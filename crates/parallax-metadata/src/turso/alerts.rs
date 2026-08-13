use super::*;

/// A stored alert rule (plan 167). List-valued scoping fields
/// (`services`, `exclude_services`, `attribute_filters`, `destination_ids`)
/// are JSON-encoded strings — the metadata layer stores them opaquely and the
/// evaluator/API layers own their shape.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertRuleRecord {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    /// `error_rate|p95_latency|p99_latency|throughput|log_count|metric`.
    pub signal_type: String,
    /// JSON array of service names ("[]" = all services).
    pub services: String,
    /// JSON array of excluded service names.
    pub exclude_services: String,
    /// JSON array of attribute filters (shape owned by the API layer).
    pub attribute_filters: String,
    /// Optional group-by dimension (`service`).
    pub group_by: Option<String>,
    /// `gt|gte|lt|lte|between|not_between`.
    pub comparator: String,
    pub threshold: f64,
    pub threshold_upper: Option<f64>,
    pub window_minutes: u32,
    pub minimum_sample_count: u64,
    pub consecutive_breaches_required: u32,
    pub consecutive_healthy_required: u32,
    /// `skip|zero`.
    pub no_data_behavior: String,
    /// `warning|critical`.
    pub severity: String,
    pub renotify_interval_minutes: u32,
    /// JSON array of destination ids.
    pub destination_ids: String,
    pub metric_name: Option<String>,
    pub metric_aggregation: Option<String>,
    pub created_at_nanos: u128,
    pub updated_at_nanos: u128,
}

/// Rolling evaluation state per (rule, group).
#[derive(Debug, Clone, PartialEq)]
pub struct AlertRuleStateRecord {
    pub rule_id: String,
    pub group_key: String,
    pub consecutive_breaches: u32,
    pub consecutive_healthy: u32,
    pub incident_open: bool,
    pub last_notified_at_nanos: Option<u128>,
    /// `breach|healthy|no_data|error`.
    pub last_status: Option<String>,
    pub last_value: Option<f64>,
    pub last_sample_count: u64,
    pub last_evaluated_at_nanos: Option<u128>,
    pub last_error: Option<String>,
}

/// An incident opened by the evaluator.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertIncidentRecord {
    pub id: String,
    pub rule_id: String,
    pub group_key: String,
    /// `open|resolved`.
    pub status: String,
    pub severity: String,
    pub first_triggered_at_nanos: u128,
    pub last_triggered_at_nanos: u128,
    pub resolved_at_nanos: Option<u128>,
    pub last_value: Option<f64>,
    pub last_notified_at_nanos: Option<u128>,
}

/// A notification destination. `config` is JSON (URL / address); V1 stores it
/// plaintext-at-rest per the plan's single-operator scope.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertDestinationRecord {
    pub id: String,
    pub name: String,
    /// `webhook|slack_webhook|email`.
    pub kind: String,
    pub config: String,
    pub created_at_nanos: u128,
    pub updated_at_nanos: u128,
}

/// One outbox row for the delivery worker.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertDeliveryEventRecord {
    pub id: String,
    pub incident_id: String,
    pub destination_id: String,
    /// `triggered|resolved|renotify`.
    pub event_type: String,
    /// `pending|delivered|dead`.
    pub status: String,
    pub attempt_count: u32,
    pub next_attempt_at_nanos: u128,
    pub claimed_by: Option<String>,
    pub claim_expires_at_nanos: Option<u128>,
    pub delivered_at_nanos: Option<u128>,
    pub last_error: Option<String>,
    /// Uniqueness key (`incident:destination:event:seq`); duplicate enqueues
    /// are ignored.
    pub delivery_key: String,
    pub created_at_nanos: u128,
}

/// One evaluation audit row.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertCheckRecord {
    pub rule_id: String,
    pub group_key: String,
    pub checked_at_nanos: u128,
    pub value: Option<f64>,
    pub sample_count: u64,
    /// `breach|healthy|no_data|error`.
    pub status: String,
    pub error: Option<String>,
}

/// Audit retention: newest rows kept per rule.
pub const ALERT_CHECKS_KEEP_PER_RULE: usize = 500;

fn real(row: &turso::Row, index: usize) -> f64 {
    opt_real(row, index).unwrap_or(0.0)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "alert thresholds/values fit well within f64 mantissa range"
)]
fn opt_real(row: &turso::Row, index: usize) -> Option<f64> {
    match row.get_value(index) {
        Ok(Value::Real(v)) => Some(v),
        Ok(Value::Integer(v)) => Some(v as f64),
        _ => None,
    }
}

fn flag(row: &turso::Row, index: usize) -> bool {
    integer(row, index) != 0
}

fn opt_millis_to_nanos(value: Option<i64>) -> Option<u128> {
    value.map(millis_to_nanos)
}

fn opt_text_value(value: Option<String>) -> Value {
    value.map_or(Value::Null, Value::Text)
}

impl TursoMetadataStore {
    const RULE_COLUMNS: &'static str = "id, name, enabled, signal_type, services, \
         exclude_services, attribute_filters, group_by, comparator, threshold, \
         threshold_upper, window_minutes, minimum_sample_count, \
         consecutive_breaches_required, consecutive_healthy_required, \
         no_data_behavior, severity, renotify_interval_minutes, destination_ids, \
         metric_name, metric_aggregation, created_at, updated_at";

