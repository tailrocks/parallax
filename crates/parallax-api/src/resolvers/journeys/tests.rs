use crate::resolvers::test_support::*;
use crate::{build_schema, execute};
use parallax_test_support::builders::MemoryStore;
use std::sync::Arc;

fn now_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn journey_log(
    ts: u128,
    event: &str,
    session: &str,
    attributes: serde_json::Value,
) -> parallax_storage::model::LogRow {
    let mut row = log_row("cli", "", ts, "");
    row.invocation_id = Some("inv-1".to_string());
    row.session_id = Some(session.to_string());
    row.event_name = event.to_string();
    row.attributes = attributes;
    row
}

fn journey_span(
    ts: u128,
    name: &str,
    kind: &str,
    attributes: serde_json::Value,
) -> parallax_storage::model::SpanRow {
    let mut row = span(
        "cli",
        &format!("trace-{ts}"),
        &format!("span-{ts}"),
        ts,
        1_000,
    );
    row.invocation_id = Some("inv-1".to_string());
    row.name = name.to_string();
    row.kind = kind.to_string();
    row.attributes = attributes;
    row
}

async fn journey_query(store: Arc<MemoryStore>, query: &str) -> serde_json::Value {
    let context = context_with_memory(store).await;
    let schema = build_schema();
    let request = juniper::http::GraphQLRequest::new(query.to_string(), None, None);
    serde_json::to_value(execute(&schema, &context, request).await).unwrap()
}

#[tokio::test]
async fn sessions_resolver_pairs_start_and_end_events() {
    let store = Arc::new(MemoryStore::new());
    let base = now_nanos();
    store.push_logs(vec![
        journey_log(base, "session.start", "s1", serde_json::json!({})),
        journey_log(base + 10, "session.end", "s1", serde_json::json!({})),
        journey_log(
            base + 20,
            "session.start",
            "s2",
            serde_json::json!({"session.previous_id": "s1"}),
        ),
    ]);
    let json = journey_query(
        store,
        r#"{ sessions(invocationId: "inv-1") { sessionId previousSessionId endNanos } }"#,
    )
    .await;
    assert!(error_messages(&json).is_empty(), "{json}");
    let sessions = json.pointer("/data/sessions").unwrap().as_array().unwrap();
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0]["sessionId"], "s1");
    assert!(sessions[0]["endNanos"].is_string());
    assert_eq!(sessions[1]["previousSessionId"], "s1");
    assert!(sessions[1]["endNanos"].is_null());
}

#[tokio::test]
async fn screen_visits_resolver_requires_scope_and_pairs_visits() {
    let store = Arc::new(MemoryStore::new());
    let base = now_nanos();
    store.push_logs(vec![
        journey_log(
            base,
            "ui.screen.entered",
            "s1",
            serde_json::json!({
                "ui.screen.visit.id": "v1", "app.screen.id": "home",
                "ui.navigation.sequence": 1
            }),
        ),
        journey_log(
            base + 10,
            "ui.screen.exited",
            "s1",
            serde_json::json!({"ui.screen.visit.id": "v1"}),
        ),
    ]);
    let missing_scope = journey_query(store.clone(), r"{ screenVisits { visitId } }").await;
    assert!(!error_messages(&missing_scope).is_empty());
    let json = journey_query(
        store,
        r#"{ screenVisits(invocationId: "inv-1") { screenId visitId exitedNanos } }"#,
    )
    .await;
    assert!(error_messages(&json).is_empty(), "{json}");
    let visits = json
        .pointer("/data/screenVisits")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(visits.len(), 1);
    assert_eq!(visits[0]["screenId"], "home");
    assert!(visits[0]["exitedNanos"].is_string());
}

#[tokio::test]
async fn ui_actions_background_cycles_jobs_and_conversations_resolve() {
    let store = Arc::new(MemoryStore::new());
    let base = now_nanos();
    store.push_spans(vec![
        journey_span(
            base,
            "ui.action",
            "SPAN_KIND_INTERNAL",
            serde_json::json!({"ui.action.name": "submit", "outcome": "success"}),
        ),
        journey_span(
            base + 10,
            "background.cycle",
            "SPAN_KIND_INTERNAL",
            serde_json::json!({"background.cycle.name": "sync"}),
        ),
        journey_span(
            base + 20,
            "job.publish",
            "SPAN_KIND_PRODUCER",
            serde_json::json!({"job.id": "j1", "job.type": "index.rebuild"}),
        ),
        journey_span(
            base + 30,
            "job.consume",
            "SPAN_KIND_CONSUMER",
            serde_json::json!({"job.id": "j1", "outcome": "success"}),
        ),
        journey_span(
            base + 40,
            "chat claude",
            "SPAN_KIND_CLIENT",
            serde_json::json!({
                "gen_ai.conversation.id": "c1", "gen_ai.agent.name": "navigator",
                "gen_ai.provider.name": "anthropic",
                "gen_ai.usage.input_tokens": 12, "gen_ai.usage.output_tokens": 3
            }),
        ),
    ]);
    let from = base.saturating_sub(1_000_000_000).to_string();
    let to = (base + 60_000_000_000).to_string();
    let json = journey_query(
        store,
        &format!(
            r#"{{
              uiActions(invocationId: "inv-1") {{ name outcome hasError }}
              backgroundCycles(invocationId: "inv-1", fromNanos: "{from}", toNanos: "{to}") {{
                name count errorCount lastTraceId
              }}
              jobs(invocationId: "inv-1", fromNanos: "{from}", toNanos: "{to}") {{
                jobId jobType producedNanos attempts {{ outcome }}
              }}
              conversations(invocationId: "inv-1") {{
                conversationId agentName providerName spanCount inputTokens outputTokens
              }}
            }}"#
        ),
    )
    .await;
    assert!(error_messages(&json).is_empty(), "{json}");
    assert_eq!(json.pointer("/data/uiActions/0/name").unwrap(), "submit");
    assert_eq!(
        json.pointer("/data/backgroundCycles/0/name").unwrap(),
        "sync"
    );
    assert_eq!(json.pointer("/data/jobs/0/jobId").unwrap(), "j1");
    assert_eq!(
        json.pointer("/data/jobs/0/attempts/0/outcome").unwrap(),
        "success"
    );
    let conversation = json.pointer("/data/conversations/0").unwrap();
    assert_eq!(conversation["agentName"], "navigator");
    assert_eq!(conversation["spanCount"], 1);
    assert_eq!(conversation["inputTokens"], 12.0);
    assert_eq!(conversation["outputTokens"], 3.0);
}
