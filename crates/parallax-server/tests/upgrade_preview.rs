//! Cross-release upgrade harness (plan 174 Step 1).
//!
//! Downloads the rolling `preview` GitHub release for this host (or uses
//! `PARALLAX_UPGRADE_PREVIOUS`), seeds a data dir, then reopens it with the
//! workspace server. Ignored: downloads a ~60MiB archive + may start Greptime.

use parallax_metadata::TursoMetadataStore;
use parallax_server::Config;
use parallax_spool::Signal;
use prost::Message;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const PREVIEW_BASE: &str = "https://github.com/tailrocks/parallax/releases/download/preview";

fn host_archive() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Some("parallax-aarch64-apple-darwin.tar.gz"),
        ("macos", "x86_64") => Some("parallax-x86_64-apple-darwin.tar.gz"),
        ("linux", "aarch64") => Some("parallax-aarch64-unknown-linux-gnu.tar.gz"),
        ("linux", "x86_64") => Some("parallax-x86_64-unknown-linux-gnu.tar.gz"),
        _ => None,
    }
}

fn cache_dir() -> PathBuf {
    std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("parallax-upgrade-preview")
}

async fn download_preview_bin() -> anyhow::Result<PathBuf> {
    if let Ok(path) = std::env::var("PARALLAX_UPGRADE_PREVIOUS") {
        let path = PathBuf::from(path);
        anyhow::ensure!(path.is_file(), "PARALLAX_UPGRADE_PREVIOUS is not a file");
        return Ok(path);
    }
    let archive = host_archive().ok_or_else(|| {
        anyhow::anyhow!(
            "no preview archive for {}-{}",
            std::env::consts::OS,
            std::env::consts::ARCH
        )
    })?;
    let dest_dir = cache_dir();
    std::fs::create_dir_all(&dest_dir)?;
    let bin = dest_dir.join("parallax");
    if bin.is_file() {
        return Ok(bin);
    }
    let archive_path = dest_dir.join(archive);
    let url = format!("{PREVIEW_BASE}/{archive}");
    let bytes = reqwest::get(&url)
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let expected = reqwest::get(format!("{url}.sha256"))
        .await?
        .error_for_status()?
        .text()
        .await?;
    let digest = sha256_hex(&bytes);
    anyhow::ensure!(
        expected.trim().starts_with(&digest),
        "preview checksum mismatch"
    );
    std::fs::write(&archive_path, &bytes)?;
    let status = Command::new("tar")
        .arg("-xzf")
        .arg(&archive_path)
        .arg("-C")
        .arg(&dest_dir)
        .status()?;
    anyhow::ensure!(status.success(), "tar extract failed");
    anyhow::ensure!(bin.is_file(), "archive missing parallax binary");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&bin)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&bin, perms)?;
    }
    Ok(bin)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn write_config(
    path: &Path,
    data_dir: &Path,
    api_port: u16,
    otlp_http: u16,
    otlp_grpc: u16,
) -> anyhow::Result<()> {
    let body = format!(
        "[server]\nbind = \"127.0.0.1\"\napi_port = {api_port}\notlp_grpc_port = {otlp_grpc}\notlp_http_port = {otlp_http}\n\n[storage]\nmode = \"managed\"\ndata_dir = \"{}\"\n",
        data_dir.display()
    );
    std::fs::write(path, body)?;
    Ok(())
}

fn spawn_serve(bin: &Path, config: &Path, home: &Path) -> anyhow::Result<Child> {
    Ok(Command::new(bin)
        .arg("serve")
        .arg("--config")
        .arg(config)
        .env("HOME", home)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?)
}

async fn sleep_ms(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

async fn wait_http(url: &str, timeout: Duration) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > timeout {
            anyhow::bail!("timeout waiting for {url}");
        }
        if let Ok(response) = client.get(url).send().await
            && (response.status().is_success() || response.status().as_u16() == 503)
        {
            return Ok(response.text().await.unwrap_or_default());
        }
        sleep_ms(200).await;
    }
}

