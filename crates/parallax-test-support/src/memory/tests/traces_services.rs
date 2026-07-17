#[tokio::test]
async fn service_map_derives_trace_path_edges() {
    let store = MemoryStore::new();
    let mut a_client = span("trace-ab", "a-client", None, "A", 100);
    a_client.kind = "SPAN_KIND_CLIENT".into();
    let mut b_server = span("trace-ab", "b-server", Some("a-client"), "B", 101);
    b_server.kind = "SPAN_KIND_SERVER".into();
    b_server.status_code = "STATUS_CODE_ERROR".into();
    b_server.duration_ns = 20_000_000;
    let mut b_client = span("trace-bc", "b-client", None, "B", 110);
    b_client.kind = "SPAN_KIND_CLIENT".into();
    let mut c_server = span("trace-bc", "c-server", Some("b-client"), "C", 111);
    c_server.kind = "SPAN_KIND_SERVER".into();
    c_server.duration_ns = 30_000_000;
    let mut outside_client = span("trace-out", "a-client", None, "A", 1_000);
    outside_client.kind = "SPAN_KIND_CLIENT".into();
    let mut outside_server = span("trace-out", "d-server", Some("a-client"), "D", 1_001);
    outside_server.kind = "SPAN_KIND_SERVER".into();
    store.push_spans(vec![
        a_client,
        b_server,
        b_client,
        c_server,
        outside_client,
        outside_server,
    ]);

    let edges = store.service_map(0..=200, 100).await.unwrap();

    let edge_ab = edges
        .iter()
        .find(|edge| edge.source == "A" && edge.target == "B")
        .expect("A -> B edge");
    assert_eq!(edge_ab.call_count, 1);
    assert_eq!(edge_ab.error_count, 1);
    assert_eq!(edge_ab.p50_ms, 20.0);
    assert!(
        edges
            .iter()
            .any(|edge| edge.source == "B" && edge.target == "C")
    );
    assert!(!edges.iter().any(|edge| edge.target == "D"));
}

#[tokio::test]
async fn service_map_is_deterministic_and_trace_bounded() {
    let store = MemoryStore::new();
    let mut a_client = span("trace-ab", "a-client", None, "A", 100);
    a_client.kind = "SPAN_KIND_CLIENT".into();
    let mut b_server = span("trace-ab", "b-server", Some("a-client"), "B", 101);
    b_server.kind = "SPAN_KIND_SERVER".into();
    let mut b_client = span("trace-bc", "b-client", None, "B", 110);
    b_client.kind = "SPAN_KIND_CLIENT".into();
    let mut c_server = span("trace-bc", "c-server", Some("b-client"), "C", 111);
    c_server.kind = "SPAN_KIND_SERVER".into();
    store.push_spans(vec![a_client, b_server, b_client, c_server]);

    let first = store.service_map(0..=200, 100).await.unwrap();
    let second = store.service_map(0..=200, 100).await.unwrap();
    let bounded = store.service_map(0..=200, 1).await.unwrap();

    assert_eq!(first, second);
    assert_eq!(bounded.len(), 1);
    assert_eq!(bounded[0].source, "B");
    assert_eq!(bounded[0].target, "C");
}

#[tokio::test]
async fn run_anchored_reads_keep_newest_limit_in_ascending_order() {
    let store = MemoryStore::new();
    let mut spans = Vec::new();
    let mut logs = Vec::new();
    for index in 0..250u128 {
        let mut span = span(
            &format!("trace-{index}"),
            &format!("span-{index}"),
            None,
            "api",
            index,
        );
        span.invocation_id = Some("run-1".into());
        spans.push(span);
        logs.push(log(Some("run-1"), index, 9));
    }
    store.push_spans(spans);
    store.push_logs(logs);

    let spans = store
        .spans_by_invocation("run-1", 200, 0..=u128::MAX)
        .await
        .unwrap();
    let logs = store.logs_by_invocation("run-1", 200).await.unwrap();

    assert_eq!(spans.len(), 200);
    assert_eq!(logs.len(), 200);
    assert_eq!(spans.first().map(|span| span.ts_nanos), Some(50));
    assert_eq!(logs.first().map(|log| log.ts_nanos), Some(50));
    assert_eq!(spans.last().map(|span| span.ts_nanos), Some(249));
    assert_eq!(logs.last().map(|log| log.ts_nanos), Some(249));
}

#[tokio::test]
async fn log_severity_max_bounds_search_and_count_series() {
    let store = MemoryStore::new();
    store.push_logs(vec![
        log(None, 5, 5),
        log(None, 9, 9),
        log(None, 13, 13),
        log(None, 17, 17),
    ]);

    let logs = store
        .logs_search(None, 0..=100, Some(5), Some(8), None, 10)
        .await
        .unwrap();
    let series = store
        .log_count_series(None, 0..=100, Some(5), Some(8), None, 1)
        .await
        .unwrap();

    assert_eq!(
        logs.iter().map(|log| log.severity_num).collect::<Vec<_>>(),
        vec![5]
    );
    assert_eq!(series.iter().map(|point| point.value).sum::<f64>(), 1.0);
}