    fn alert_rule_from_row(row: &turso::Row) -> AlertRuleRecord {
        AlertRuleRecord {
            id: text(row, 0),
            name: text(row, 1),
            enabled: flag(row, 2),
            signal_type: text(row, 3),
            services: text(row, 4),
            exclude_services: text(row, 5),
            attribute_filters: text(row, 6),
            group_by: opt_text(row, 7),
            comparator: text(row, 8),
            threshold: real(row, 9),
            threshold_upper: opt_real(row, 10),
            window_minutes: u32::try_from(integer(row, 11)).unwrap_or(0),
            minimum_sample_count: u64::try_from(integer(row, 12)).unwrap_or(0),
            consecutive_breaches_required: u32::try_from(integer(row, 13)).unwrap_or(1),
            consecutive_healthy_required: u32::try_from(integer(row, 14)).unwrap_or(1),
            no_data_behavior: text(row, 15),
            severity: text(row, 16),
            renotify_interval_minutes: u32::try_from(integer(row, 17)).unwrap_or(0),
            destination_ids: text(row, 18),
            metric_name: opt_text(row, 19),
            metric_aggregation: opt_text(row, 20),
            created_at_nanos: millis_to_nanos(integer(row, 21)),
            updated_at_nanos: millis_to_nanos(integer(row, 22)),
        }
    }

