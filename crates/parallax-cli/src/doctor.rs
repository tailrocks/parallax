#![expect(
    clippy::cast_precision_loss,
    reason = "human-readable storage percentages"
)]

//! Self-sufficiency commands: `doctor` (diagnose the local install),
//! `prune` (lifecycle plan + reclaim), `uninstall --purge` (remove the data
//! directory). These inspect the local installation directly — they are
//! install tooling, not telemetry queries, so the API boundary does not
//! apply to them.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use parallax_metadata::TursoMetadataStore;
use parallax_server::Config;
use parallax_storage::{
    MetadataPruneStore, PruneClass, PruneEstimate, PruneExecutionRequest, PruneItem,
    PruneJournalStepState, PrunePlan, PrunePlanLimits, PruneSnapshot, PruneStepStart, PruneStore,
};

const SPOOL_SIGNALS: [(&str, &str); 3] = [
    ("traces", "traces.pspl"),
    ("logs", "logs.pspl"),
    ("metrics", "metrics.pspl"),
];
const SPOOL_LEGACY: [(&str, &str); 3] = [
    ("traces", "traces.ndjson"),
    ("logs", "logs.ndjson"),
    ("metrics", "metrics.ndjson"),
];

#[derive(Debug, Default, PartialEq, Eq)]
struct SignalSpoolStats {
    active_lines: usize,
    active_bytes: u64,
    rotated_segments: usize,
    rotated_bytes: u64,
}

fn default_data_dir() -> PathBuf {
    std::env::home_dir().map_or_else(|| PathBuf::from(".parallax"), |h| h.join(".parallax"))
}

fn config_path() -> PathBuf {
    default_data_dir().join("config.toml")
}

fn load_config() -> Config {
    Config::load(Some(&config_path())).unwrap_or_default()
}

fn data_dir() -> PathBuf {
    load_config().data_dir()
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

fn rotated_segment_paths(spool_dir: &Path, stem: &str) -> Vec<PathBuf> {
    let prefix = format!("{stem}.");
    let active_pspl = format!("{stem}.pspl");
    let active_ndjson = format!("{stem}.ndjson");
    let Ok(entries) = std::fs::read_dir(spool_dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name != active_pspl
                        && name != active_ndjson
                        && name.starts_with(&prefix)
                        && (name.ends_with(".pspl") || name.ends_with(".ndjson"))
                })
        })
        .collect()
}

fn count_pspl_frames(path: &Path) -> usize {
    use std::io::Read;
    let Ok(mut file) = std::fs::File::open(path) else {
        return 0;
    };
    let mut magic = [0u8; 5];
    let Ok(n) = file.read(&mut magic) else {
        return 0;
    };
    if n < 5 || &magic != b"PSPL1" {
        return 0;
    }
    let mut count = 0usize;
    loop {
        let mut len_buf = [0u8; 4];
        if file.read_exact(&mut len_buf).is_err() {
            break;
        }
        let len = u64::from(u32::from_le_bytes(len_buf));
        if std::io::copy(&mut file.by_ref().take(len), &mut std::io::sink()).is_err() {
            break;
        }
        count += 1;
    }
    count
}

