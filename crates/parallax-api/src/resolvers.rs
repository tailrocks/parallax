//! Domain-split GraphQL resolvers and wrapper types.

pub(crate) mod alerts;
pub(crate) mod common;
pub(crate) mod dashboards;
pub(crate) mod fields;
pub(crate) mod helpers;
pub(crate) mod investigations;
pub(crate) mod invocations;
#[expect(clippy::too_many_lines, reason = "issue assembly")]
pub(crate) mod issues;
pub(crate) mod journeys;
pub(crate) mod logs;
pub(crate) mod metrics;
#[expect(
    clippy::cast_precision_loss,
    clippy::excessive_nesting,
    reason = "telemetry analytics"
)]
pub(crate) mod services;
pub(crate) mod sql;
pub(crate) mod story;
pub(crate) mod traces;

#[cfg(test)]
pub(crate) mod test_support;

pub(crate) use alerts::AlertCheck;
pub(crate) use alerts::AlertDestination;
pub(crate) use alerts::AlertIncident;
pub(crate) use alerts::AlertRule;
pub(crate) use alerts::AlertRuleInput;
pub(crate) use alerts::AlertRuleState;
pub(crate) use common::Point;
pub(crate) use dashboards::Dashboard;
pub(crate) use fields::AttributeCompareRow;
pub(crate) use fields::EvidenceGap;
pub(crate) use fields::FieldKey;
pub(crate) use fields::FieldStats;
pub(crate) use investigations::Investigation;
pub(crate) use investigations::SavedView;
pub(crate) use invocations::Invocation;
pub(crate) use invocations::ObservedInvocation;
pub(crate) use issues::BundleOut;
pub(crate) use issues::Issue;
pub(crate) use issues::IssueList;
pub(crate) use issues::IssueSort;
pub(crate) use issues::TrendPoint;
pub(crate) use journeys::BackgroundCycleOut;
pub(crate) use journeys::ConversationOut;
pub(crate) use journeys::JobOut;
pub(crate) use journeys::ScreenVisitOut;
pub(crate) use journeys::SessionOut;
pub(crate) use journeys::UiActionOut;
pub(crate) use logs::LogRecord;
pub(crate) use metrics::MetricCatalogRow;
pub(crate) use metrics::MetricExemplar;
pub(crate) use metrics::RuntimeMetric;
pub(crate) use metrics::Series;
pub(crate) use services::Overview;
pub(crate) use services::ReleaseWindow;
pub(crate) use services::ServiceCatalogRow;
pub(crate) use services::ServiceMap;
pub(crate) use services::ServiceOverview;
pub(crate) use services::ServiceSummary;
pub(crate) use services::SignalKind;
pub(crate) use services::SpanRed;
pub(crate) use sql::SqlResultOut;
pub(crate) use story::AgentSessionOut;
pub(crate) use story::StoryBeat;
pub(crate) use traces::AttributeFilterInput;
pub(crate) use traces::CriticalPath;
pub(crate) use traces::DurationStats;
pub(crate) use traces::Facet;
pub(crate) use traces::Trace;
pub(crate) use traces::TraceDiff;
pub(crate) use traces::TraceEventsOut;
pub(crate) use traces::TraceList;
pub(crate) use traces::TraceSort;
pub(crate) use traces::TraceSummary;
