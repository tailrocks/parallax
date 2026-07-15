use super::*;

const CUSTOM_ENDPOINT: &str = "http://127.0.0.1:14317";

#[test]
fn protocol_follows_port() {
    assert_eq!(
        (
            protocol_for("http://localhost:4317"),
            protocol_for("http://localhost:4318"),
            protocol_for("http://host.docker.internal:14317"),
        ),
        (OTLP_GRPC_PROTOCOL, OTLP_HTTP_PROTOCOL, OTLP_GRPC_PROTOCOL)
    );
}

#[test]
fn flag_beats_env() {
    let fwd = resolve_forward_from(
        Some("http://localhost:4317"),
        Some("off".to_string()),
        None,
        CUSTOM_ENDPOINT,
    )
    .unwrap();
    assert_eq!(
        (fwd.endpoint.as_str(), fwd.compare),
        ("http://localhost:4317", true)
    );
}

#[test]
fn flag_off_forces_default() {
    let fwd = resolve_forward_from(
        Some("off"),
        Some("rotel".to_string()),
        None,
        CUSTOM_ENDPOINT,
    )
    .unwrap();
    assert_eq!(
        (fwd.endpoint.as_str(), fwd.compare),
        (CUSTOM_ENDPOINT, false)
    );
}

#[test]
fn rotel_alias_resolves() {
    let fwd = resolve_forward_from(None, Some("rotel".to_string()), None, CUSTOM_ENDPOINT).unwrap();
    assert_eq!(
        (fwd.endpoint.as_str(), fwd.compare),
        (DEFAULT_ROTEL_ENDPOINT, true)
    );
}

#[test]
fn explicit_url_from_env() {
    let fwd = resolve_forward_from(
        None,
        Some("http://collector:4318".to_string()),
        None,
        CUSTOM_ENDPOINT,
    )
    .unwrap();
    assert_eq!(
        (fwd.endpoint.as_str(), fwd.protocol, fwd.compare),
        ("http://collector:4318", OTLP_HTTP_PROTOCOL, true)
    );
}

#[test]
fn respects_preexisting_otel_endpoint() {
    let fwd = resolve_forward_from(
        None,
        None,
        Some("http://localhost:4317".to_string()),
        CUSTOM_ENDPOINT,
    )
    .unwrap();
    assert_eq!(
        (fwd.endpoint.as_str(), fwd.compare),
        ("http://localhost:4317", true)
    );
}

#[test]
fn default_when_nothing_set() {
    let fwd = resolve_forward_from(None, None, None, CUSTOM_ENDPOINT).unwrap();
    assert_eq!(
        (fwd.endpoint.as_str(), fwd.compare),
        (CUSTOM_ENDPOINT, false)
    );
}

#[test]
fn preexisting_parallax_endpoint_is_not_compare() {
    let fwd = resolve_forward_from(
        None,
        None,
        Some(CUSTOM_ENDPOINT.to_string()),
        CUSTOM_ENDPOINT,
    )
    .unwrap();
    assert!(!fwd.compare);
}

#[test]
fn invalid_target_errors() {
    assert!(resolve_forward_from(Some("nonsense"), None, None, CUSTOM_ENDPOINT).is_err());
}

#[test]
fn endpoint_uses_api_host_and_reported_grpc_port() {
    assert_eq!(
        endpoint_from_api_url_and_port("http://127.0.0.1:4000", 14317).unwrap(),
        CUSTOM_ENDPOINT
    );
}

#[test]
fn browser_traces_follow_rotel_http_receiver_in_compare_mode() {
    let forward =
        resolve_forward_from(None, Some("rotel".to_string()), None, CUSTOM_ENDPOINT).unwrap();
    assert_eq!(
        http_traces_endpoint(&forward, "http://127.0.0.1:14318/v1/traces"),
        "http://localhost:4318/v1/traces"
    );
}

#[test]
fn browser_traces_use_explicit_http_forward() {
    let forward =
        resolve_forward_from(Some("http://collector:4318"), None, None, CUSTOM_ENDPOINT).unwrap();
    assert_eq!(
        http_traces_endpoint(&forward, "http://127.0.0.1:14318/v1/traces"),
        "http://collector:4318/v1/traces"
    );
}

