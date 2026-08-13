//! Resolver tests for `alertRulePreview` (plan 171).

use crate::resolvers::test_support::context_with_memory;
use crate::{build_schema, execute};

use parallax_test_support::builders::MemoryStore;
use std::sync::Arc;

// Re-export helpers if tests module doesn't pub them.
// context_with_memory / error_messages / run live in tests.rs as private.
// Duplicate the tiny run helper to avoid coupling.

async fn exec(
    schema: &crate::Schema,
    context: &crate::ApiContext,
    query: impl Into<String>,
) -> serde_json::Value {
    let request = juniper::http::GraphQLRequest::new(query.into(), None, None);
    serde_json::to_value(execute(schema, context, request).await).unwrap()
}

fn errors(json: &serde_json::Value) -> Vec<String> {
    json.pointer("/errors")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|error| error.get("message").and_then(|message| message.as_str()))
        .map(str::to_string)
        .collect()
}

struct StubPreviewer;

impl crate::AlertPreviewer for StubPreviewer {
    fn preview(
        &self,
        _rule: parallax_metadata::AlertRuleRecord,
        window_minutes: u32,
        _now_nanos: u128,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<crate::AlertPreviewData>> + Send + '_>,
    > {
        Box::pin(async move {
            Ok(crate::AlertPreviewData {
                window_minutes,
                groups: vec![crate::AlertPreviewGroupData {
                    group_key: "checkout".into(),
                    samples_sufficient: true,
                    points: vec![crate::AlertPreviewPointData {
                        ts_nanos: "1".into(),
                        value: Some(0.4),
                        sample_count: 8,
                        would_fire: true,
                    }],
                }],
            })
        })
    }
}

const RULE_PREVIEW: &str = r#"
    {
      alertRulePreview(input: {
        name: "High error rate",
        signalType: "error_rate",
        services: ["checkout"],
        comparator: "gt",
        threshold: 0.2,
        windowMinutes: 5,
        severity: "critical"
      }) {
        windowMinutes
        groups { groupKey samplesSufficient points { value wouldFire sampleCount } }
      }
    }
    "#;

#[tokio::test]
async fn alert_rule_preview_does_not_persist() {
    let schema = build_schema();
    let mut context = context_with_memory(Arc::new(MemoryStore::new())).await;
    context.alert_previewer = Some(Arc::new(StubPreviewer));
    let json = exec(&schema, &context, RULE_PREVIEW).await;
    assert!(errors(&json).is_empty(), "preview: {json}");
    assert_eq!(
        json.pointer("/data/alertRulePreview/groups/0/groupKey"),
        Some(&serde_json::json!("checkout"))
    );
    assert_eq!(
        json.pointer("/data/alertRulePreview/groups/0/points/0/wouldFire"),
        Some(&serde_json::json!(true))
    );
    let json = exec(&schema, &context, r#"{ alertRules { id } }"#).await;
    assert_eq!(
        json.pointer("/data/alertRules"),
        Some(&serde_json::json!([])),
        "preview must not persist a rule"
    );
}

#[tokio::test]
async fn alert_rule_preview_unavailable_without_previewer() {
    let schema = build_schema();
    let context = context_with_memory(Arc::new(MemoryStore::new())).await;
    let json = exec(&schema, &context, RULE_PREVIEW).await;
    assert!(
        errors(&json)
            .iter()
            .any(|message| message.contains("not available")),
        "expected preview unavailable: {json}"
    );
}