fn spool_stats(spool_dir: &Path, stem: &str, active_file: &str) -> SignalSpoolStats {
    let active_path = spool_dir.join(active_file);
    let legacy_path = spool_dir.join(format!("{stem}.ndjson"));
    let mut active_lines = if active_path.exists() {
        count_pspl_frames(&active_path)
    } else {
        0
    };
    if legacy_path.exists() {
        active_lines += std::fs::read_to_string(&legacy_path).map_or(0, |s| s.lines().count());
    }
    let active_bytes = active_path.metadata().map_or(0, |m| m.len())
        + legacy_path.metadata().map_or(0, |m| m.len());
    let rotated_paths = rotated_segment_paths(spool_dir, stem);
    let rotated_bytes = rotated_paths
        .iter()
        .filter_map(|path| path.metadata().ok().map(|metadata| metadata.len()))
        .sum();
    SignalSpoolStats {
        active_lines,
        active_bytes,
        rotated_segments: rotated_paths.len(),
        rotated_bytes,
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

pub(crate) async fn doctor() -> anyhow::Result<()> {
    let config = load_config();
    let dir = config.data_dir();
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
        let version = tokio::process::Command::new(&engine)
            .arg("--version")
            .output()
            .await
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
        println!(
            "  spool caps: segment {}, total {}, max age {}h",
            human(config.retention.spool_max_segment_bytes),
            human(config.retention.spool_max_total_bytes),
            config.retention.spool_max_age_hours
        );
        for (stem, file) in SPOOL_SIGNALS {
            let stats = spool_stats(&spool, stem, file);
            println!(
                "  spool {file}: active {} ({} request(s)) + {} rotated segment(s) ({})",
                human(stats.active_bytes),
                stats.active_lines,
                stats.rotated_segments,
                human(stats.rotated_bytes)
            );
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
            human(meta.metadata().map_or(0, |m| m.len()))
        );
    }
    let log = dir.join("greptime.log");
    if log.exists() {
        println!(
            "  engine log: {} ({})",
            log.display(),
            human(log.metadata().map_or(0, |m| m.len()))
        );
    }

    // Deploy/change context adapter (plan 121 residual doctor surface).
    // Provider text is untrusted evidence; this only reports configuration and
    // durable delivery inventory, never causal claims.
    let webhook_secret = config.resolved_github_webhook_secret();
    println!(
        "  deploy-context webhook: {}",
        if webhook_secret.is_some() {
            "secret configured (POST /webhooks/github)"
        } else {
            "disabled (set PARALLAX_GITHUB_WEBHOOK_SECRET or [github_deploy].webhook_secret)"
        }
    );
    if meta.exists() {
        match TursoMetadataStore::open(&meta).await {
            Ok(store) => match store.count_deploy_deliveries().await {
                Ok(count) => println!("  deploy-context deliveries: {count}"),
                Err(err) => println!("  deploy-context deliveries: unavailable ({err})"),
            },
            Err(err) => println!("  deploy-context deliveries: open failed ({err})"),
        }
    } else {
        println!("  deploy-context deliveries: no meta.db yet");
    }

    Ok(())
}

/// Contract grace for resolved issues / terminal invocations (30 days).
const PRUNE_GRACE_NANOS: u128 = 30 * 24 * 60 * 60 * 1_000_000_000;

/// Deterministic lifecycle prune plan (dry-run default) with optional execute.
#[expect(
    clippy::too_many_lines,
    reason = "one linear prune walkthrough with narration"
)]
pub(crate) async fn prune(execute: bool, yes: bool, json: bool) -> anyhow::Result<()> {
    let config = load_config();
    let dir = config.data_dir();
    let meta_path = dir.join("meta.db");
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let cutoff_nanos = now_nanos.saturating_sub(PRUNE_GRACE_NANOS);

    let mut items: Vec<PruneItem> = Vec::new();
    let mut protection_generation = "pins:none".to_string();
    if meta_path.exists() {
        let store = TursoMetadataStore::open(&meta_path).await?;
        protection_generation = store.evidence_pin_protection(now_nanos).await?.generation;
        items.extend(store.metadata_prune_items(cutoff_nanos, now_nanos).await?);
    } else {
        eprintln!(
            "warning: no Turso metadata at {} — planning spool only",
            meta_path.display()
        );
    }

    let spool_dir = dir.join("spool");
    let spool_bytes = spool_dir_bytes(&spool_dir);
    items.push(PruneItem {
        store: PruneStore::LocalDisk,
        class: PruneClass::Spool,
        target: "spool".to_string(),
        cutoff_nanos,
        estimate: PruneEstimate {
            rows: None,
            // Plan contract requires a row or object estimate on every item.
            objects: Some(u64::from(spool_bytes > 0)),
            bytes: Some(spool_bytes),
        },
        exclusions: Vec::new(),
        warnings: vec!["truncates active segments and removes rotated spool files".to_string()],
    });

    let snapshot = PruneSnapshot {
        config_generation: format!(
            "retention:{}:{}:{}",
            config.retention.traces_ttl, config.retention.logs_ttl, config.retention.metrics_ttl
        ),
        protection_generation,
        catalog_fingerprint: "metadata+spool".to_string(),
    };
    let plan = PrunePlan::build(
        cutoff_nanos,
        snapshot.clone(),
        items,
        PrunePlanLimits::default(),
    )?;

    if json {
        println!("{}", serde_json::to_string_pretty(&plan)?);
    } else {
        print_plan_human(&plan);
    }

    if !execute {
        if !json {
            println!();
            println!(
                "dry-run only — re-run with --execute (and --yes for non-interactive) to apply"
            );
            println!("raw telemetry remains engine-TTL managed (see config [retention])");
        }
        return Ok(());
    }

    if !yes {
        println!();
        println!("This permanently deletes eligible Turso lifecycle rows and truncates the spool.");
        println!("Re-run with --execute --yes to confirm.");
        return Ok(());
    }

    // Journal + execute Turso steps; spool is local-disk.
    let mut report = serde_json::Map::new();
    report.insert(
        "plan_id".into(),
        serde_json::Value::String(plan.plan_id().to_string()),
    );
    let mut deleted = serde_json::Map::new();

    if meta_path.exists() {
        let store = TursoMetadataStore::open(&meta_path).await?;
        let execution_now_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let current_protection = store.evidence_pin_protection(execution_now_nanos).await?;
        let current_snapshot = PruneSnapshot {
            protection_generation: current_protection.generation,
            ..snapshot.clone()
        };
        let request = PruneExecutionRequest::execute(plan.plan_id().to_string(), true)?;
        let authorization = plan.authorize(&request, &current_snapshot)?;
        let journal = store
            .create_prune_journal(&plan, now_nanos, PrunePlanLimits::default())
            .await?;
        for step in &journal.steps {
            if step.item.store != PruneStore::Turso {
                continue;
            }
            match store
                .begin_prune_step(plan.plan_id(), step.step_index, now_nanos)
                .await?
            {
                PruneStepStart::AlreadyComplete => continue,
                PruneStepStart::Execute => {}
            }
            match store
                .execute_prune_item(&step.item, &authorization, execution_now_nanos)
                .await
            {
                Ok(rows) => {
                    deleted.insert(step.item.target.clone(), serde_json::Value::from(rows));
                    store
                        .complete_prune_step(plan.plan_id(), step.step_index, now_nanos)
                        .await?;
                }
                Err(error) => {
                    let message = error.to_string();
                    let _unused = store
                        .record_prune_step_failure(
                            plan.plan_id(),
                            step.step_index,
                            &message,
                            now_nanos,
                        )
                        .await;
                    anyhow::bail!(
                        "prune step {} ({}) failed: {message}",
                        step.step_index,
                        step.item.target
                    );
                }
            }
        }
        // Spool is not journaled in Turso; execute after metadata succeeds.
        let reclaimed = prune_dir(&spool_dir)?;
        deleted.insert("spool_bytes".into(), serde_json::Value::from(reclaimed));
        // Mark any spool step complete if present.
        if let Some(step) = journal
            .steps
            .iter()
            .find(|s| s.item.class == PruneClass::Spool)
            && step.state != PruneJournalStepState::Complete
            && let Ok(PruneStepStart::Execute) = store
                .begin_prune_step(plan.plan_id(), step.step_index, now_nanos)
                .await
        {
            let _unused = store
                .complete_prune_step(plan.plan_id(), step.step_index, now_nanos)
                .await;
        }
    } else {
        let reclaimed = prune_dir(&spool_dir)?;
        deleted.insert("spool_bytes".into(), serde_json::Value::from(reclaimed));
    }

    report.insert("deleted".into(), serde_json::Value::Object(deleted));
    report.insert(
        "status".into(),
        serde_json::Value::String("complete".into()),
    );
    report.insert(
        "physical_bytes_note".into(),
        serde_json::Value::String(
            "GreptimeDB TTL/compaction is asynchronous; logical deletions reported above".into(),
        ),
    );

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!();
        println!("prune complete");
        if let Some(map) = report.get("deleted").and_then(|v| v.as_object()) {
            for (k, v) in map {
                println!("  {k}: {v}");
            }
        }
        println!("raw telemetry remains engine-TTL managed (physical reclaim via compaction)");
    }
    Ok(())
}

