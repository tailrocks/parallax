use super::*;

#[test]
fn preserves_load_bearing_wire_names() -> Result<(), String> {
    let actual = (
        SERVICE_NAME,
        EVENT_NAME,
        CLI_INVOCATION_ID,
        BUNDLE_WINDOW_METRICS,
    );
    let expected = (
        "service.name",
        "event.name",
        "cli.invocation.id",
        &[
            "process.cpu.utilization",
            "process.memory.usage",
            "tokio.runtime.alive_tasks",
        ][..],
    );
    if actual != expected {
        return Err(format!("semantic-convention wire-name drift: {actual:?}"));
    }
    Ok(())
}

#[test]
fn catalog_prom_name_matches_stored_otel_exemplar_name() {
    assert!(metric_names_match(
        "catalog_product_queries_total",
        "catalog.product.queries"
    ));
    assert!(metric_names_match(
        "http_server_request_duration",
        "http.server.request.duration"
    ));
    assert!(metric_names_match(
        "http.server.request.duration",
        "http.server.request.duration"
    ));
    assert!(!metric_names_match(
        "catalog_product_queries_total",
        "http.server.request.duration"
    ));
}

proptest::proptest! {
    #[test]
    fn native_metric_table_names_use_only_greptime_safe_characters(name in ".*") {
        let normalized = native_metric_table_base(&name);
        proptest::prop_assert!(normalized
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_'));
        proptest::prop_assert_eq!(native_metric_table_base(&normalized), normalized);
    }

    #[test]
    fn resource_json_paths_quote_the_attribute(attribute in "[^\\p{C}]*") {
        let path = resource_json_path(&attribute);
        proptest::prop_assert!(path.starts_with(r#"$.""#));
        proptest::prop_assert!(path.ends_with('"'));
        proptest::prop_assert!(!path.starts_with(r#"$.\\""#));
    }
}