    /// Insert or update a rule; `created_at` is preserved on update and
    /// `last_scheduled_at` is never touched here (the CAS claim owns it).
    pub async fn alert_rule_save(&self, rule: &AlertRuleRecord) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "INSERT INTO alert_rules
                   (id, name, enabled, signal_type, services, exclude_services,
                    attribute_filters, group_by, comparator, threshold,
                    threshold_upper, window_minutes, minimum_sample_count,
                    consecutive_breaches_required, consecutive_healthy_required,
                    no_data_behavior, severity, renotify_interval_minutes,
                    destination_ids, metric_name, metric_aggregation,
                    created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                         ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name, enabled = excluded.enabled,
                   signal_type = excluded.signal_type,
                   services = excluded.services,
                   exclude_services = excluded.exclude_services,
                   attribute_filters = excluded.attribute_filters,
                   group_by = excluded.group_by,
                   comparator = excluded.comparator,
                   threshold = excluded.threshold,
                   threshold_upper = excluded.threshold_upper,
                   window_minutes = excluded.window_minutes,
                   minimum_sample_count = excluded.minimum_sample_count,
                   consecutive_breaches_required = excluded.consecutive_breaches_required,
                   consecutive_healthy_required = excluded.consecutive_healthy_required,
                   no_data_behavior = excluded.no_data_behavior,
                   severity = excluded.severity,
                   renotify_interval_minutes = excluded.renotify_interval_minutes,
                   destination_ids = excluded.destination_ids,
                   metric_name = excluded.metric_name,
                   metric_aggregation = excluded.metric_aggregation,
                   updated_at = excluded.updated_at",
                vec![
                    Value::Text(rule.id.clone()),
                    Value::Text(rule.name.clone()),
                    Value::Integer(i64::from(rule.enabled)),
                    Value::Text(rule.signal_type.clone()),
                    Value::Text(rule.services.clone()),
                    Value::Text(rule.exclude_services.clone()),
                    Value::Text(rule.attribute_filters.clone()),
                    opt_text_value(rule.group_by.clone()),
                    Value::Text(rule.comparator.clone()),
                    Value::Real(rule.threshold),
                    rule.threshold_upper.map_or(Value::Null, Value::Real),
                    Value::Integer(i64::from(rule.window_minutes)),
                    Value::Integer(i64::try_from(rule.minimum_sample_count).unwrap_or(i64::MAX)),
                    Value::Integer(i64::from(rule.consecutive_breaches_required)),
                    Value::Integer(i64::from(rule.consecutive_healthy_required)),
                    Value::Text(rule.no_data_behavior.clone()),
                    Value::Text(rule.severity.clone()),
                    Value::Integer(i64::from(rule.renotify_interval_minutes)),
                    Value::Text(rule.destination_ids.clone()),
                    opt_text_value(rule.metric_name.clone()),
                    opt_text_value(rule.metric_aggregation.clone()),
                    Value::Integer(nanos_to_millis(rule.created_at_nanos)),
                    Value::Integer(nanos_to_millis(rule.updated_at_nanos)),
                ],
            )
            .await?;
        Ok(())
    }

    /// Delete a rule and its dependent state/audit rows (incidents are kept as
    /// history; their `rule_id` becomes a dangling reference by design).
    pub async fn alert_rule_delete(&self, id: &str) -> anyhow::Result<bool> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM alert_rule_states WHERE rule_id = ?1", (id,))
            .await?;
        conn.execute("DELETE FROM alert_checks WHERE rule_id = ?1", (id,))
            .await?;
        let affected = conn
            .execute("DELETE FROM alert_rules WHERE id = ?1", (id,))
            .await?;
        Ok(affected > 0)
    }

    pub async fn alert_rules(&self) -> anyhow::Result<Vec<AlertRuleRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {} FROM alert_rules ORDER BY updated_at DESC",
                    Self::RULE_COLUMNS
                ),
                (),
            )
            .await?;
        let mut rules = Vec::new();
        while let Some(row) = rows.next().await? {
            rules.push(Self::alert_rule_from_row(&row));
        }
        Ok(rules)
    }

    pub async fn alert_rule(&self, id: &str) -> anyhow::Result<Option<AlertRuleRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {} FROM alert_rules WHERE id = ?1",
                    Self::RULE_COLUMNS
                ),
                (id,),
            )
            .await?;
        Ok(rows
            .next()
            .await?
            .map(|row| Self::alert_rule_from_row(&row)))
    }

    pub async fn alert_rule_set_enabled(&self, id: &str, enabled: bool) -> anyhow::Result<bool> {
        let affected = self
            .conn
            .lock()
            .await
            .execute(
                "UPDATE alert_rules SET enabled = ?2 WHERE id = ?1",
                (id, i64::from(enabled)),
            )
            .await?;
        Ok(affected > 0)
    }

    /// CAS-claim a rule for one evaluation tick: succeeds only when the rule
    /// is enabled and was last scheduled at least `min_interval_secs` ago (or
    /// never). Safe under multiple concurrent server instances.
    pub async fn alert_rule_claim(
        &self,
        id: &str,
        now_nanos: u128,
        min_interval_secs: u32,
    ) -> anyhow::Result<bool> {
        let now = nanos_to_millis(now_nanos);
        let cutoff = now - i64::from(min_interval_secs) * 1_000;
        let affected = self
            .conn
            .lock()
            .await
            .execute(
                "UPDATE alert_rules SET last_scheduled_at = ?2
                 WHERE id = ?1 AND enabled = 1
                   AND (last_scheduled_at IS NULL OR last_scheduled_at <= ?3)",
                (id, now, cutoff),
            )
            .await?;
        Ok(affected > 0)
    }

    const STATE_COLUMNS: &'static str = "rule_id, group_key, consecutive_breaches, \
         consecutive_healthy, incident_open, last_notified_at, last_status, \
         last_value, last_sample_count, last_evaluated_at, last_error";

    fn alert_rule_state_from_row(row: &turso::Row) -> AlertRuleStateRecord {
        AlertRuleStateRecord {
            rule_id: text(row, 0),
            group_key: text(row, 1),
            consecutive_breaches: u32::try_from(integer(row, 2)).unwrap_or(0),
            consecutive_healthy: u32::try_from(integer(row, 3)).unwrap_or(0),
            incident_open: flag(row, 4),
            last_notified_at_nanos: opt_millis_to_nanos(opt_integer(row, 5)),
            last_status: opt_text(row, 6),
            last_value: opt_real(row, 7),
            last_sample_count: u64::try_from(integer(row, 8)).unwrap_or(0),
            last_evaluated_at_nanos: opt_millis_to_nanos(opt_integer(row, 9)),
            last_error: opt_text(row, 10),
        }
    }

    pub async fn alert_rule_state_upsert(
        &self,
        state: &AlertRuleStateRecord,
    ) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "INSERT INTO alert_rule_states
                   (rule_id, group_key, consecutive_breaches, consecutive_healthy,
                    incident_open, last_notified_at, last_status, last_value,
                    last_sample_count, last_evaluated_at, last_error)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(rule_id, group_key) DO UPDATE SET
                   consecutive_breaches = excluded.consecutive_breaches,
                   consecutive_healthy = excluded.consecutive_healthy,
                   incident_open = excluded.incident_open,
                   last_notified_at = excluded.last_notified_at,
                   last_status = excluded.last_status,
                   last_value = excluded.last_value,
                   last_sample_count = excluded.last_sample_count,
                   last_evaluated_at = excluded.last_evaluated_at,
                   last_error = excluded.last_error",
                (
                    state.rule_id.as_str(),
                    state.group_key.as_str(),
                    i64::from(state.consecutive_breaches),
                    i64::from(state.consecutive_healthy),
                    i64::from(state.incident_open),
                    state.last_notified_at_nanos.map(nanos_to_millis),
                    state.last_status.clone(),
                    state.last_value,
                    i64::try_from(state.last_sample_count).unwrap_or(i64::MAX),
                    state.last_evaluated_at_nanos.map(nanos_to_millis),
                    state.last_error.clone(),
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn alert_rule_state(
        &self,
        rule_id: &str,
        group_key: &str,
    ) -> anyhow::Result<Option<AlertRuleStateRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {} FROM alert_rule_states
                     WHERE rule_id = ?1 AND group_key = ?2",
                    Self::STATE_COLUMNS
                ),
                (rule_id, group_key),
            )
            .await?;
        Ok(rows
            .next()
            .await?
            .map(|row| Self::alert_rule_state_from_row(&row)))
    }

    pub async fn alert_rule_states(
        &self,
        rule_id: &str,
    ) -> anyhow::Result<Vec<AlertRuleStateRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {} FROM alert_rule_states
                     WHERE rule_id = ?1 ORDER BY group_key ASC",
                    Self::STATE_COLUMNS
                ),
                (rule_id,),
            )
            .await?;
        let mut states = Vec::new();
        while let Some(row) = rows.next().await? {
            states.push(Self::alert_rule_state_from_row(&row));
        }
        Ok(states)
    }

    const INCIDENT_COLUMNS: &'static str = "id, rule_id, group_key, status, severity, \
         first_triggered_at, last_triggered_at, resolved_at, last_value, \
         last_notified_at";

    fn alert_incident_from_row(row: &turso::Row) -> AlertIncidentRecord {
        AlertIncidentRecord {
            id: text(row, 0),
            rule_id: text(row, 1),
            group_key: text(row, 2),
            status: text(row, 3),
            severity: text(row, 4),
            first_triggered_at_nanos: millis_to_nanos(integer(row, 5)),
            last_triggered_at_nanos: millis_to_nanos(integer(row, 6)),
            resolved_at_nanos: opt_millis_to_nanos(opt_integer(row, 7)),
            last_value: opt_real(row, 8),
            last_notified_at_nanos: opt_millis_to_nanos(opt_integer(row, 9)),
        }
    }

    /// Open an incident unless one is already open for (rule, group); returns
    /// whether a new incident row was created. The dedupe guard runs under the
    /// single connection lock, so concurrent ticks cannot double-open.
    pub async fn alert_incident_open(
        &self,
        incident: &AlertIncidentRecord,
    ) -> anyhow::Result<bool> {
        let conn = self.conn.lock().await;
        let existing = {
            let mut rows = conn
                .query(
                    "SELECT id FROM alert_incidents
                     WHERE rule_id = ?1 AND group_key = ?2 AND status = 'open'
                     LIMIT 1",
                    (incident.rule_id.as_str(), incident.group_key.as_str()),
                )
                .await?;
            rows.next().await?.is_some()
        };
        if existing {
            return Ok(false);
        }
        conn.execute(
            "INSERT INTO alert_incidents
               (id, rule_id, group_key, status, severity, first_triggered_at,
                last_triggered_at, resolved_at, last_value, last_notified_at)
             VALUES (?1, ?2, ?3, 'open', ?4, ?5, ?6, NULL, ?7, ?8)",
            (
                incident.id.as_str(),
                incident.rule_id.as_str(),
                incident.group_key.as_str(),
                incident.severity.as_str(),
                nanos_to_millis(incident.first_triggered_at_nanos),
                nanos_to_millis(incident.last_triggered_at_nanos),
                incident.last_value,
                incident.last_notified_at_nanos.map(nanos_to_millis),
            ),
        )
        .await?;
        Ok(true)
    }

    /// Resolve the open incident for (rule, group); returns the resolved
    /// incident id when one existed.
    pub async fn alert_incident_resolve(
        &self,
        rule_id: &str,
        group_key: &str,
        resolved_at_nanos: u128,
        last_value: Option<f64>,
    ) -> anyhow::Result<Option<String>> {
        let conn = self.conn.lock().await;
        let id = {
            let mut rows = conn
                .query(
                    "SELECT id FROM alert_incidents
                     WHERE rule_id = ?1 AND group_key = ?2 AND status = 'open'
                     LIMIT 1",
                    (rule_id, group_key),
                )
                .await?;
            rows.next().await?.map(|row| text(&row, 0))
        };
        let Some(id) = id else {
            return Ok(None);
        };
        conn.execute(
            "UPDATE alert_incidents
             SET status = 'resolved', resolved_at = ?2, last_value = ?3
             WHERE id = ?1",
            (id.as_str(), nanos_to_millis(resolved_at_nanos), last_value),
        )
        .await?;
        Ok(Some(id))
    }

    /// Record a re-trigger/renotify touch on an open incident.
    pub async fn alert_incident_touch(
        &self,
        id: &str,
        triggered_at_nanos: u128,
        last_value: Option<f64>,
        notified: bool,
    ) -> anyhow::Result<()> {
        let millis = nanos_to_millis(triggered_at_nanos);
        self.conn
            .lock()
            .await
            .execute(
                "UPDATE alert_incidents
                 SET last_triggered_at = ?2, last_value = ?3,
                     last_notified_at = CASE WHEN ?4 THEN ?2 ELSE last_notified_at END
                 WHERE id = ?1",
                (id, millis, last_value, i64::from(notified)),
            )
            .await?;
        Ok(())
    }

    pub async fn alert_incident(&self, id: &str) -> anyhow::Result<Option<AlertIncidentRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {} FROM alert_incidents WHERE id = ?1",
                    Self::INCIDENT_COLUMNS
                ),
                (id,),
            )
            .await?;
        Ok(rows
            .next()
            .await?
            .map(|row| Self::alert_incident_from_row(&row)))
    }

    pub async fn alert_incident_open_for(
        &self,
        rule_id: &str,
        group_key: &str,
    ) -> anyhow::Result<Option<AlertIncidentRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {} FROM alert_incidents
                     WHERE rule_id = ?1 AND group_key = ?2 AND status = 'open'
                     LIMIT 1",
                    Self::INCIDENT_COLUMNS
                ),
                (rule_id, group_key),
            )
            .await?;
        Ok(rows
            .next()
            .await?
            .map(|row| Self::alert_incident_from_row(&row)))
    }

    /// Incidents newest-first, optionally filtered by status and/or rule.
    pub async fn alert_incidents(
        &self,
        status: Option<&str>,
        rule_id: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<AlertIncidentRecord>> {
        let mut clauses: Vec<String> = Vec::new();
        let mut params: Vec<Value> = Vec::new();
        if let Some(status) = status {
            params.push(Value::Text(status.to_string()));
            clauses.push(format!("status = ?{}", params.len()));
        }
        if let Some(rule_id) = rule_id {
            params.push(Value::Text(rule_id.to_string()));
            clauses.push(format!("rule_id = ?{}", params.len()));
        }
        let where_clause = if clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", clauses.join(" AND "))
        };
        params.push(Value::Integer(i64::try_from(limit).unwrap_or(i64::MAX)));
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {} FROM alert_incidents{where_clause}
                     ORDER BY last_triggered_at DESC LIMIT ?{}",
                    Self::INCIDENT_COLUMNS,
                    params.len()
                ),
                params,
            )
            .await?;
        let mut incidents = Vec::new();
        while let Some(row) = rows.next().await? {
            incidents.push(Self::alert_incident_from_row(&row));
        }
        Ok(incidents)
    }

    fn alert_destination_from_row(row: &turso::Row) -> AlertDestinationRecord {
        AlertDestinationRecord {
            id: text(row, 0),
            name: text(row, 1),
            kind: text(row, 2),
            config: text(row, 3),
            created_at_nanos: millis_to_nanos(integer(row, 4)),
            updated_at_nanos: millis_to_nanos(integer(row, 5)),
        }
    }

    pub async fn alert_destination_save(
        &self,
        destination: &AlertDestinationRecord,
    ) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "INSERT INTO alert_destinations
                   (id, name, kind, config, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name, kind = excluded.kind,
                   config = excluded.config, updated_at = excluded.updated_at",
                (
                    destination.id.as_str(),
                    destination.name.as_str(),
                    destination.kind.as_str(),
                    destination.config.as_str(),
                    nanos_to_millis(destination.created_at_nanos),
                    nanos_to_millis(destination.updated_at_nanos),
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn alert_destination_delete(&self, id: &str) -> anyhow::Result<bool> {
        let affected = self
            .conn
            .lock()
            .await
            .execute("DELETE FROM alert_destinations WHERE id = ?1", (id,))
            .await?;
        Ok(affected > 0)
    }

    pub async fn alert_destinations(&self) -> anyhow::Result<Vec<AlertDestinationRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, name, kind, config, created_at, updated_at
                 FROM alert_destinations ORDER BY updated_at DESC",
                (),
            )
            .await?;
        let mut destinations = Vec::new();
        while let Some(row) = rows.next().await? {
            destinations.push(Self::alert_destination_from_row(&row));
        }
        Ok(destinations)
    }

    pub async fn alert_destination(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<AlertDestinationRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, name, kind, config, created_at, updated_at
                 FROM alert_destinations WHERE id = ?1",
                (id,),
            )
            .await?;
        Ok(rows
            .next()
            .await?
            .map(|row| Self::alert_destination_from_row(&row)))
    }

    const DELIVERY_COLUMNS: &'static str = "id, incident_id, destination_id, event_type, \
         status, attempt_count, next_attempt_at, claimed_by, claim_expires_at, \
         delivered_at, last_error, delivery_key, created_at";

    fn alert_delivery_from_row(row: &turso::Row) -> AlertDeliveryEventRecord {
        AlertDeliveryEventRecord {
            id: text(row, 0),
            incident_id: text(row, 1),
            destination_id: text(row, 2),
            event_type: text(row, 3),
            status: text(row, 4),
            attempt_count: u32::try_from(integer(row, 5)).unwrap_or(0),
            next_attempt_at_nanos: millis_to_nanos(integer(row, 6)),
            claimed_by: opt_text(row, 7),
            claim_expires_at_nanos: opt_millis_to_nanos(opt_integer(row, 8)),
            delivered_at_nanos: opt_millis_to_nanos(opt_integer(row, 9)),
            last_error: opt_text(row, 10),
            delivery_key: text(row, 11),
            created_at_nanos: millis_to_nanos(integer(row, 12)),
        }
    }

    /// Enqueue an outbox row; a duplicate `delivery_key` is silently ignored
    /// (idempotent enqueue). Returns whether a new row was inserted.
    pub async fn alert_delivery_enqueue(
        &self,
        event: &AlertDeliveryEventRecord,
    ) -> anyhow::Result<bool> {
        let affected = self
            .conn
            .lock()
            .await
            .execute(
                "INSERT OR IGNORE INTO alert_delivery_events
                   (id, incident_id, destination_id, event_type, status,
                    attempt_count, next_attempt_at, claimed_by, claim_expires_at,
                    delivered_at, last_error, delivery_key, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, NULL, NULL, NULL, ?8, ?9)",
                (
                    event.id.as_str(),
                    event.incident_id.as_str(),
                    event.destination_id.as_str(),
                    event.event_type.as_str(),
                    event.status.as_str(),
                    i64::from(event.attempt_count),
                    nanos_to_millis(event.next_attempt_at_nanos),
                    event.delivery_key.as_str(),
                    nanos_to_millis(event.created_at_nanos),
                ),
            )
            .await?;
        Ok(affected > 0)
    }

    /// Claim up to `limit` due pending deliveries with a lease. A row is due
    /// when pending, its `next_attempt_at` has passed, and any previous claim
    /// lease has expired. Runs entirely under the connection lock.
    pub async fn alert_deliveries_claim(
        &self,
        claimer: &str,
        now_nanos: u128,
        lease_secs: u32,
        limit: usize,
    ) -> anyhow::Result<Vec<AlertDeliveryEventRecord>> {
        let now = nanos_to_millis(now_nanos);
        let expires = now + i64::from(lease_secs) * 1_000;
        let conn = self.conn.lock().await;
        let due_ids: Vec<String> = {
            let mut rows = conn
                .query(
                    "SELECT id FROM alert_delivery_events
                     WHERE status = 'pending' AND next_attempt_at <= ?1
                       AND (claim_expires_at IS NULL OR claim_expires_at <= ?1)
                     ORDER BY next_attempt_at ASC LIMIT ?2",
                    (now, i64::try_from(limit).unwrap_or(i64::MAX)),
                )
                .await?;
            let mut ids = Vec::new();
            while let Some(row) = rows.next().await? {
                ids.push(text(&row, 0));
            }
            ids
        };
        let mut claimed = Vec::new();
        for id in due_ids {
            let affected = conn
                .execute(
                    "UPDATE alert_delivery_events
                     SET claimed_by = ?2, claim_expires_at = ?3
                     WHERE id = ?1 AND status = 'pending'
                       AND (claim_expires_at IS NULL OR claim_expires_at <= ?4)",
                    (id.as_str(), claimer, expires, now),
                )
                .await?;
            if affected == 0 {
                continue;
            }
            let mut rows = conn
                .query(
                    &format!(
                        "SELECT {} FROM alert_delivery_events WHERE id = ?1",
                        Self::DELIVERY_COLUMNS
                    ),
                    (id.as_str(),),
                )
                .await?;
            if let Some(row) = rows.next().await? {
                claimed.push(Self::alert_delivery_from_row(&row));
            }
        }
        Ok(claimed)
    }

    pub async fn alert_delivery_mark_delivered(
        &self,
        id: &str,
        delivered_at_nanos: u128,
    ) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "UPDATE alert_delivery_events
                 SET status = 'delivered', delivered_at = ?2,
                     claimed_by = NULL, claim_expires_at = NULL, last_error = NULL
                 WHERE id = ?1",
                (id, nanos_to_millis(delivered_at_nanos)),
            )
            .await?;
        Ok(())
    }

    /// Record a failed attempt. When `dead` the row leaves the pending pool;
    /// otherwise the caller supplies the backed-off `next_attempt_at`.
    pub async fn alert_delivery_mark_failed(
        &self,
        id: &str,
        error: &str,
        next_attempt_at_nanos: u128,
        dead: bool,
    ) -> anyhow::Result<()> {
        let status = if dead { "dead" } else { "pending" };
        self.conn
            .lock()
            .await
            .execute(
                "UPDATE alert_delivery_events
                 SET attempt_count = attempt_count + 1, last_error = ?2,
                     next_attempt_at = ?3, status = ?4,
                     claimed_by = NULL, claim_expires_at = NULL
                 WHERE id = ?1",
                (id, error, nanos_to_millis(next_attempt_at_nanos), status),
            )
            .await?;
        Ok(())
    }

    pub async fn alert_deliveries_for_incident(
        &self,
        incident_id: &str,
    ) -> anyhow::Result<Vec<AlertDeliveryEventRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {} FROM alert_delivery_events
                     WHERE incident_id = ?1 ORDER BY created_at ASC",
                    Self::DELIVERY_COLUMNS
                ),
                (incident_id,),
            )
            .await?;
        let mut events = Vec::new();
        while let Some(row) = rows.next().await? {
            events.push(Self::alert_delivery_from_row(&row));
        }
        Ok(events)
    }

    /// Append one audit row and prune to the newest
    /// [`ALERT_CHECKS_KEEP_PER_RULE`] rows for that rule.
    pub async fn alert_check_insert(&self, check: &AlertCheckRecord) -> anyhow::Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO alert_checks
               (rule_id, group_key, checked_at, value, sample_count, status, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                check.rule_id.as_str(),
                check.group_key.as_str(),
                nanos_to_millis(check.checked_at_nanos),
                check.value,
                i64::try_from(check.sample_count).unwrap_or(i64::MAX),
                check.status.as_str(),
                check.error.clone(),
            ),
        )
        .await?;
        conn.execute(
            "DELETE FROM alert_checks
             WHERE rule_id = ?1 AND rowid NOT IN (
               SELECT rowid FROM alert_checks WHERE rule_id = ?1
               ORDER BY checked_at DESC, rowid DESC LIMIT ?2
             )",
            (
                check.rule_id.as_str(),
                i64::try_from(ALERT_CHECKS_KEEP_PER_RULE).unwrap_or(i64::MAX),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn alert_checks(
        &self,
        rule_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<AlertCheckRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT rule_id, group_key, checked_at, value, sample_count, status, error
                 FROM alert_checks WHERE rule_id = ?1
                 ORDER BY checked_at DESC LIMIT ?2",
                (rule_id, i64::try_from(limit).unwrap_or(i64::MAX)),
            )
            .await?;
        let mut checks = Vec::new();
        while let Some(row) = rows.next().await? {
            checks.push(AlertCheckRecord {
                rule_id: text(&row, 0),
                group_key: text(&row, 1),
                checked_at_nanos: millis_to_nanos(integer(&row, 2)),
                value: opt_real(&row, 3),
                sample_count: u64::try_from(integer(&row, 4)).unwrap_or(0),
                status: text(&row, 5),
                error: opt_text(&row, 6),
            });
        }
        Ok(checks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> (tempfile::TempDir, std::path::PathBuf) {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("metadata.db");
        (directory, path)
    }

    const MINUTE_NANOS: u128 = 60 * 1_000_000_000;

    fn rule(id: &str) -> AlertRuleRecord {
        AlertRuleRecord {
            id: id.to_string(),
            name: "High error rate".to_string(),
            enabled: true,
            signal_type: "error_rate".to_string(),
            services: "[\"checkout\"]".to_string(),
            exclude_services: "[]".to_string(),
            attribute_filters: "[]".to_string(),
            group_by: Some("service".to_string()),
            comparator: "gt".to_string(),
            threshold: 0.2,
            threshold_upper: None,
            window_minutes: 5,
            minimum_sample_count: 10,
            consecutive_breaches_required: 2,
            consecutive_healthy_required: 2,
            no_data_behavior: "skip".to_string(),
            severity: "critical".to_string(),
            renotify_interval_minutes: 30,
            destination_ids: "[\"dest-1\"]".to_string(),
            metric_name: None,
            metric_aggregation: None,
            created_at_nanos: MINUTE_NANOS,
            updated_at_nanos: MINUTE_NANOS,
        }
    }

    #[tokio::test]
    async fn rule_round_trip_and_update_preserves_created_at() {
        let (_dir, path) = temp_db();
        let store = TursoMetadataStore::open(path).await.expect("open");
        let original = rule("r1");
        store.alert_rule_save(&original).await.expect("save");
        let loaded = store.alert_rule("r1").await.expect("get").expect("some");
        assert_eq!(loaded, original);

        let mut updated = original.clone();
        updated.name = "Renamed".to_string();
        updated.threshold = 0.5;
        updated.created_at_nanos = 99 * MINUTE_NANOS; // must be ignored on update
        updated.updated_at_nanos = 2 * MINUTE_NANOS;
        store.alert_rule_save(&updated).await.expect("update");
        let reloaded = store.alert_rule("r1").await.expect("get").expect("some");
        assert_eq!(reloaded.name, "Renamed");
        assert!((reloaded.threshold - 0.5).abs() < f64::EPSILON);
        assert_eq!(reloaded.created_at_nanos, MINUTE_NANOS);
        assert_eq!(reloaded.updated_at_nanos, 2 * MINUTE_NANOS);
        assert_eq!(store.alert_rules().await.expect("list").len(), 1);
    }

    #[tokio::test]
    async fn rule_claim_is_cas_and_respects_interval() {
        let (_dir, path) = temp_db();
        let store = TursoMetadataStore::open(path).await.expect("open");
        store.alert_rule_save(&rule("r1")).await.expect("save");

        let t0 = 100 * MINUTE_NANOS;
        assert!(store.alert_rule_claim("r1", t0, 30).await.expect("claim"));
        // Re-claim within the interval fails.
        assert!(
            !store
                .alert_rule_claim("r1", t0 + MINUTE_NANOS / 6, 30)
                .await
                .expect("claim")
        );
        // After the interval it succeeds again.
        assert!(
            store
                .alert_rule_claim("r1", t0 + MINUTE_NANOS, 30)
                .await
                .expect("claim")
        );
        // Disabled rules are never claimable.
        store
            .alert_rule_set_enabled("r1", false)
            .await
            .expect("disable");
        assert!(
            !store
                .alert_rule_claim("r1", t0 + 10 * MINUTE_NANOS, 30)
                .await
                .expect("claim")
        );
    }

    #[tokio::test]
    async fn state_upsert_round_trip() {
        let (_dir, path) = temp_db();
        let store = TursoMetadataStore::open(path).await.expect("open");
        let state = AlertRuleStateRecord {
            rule_id: "r1".to_string(),
            group_key: "checkout".to_string(),
            consecutive_breaches: 2,
            consecutive_healthy: 0,
            incident_open: true,
            last_notified_at_nanos: Some(MINUTE_NANOS),
            last_status: Some("breach".to_string()),
            last_value: Some(0.42),
            last_sample_count: 120,
            last_evaluated_at_nanos: Some(2 * MINUTE_NANOS),
            last_error: None,
        };
        store.alert_rule_state_upsert(&state).await.expect("upsert");
        let loaded = store
            .alert_rule_state("r1", "checkout")
            .await
            .expect("get")
            .expect("some");
        assert_eq!(loaded, state);

        let mut healed = state.clone();
        healed.incident_open = false;
        healed.consecutive_healthy = 2;
        healed.consecutive_breaches = 0;
        healed.last_status = Some("healthy".to_string());
        store
            .alert_rule_state_upsert(&healed)
            .await
            .expect("upsert");
        let states = store.alert_rule_states("r1").await.expect("list");
        assert_eq!(states, vec![healed]);
    }

    #[tokio::test]
    async fn incident_lifecycle_never_double_opens() {
        let (_dir, path) = temp_db();
        let store = TursoMetadataStore::open(path).await.expect("open");
        let incident = AlertIncidentRecord {
            id: "i1".to_string(),
            rule_id: "r1".to_string(),
            group_key: "checkout".to_string(),
            status: "open".to_string(),
            severity: "critical".to_string(),
            first_triggered_at_nanos: MINUTE_NANOS,
            last_triggered_at_nanos: MINUTE_NANOS,
            resolved_at_nanos: None,
            last_value: Some(0.4),
            last_notified_at_nanos: Some(MINUTE_NANOS),
        };
        assert!(store.alert_incident_open(&incident).await.expect("open"));
        // Second open for the same (rule, group) is a no-op.
        let mut duplicate = incident.clone();
        duplicate.id = "i2".to_string();
        assert!(!store.alert_incident_open(&duplicate).await.expect("open"));

        store
            .alert_incident_touch("i1", 2 * MINUTE_NANOS, Some(0.6), true)
            .await
            .expect("touch");
        let open = store
            .alert_incident_open_for("r1", "checkout")
            .await
            .expect("get")
            .expect("some");
        assert_eq!(open.id, "i1");
        assert_eq!(open.last_triggered_at_nanos, 2 * MINUTE_NANOS);
        assert_eq!(open.last_value, Some(0.6));
        assert_eq!(open.last_notified_at_nanos, Some(2 * MINUTE_NANOS));

        let resolved = store
            .alert_incident_resolve("r1", "checkout", 3 * MINUTE_NANOS, Some(0.01))
            .await
            .expect("resolve");
        assert_eq!(resolved.as_deref(), Some("i1"));
        // No open incident remains; resolving again is a no-op.
        assert!(
            store
                .alert_incident_open_for("r1", "checkout")
                .await
                .expect("get")
                .is_none()
        );
        assert!(
            store
                .alert_incident_resolve("r1", "checkout", 4 * MINUTE_NANOS, None)
                .await
                .expect("resolve")
                .is_none()
        );
        // A fresh breach can open a new incident after resolution.
        let mut second = incident.clone();
        second.id = "i3".to_string();
        assert!(store.alert_incident_open(&second).await.expect("open"));

        let all = store
            .alert_incidents(None, Some("r1"), 10)
            .await
            .expect("list");
        assert_eq!(all.len(), 2);
        let open_only = store
            .alert_incidents(Some("open"), None, 10)
            .await
            .expect("list");
        assert_eq!(open_only.len(), 1);
        assert_eq!(open_only[0].id, "i3");
    }

    #[tokio::test]
    async fn destination_round_trip() {
        let (_dir, path) = temp_db();
        let store = TursoMetadataStore::open(path).await.expect("open");
        let destination = AlertDestinationRecord {
            id: "d1".to_string(),
            name: "Ops webhook".to_string(),
            kind: "webhook".to_string(),
            config: "{\"url\":\"http://127.0.0.1:9000/hook\"}".to_string(),
            created_at_nanos: MINUTE_NANOS,
            updated_at_nanos: MINUTE_NANOS,
        };
        store
            .alert_destination_save(&destination)
            .await
            .expect("save");
        assert_eq!(
            store.alert_destination("d1").await.expect("get"),
            Some(destination)
        );
        assert!(store.alert_destination_delete("d1").await.expect("delete"));
        assert!(store.alert_destinations().await.expect("list").is_empty());
    }

    fn delivery(id: &str, key: &str, due_nanos: u128) -> AlertDeliveryEventRecord {
        AlertDeliveryEventRecord {
            id: id.to_string(),
            incident_id: "i1".to_string(),
            destination_id: "d1".to_string(),
            event_type: "triggered".to_string(),
            status: "pending".to_string(),
            attempt_count: 0,
            next_attempt_at_nanos: due_nanos,
            claimed_by: None,
            claim_expires_at_nanos: None,
            delivered_at_nanos: None,
            last_error: None,
            delivery_key: key.to_string(),
            created_at_nanos: due_nanos,
        }
    }

    #[tokio::test]
    async fn delivery_enqueue_is_idempotent_and_lease_claims() {
        let (_dir, path) = temp_db();
        let store = TursoMetadataStore::open(path).await.expect("open");
        let due = MINUTE_NANOS;
        assert!(
            store
                .alert_delivery_enqueue(&delivery("e1", "i1:d1:triggered:0", due))
                .await
                .expect("enqueue")
        );
        // Same delivery key: ignored.
        assert!(
            !store
                .alert_delivery_enqueue(&delivery("e2", "i1:d1:triggered:0", due))
                .await
                .expect("enqueue")
        );
        // Not yet due.
        assert!(
            store
                .alert_deliveries_claim("w1", due - 1_000_000_000, 60, 10)
                .await
                .expect("claim")
                .is_empty()
        );
        // Due: claimed with a lease.
        let claimed = store
            .alert_deliveries_claim("w1", due, 60, 10)
            .await
            .expect("claim");
        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].id, "e1");
        assert_eq!(claimed[0].claimed_by.as_deref(), Some("w1"));
        // While leased, a second worker gets nothing.
        assert!(
            store
                .alert_deliveries_claim("w2", due + 1_000_000_000, 60, 10)
                .await
                .expect("claim")
                .is_empty()
        );
        // After lease expiry another worker can take it over.
        let takeover = store
            .alert_deliveries_claim("w2", due + 2 * MINUTE_NANOS, 60, 10)
            .await
            .expect("claim");
        assert_eq!(takeover.len(), 1);
        assert_eq!(takeover[0].claimed_by.as_deref(), Some("w2"));
    }

    #[tokio::test]
    async fn concurrent_claimers_exactly_one_wins() {
        let (_dir, path) = temp_db();
        let store = std::sync::Arc::new(TursoMetadataStore::open(path).await.expect("open"));
        let due = MINUTE_NANOS;
        store
            .alert_delivery_enqueue(&delivery("e-conc", "k-conc", due))
            .await
            .expect("enqueue");
        let first = {
            let store = std::sync::Arc::clone(&store);
            tokio::spawn(async move { store.alert_deliveries_claim("w1", due, 60, 10).await })
        };
        let second = {
            let store = std::sync::Arc::clone(&store);
            tokio::spawn(async move { store.alert_deliveries_claim("w2", due, 60, 10).await })
        };
        let left = first.await.expect("join").expect("claim");
        let right = second.await.expect("join").expect("claim");
        assert_eq!(left.len() + right.len(), 1);
        let later = due + 2 * MINUTE_NANOS;
        let reclaim = store
            .alert_deliveries_claim("w3", later, 60, 10)
            .await
            .expect("reclaim");
        assert_eq!(reclaim.len(), 1);
        assert_eq!(reclaim[0].claimed_by.as_deref(), Some("w3"));
    }

    #[tokio::test]
    async fn delivery_failure_backoff_and_dead_letter() {
        let (_dir, path) = temp_db();
        let store = TursoMetadataStore::open(path).await.expect("open");
        let due = MINUTE_NANOS;
        store
            .alert_delivery_enqueue(&delivery("e1", "k1", due))
            .await
            .expect("enqueue");
        store
            .alert_deliveries_claim("w1", due, 60, 10)
            .await
            .expect("claim");
        store
            .alert_delivery_mark_failed("e1", "HTTP 500", due + 5 * MINUTE_NANOS, false)
            .await
            .expect("fail");
        let events = store
            .alert_deliveries_for_incident("i1")
            .await
            .expect("list");
        assert_eq!(events[0].attempt_count, 1);
        assert_eq!(events[0].status, "pending");
        assert_eq!(events[0].last_error.as_deref(), Some("HTTP 500"));
        assert_eq!(events[0].claimed_by, None);
        // Not claimable before the backed-off next attempt.
        assert!(
            store
                .alert_deliveries_claim("w1", due + MINUTE_NANOS, 60, 10)
                .await
                .expect("claim")
                .is_empty()
        );
        // Dead-letter removes it from the pending pool entirely.
        store
            .alert_delivery_mark_failed("e1", "HTTP 500", due + 6 * MINUTE_NANOS, true)
            .await
            .expect("fail");
        assert!(
            store
                .alert_deliveries_claim("w1", due + 10 * MINUTE_NANOS, 60, 10)
                .await
                .expect("claim")
                .is_empty()
        );
        // Success path on a fresh row.
        store
            .alert_delivery_enqueue(&delivery("e2", "k2", due))
            .await
            .expect("enqueue");
        store
            .alert_delivery_mark_delivered("e2", due + MINUTE_NANOS)
            .await
            .expect("deliver");
        let events = store
            .alert_deliveries_for_incident("i1")
            .await
            .expect("list");
        let delivered = events.iter().find(|e| e.id == "e2").expect("row");
        assert_eq!(delivered.status, "delivered");
        assert_eq!(delivered.delivered_at_nanos, Some(due + MINUTE_NANOS));
    }

    #[tokio::test]
    async fn checks_append_and_prune_to_retention() {
        let (_dir, path) = temp_db();
        let store = TursoMetadataStore::open(path).await.expect("open");
        let total = ALERT_CHECKS_KEEP_PER_RULE + 25;
        for index in 0..total {
            store
                .alert_check_insert(&AlertCheckRecord {
                    rule_id: "r1".to_string(),
                    group_key: "checkout".to_string(),
                    checked_at_nanos: (index as u128 + 1) * MINUTE_NANOS,
                    value: Some(f64::from(u32::try_from(index).expect("small index")) / 100.0),
                    sample_count: 10,
                    status: "healthy".to_string(),
                    error: None,
                })
                .await
                .expect("insert");
        }
        let checks = store.alert_checks("r1", total + 10).await.expect("list");
        assert_eq!(checks.len(), ALERT_CHECKS_KEEP_PER_RULE);
        // Newest first; oldest retained row is `total - keep + 1`.
        assert_eq!(checks[0].checked_at_nanos, total as u128 * MINUTE_NANOS);
        let oldest = checks.last().expect("row");
        assert_eq!(
            oldest.checked_at_nanos,
            (total - ALERT_CHECKS_KEEP_PER_RULE + 1) as u128 * MINUTE_NANOS
        );
        // Other rules are untouched by pruning.
        store
            .alert_check_insert(&AlertCheckRecord {
                rule_id: "r2".to_string(),
                group_key: String::new(),
                checked_at_nanos: MINUTE_NANOS,
                value: None,
                sample_count: 0,
                status: "no_data".to_string(),
                error: None,
            })
            .await
            .expect("insert");
        assert_eq!(store.alert_checks("r2", 10).await.expect("list").len(), 1);
    }
}
