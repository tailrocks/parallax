use std::{fmt::Display, process::ExitStatus};

pub(crate) fn note<T, E: Display>(result: Result<T, E>, operation: &str) {
    if let Err(error) = result {
        tracing::warn!(%error, operation, "best-effort operation failed");
    }
}

fn warn_status(result: std::io::Result<ExitStatus>, operation: &str) {
    match result {
        Ok(status) if status.success() => {}
        Ok(status) => tracing::warn!(%status, operation, "process exited unsuccessfully"),
        Err(error) => tracing::warn!(%error, operation, "process could not start"),
    }
}

pub(crate) fn kill_stale(pid: u32) {
    warn_status(
        std::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .status(),
        "kill stale GreptimeDB child",
    );
}

pub(crate) async fn drain_workers(workers: Vec<tokio::task::JoinHandle<()>>) {
    let drain = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        for worker in workers {
            note(worker.await, "join ingest worker");
        }
    })
    .await;
    note(drain, "drain ingest workers within 5 seconds");
}

pub(crate) fn cleanup_asset(bin_dir: &std::path::Path, asset: &str, archive: &std::path::Path) {
    note(
        std::fs::remove_dir_all(bin_dir.join(asset)),
        "asset cleanup",
    );
    note(std::fs::remove_file(archive), "archive cleanup");
}