fn stop_child(mut child: Child) {
    drop(child.kill());
    drop(child.wait());
}

/// SIGKILL of preview `serve` leaves a managed Greptime grandchild. Reap by
/// data-home, not pidfile: an older preview or a kill-before-write leaves no
/// `greptime.pid`, and workspace start then fails preflight on 24000.
fn stop_preview_engine(data_dir: &Path) {
    let data_home = data_dir.join("greptime-data");
    let needle = data_home.display().to_string();
    let Ok(output) = Command::new("ps")
        .args(["-ax", "-o", "pid=,command="])
        .output()
    else {
        return;
    };
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let line = line.trim();
        if !line.contains("greptime") || !line.contains(&needle) {
            continue;
        }
        if let Some(pid) = line.split_whitespace().next() {
            drop(Command::new("kill").args(["-TERM", pid]).status());
        }
    }
}

fn engine_port_held(port: u16) -> bool {
    std::net::TcpStream::connect_timeout(
        &(std::net::Ipv4Addr::LOCALHOST, port).into(),
        Duration::from_millis(20),
    )
    .is_ok()
}

async fn wait_engine_ports_free() -> anyhow::Result<()> {
    for _ in 0..200 {
        if ![24_000_u16, 24_001, 24_002, 24_003]
            .into_iter()
            .any(engine_port_held)
        {
            return Ok(());
        }
        sleep_ms(50).await;
    }
    anyhow::bail!("managed Greptime ports 24000–24003 still held after preview stop")
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads preview release binary; run with --ignored"]
async fn upgrade_preview_data_dir_opens_losslessly_under_workspace() -> anyhow::Result<()> {
    let old_bin = download_preview_bin().await?;
    let tmp = tempfile::tempdir()?;
    let home = tmp.path().join("home");
    let data_dir = home.join(".parallax");
    std::fs::create_dir_all(&data_dir)?;
    let api_port = 18_000 + u16::try_from(std::process::id() % 500).unwrap_or(0);
    let otlp_http = api_port + 1;
    let otlp_grpc = api_port + 2;
    let config_path = data_dir.join("config.toml");
    write_config(&config_path, &data_dir, api_port, otlp_http, otlp_grpc)?;

    let child = spawn_serve(&old_bin, &config_path, &home)?;
    let health_url = format!("http://127.0.0.1:{api_port}/health");
    if let Err(error) = wait_http(&health_url, Duration::from_secs(180)).await {
        stop_child(child);
        anyhow::bail!("preview serve never became ready: {error}");
    }

    let body =
        parallax_proto::collector_trace::ExportTraceServiceRequest::default().encode_to_vec();
    let client = reqwest::Client::new();
    let posted = client
        .post(format!("http://127.0.0.1:{otlp_http}/v1/traces"))
        .header("content-type", "application/x-protobuf")
        .body(body)
        .send()
        .await?;
    anyhow::ensure!(
        posted.status().is_success(),
        "preview OTLP rejected: {}",
        posted.status()
    );
    sleep_ms(400).await;
    stop_child(child);
    stop_preview_engine(&data_dir);
    wait_engine_ports_free().await?;

    let spool_before =
        parallax_spool::Spool::open(data_dir.join("spool"))?.line_count(Signal::Traces)?;
    anyhow::ensure!(spool_before >= 1, "preview write must land a spool frame");

    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.data_dir = data_dir.to_string_lossy().into_owned();
    config.storage.mode = "managed".into();
    let handle = parallax_server::start(&config).await?;
    let health = reqwest::get(format!("http://{}/health", handle.api_addr))
        .await?
        .text()
        .await?;
    anyhow::ensure!(
        health == "ok" || health.starts_with("degraded:"),
        "{health}"
    );

    TursoMetadataStore::open(data_dir.join("meta.db")).await?;

    let spool_after = handle.spool.line_count(Signal::Traces)?;
    anyhow::ensure!(
        spool_after == spool_before,
        "spool frames must not vanish: {spool_after} != {spool_before}"
    );
    handle.shutdown();
    Ok(())
}
