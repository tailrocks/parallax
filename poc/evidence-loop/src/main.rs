use anyhow::Context;
use evidence_loop_poc::budget::{compute_budget, OutcomesData};
use evidence_loop_poc::dispatch::build_fix_candidate;
use evidence_loop_poc::run_pipeline;
use std::fs;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let fixtures = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fixtures"));

    let trace_json = fs::read_to_string(fixtures.join("otlp-trace.json"))
        .with_context(|| format!("reading {}/otlp-trace.json", fixtures.display()))?;
    let logs_json = fs::read_to_string(fixtures.join("otlp-logs.json"))
        .with_context(|| format!("reading {}/otlp-logs.json", fixtures.display()))?;
    let deploys_json = fs::read_to_string(fixtures.join("deploy-events.json")).ok();

    let output = run_pipeline("proj_checkout", &trace_json, &logs_json, deploys_json.as_deref())?;

    println!("derived error events: {}", output.error_events.len());
    for ev in &output.error_events {
        println!(
            "  [{}] {:?} {} fp={}",
            ev.span_id, ev.source, ev.error_type, ev.fingerprint
        );
    }

    let out_dir = PathBuf::from("out");
    fs::create_dir_all(&out_dir)?;
    println!("bundles: {}", output.bundles.len());
    for b in &output.bundles {
        let path = out_dir.join(format!("{}.json", b.bundle_id));
        fs::write(&path, serde_json::to_string_pretty(b)?)?;
        println!(
            "  {} anchor_fp={} nodes={} edges={} redactions={} hash={}",
            path.display(),
            b.anchor.fingerprint,
            b.nodes.len(),
            b.edges.len(),
            b.redaction_report.total(),
            b.canonical_hash.as_deref().unwrap_or("-")
        );
    }

    // Dispatch: compute the autonomy budget from outcome history (if fixture
    // present) and emit one fix_candidate wake payload per bundle.
    if let Ok(outcomes_json) = fs::read_to_string(fixtures.join("outcome-rows.json")) {
        let outcomes: OutcomesData =
            serde_json::from_str(&outcomes_json).context("parsing outcome rows JSON")?;
        println!("fix candidates:");
        for b in &output.bundles {
            let budget = compute_budget(&outcomes.outcomes, "backend_error");
            let candidate = build_fix_candidate(
                b,
                budget,
                vec!["cargo test -p checkout".to_string()],
            );
            let path = out_dir.join(format!("fix-candidate-{}.json", b.anchor.fingerprint));
            fs::write(&path, serde_json::to_string_pretty(&candidate)?)?;
            println!(
                "  {} issue={} trigger={} budget={} key={}",
                path.display(),
                candidate.issue_id,
                candidate.trigger,
                candidate.autonomy_budget.max_level,
                candidate.idempotency_key
            );
        }
    }
    Ok(())
}
