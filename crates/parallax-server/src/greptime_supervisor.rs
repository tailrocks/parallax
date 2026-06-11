//! Managed GreptimeDB supervision, per implementation-spec §11:
//! resolve binary (data-dir bin → PATH → checksum-verified download of the
//! pinned/latest release), spawn `greptime standalone start` on the shifted
//! ports (24000–24003), health-check, restart with backoff.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

pub const GREPTIME_HTTP_PORT: u16 = 24000;
pub const GREPTIME_GRPC_PORT: u16 = 24001;
pub const GREPTIME_MYSQL_PORT: u16 = 24002;
pub const GREPTIME_POSTGRES_PORT: u16 = 24003;

pub struct GreptimeSupervisor {
    binary: PathBuf,
    data_home: PathBuf,
    log_path: PathBuf,
    pub http_url: String,
    task: Option<tokio::task::JoinHandle<()>>,
}

fn host_target() -> anyhow::Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Ok("darwin-arm64"),
        ("macos", "x86_64") => Ok("darwin-amd64"),
        ("linux", "aarch64") => Ok("linux-arm64"),
        ("linux", "x86_64") => Ok("linux-amd64"),
        (os, arch) => anyhow::bail!("unsupported host for managed GreptimeDB: {os}/{arch}"),
    }
}

/// Known-good fallback when "latest" cannot be resolved (no network to the
/// GitHub API): the compatible floor from the implementation spec.
const FALLBACK_VERSION: &str = "1.0.2";

/// Resolve "latest" to a concrete release tag via the GitHub API, falling
/// back to the pinned floor when the API is unreachable.
async fn resolve_version(version: &str) -> anyhow::Result<String> {
    if version != "latest" {
        return Ok(version.trim_start_matches('v').to_string());
    }
    let response = reqwest::Client::new()
        .get("https://api.github.com/repos/GreptimeTeam/greptimedb/releases/latest")
        .header("user-agent", "parallax")
        .send()
        .await;
    match response {
        Ok(r) => {
            let body: serde_json::Value = r.error_for_status()?.json().await?;
            let tag = body
                .get("tag_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("releases/latest response missing tag_name"))?;
            Ok(tag.trim_start_matches('v').to_string())
        }
        Err(e) => {
            tracing::warn!(
                "could not resolve latest GreptimeDB release ({e}); \
                 falling back to v{FALLBACK_VERSION}"
            );
            Ok(FALLBACK_VERSION.to_string())
        }
    }
}

/// Locate or install the engine binary. Returns (path, resolved_version_hint).
pub async fn ensure_binary(
    bin_dir: &Path,
    version: &str,
    allow_download: bool,
) -> anyhow::Result<PathBuf> {
    let managed = bin_dir.join("greptime");
    if managed.exists() {
        return Ok(managed);
    }
    if let Ok(output) = std::process::Command::new("greptime")
        .arg("--version")
        .output()
        && output.status.success()
    {
        return Ok(PathBuf::from("greptime"));
    }
    anyhow::ensure!(
        allow_download,
        "greptime binary not found (looked in {} and PATH); re-run with download allowed, \
         install via the Greptime brew tap, or use --greptime-url / --no-greptime",
        bin_dir.display()
    );

    let version = resolve_version(version).await?;
    let target = host_target()?;
    let asset = format!("greptime-{target}-v{version}");
    let base = format!("https://github.com/GreptimeTeam/greptimedb/releases/download/v{version}");
    tracing::info!("downloading GreptimeDB v{version} ({target}) — one-time setup");

    let client = reqwest::Client::new();
    let checksum_line = client
        .get(format!("{base}/{asset}.sha256sum"))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let expected = checksum_line
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow::anyhow!("empty sha256sum asset"))?
        .to_lowercase();

    let archive = client
        .get(format!("{base}/{asset}.tar.gz"))
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let actual = format!("{:x}", Sha256::digest(&archive));
    anyhow::ensure!(
        actual == expected,
        "GreptimeDB download checksum mismatch: expected {expected}, got {actual}"
    );

    std::fs::create_dir_all(bin_dir)?;
    let archive_path = bin_dir.join(format!("{asset}.tar.gz"));
    std::fs::write(&archive_path, &archive)?;
    let status = std::process::Command::new("tar")
        .arg("-xzf")
        .arg(&archive_path)
        .arg("-C")
        .arg(bin_dir)
        .status()?;
    anyhow::ensure!(status.success(), "extracting GreptimeDB archive failed");
    std::fs::rename(bin_dir.join(&asset).join("greptime"), &managed)?;
    let _ = std::fs::remove_dir_all(bin_dir.join(&asset));
    let _ = std::fs::remove_file(&archive_path);
    tracing::info!("GreptimeDB v{version} installed to {}", managed.display());
    Ok(managed)
}

