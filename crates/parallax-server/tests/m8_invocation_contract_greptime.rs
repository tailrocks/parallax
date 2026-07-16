//! Real-engine acceptance for the neutral CLI-invocation contract (plan 156):
//! root-span-attribute and resource-attribute emitters resolve, the legacy
//! `parallax.run.id` emitter does not, extract-keys promotes the log columns,
//! invocation-scoped metric points land in `invocation_metric_points`, and the
//! session/screen/action/cycle/job/conversation projections answer over an
//! OTLP-seeded corpus.
//!
//! Run with: `cargo nextest run -p parallax-server --test m8_invocation_contract_greptime --run-ignored only`

#![allow(clippy::expect_used, clippy::panic, reason = "test fixture assertions")]
#![expect(clippy::too_many_lines, reason = "one seeded end-to-end scenario")]

use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};
use parallax_server::Config;
use prost::Message;
use std::time::Duration;

fn kv(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueEnum::StringValue(value.to_string())),
        }),
        key_strindex: 0,
    }
}

fn kv_int(key: &str, value: i64) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueEnum::IntValue(value)),
        }),
        key_strindex: 0,
    }
}

fn now_nanos() -> u64 {
    u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
    )
    .expect("fits u64")
}

struct SpanSpec {
    id: u8,
    parent: Option<u8>,
    trace: u8,
    name: &'static str,
    kind: i32,
    offset_nanos: u64,
    attrs: Vec<KeyValue>,
}