#[test]
fn generated_trace_carrier_is_w3c_shaped() {
    let carrier = generated_traceparent();
    let fields = carrier.split('-').collect::<Vec<_>>();
    assert_eq!(fields.len(), 4);
    assert_eq!(fields[0], "00");
    assert_eq!(fields[1].len(), 32);
    assert_eq!(fields[2].len(), 16);
    assert_eq!(fields[3], "01");
}

#[test]
fn render_bundle_markdown_matches_legacy_trailer() {
    let bundle = serde_json::json!({
        "markdown": "# Evidence\nline two",
        "canonicalHash": "abc123",
        "json": r#"{"schema_version":"bundle-v1","canonical_hash":"abc123"}"#,
    });
    let (stdout, stderr) = render_bundle(OutputFormat::Markdown, &bundle);
    assert_eq!(stdout, "# Evidence\nline two\n\n---\nbundle: abc123\n");
    assert!(stderr.is_empty());
}

#[test]
fn render_bundle_json_is_verbatim_without_trailer() {
    let canonical = r#"{"schema_version":"bundle-v1","canonical_hash":"abc123"}"#;
    let bundle = serde_json::json!({
        "markdown": "# Evidence",
        "canonicalHash": "abc123",
        "json": canonical,
    });
    let (stdout, stderr) = render_bundle(OutputFormat::Json, &bundle);
    assert_eq!(stdout, format!("{canonical}\n"));
    assert!(!stdout.contains("---\nbundle:"));
    assert!(stderr.is_empty());
}

#[test]
fn render_agent_session_json_is_object() {
    let session = serde_json::json!({
        "rootSpanId": "root-1",
        "totalInputTokens": "10",
        "totalOutputTokens": "20",
        "errorCount": 0,
        "truncated": false,
        "steps": []
    });
    let (stdout, stderr) = render_agent_session(OutputFormat::Json, "run-x", &session);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(parsed["rootSpanId"], "root-1");
    assert!(stderr.is_empty());
}

#[test]
fn render_agent_session_markdown_lists_steps() {
    let session = serde_json::json!({
        "rootSpanId": "root-1",
        "totalInputTokens": "10",
        "totalOutputTokens": "20",
        "errorCount": 1,
        "truncated": true,
        "steps": [{
            "spanId": "s1",
            "traceId": "t1",
            "kind": "EXECUTE_TOOL",
            "name": "search",
            "startNanos": "1",
            "durationNs": "100",
            "isError": true,
            "genAiOperation": "tool",
            "inputTokens": null,
            "outputTokens": null
        }]
    });
    let (stdout, stderr) = render_agent_session(OutputFormat::Markdown, "run-x", &session);
    assert!(stdout.contains("agent session for run run-x"));
    assert!(stdout.contains("EXECUTE_TOOL"));
    assert!(stdout.contains("search"));
    assert!(stdout.contains("ERR"));
    assert!(stdout.contains("truncated: true"));
    assert!(stderr.is_empty());
}

#[test]
fn endpoint_uses_remote_api_host_and_reported_grpc_port() {
    assert_eq!(
        endpoint_from_api_url_and_port("https://parallax.example.com:4000", 14317).unwrap(),
        "https://parallax.example.com:14317"
    );
}

#[test]
fn endpoint_brackets_ipv6_api_host() {
    assert_eq!(
        endpoint_from_api_url_and_port("http://[::1]:4000", 14317).unwrap(),
        "http://[::1]:14317"
    );
}

#[test]
fn compare_adds_lab_attrs() {
    let attrs = forward_resource_attrs("abc123", true);
    assert!(attrs.contains("parallax.run.id=abc123"));
    assert!(attrs.contains("parallax.lab=1"));
    assert!(attrs.contains("deployment.environment.name="));
}

#[test]
fn default_mode_run_id_only() {
    let attrs = forward_resource_attrs("abc123", false);
    assert_eq!(attrs, "parallax.run.id=abc123");
}
