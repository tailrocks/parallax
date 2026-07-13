use super::ci_partitions;

#[test]
fn ci_inventory_has_no_empty_or_placeholder_partition() {
    assert_eq!(
        ci_partitions(false),
        ["lint", "policy", "facade", "docs-links", "ui"]
    );
    assert_eq!(
        ci_partitions(true),
        [
            "lint",
            "policy",
            "facade",
            "docs-links",
            "ui",
            "test",
            "integration",
            "dependencies"
        ]
    );
    assert!(
        ci_partitions(true)
            .iter()
            .all(|partition| !matches!(*partition, "todo" | "placeholder"))
    );
}
