use std::fmt::Display;

pub(crate) fn note<T, E: Display>(result: Result<T, E>, operation: &str) {
    if let Err(error) = result {
        tracing::warn!(%error, operation, "best-effort operation failed");
    }
}

pub(crate) async fn drain_workers(
    workers: Vec<tokio::task::JoinHandle<()>>,
    health: &crate::ingest_health::IngestHealth,
) {
    let started = std::time::Instant::now();
    let drain = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        for worker in workers {
            note(worker.await, "join ingest worker");
        }
    })
    .await;
    let completed = drain.is_ok();
    health.drained(started.elapsed(), completed);
    note(drain, "drain ingest workers within 5 seconds");
}

pub(crate) async fn open_spool(
    data_dir: &std::path::Path,
    max_segment_bytes: u64,
) -> anyhow::Result<std::sync::Arc<parallax_spool::Spool>> {
    let dir = data_dir.join("spool");
    tokio::task::spawn_blocking(move || {
        parallax_spool::Spool::open_with_max_segment_bytes(dir, max_segment_bytes)
            .map(std::sync::Arc::new)
    })
    .await?
}
