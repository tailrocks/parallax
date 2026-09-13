use super::*;

type IssueEvents = HashMap<(String, String), Vec<model::ErrorEventRow>>;
type IssueEventCache = HashMap<IssueEventQuery, Arc<IssueEvents>>;

/// Request-scoped memo for the highest-fan-in anchored reads. Built fresh on
/// every GraphQL request so sibling fields share one store round-trip per
/// (`trace_id`) without caching across requests.
#[derive(Debug, Default)]
pub struct RequestMemo {
    spans: tokio::sync::Mutex<HashMap<String, Arc<Vec<model::SpanRow>>>>,
    logs: tokio::sync::Mutex<HashMap<String, Arc<Vec<model::LogRow>>>>,
    issue_events: tokio::sync::Mutex<IssueEventCache>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IssueEventQuery {
    issue_keys: Vec<(String, String)>,
    from_nanos: u128,
    to_nanos: u128,
    limit: usize,
    environment: Option<String>,
}

impl ApiContext {
    #[expect(
        clippy::too_many_arguments,
        reason = "identity and window fields keep the memo key explicit at callsites"
    )]
    pub(crate) async fn issue_events_for(
        &self,
        issue_keys: &[(String, String)],
        service: &str,
        fingerprint: &str,
        from_nanos: u128,
        to_nanos: u128,
        limit: usize,
        environment: Option<&str>,
    ) -> FieldResult<Vec<model::ErrorEventRow>> {
        let key = IssueEventQuery {
            issue_keys: issue_keys.to_vec(),
            from_nanos,
            to_nanos,
            limit,
            environment: environment.map(str::to_string),
        };
        let issue_key = (service.to_string(), fingerprint.to_string());
        let mut cache = self.memo.issue_events.lock().await;
        if let Some(events) = cache.get(&key) {
            return Ok(events.get(&issue_key).cloned().unwrap_or_default());
        }
        let events = Arc::new(
            self.store
                .error_events_by_fingerprints(issue_keys, from_nanos..=to_nanos, limit, environment)
                .await
                .map_err(internal_field_err)?,
        );
        let result = events.get(&issue_key).cloned().unwrap_or_default();
        cache.insert(key, events);
        Ok(result)
    }

    pub async fn spans_for(&self, trace_id: &str) -> FieldResult<Arc<Vec<model::SpanRow>>> {
        {
            let cache = self.memo.spans.lock().await;
            if let Some(rows) = cache.get(trace_id) {
                return Ok(Arc::clone(rows));
            }
        }
        let mut rows = self
            .store
            .spans_by_trace(trace_id)
            .await
            .map_err(internal_field_err)?;
        if rows.len() > TRACE_SPANS_MAX {
            tracing::warn!(
                trace_id,
                fetched = rows.len(),
                cap = TRACE_SPANS_MAX,
                "anchored spans truncated to TRACE_SPANS_MAX"
            );
            rows.truncate(TRACE_SPANS_MAX);
        }
        let rows = Arc::new(rows);
        let mut cache = self.memo.spans.lock().await;
        Ok(Arc::clone(
            cache
                .entry(trace_id.to_string())
                .or_insert_with(|| Arc::clone(&rows)),
        ))
    }

    pub async fn logs_for(&self, trace_id: &str) -> FieldResult<Arc<Vec<model::LogRow>>> {
        {
            let cache = self.memo.logs.lock().await;
            if let Some(rows) = cache.get(trace_id) {
                return Ok(Arc::clone(rows));
            }
        }
        let mut rows = self
            .store
            .logs_by_trace(trace_id)
            .await
            .map_err(internal_field_err)?;
        if rows.len() > TRACE_SPANS_MAX {
            tracing::warn!(
                trace_id,
                fetched = rows.len(),
                cap = TRACE_SPANS_MAX,
                "anchored logs truncated to TRACE_SPANS_MAX"
            );
            rows.truncate(TRACE_SPANS_MAX);
        }
        let rows = Arc::new(rows);
        let mut cache = self.memo.logs.lock().await;
        Ok(Arc::clone(
            cache
                .entry(trace_id.to_string())
                .or_insert_with(|| Arc::clone(&rows)),
        ))
    }
}