fn spool_dir_bytes(dir: &Path) -> u64 {
    if !dir.exists() {
        return 0;
    }
    dir_size(dir)
}

fn print_plan_human(plan: &PrunePlan) {
    println!("prune plan {}", plan.plan_id());
    println!("  cutoff_nanos: {}", plan.cutoff_nanos());
    println!("  items: {}", plan.items().len());
    for item in plan.items() {
        let rows = item
            .estimate
            .rows
            .map(|n| n.to_string())
            .unwrap_or_else(|| "-".into());
        let bytes = item.estimate.bytes.map(human).unwrap_or_else(|| "-".into());
        println!(
            "  - {:?} / {}  rows≈{rows}  bytes≈{bytes}",
            item.store, item.target
        );
        for exclusion in &item.exclusions {
            println!("      exclude {:?}: {}", exclusion.kind, exclusion.count);
        }
        for warning in &item.warnings {
            println!("      warn: {warning}");
        }
    }
}

fn prune_dir(dir: &Path) -> anyhow::Result<u64> {
    let mut reclaimed = 0u64;
    for (_stem, file) in SPOOL_SIGNALS.iter().chain(SPOOL_LEGACY.iter()) {
        let path = dir.join(file);
        if let Ok(meta) = path.metadata() {
            reclaimed += meta.len();
            std::fs::write(&path, b"")?;
        }
    }
    for (stem, _) in SPOOL_SIGNALS {
        for rotated in rotated_segment_paths(dir, stem) {
            if let Ok(meta) = rotated.metadata() {
                reclaimed += meta.len();
            }
            std::fs::remove_file(rotated)?;
        }
    }
    Ok(reclaimed)
}

/// Remove the entire data directory. Destructive; requires --purge.
pub(crate) fn uninstall(purge: bool, yes: bool) -> anyhow::Result<()> {
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

#[cfg(test)]
mod tests;
