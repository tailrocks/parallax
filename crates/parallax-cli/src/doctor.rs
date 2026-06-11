//! Self-sufficiency commands: `doctor` (diagnose the local install),
//! `prune` (reclaim spool space now), `uninstall --purge` (remove the data
//! directory). These inspect the local installation directly — they are
//! install tooling, not telemetry queries, so the API boundary does not
//! apply to them.

use std::path::{Path, PathBuf};

fn data_dir() -> PathBuf {
    // Mirrors the server's default; a custom data_dir is read from the
    // default config file when present.
    let default = std::env::home_dir()
        .map(|h| h.join(".parallax"))
        .unwrap_or_else(|| PathBuf::from(".parallax"));
    let config_path = default.join("config.toml");
    if let Ok(raw) = std::fs::read_to_string(&config_path)
        && let Ok(value) = raw.parse::<toml::Table>()
        && let Some(dir) = value
            .get("storage")
            .and_then(|s| s.get("data_dir"))
            .and_then(|d| d.as_str())
    {
        if let Some(rest) = dir.strip_prefix("~/")
            && let Some(home) = std::env::home_dir()
        {
            return home.join(rest);
        }
        return PathBuf::from(dir);
    }
    default
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

fn human(bytes: u64) -> String {
    match bytes {
        0..=1023 => format!("{bytes} B"),
        1024..=1_048_575 => format!("{:.1} KiB", bytes as f64 / 1024.0),
        1_048_576..=1_073_741_823 => format!("{:.1} MiB", bytes as f64 / 1_048_576.0),
        _ => format!("{:.2} GiB", bytes as f64 / 1_073_741_824.0),
    }
}

async fn check_http(url: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok()?;
    let response = client.get(url).send().await.ok()?;
    response.status().is_success().then(|| "ok".to_string())
}

pub async fn doctor() -> anyhow::Result<()> {
    let dir = data_dir();
    println!("parallax doctor");
    println!("  data dir: {} ({})", dir.display(), human(dir_size(&dir)));

    // Server + listeners.
    for (name, url) in [
        ("api (:4000)", "http://127.0.0.1:4000/health"),
        ("greptime child (:24000)", "http://127.0.0.1:24000/health"),
    ] {
        match check_http(url).await {
            Some(_) => println!("  {name}: ok"),
            None => println!("  {name}: NOT RESPONDING"),
        }
    }
    match check_http("http://127.0.0.1:4000/version").await {
        Some(_) => {
            let version = reqwest::get("http://127.0.0.1:4000/version")
                .await?
                .text()
                .await
                .unwrap_or_default();
            println!("  server version: {version}");
        }
        None => println!("  server version: unavailable (is `parallax serve` running?)"),
    }

    // Engine binary.
    let engine = dir.join("bin/greptime");
    if engine.exists() {
        let version = std::process::Command::new(&engine)
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| {
                s.lines()
                    .find(|l| l.contains("version"))
                    .map(|l| l.trim().to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());
        println!("  engine binary: {} ({version})", engine.display());
    } else {
        println!("  engine binary: not installed in data dir (PATH or external mode?)");
    }

    // Spool backlog and storage sizes.
    let spool = dir.join("spool");
    if spool.exists() {
        for file in ["traces.ndjson", "logs.ndjson", "metrics.ndjson"] {
            let path = spool.join(file);
            let lines = std::fs::read_to_string(&path)
                .map(|s| s.lines().count())
                .unwrap_or(0);
            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
            println!("  spool {file}: {lines} request(s), {}", human(size));
        }
    }
    let engine_data = dir.join("greptime-data");
    if engine_data.exists() {
        println!("  engine data: {}", human(dir_size(&engine_data)));
    }
    let meta = dir.join("meta.db");
    if meta.exists() {
        println!(
            "  metadata db: {}",
            human(meta.metadata().map(|m| m.len()).unwrap_or(0))
        );
    }
    let log = dir.join("greptime.log");
    if log.exists() {
        println!(
            "  engine log: {} ({})",
            log.display(),
            human(log.metadata().map(|m| m.len()).unwrap_or(0))
        );
    }
    Ok(())
}

/// Truncate the ingest spool (telemetry TTLs are enforced by the engine).
pub fn prune() -> anyhow::Result<()> {
    let dir = data_dir().join("spool");
    let mut reclaimed = 0u64;
    for file in ["traces.ndjson", "logs.ndjson", "metrics.ndjson"] {
        let path = dir.join(file);
        if let Ok(meta) = path.metadata() {
            reclaimed += meta.len();
            std::fs::write(&path, b"")?;
        }
    }
    println!("pruned spool: reclaimed {}", human(reclaimed));
    println!("telemetry retention is TTL-managed by the engine (see config [retention])");
    Ok(())
}

/// Remove the entire data directory. Destructive; requires --purge.
pub fn uninstall(purge: bool, yes: bool) -> anyhow::Result<()> {
    if !purge {
        println!("nothing removed. Use `parallax uninstall --purge` to delete the data dir;");
        println!("remove the binary with your package manager (e.g. brew uninstall parallax).");
        return Ok(());
    }
    let dir = data_dir();
    if !dir.exists() {
        println!("{} does not exist — nothing to remove", dir.display());
        return Ok(());
    }
    let size = human(dir_size(&dir));
    if !yes {
        println!(
            "This permanently deletes {} ({size}) including all telemetry, issues, and the \
             managed engine. Re-run with --yes to confirm.",
            dir.display()
        );
        return Ok(());
    }
    std::fs::remove_dir_all(&dir)?;
    println!("removed {} ({size})", dir.display());
    println!("remove the binary with your package manager (e.g. brew uninstall parallax).");
    Ok(())
}
