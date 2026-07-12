use std::path::Path;

pub(crate) async fn reap_stale_child(pid_path: &Path, port: u16) {
    let Some(pid) = tokio::fs::read_to_string(pid_path)
        .await
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
    else {
        return;
    };
    let command = tokio::process::Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output()
        .await
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default();
    if command.contains("greptime") {
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
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
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