impl GreptimeSupervisor {
    /// Spawn the child and wait until /health answers (or time out).
    pub async fn start(binary: PathBuf, data_dir: &Path) -> anyhow::Result<Self> {
        let data_home = data_dir.join("greptime-data");
        std::fs::create_dir_all(&data_home)?;
        let log_path = data_dir.join("greptime.log");
        let http_url = format!("http://127.0.0.1:{GREPTIME_HTTP_PORT}");

        let mut supervisor = Self {
            binary,
            data_home,
            log_path,
            http_url: http_url.clone(),
            task: None,
        };
        let child = supervisor.spawn()?;
        supervisor.monitor(child);
        supervisor.wait_healthy(Duration::from_secs(30)).await?;
        Ok(supervisor)
    }

    fn spawn(&self) -> anyhow::Result<tokio::process::Child> {
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        let child = Command::new(&self.binary)
            .args(["standalone", "start"])
            .arg("--http-addr")
            .arg(format!("127.0.0.1:{GREPTIME_HTTP_PORT}"))
            .arg("--rpc-bind-addr")
            .arg(format!("127.0.0.1:{GREPTIME_GRPC_PORT}"))
            .arg("--mysql-addr")
            .arg(format!("127.0.0.1:{GREPTIME_MYSQL_PORT}"))
            .arg("--postgres-addr")
            .arg(format!("127.0.0.1:{GREPTIME_POSTGRES_PORT}"))
            .arg("--data-home")
            .arg(&self.data_home)
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log))
            .kill_on_drop(true)
            .spawn()?;
        Ok(child)
    }

    /// Restart on exit with linear backoff; give up after repeated fast deaths.
    fn monitor(&mut self, mut child: tokio::process::Child) {
        let binary = self.binary.clone();
        let data_home = self.data_home.clone();
        let log_path = self.log_path.clone();
        self.task = Some(tokio::spawn(async move {
            let mut fast_failures = 0u32;
            loop {
                let started = std::time::Instant::now();
                let status = child.wait().await;
                tracing::warn!("greptime child exited: {status:?}");
                if started.elapsed() < Duration::from_secs(5) {
                    fast_failures += 1;
                } else {
                    fast_failures = 0;
                }
                if fast_failures >= 5 {
                    tracing::error!(
                        "greptime crashed {fast_failures} times in a row; giving up — \
                         see {} and run `parallax doctor`",
                        log_path.display()
                    );
                    return;
                }
                tokio::time::sleep(Duration::from_secs(u64::from(fast_failures) + 1)).await;
                let respawned = Self {
                    binary: binary.clone(),
                    data_home: data_home.clone(),
                    log_path: log_path.clone(),
                    http_url: String::new(),
                    task: None,
                }
                .spawn();
                match respawned {
                    Ok(c) => child = c,
                    Err(e) => {
                        tracing::error!("greptime respawn failed: {e}");
                        return;
                    }
                }
            }
        }));
    }

    async fn wait_healthy(&self, timeout: Duration) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let deadline = std::time::Instant::now() + timeout;
        let url = format!("{}/health", self.http_url);
        loop {
            if let Ok(response) = client.get(&url).send().await
                && response.status().is_success()
            {
                return Ok(());
            }
            anyhow::ensure!(
                std::time::Instant::now() < deadline,
                "GreptimeDB did not become healthy within {timeout:?}; see {}",
                self.log_path.display()
            );
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    pub fn stop(&self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}