// A non-root span of a participating service surfaces the whole trace,
// represented by its real root (the cross-service `--service catalog` bug).
#[tokio::test]
async fn service_filter_matches_participation_not_just_root() {
    let store = MemoryStore::new();
    store.push_spans(vec![
        span("t1", "a", None, "checkout", 10),
        span("t1", "b", Some("a"), "catalog", 20),
    ]);

    let by_catalog = store.traces_search(&query(Some("catalog"))).await.unwrap();
    let by_catalog = by_catalog.items;
    assert_eq!(by_catalog.len(), 1, "catalog participates in t1");
    assert_eq!(by_catalog[0].trace_id, "t1");
    assert_eq!(
        by_catalog[0].service, "checkout",
        "summary uses the trace root, not the filtered service"
    );
    assert_eq!(by_catalog[0].span_count, 2);

    let absent = store.traces_search(&query(Some("payment"))).await.unwrap();
    assert!(absent.items.is_empty(), "payment is in no trace");
}

// A trace with no stored root (all spans parented elsewhere) still lists,
// represented by its earliest span.
#[tokio::test]
async fn rootless_trace_lists_via_earliest_span() {
    let store = MemoryStore::new();
    store.push_spans(vec![
        span("t2", "y", Some("missing-parent"), "catalog", 30),
        span("t2", "x", Some("missing-parent"), "catalog", 15),
    ]);

    let traces = store.traces_search(&query(None)).await.unwrap();
    let traces = traces.items;
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0].trace_id, "t2");
    assert_eq!(
        traces[0].start_nanos, 15,
        "earliest span represents a rootless trace"
    );
    assert_eq!(traces[0].span_count, 2);
}

#[tokio::test]
async fn traces_by_ids_preserves_requested_order_and_summarizes_targets() {
    let store = MemoryStore::new();
    let mut target_b = span("target-b", "root-b", None, "worker", 20);
    target_b.name = "consume-b".into();
    target_b.status_code = "STATUS_CODE_ERROR".into();
    let mut target_a = span("target-a", "root-a", None, "api", 10);
    target_a.name = "consume-a".into();
    store.push_spans(vec![
        target_a,
        span("target-a", "child-a", Some("root-a"), "api", 12),
        target_b,
    ]);

    let summaries = store
        .traces_by_ids(&[
            "target-b".to_string(),
            "missing".to_string(),
            "target-a".to_string(),
            "target-b".to_string(),
        ])
        .await
        .unwrap();

    assert_eq!(
        summaries
            .iter()
            .map(|summary| summary.trace_id.as_str())
            .collect::<Vec<_>>(),
        vec!["target-b", "target-a"]
    );
    assert_eq!(summaries[0].service, "worker");
    assert_eq!(summaries[0].root_name, "consume-b");
    assert!(summaries[0].has_error);
    assert_eq!(summaries[1].span_count, 2);
}

#[tokio::test]
async fn trace_search_sorts_offsets_and_filters_duration_band() {
    let store = MemoryStore::new();
    store.push_spans(vec![
        span_with_duration("fast", "a", None, "api", 10, 10),
        span_with_duration("mid", "b", None, "api", 20, 20),
        span_with_duration("slow", "c", None, "api", 30, 30),
        span_with_duration("wide", "d", None, "api", 40, 25),
        span_with_duration("wide", "e", Some("d"), "api", 45, 5),
    ]);

    let result = store
        .traces_search(&TraceQuery {
            min_duration_ns: Some(15),
            max_duration_ns: Some(30),
            sort: TraceSort::DurationDesc,
            limit: 2,
            offset: 1,
            ..TraceQuery::default()
        })
        .await
        .unwrap();
    assert_eq!(result.total, 3);
    assert_eq!(
        result
            .items
            .iter()
            .map(|t| t.trace_id.as_str())
            .collect::<Vec<_>>(),
        vec!["wide", "mid"]
    );

    let by_span_count = store
        .traces_search(&TraceQuery {
            sort: TraceSort::SpanCountDesc,
            limit: 1,
            ..TraceQuery::default()
        })
        .await
        .unwrap();
    assert_eq!(by_span_count.items[0].trace_id, "wide");
    assert_eq!(by_span_count.items[0].span_count, 2);
}

