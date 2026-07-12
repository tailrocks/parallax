use super::*;

pub(crate) struct Issue {
    row: model::Issue,
    cohort: Arc<Vec<String>>,
}

impl Issue {
    pub(crate) fn single(row: model::Issue) -> Self {
        Self {
            cohort: Arc::new(vec![row.fingerprint.clone()]),
            row,
        }
    }

    pub(crate) fn from_rows(rows: Vec<model::Issue>) -> Vec<Self> {
        let cohort = Arc::new(rows.iter().map(|issue| issue.fingerprint.clone()).collect());
        rows.into_iter()
            .map(|row| Self {
                row,
                cohort: Arc::clone(&cohort),
            })
            .collect()
    }
}

#[graphql_object(context = ApiContext)]
impl Issue {
    fn fingerprint(&self) -> &str {
        &self.row.fingerprint
    }
    fn title(&self) -> &str {
        &self.row.title
    }
    fn error_type(&self) -> &str {
        &self.row.error_type
    }
    fn culprit(&self) -> Option<&str> {
        self.row.culprit.as_deref()
    }
    fn service(&self) -> &str {
        &self.row.service
    }
    fn status(&self) -> &str {
        &self.row.status
    }
    fn first_seen_nanos(&self) -> String {
        nanos_string(self.row.first_seen_nanos)
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.row.last_seen_nanos)
    }
    fn event_count(&self) -> i32 {
        saturate_i32(self.row.event_count)
    }
    fn last_trace_id(&self) -> Option<&str> {
        self.row.last_trace_id.as_deref()
    }
    /// Bounded top-tag-values cache as JSON: `{key: {value: count}}`.
    fn tags(&self) -> &str {
        &self.row.tags
    }

    /// The last-24h occurrence sparkline (hourly buckets), oldest first.
    async fn trend(&self, context: &ApiContext) -> FieldResult<Vec<TrendPoint>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(field_err)?
            .as_nanos();
        let since = now.saturating_sub(24 * 3_600_000_000_000);
        let points = context
            .metadata
            .issue_trend(&self.row.fingerprint, since, 3600)
            .await
            .map_err(field_err)?;
        Ok(points.into_iter().map(TrendPoint).collect())
    }

    /// The most recent stored occurrence.
    async fn latest_event(&self, context: &ApiContext) -> FieldResult<Option<ErrorEvent>> {
        let events = context
            .issue_events_for(&self.cohort, &self.row.fingerprint, 0, u128::MAX, 1)
            .await?;
        Ok(events.into_iter().next().map(ErrorEvent))
    }

    /// Recent occurrences of this issue, newest first, optionally
    /// range-bounded (`fromNanos`/`toNanos`).
    async fn events(
        &self,
        context: &ApiContext,
        limit: Option<i32>,
        from_nanos: Option<String>,
        to_nanos: Option<String>,
    ) -> FieldResult<Vec<ErrorEvent>> {
        let from = match from_nanos {
            Some(s) => s.parse().map_err(|_| field_err("invalid fromNanos"))?,
            None => 0,
        };
        let to = match to_nanos {
            Some(s) => s.parse().map_err(|_| field_err("invalid toNanos"))?,
            None => u128::MAX,
        };
        let events = context
            .issue_events_for(
                &self.cohort,
                &self.row.fingerprint,
                from,
                to,
                clamp_limit(limit, 50),
            )
            .await?;
        Ok(events.into_iter().map(ErrorEvent).collect())
    }
}

/// Page of issues plus the (scan-capped) total for pagination.
pub(crate) struct IssueList {
    items: Vec<model::Issue>,
    total: usize,
}

impl IssueList {
    pub(super) fn new(items: Vec<model::Issue>, total: usize) -> Self {
        Self { items, total }
    }
}

#[graphql_object(context = ApiContext)]
impl IssueList {
    fn items(&self) -> Vec<Issue> {
        Issue::from_rows(self.items.clone())
    }
    /// Matching issues before paging — exact up to the 1000-row scan window.
    fn total(&self) -> i32 {
        i32::try_from(self.total).unwrap_or(i32::MAX)
    }
}

/// How `issues` lists are ordered. TREND = last-24h occurrence sum.
#[derive(juniper::GraphQLEnum, Clone, Copy)]
pub(crate) enum IssueSort {
    LastSeen,
    FirstSeen,
    Events,
    Trend,
}

impl IssueSort {
    pub(super) fn key(self) -> model::IssueSortKey {
        match self {
            Self::LastSeen => model::IssueSortKey::LastSeen,
            Self::FirstSeen => model::IssueSortKey::FirstSeen,
            Self::Events => model::IssueSortKey::Events,
            Self::Trend => model::IssueSortKey::Trend,
        }
    }
}

pub(crate) struct ErrorEvent(pub(crate) model::ErrorEventRow);

#[graphql_object(context = ApiContext)]
impl ErrorEvent {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn fingerprint(&self) -> &str {
        &self.0.fingerprint
    }
    fn error_type(&self) -> &str {
        &self.0.error_type
    }
    fn message(&self) -> &str {
        &self.0.message
    }
    fn stacktrace(&self) -> Option<&str> {
        self.0.stacktrace.as_deref()
    }
    fn source(&self) -> String {
        serde_json::to_string(&self.0.source)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string()
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn attributes(&self) -> String {
        self.0.attributes.to_string()
    }
}

pub(crate) struct TrendPoint(pub(crate) model::TrendPoint);

#[graphql_object(context = ApiContext)]
impl TrendPoint {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn count(&self) -> i32 {
        i32::try_from(self.0.count).unwrap_or(i32::MAX)
    }
}
