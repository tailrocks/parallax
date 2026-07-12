use super::*;

fn findings(source: &str) -> Vec<Finding> {
    let syntax = syn::parse_file(source).expect("fixture parses");
    let mut findings = Vec::new();
    BlockingVisitor {
        file: "fixture.rs",
        async_depth: 0,
        findings: &mut findings,
    }
    .visit_file(&syntax);
    findings
}

#[test]
fn rejects_blocking_only_in_async_context_and_accepts_owned_boundary() -> Result<()> {
    let direct = findings("async fn bad() { std::fs::read_to_string(\"x\"); }");
    anyhow::ensure!(direct.len() == 1, "direct blocking call was not rejected");
    anyhow::ensure!(
        findings("fn startup() { std::fs::read_to_string(\"x\"); }").is_empty(),
        "synchronous startup was rejected"
    );
    anyhow::ensure!(
        findings(
            "async fn owned() { tokio::task::spawn_blocking(|| std::fs::read_to_string(\"x\")); }"
        )
        .is_empty(),
        "owned blocking boundary was rejected"
    );
    Ok(())
}
