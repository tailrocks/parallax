#![expect(
    clippy::too_many_arguments,
    reason = "compact trace fixture constructor"
)]

use super::*;

fn span(id: &str, parent: Option<&str>, start: u128, duration: u128) -> SpanRow {
    SpanRow {
        ts_nanos: start,
        service: "api".into(),
        trace_id: "trace".into(),
        span_id: id.into(),
        parent_span_id: parent.map(Into::into),
        name: id.into(),
        kind: "SPAN_KIND_INTERNAL".into(),
        status_code: "STATUS_CODE_UNSET".into(),
        status_message: String::new(),
        duration_ns: duration,
        invocation_id: None,
        session_id: None,
        scope_name: String::new(),
        events: None,
        links: serde_json::Value::Null,
        attributes: serde_json::Value::Null,
        resource: serde_json::Value::Null,
    }
}

fn named(
    id: &str,
    parent: Option<&str>,
    service: &str,
    name: &str,
    start: u128,
    duration: u128,
    status: &str,
) -> SpanRow {
    let mut row = span(id, parent, start, duration);
    row.service = service.into();
    row.name = name.into();
    row.status_code = status.into();
    row
}

fn path_ids(path: &CriticalPath) -> Vec<&str> {
    path.hops
        .iter()
        .map(|hop| hop.span_id.as_str())
        .collect::<Vec<_>>()
}

#[test]
fn critical_path_linear_chain_counts_parent_self_time() {
    let spans = vec![
        span("root", None, 0, 100),
        span("child", Some("root"), 20, 50),
        span("grandchild", Some("child"), 30, 20),
    ];

    let path = critical_path(&spans);

    assert_eq!(path_ids(&path), vec!["root", "child", "grandchild"]);
    assert_eq!(path.total_gated_ns, 100);
    assert_eq!(path.hops[0].self_time_ns, 50);
    assert_eq!(path.hops[1].self_time_ns, 30);
    assert_eq!(path.hops[2].self_time_ns, 20);
}

#[test]
fn critical_path_parallel_fanout_chooses_latest_end() {
    let spans = vec![
        span("root", None, 0, 100),
        span("fast", Some("root"), 10, 20),
        span("slow", Some("root"), 12, 70),
    ];

    let path = critical_path(&spans);

    assert_eq!(path_ids(&path), vec!["root", "slow"]);
    assert_eq!(path.hops[0].gated_by_child.as_deref(), Some("slow"));
    assert_eq!(path.total_gated_ns, 100);
}

#[test]
fn critical_path_switches_across_sequential_waves() {
    let spans = vec![
        span("root", None, 0, 120),
        span("a", Some("root"), 0, 40),
        span("b", Some("root"), 5, 20),
        span("c", Some("root"), 45, 65),
    ];

    let path = critical_path(&spans);

    assert_eq!(path_ids(&path), vec!["root", "a", "c"]);
    assert_eq!(path.total_gated_ns, 120);
}

#[test]
fn critical_path_reports_unattached_roots() {
    let spans = vec![span("root-a", None, 20, 10), span("root-b", None, 10, 10)];

    let path = critical_path(&spans);

    assert_eq!(path_ids(&path), vec!["root-b"]);
    assert_eq!(path.unattached, vec!["root-a"]);
}

#[test]
fn critical_path_clamps_child_overrun_and_flags_clock_suspect() {
    let spans = vec![
        span("root", None, 10, 50),
        span("child", Some("root"), 0, 100),
    ];

    let path = critical_path(&spans);

    assert_eq!(path_ids(&path), vec!["root", "child"]);
    assert!(path.hops[0].clock_suspect);
    assert_eq!(path.total_gated_ns, 50);
}

#[test]
fn critical_path_handles_zero_duration_spans() {
    let spans = vec![span("root", None, 0, 0), span("child", Some("root"), 0, 0)];

    let path = critical_path(&spans);

    assert_eq!(path_ids(&path), vec!["root"]);
    assert_eq!(path.total_gated_ns, 0);
}

