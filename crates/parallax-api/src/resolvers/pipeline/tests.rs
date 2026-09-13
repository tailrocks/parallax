//! Pipeline resolver unit tests (filter semantics; live counters are proven
//! by the server integration test over real HTTP + GraphQL).

use super::{IngestDrop, IngestQueue, PipelineSnapshot, SamplingPolicy};
use crate::resolvers::test_support::context_with_memory;
use parallax_test_support::builders::MemoryStore;
use std::sync::Arc;

#[derive(Debug)]
struct StubPipeline;

impl PipelineSnapshot for StubPipeline {
    fn sampling_policies(&self) -> Vec<SamplingPolicy> {
        vec![
            SamplingPolicy {
                signal: "traces".to_string(),
                service: None,
                rule: "head".to_string(),
                rate: 1.0,
                enforced_by: "server-ingest".to_string(),
                description: "keep-all".to_string(),
            },
            SamplingPolicy {
                signal: "logs".to_string(),
                service: None,
                rule: "head".to_string(),
                rate: 0.5,
                enforced_by: "server-ingest".to_string(),
                description: "declared".to_string(),
            },
        ]
    }

    fn ingest_drops(&self) -> Vec<IngestDrop> {
        vec![
            IngestDrop {
                signal: Some("traces".to_string()),
                reason: "ingress_reject".to_string(),
                count: 3,
                detail: "malformed".to_string(),
            },
            IngestDrop {
                signal: None,
                reason: "live_tail_lag".to_string(),
                count: 7,
                detail: "global".to_string(),
            },
        ]
    }

    fn ingest_queues(&self) -> Vec<IngestQueue> {
        vec![IngestQueue {
            signal: "traces".to_string(),
            depth: 1,
            capacity: 8,
            high_water: 2,
            accepted: 9,
        }]
    }
}

async fn stubbed_context() -> crate::ApiContext {
    let mut context = context_with_memory(Arc::new(MemoryStore::new())).await;
    context.pipeline = Some(Arc::new(StubPipeline));
    context
}

#[tokio::test]
async fn sampling_policy_lists_and_filters_by_signal() {
    let context = stubbed_context().await;
    let all = super::sampling_policy(&context, None, None).await.unwrap();
    assert_eq!(all.len(), 2);
    let traces = super::sampling_policy(&context, None, Some("traces".to_string()))
        .await
        .unwrap();
    assert_eq!(traces.len(), 1);
    assert!((traces[0].rate - 1.0).abs() < f64::EPSILON);
    // Global rows apply to any requested service.
    let scoped = super::sampling_policy(
        &context,
        Some("checkout".to_string()),
        Some("logs".to_string()),
    )
    .await
    .unwrap();
    assert_eq!(scoped.len(), 1);
    assert!((scoped[0].rate - 0.5).abs() < f64::EPSILON);
}

#[tokio::test]
async fn unknown_signal_is_rejected() {
    let context = stubbed_context().await;
    let error = super::sampling_policy(&context, None, Some("nope".to_string()))
        .await
        .unwrap_err();
    assert!(error.message().contains("unknown signal"));
    let error = super::ingest_drops(&context, Some("nope".to_string()))
        .await
        .unwrap_err();
    assert!(error.message().contains("unknown signal"));
}

#[tokio::test]
async fn drops_keep_global_reasons_under_signal_filter() {
    let context = stubbed_context().await;
    let all = super::ingest_drops(&context, None).await.unwrap();
    assert_eq!(all.len(), 2);
    let traces = super::ingest_drops(&context, Some("traces".to_string()))
        .await
        .unwrap();
    assert_eq!(traces.len(), 2);
    let logs = super::ingest_drops(&context, Some("logs".to_string()))
        .await
        .unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].reason, "live_tail_lag");
}

#[tokio::test]
async fn queues_pass_through() {
    let context = stubbed_context().await;
    let queues = super::ingest_queues(&context).await.unwrap();
    assert_eq!(queues.len(), 1);
    assert_eq!(queues[0].accepted, 9);
}

#[tokio::test]
async fn missing_pipeline_reads_back_empty() {
    let context = context_with_memory(Arc::new(MemoryStore::new())).await;
    assert!(context.pipeline.is_none());
    assert!(
        super::sampling_policy(&context, None, None)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        super::ingest_drops(&context, None)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(super::ingest_queues(&context).await.unwrap().is_empty());
}
