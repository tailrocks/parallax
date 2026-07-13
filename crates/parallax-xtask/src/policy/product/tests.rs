use super::*;

#[test]
fn ast_visitors_detect_clone_memory_and_ingest_logging() {
    let syntax = syn::parse_file(
        "fn ingest_logs(x: T) { let _ = x.clone(); tracing::warn!(\"x\"); MemoryStore::new(); }",
    )
    .expect("fixture parse");
    let mut clones = CloneVisitor::default();
    clones.visit_file(&syntax);
    assert_eq!(clones.count, 1);
    let mut ids = IdentifierVisitor::default();
    ids.visit_file(&syntax);
    assert!(ids.identifiers.contains("MemoryStore"));
    let mut logs = IngestLogVisitor::default();
    logs.visit_file(&syntax);
    assert_eq!(logs.violations, ["ingest_logs"]);
}

#[test]
fn bun_policy_has_positive_and_negative_fixtures() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let root = directory.path();
    fs::create_dir_all(root.join("ui")).expect("fixture directory");
    fs::write(
        root.join("ui/package.json"),
        r#"{"scripts":{"build":"bunx --bun --no-install vite build","lint":"bun run lint:native","typecheck":"bun ./node_modules/typescript/bin/tsc --noEmit"}}"#,
    )
    .expect("package fixture");
    fs::write(
        root.join("ui/bunfig.toml"),
        "[run]\nbun = true\n[install]\nauto = \"disable\"\n",
    )
    .expect("bunfig fixture");
    let mut findings = Vec::new();
    check_bun(root, &mut findings).expect("positive fixture check");
    assert!(findings.is_empty());
    fs::write(
        root.join("ui/package.json"),
        r#"{"scripts":{"build":"node ./node_modules/vite/bin/vite.js build"}}"#,
    )
    .expect("runtime-negative package fixture");
    check_bun(root, &mut findings).expect("runtime-negative fixture check");
    let rejects_node = findings
        .iter()
        .any(|finding| finding.rule_id == "product.bun");
    findings.clear();
    fs::write(
        root.join("ui/package.json"),
        r#"{"scripts":{"build":"bunx --bun --no-install vite build"}}"#,
    )
    .expect("restore positive package fixture");
    fs::write(root.join("ui/package-lock.json"), "{}").expect("negative fixture");
    check_bun(root, &mut findings).expect("negative fixture check");
    assert!(
        rejects_node
            && findings
                .iter()
                .any(|finding| finding.rule_id == "product.bun")
    );
}
