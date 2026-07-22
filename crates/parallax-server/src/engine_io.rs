use std::path::Path;
use std::time::Duration;

pub(crate) async fn wait_for_ports_free(ports: &[u16], timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        let mut listeners = Vec::with_capacity(ports.len());
        for port in ports {
            match tokio::net::TcpListener::bind(("127.0.0.1", *port)).await {
                Ok(listener) => listeners.push(listener),
                Err(_) => break,
            }
        }
        if listeners.len() == ports.len() {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

pub(crate) async fn stop_child_and_wait(
    task: Option<tokio::task::JoinHandle<()>>,
    pid_path: &Path,
    ports: &[u16],
) -> bool {
    if let Some(task) = task {
        task.abort();
        let _cancelled = task.await;
    }
    crate::outcomes::note(tokio::fs::remove_file(pid_path).await, "remove pid file");
    wait_for_ports_free(ports, Duration::from_secs(10)).await
}

pub(crate) async fn reap_stale_child(pid_path: &Path, port: u16) {
    let Some(pid) = tokio::fs::read_to_string(pid_path)
        .await
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
    else {
        return;
    };
    let probe = tokio::process::Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command=,ppid="])
        .output()
        .await
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default();
    let is_greptime = probe.contains("greptime");
    // Only reap a true orphan (its owning serve died, so init adopted it —
    // ppid 1). A live parent means another serve actively supervises this
    // engine: killing it would crash-loop that stack; leave the pidfile and
    // let our own port preflight fail this start instead.
    let orphaned = probe
        .split_whitespace()
        .next_back()
        .and_then(|ppid| ppid.parse::<u32>().ok())
        == Some(1);
    if is_greptime && !orphaned {
        tracing::warn!("greptime child (pid {pid}) is still owned by a live serve; not reaping");
        return;
    }
    if is_greptime {
        tracing::warn!("reaping stale greptime child (pid {pid}) from a previous serve");
        let status = tokio::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .await;
        crate::outcomes::note(
            status.map(|status| status.success()),
            "stale child termination",
        );
        for _ in 0..40 {
            if tokio::net::TcpListener::bind(("127.0.0.1", port))
                .await
                .is_ok()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
    crate::outcomes::note(
        tokio::fs::remove_file(pid_path).await,
        "remove stale pid file",
    );
}

pub(crate) async fn install_archive(
    bin_dir: &Path,
    asset: &str,
    managed: &Path,
    archive: &[u8],
) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(bin_dir).await?;
    let archive_path = bin_dir.join(format!("{asset}.tar.gz"));
    tokio::fs::write(&archive_path, archive).await?;
    let status = tokio::process::Command::new("tar")
        .arg("-xzf")
        .arg(&archive_path)
        .arg("-C")
        .arg(bin_dir)
        .status()
        .await?;
    anyhow::ensure!(status.success(), "extracting GreptimeDB archive failed");
    tokio::fs::rename(bin_dir.join(asset).join("greptime"), managed).await?;
    crate::outcomes::note(
        tokio::fs::remove_dir_all(bin_dir.join(asset)).await,
        "extracted asset cleanup",
    );
    crate::outcomes::note(
        tokio::fs::remove_file(archive_path).await,
        "archive cleanup",
    );
    Ok(())
}
