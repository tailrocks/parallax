pub(crate) const ALERT_SIGNAL_TYPES: [&str; 6] = [
    "error_rate",
    "p95_latency",
    "p99_latency",
    "throughput",
    "log_count",
    "metric",
];
pub(crate) const ALERT_COMPARATORS: [&str; 6] =
    ["gt", "gte", "lt", "lte", "between", "not_between"];
pub(crate) const ALERT_SEVERITIES: [&str; 2] = ["warning", "critical"];
pub(crate) const ALERT_NO_DATA_BEHAVIORS: [&str; 2] = ["skip", "zero"];
pub(crate) const ALERT_DESTINATION_KINDS: [&str; 2] = ["webhook", "slack_webhook"];
pub(crate) const ALERT_NAME_MAX: usize = 120;
pub(crate) const ALERT_INCIDENTS_DEFAULT_LIMIT: usize = 100;
pub(crate) const ALERT_CHECKS_DEFAULT_LIMIT: usize = 100;