#[test]
fn compare_identical_traces_has_empty_diff() {
    let spans = vec![named(
        "a",
        None,
        "api",
        "GET /orders/123",
        0,
        10,
        "STATUS_CODE_UNSET",
    )];

    let diff = compare(&spans, &spans);

    assert!(diff.added.is_empty());
    assert!(diff.removed.is_empty());
    assert!(diff.changed.is_empty());
}

#[test]
fn compare_matches_names_with_volatile_ids() {
    let before = vec![named(
        "a",
        None,
        "api",
        "GET /orders/order-123",
        0,
        10,
        "STATUS_CODE_UNSET",
    )];
    let after = vec![named(
        "b",
        None,
        "api",
        "GET /orders/order-456",
        0,
        15,
        "STATUS_CODE_UNSET",
    )];

    let diff = compare(&before, &after);

    assert!(diff.added.is_empty());
    assert!(diff.removed.is_empty());
    assert_eq!(diff.changed.len(), 1);
    assert_eq!(diff.changed[0].duration_delta_ns, 5);
}

#[test]
fn compare_reports_added_retry_sibling() {
    let before = vec![named(
        "root",
        None,
        "api",
        "checkout",
        0,
        20,
        "STATUS_CODE_UNSET",
    )];
    let after = vec![
        named("root", None, "api", "checkout", 0, 20, "STATUS_CODE_UNSET"),
        named(
            "retry",
            Some("root"),
            "api",
            "retry",
            10,
            5,
            "STATUS_CODE_UNSET",
        ),
    ];

    let diff = compare(&before, &after);

    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.added[0].name, "retry");
}

#[test]
fn compare_keeps_same_operation_children_under_different_parents_distinct() {
    let before = vec![
        named(
            "root-a",
            None,
            "api",
            "checkout",
            0,
            100,
            "STATUS_CODE_UNSET",
        ),
        named(
            "branch-a",
            Some("root-a"),
            "api",
            "fanout",
            10,
            40,
            "STATUS_CODE_UNSET",
        ),
        named(
            "branch-b",
            Some("root-a"),
            "api",
            "fanout",
            20,
            40,
            "STATUS_CODE_UNSET",
        ),
        named(
            "select-a",
            Some("branch-a"),
            "db",
            "SELECT stock",
            12,
            10,
            "STATUS_CODE_UNSET",
        ),
        named(
            "select-b",
            Some("branch-b"),
            "db",
            "SELECT stock",
            22,
            10,
            "STATUS_CODE_UNSET",
        ),
    ];
    let after = vec![
        named(
            "root-b",
            None,
            "api",
            "checkout",
            0,
            100,
            "STATUS_CODE_UNSET",
        ),
        named(
            "branch-c",
            Some("root-b"),
            "api",
            "fanout",
            10,
            40,
            "STATUS_CODE_UNSET",
        ),
        named(
            "branch-d",
            Some("root-b"),
            "api",
            "fanout",
            20,
            40,
            "STATUS_CODE_UNSET",
        ),
        named(
            "select-d",
            Some("branch-d"),
            "db",
            "SELECT stock",
            22,
            10,
            "STATUS_CODE_UNSET",
        ),
    ];

    let diff = compare(&before, &after);

    assert_eq!(diff.removed.len(), 1);
    assert_eq!(diff.removed[0].span_id, "select-a");
    assert!(diff.added.is_empty());
}

#[test]
fn compare_reports_status_change() {
    let before = vec![named(
        "a",
        None,
        "api",
        "checkout",
        0,
        10,
        "STATUS_CODE_UNSET",
    )];
    let after = vec![named(
        "b",
        None,
        "api",
        "checkout",
        0,
        10,
        "STATUS_CODE_ERROR",
    )];

    let diff = compare(&before, &after);

    assert_eq!(diff.changed.len(), 1);
    assert!(diff.changed[0].status_changed);
}