fn spans_request(resource_attrs: Vec<KeyValue>, base: u64, specs: Vec<SpanSpec>) -> Vec<u8> {
    let spans = specs
        .into_iter()
        .map(|spec| parallax_proto::trace::Span {
            trace_id: vec![spec.trace; 16],
            span_id: vec![spec.id; 8],
            parent_span_id: spec.parent.map(|p| vec![p; 8]).unwrap_or_default(),
            name: spec.name.to_string(),
            kind: spec.kind,
            start_time_unix_nano: base + spec.offset_nanos,
            end_time_unix_nano: base + spec.offset_nanos + 5_000_000,
            attributes: spec.attrs,
            ..Default::default()
        })
        .collect();
    ExportTraceServiceRequest {
        resource_spans: vec![parallax_proto::trace::ResourceSpans {
            resource: Some(parallax_proto::resource::Resource {
                attributes: resource_attrs,
                ..Default::default()
            }),
            scope_spans: vec![parallax_proto::trace::ScopeSpans {
                spans,
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
    .encode_to_vec()
}

fn logs_request(resource_attrs: Vec<KeyValue>, records: Vec<(u64, Vec<KeyValue>)>) -> Vec<u8> {
    ExportLogsServiceRequest {
        resource_logs: vec![parallax_proto::logs::ResourceLogs {
            resource: Some(parallax_proto::resource::Resource {
                attributes: resource_attrs,
                ..Default::default()
            }),
            scope_logs: vec![parallax_proto::logs::ScopeLogs {
                log_records: records
                    .into_iter()
                    .map(|(ts, attributes)| parallax_proto::logs::LogRecord {
                        time_unix_nano: ts,
                        observed_time_unix_nano: ts,
                        severity_number: 9,
                        severity_text: "INFO".to_string(),
                        body: Some(AnyValue {
                            value: Some(AnyValueEnum::StringValue("journey".to_string())),
                        }),
                        attributes,
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
    .encode_to_vec()
}

fn metrics_request(resource_attrs: Vec<KeyValue>, ts: u64) -> Vec<u8> {
    ExportMetricsServiceRequest {
        resource_metrics: vec![parallax_proto::metrics::ResourceMetrics {
            resource: Some(parallax_proto::resource::Resource {
                attributes: resource_attrs,
                ..Default::default()
            }),
            scope_metrics: vec![parallax_proto::metrics::ScopeMetrics {
                metrics: vec![parallax_proto::metrics::Metric {
                    name: "process.cpu.utilization".to_string(),
                    data: Some(parallax_proto::metrics::metric::Data::Gauge(
                        parallax_proto::metrics::Gauge {
                            data_points: vec![parallax_proto::metrics::NumberDataPoint {
                                time_unix_nano: ts,
                                value: Some(
                                    parallax_proto::metrics::number_data_point::Value::AsDouble(
                                        0.42,
                                    ),
                                ),
                                ..Default::default()
                            }],
                        },
                    )),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
    .encode_to_vec()
}

async fn post_otlp(client: &reqwest::Client, url: &str, body: Vec<u8>) {
    let response = client
        .post(url)
        .header("content-type", "application/x-protobuf")
        .body(body)
        .send()
        .await
        .unwrap_or_else(|error| panic!("POST {url}: {error}"));
    assert!(
        response.status().is_success(),
        "POST {url}: {}",
        response.status()
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn invocation_contract_resolves_projections_on_live_engine() {
    let _subscriber_already_installed = tracing_subscriber::fmt()
        .with_env_filter("parallax_server=info")
        .try_init();
    let cache_bin = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("greptime-bin");
    let home_bin = std::env::home_dir()
        .map(|home| home.join(".parallax/bin/greptime"))
        .filter(|path| path.exists());
    let tmp = tempfile::tempdir().expect("tempdir");
    let data_bin = tmp.path().join("bin");
    let seed = home_bin.or_else(|| {
        let cached = cache_bin.join("greptime");
        cached.exists().then_some(cached)
    });
    if let Some(existing) = seed {
        std::fs::create_dir_all(&data_bin).expect("bin dir");
        std::fs::copy(&existing, data_bin.join("greptime")).expect("seed engine");
        let status = std::process::Command::new("chmod")
            .arg("+x")
            .arg(data_bin.join("greptime"))
            .status()
            .expect("chmod");
        assert!(status.success());
    }

    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.mode = "managed".to_string();
    config.storage.data_dir = tmp.path().to_string_lossy().into_owned();
    let handle = parallax_server::start(&config)
        .await
        .expect("managed server starts");
    if !cache_bin.join("greptime").exists() && data_bin.join("greptime").exists() {
        std::fs::create_dir_all(&cache_bin).expect("cache dir");
        std::fs::copy(data_bin.join("greptime"), cache_bin.join("greptime"))
            .expect("cache downloaded engine");
    }

    let client = reqwest::Client::new();
    let traces_url = format!("http://{}/v1/traces", handle.otlp_http_addr);
    let logs_url = format!("http://{}/v1/logs", handle.otlp_http_addr);
    let metrics_url = format!("http://{}/v1/metrics", handle.otlp_http_addr);
    let base = now_nanos();

    // (a) jackin shape: ids live on root spans, never on Resource.
    let jackin_corpus = spans_request(
        vec![kv("service.name", "cli-app")],
        base,
        vec![
            SpanSpec {
                id: 0x11,
                parent: None,
                trace: 0xa1,
                name: "cli.command",
                kind: 1,
                offset_nanos: 0,
                attrs: vec![
                    kv("cli.invocation.id", "inv-span"),
                    kv("cli.command.name", "workspace.env.set"),
                    kv("app.mode", "interactive"),
                    kv("outcome", "success"),
                ],
            },
            SpanSpec {
                id: 0x12,
                parent: Some(0x11),
                trace: 0xa1,
                name: "child.work",
                kind: 1,
                offset_nanos: 1_000_000,
                attrs: vec![],
            },
            SpanSpec {
                id: 0x13,
                parent: None,
                trace: 0xa2,
                name: "ui.action",
                kind: 1,
                offset_nanos: 2_000_000,
                attrs: vec![
                    kv("cli.invocation.id", "inv-span"),
                    kv("ui.action.name", "submit_form"),
                    kv("app.screen.id", "settings"),
                    kv("outcome", "success"),
                ],
            },
            SpanSpec {
                id: 0x14,
                parent: None,
                trace: 0xa3,
                name: "background.cycle",
                kind: 1,
                offset_nanos: 3_000_000,
                attrs: vec![
                    kv("cli.invocation.id", "inv-span"),
                    kv("background.cycle.name", "sync.remotes"),
                ],
            },
            SpanSpec {
                id: 0x15,
                parent: None,
                trace: 0xa4,
                name: "job.publish",
                kind: 4,
                offset_nanos: 4_000_000,
                attrs: vec![
                    kv("cli.invocation.id", "inv-span"),
                    kv("job.id", "job-1"),
                    kv("job.type", "index.rebuild"),
                ],
            },
            SpanSpec {
                id: 0x16,
                parent: None,
                trace: 0xa5,
                name: "job.consume",
                kind: 5,
                offset_nanos: 5_000_000,
                attrs: vec![
                    kv("cli.invocation.id", "inv-span"),
                    kv("job.id", "job-1"),
                    kv("outcome", "success"),
                ],
            },
            SpanSpec {
                id: 0x17,
                parent: None,
                trace: 0xa6,
                name: "chat claude",
                kind: 3,
                offset_nanos: 6_000_000,
                attrs: vec![
                    kv("cli.invocation.id", "inv-span"),
                    kv("gen_ai.conversation.id", "conv-1"),
                    kv("gen_ai.agent.name", "navigator"),
                    kv("gen_ai.provider.name", "anthropic"),
                    kv_int("gen_ai.usage.input_tokens", 120),
                    kv_int("gen_ai.usage.output_tokens", 40),
                ],
            },
        ],
    );
    post_otlp(&client, &traces_url, jackin_corpus).await;

    // (b) generic wrapped emitter: resource attribute only.
    post_otlp(
        &client,
        &traces_url,
        spans_request(
            vec![
                kv("service.name", "wrapped-tool"),
                kv("cli.invocation.id", "inv-res"),
            ],
            base,
            vec![SpanSpec {
                id: 0x21,
                parent: None,
                trace: 0xb1,
                name: "wrapped.op",
                kind: 1,
                offset_nanos: 0,
                attrs: vec![],
            }],
        ),
    )
    .await;

    // (negative) legacy emitter: parallax.run.id only — must resolve nothing.
    post_otlp(
        &client,
        &traces_url,
        spans_request(
            vec![
                kv("service.name", "legacy-tool"),
                kv("parallax.run.id", "legacy-run"),
            ],
            base,
            vec![SpanSpec {
                id: 0x31,
                parent: None,
                trace: 0xc1,
                name: "legacy.op",
                kind: 1,
                offset_nanos: 0,
                attrs: vec![kv("parallax.run.id", "legacy-run")],
            }],
        ),
    )
    .await;

    // (c) journey log events: sessions + screen visits via extract-keys.
    post_otlp(
        &client,
        &logs_url,
        logs_request(
            vec![kv("service.name", "cli-app")],
            vec![
                (
                    base + 10_000_000,
                    vec![
                        kv("cli.invocation.id", "inv-span"),
                        kv("session.id", "sess-1"),
                        kv("event.name", "session.start"),
                    ],
                ),
                (
                    base + 11_000_000,
                    vec![
                        kv("cli.invocation.id", "inv-span"),
                        kv("session.id", "sess-1"),
                        kv("event.name", "ui.screen.entered"),
                        kv("ui.screen.visit.id", "visit-1"),
                        kv("app.screen.id", "dashboard"),
                        kv_int("ui.navigation.sequence", 1),
                    ],
                ),
                (
                    base + 12_000_000,
                    vec![
                        kv("cli.invocation.id", "inv-span"),
                        kv("session.id", "sess-1"),
                        kv("event.name", "ui.screen.exited"),
                        kv("ui.screen.visit.id", "visit-1"),
                    ],
                ),
                (
                    base + 13_000_000,
                    vec![
                        kv("cli.invocation.id", "inv-span"),
                        kv("session.id", "sess-1"),
                        kv("event.name", "session.end"),
                    ],
                ),
            ],
        ),
    )
    .await;

    // (d) invocation-scoped metric points land in the extension table.
    post_otlp(
        &client,
        &metrics_url,
        metrics_request(
            vec![
                kv("service.name", "wrapped-tool"),
                kv("cli.invocation.id", "inv-res"),
            ],
            base,
        ),
    )
    .await;

    let store = &handle.store;
    // Bounded window (the production callers pass retained_recent_range();
    // an i64::MAX bound does not plan as a Timestamp(ns) literal).
    let range = u128::from(base) - 3_600_000_000_000..=u128::from(base) + 3_600_000_000_000;

    // Poll until both invocations are observed (worker + engine async).
    let mut observed = Vec::new();
    for _ in 0..200 {
        observed = store
            .observed_invocations(50, range.clone())
            .await
            .expect("observed invocations");
        let ids: Vec<&str> = observed
            .iter()
            .map(|invocation| invocation.invocation_id.as_str())
            .collect();
        if ids.contains(&"inv-span") && ids.contains(&"inv-res") {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let ids: Vec<&str> = observed
        .iter()
        .map(|invocation| invocation.invocation_id.as_str())
        .collect();
    assert!(ids.contains(&"inv-span"), "root-span emitter: {ids:?}");
    assert!(ids.contains(&"inv-res"), "resource emitter: {ids:?}");
    assert!(
        !ids.contains(&"legacy-run"),
        "legacy parallax.run.id emitter must not resolve: {ids:?}"
    );
    let jackin = observed
        .iter()
        .find(|invocation| invocation.invocation_id == "inv-span")
        .expect("jackin invocation");
    assert_eq!(jackin.last_command.as_deref(), Some("workspace.env.set"));
    assert_eq!(jackin.app_mode.as_deref(), Some("interactive"));

    // Extract-keys promoted the correlation columns on opentelemetry_logs.
    let mut promoted = 0;
    for _ in 0..200 {
        promoted = store
            .raw_sql(
                r#"SELECT COUNT(*) FROM opentelemetry_logs
                   WHERE "cli.invocation.id" = 'inv-span' AND "session.id" = 'sess-1'"#,
            )
            .await
            .map(|result| result.rows[0][0].as_i64().unwrap_or(0))
            .unwrap_or(0);
        if promoted >= 4 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(promoted, 4, "extract-keys must fill the promoted columns");

    // Invocation-scoped points landed in the extension table.
    let mut points = 0;
    for _ in 0..200 {
        let result = store
            .raw_sql(
                r#"SELECT COUNT(*) FROM invocation_metric_points
                   WHERE "invocation_id" = 'inv-res'"#,
            )
            .await
            .expect("query invocation_metric_points");
        points = result.rows[0][0].as_i64().unwrap_or(0);
        if points >= 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(points >= 1, "invocation-scoped metric point stored");

    // Whole-trace resolution: children of a root-stamped trace count too.
    let spans = store
        .spans_by_invocation("inv-span", 100, range.clone())
        .await
        .expect("spans by invocation");
    assert!(
        spans.iter().any(|span| span.name == "child.work"),
        "child spans of a root-stamped trace resolve: {:?}",
        spans.iter().map(|span| &span.name).collect::<Vec<_>>()
    );

    // Projections over the seeded corpus.
    let sessions = store
        .sessions_by_invocation("inv-span", range.clone(), 50)
        .await
        .expect("sessions");
    assert_eq!(sessions.len(), 1, "{sessions:?}");
    assert_eq!(sessions[0].session_id, "sess-1");
    assert!(sessions[0].end_nanos.is_some());

    let visits = store
        .screen_visits(Some("inv-span"), None, range.clone(), 50)
        .await
        .expect("screen visits");
    assert_eq!(visits.len(), 1, "{visits:?}");
    assert_eq!(visits[0].screen_id, "dashboard");
    assert!(visits[0].exited_nanos.is_some());

    let actions = store
        .ui_actions("inv-span", range.clone(), 50)
        .await
        .expect("ui actions");
    assert_eq!(actions.len(), 1, "{actions:?}");
    assert_eq!(actions[0].name, "submit_form");
    assert_eq!(actions[0].screen_id.as_deref(), Some("settings"));

    let cycles = store
        .background_cycles(Some("inv-span"), range.clone(), 50)
        .await
        .expect("background cycles");
    assert_eq!(cycles.len(), 1, "{cycles:?}");
    assert_eq!(cycles[0].name, "sync.remotes");
    assert_eq!(cycles[0].count, 1);

    let jobs = store
        .jobs(Some("inv-span"), range.clone(), 50)
        .await
        .expect("jobs");
    assert_eq!(jobs.len(), 1, "{jobs:?}");
    assert_eq!(jobs[0].job_id, "job-1");
    assert_eq!(jobs[0].job_type.as_deref(), Some("index.rebuild"));
    assert!(jobs[0].produced_nanos.is_some());
    assert_eq!(jobs[0].attempts.len(), 1);

    let conversations = store
        .conversations("inv-span", range.clone(), 50)
        .await
        .expect("conversations");
    assert_eq!(conversations.len(), 1, "{conversations:?}");
    assert_eq!(conversations[0].conversation_id, "conv-1");
    assert_eq!(conversations[0].agent_name.as_deref(), Some("navigator"));
    assert_eq!(conversations[0].input_tokens, Some(120.0));
    assert_eq!(conversations[0].output_tokens, Some(40.0));

    handle.shutdown();
}