#[tokio::test]
async fn overview_totals_and_signal_series_cover_seeded_window() {
    let store = MemoryStore::new();
    let mut ok = span("t1", "a", None, "api", 1_000_000_000);
    ok.duration_ns = 1_000_000;
    let mut err = span("t1", "b", Some("a"), "api", 1_500_000_000);
    err.status_code = "STATUS_CODE_ERROR".into();
    err.duration_ns = 9_000_000;
    store.push_spans(vec![ok, err]);
    store.push_logs(vec![LogRow {
        ts_nanos: 1_250_000_000,
        event_name: "checkout.failed".into(),
        observed_ts_nanos: 1_300_000_000,
        service: "api".into(),
        severity_num: 17,
        severity_text: "ERROR".into(),
        body: "bad".into(),
        trace_id: "t1".into(),
        span_id: "b".into(),
        invocation_id: None,
        session_id: None,
        scope_name: String::new(),
        attributes: serde_json::Value::Null,
        resource: serde_json::Value::Null,
    }]);
    store
        .write_error_events(vec![error_event("api", 1_600_000_000)])
        .await
        .unwrap();

    let totals = store.overview_totals(0..=2_000_000_000).await.unwrap();
    assert_eq!(totals.span_count, 2);
    assert_eq!(totals.trace_count, 1);
    assert_eq!(totals.log_count, 1);
    assert_eq!(totals.error_count, 1);
    assert_eq!(totals.active_services, 1);
    assert_eq!(totals.error_rate, 0.5);

    let logs = store.logs_by_trace("t1").await.unwrap();
    assert_eq!(logs[0].event_name, "checkout.failed");
    assert_eq!(logs[0].observed_ts_nanos, 1_300_000_000);

    let spans = store
        .signal_count_series(
            SignalKind::Spans,
            Some("api"),
            0..=2_000_000_000,
            1_000_000_000,
        )
        .await
        .unwrap();
    assert_eq!(spans[0].value, 2.0);
    let errors = store
        .signal_count_series(
            SignalKind::Errors,
            Some("api"),
            0..=2_000_000_000,
            1_000_000_000,
        )
        .await
        .unwrap();
    assert_eq!(errors[0].value, 1.0);
}

#[tokio::test]
async fn service_summaries_and_red_use_trace_durations() {
    let store = MemoryStore::new();
    let mut fast = span("t1", "a", None, "api", 1_000_000_000);
    fast.duration_ns = 10_000_000;
    let mut slow = span("t2", "b", None, "api", 1_500_000_000);
    slow.duration_ns = 30_000_000;
    slow.status_code = "STATUS_CODE_ERROR".into();
    let mut other = span("t3", "c", None, "worker", 1_800_000_000);
    other.duration_ns = 50_000_000;
    store.push_spans(vec![fast, slow, other]);

    let summaries = store.service_summaries(0..=2_000_000_000).await.unwrap();
    assert_eq!(summaries[0].name, "worker");
    let api = summaries.iter().find(|s| s.name == "api").unwrap();
    assert_eq!(api.span_count, 2);
    assert_eq!(api.error_count, 1);
    assert_eq!(api.p95_ms, Some(29.0));

    let red = store
        .span_red_series(Some("api"), 0..=2_000_000_000, 1_000_000_000)
        .await
        .unwrap();
    assert_eq!(red.rate[0].value, 2.0);
    assert_eq!(red.error_rate[0].value, 0.5);
    assert_eq!(red.p50[0].value, 20.0);
    assert_eq!(red.p95[0].value, 29.0);
    assert_eq!(red.p99[0].value, 29.8);
}

#[tokio::test]
async fn conformance_scenarios_pass_on_memory() {
    let store = MemoryStore::new();
    crate::conformance::trace_search_scenario(&store)
        .await
        .expect("trace_search");
    crate::conformance::log_count_series_scenario(&store)
        .await
        .expect("log_count_series");
    crate::conformance::overview_totals_scenario(&store)
        .await
        .expect("overview_totals");
    crate::conformance::attribute_compare_scenario(&store)
        .await
        .expect("attribute_compare");
    crate::conformance::service_map_scenario(&store)
        .await
        .expect("service_map");
}

#[tokio::test]
async fn trace_duration_stats_ignores_duration_bounds_and_paging() {
    let store = MemoryStore::new();
    store.push_spans(vec![
        span_with_duration("t0", "a", None, "checkout", 1_000, 10),
        span_with_duration("t1", "b", None, "checkout", 1_001, 20),
        span_with_duration("t2", "c", None, "checkout", 1_002, 30),
        span_with_duration("t3", "d", None, "checkout", 1_003, 40),
        span_with_duration("t4", "e", None, "checkout", 1_004, 100),
    ]);

    let stats = store
        .trace_duration_stats(&TraceQuery {
            // Duration bounds and paging must not shape the distribution.
            min_duration_ns: Some(35),
            limit: 1,
            ..TraceQuery::default()
        })
        .await
        .unwrap();
    // Nearest-rank over [10,20,30,40,100]: p50 = 30, p95 = 100.
    assert_eq!(stats.p50_ns, Some(30.0));
    assert_eq!(stats.p95_ns, Some(100.0));

    let empty = store
        .trace_duration_stats(&TraceQuery {
            service: Some("missing".into()),
            ..TraceQuery::default()
        })
        .await
        .unwrap();
    assert_eq!(empty.p50_ns, None);
    assert_eq!(empty.p95_ns, None);
}
