use super::*;

impl GreptimeStore {
    pub async fn connect(
        base_url: &str,
        traces_ttl: &str,
        logs_ttl: &str,
        metrics_ttl: &str,
    ) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(HTTP_CLIENT_TIMEOUT)
            .build()?;
        let store = Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
            traces_ttl: traces_ttl.to_string(),
            logs_ttl: logs_ttl.to_string(),
            metrics_ttl: metrics_ttl.to_string(),
            traces_deviations_done: AtomicBool::new(false),
            logs_deviations_done: AtomicBool::new(false),
            metric_table_cache: Arc::new(RwLock::new(HashMap::new())),
        };
        // Liveness probe before DDL.
        store
            .client
            .get(format!("{}/health", store.base_url))
            .send()
            .await?
            .error_for_status()?;
        Ok(store)
    }

    /// Create extension tables + pre-create the native logs schema (idempotent),
    /// apply repair ALTERs, and reconcile TTLs from config.
    pub async fn bootstrap(&self, metrics_ttl: &str, error_events_ttl: &str) -> anyhow::Result<()> {
        // Pre-create opentelemetry_logs so extract-keys cannot promote high-card
        // attributes into the PRIMARY KEY (Plan 084). Schema matches GreptimeDB
        // v1.1.2 native OTLP logs + deliberate FIELD/TAG deviations.
        let logs_create = format!(
            r#"CREATE TABLE IF NOT EXISTS "opentelemetry_logs" (
                   "timestamp" TIMESTAMP(9) NOT NULL,
                   "trace_id" STRING NULL SKIPPING INDEX,
                   "span_id" STRING NULL,
                   "severity_text" STRING NULL,
                   "severity_number" INT NULL,
                   "body" STRING NULL FULLTEXT INDEX WITH(
                       analyzer = 'English',
                       backend = 'bloom',
                       case_sensitive = 'false',
                       false_positive_rate = '0.01',
                       granularity = '10240'
                   ),
                   "log_attributes" JSON NULL,
                   "trace_flags" INT UNSIGNED NULL,
                   "scope_name" STRING NULL,
                   "scope_version" STRING NULL,
                   "scope_attributes" JSON NULL,
                   "scope_schema_url" STRING NULL,
                   "resource_attributes" JSON NULL,
                   "resource_schema_url" STRING NULL,
                   "service.name" STRING NULL,
                   {} STRING NULL SKIPPING INDEX,
                   {} STRING NULL SKIPPING INDEX,
                   {} STRING NULL,
                   {} BIGINT NULL,
                   TIME INDEX ("timestamp"),
                   PRIMARY KEY ("scope_name", "service.name")
                 )
                 ENGINE=mito
                 WITH(
                   append_mode = 'true',
                   'greptime.semantic.signal_type' = 'log',
                   'greptime.semantic.source' = 'opentelemetry',
                   ttl = '{}'
                 )"#,
            wire_attr_ident(semconv::CLI_INVOCATION_ID),
            wire_attr_ident(semconv::SESSION_ID),
            wire_attr_ident(semconv::EVENT_NAME),
            wire_attr_ident(semconv::LOG_OBSERVED_TS_NANOS),
            escape(&self.logs_ttl),
        );
        self.sql(&logs_create).await?;

        let statements = [
            // Forward-only contract (operator, 2026-07-17): legacy run-keyed
            // tables are dropped, never read or migrated.
            "DROP TABLE IF EXISTS run_metric_points".to_string(),
            format!(
                r#"CREATE TABLE IF NOT EXISTS error_events (
                   "ts" TIMESTAMP(9) NOT NULL, "service" STRING, "fingerprint" STRING,
                   "error_type" STRING, "message" STRING, "stacktrace" STRING, "source" STRING,
                   "trace_id" STRING, "span_id" STRING, "attributes" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service", "fingerprint")
                 ) WITH (ttl = '{error_events_ttl}')"#
            ),
            format!(
                r#"CREATE TABLE IF NOT EXISTS invocation_metric_points (
                   "ts" TIMESTAMP(9) NOT NULL, "invocation_id" STRING SKIPPING INDEX,
                   "service" STRING, "name" STRING, "value" DOUBLE, "attributes" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service", "name")
                 ) WITH (append_mode = 'true', ttl = '{metrics_ttl}')"#
            ),
        ];
        for statement in statements {
            self.sql(&statement).await?;
        }
        self.ensure_metric_exemplars(metrics_ttl).await?;
        self.try_logs_deviations().await;
        self.reconcile_ttls(metrics_ttl, error_events_ttl).await;
        Ok(())
    }

    pub(super) fn metric_exemplars_ddl(table: &str, metrics_ttl: &str) -> String {
        format!(
            r#"CREATE TABLE IF NOT EXISTS {table} (
                   "ts" TIMESTAMP(9) NOT NULL,
                   "service" STRING, "name" STRING, "value" DOUBLE,
                   "trace_id" STRING SKIPPING INDEX, "span_id" STRING,
                   "invocation_id" STRING SKIPPING INDEX, "attributes" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service", "name")
                 ) WITH (append_mode = 'true', ttl = '{}')"#,
            escape(metrics_ttl)
        )
    }

    async fn table_exists(&self, table: &str) -> anyhow::Result<bool> {
        let rows = self
            .sql(&format!(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name = '{}'",
                escape(table)
            ))
            .await?;
        Ok(rows.first().map(|row| u128_at(row, 0)).unwrap_or(0) == 1)
    }

    async fn table_columns(&self, table: &str) -> anyhow::Result<Vec<String>> {
        Ok(self
            .sql(&format!("DESCRIBE {}", quoted_ident(table)))
            .await?
            .iter()
            .map(|row| str_at(row, 0))
            .collect())
    }

    /// Forward-only exemplar table contract: any pre-`invocation_id` shape is
    /// dropped (operator, 2026-07-17: no backward compatibility), then the
    /// canonical table is created fresh.
    async fn ensure_metric_exemplars(&self, metrics_ttl: &str) -> anyhow::Result<()> {
        for legacy in [METRIC_EXEMPLARS_REPLACEMENT, METRIC_EXEMPLARS_LEGACY] {
            self.sql(&format!("DROP TABLE IF EXISTS {}", quoted_ident(legacy)))
                .await?;
        }
        if self.table_exists(METRIC_EXEMPLARS_TABLE).await? {
            let columns = self.table_columns(METRIC_EXEMPLARS_TABLE).await?;
            if !columns.iter().any(|column| column == "invocation_id") {
                self.sql(&format!(
                    "DROP TABLE {}",
                    quoted_ident(METRIC_EXEMPLARS_TABLE)
                ))
                .await?;
            }
        }
        self.sql(&Self::metric_exemplars_ddl(
            METRIC_EXEMPLARS_TABLE,
            metrics_ttl,
        ))
        .await?;
        Ok(())
    }

    /// Apply configured retention TTLs via `ALTER TABLE … SET 'ttl'`.
    /// Fixed product tables always; existing native per-metric tables are
    /// enumerated through the bounded catalog so config changes reach them
    /// after creation (plan 116 Step 4).
    async fn reconcile_ttls(&self, metrics_ttl: &str, error_events_ttl: &str) {
        let mut targets: Vec<(String, &str)> = [
            ("opentelemetry_traces", self.traces_ttl.as_str()),
            ("opentelemetry_logs", self.logs_ttl.as_str()),
            ("error_events", error_events_ttl),
            ("invocation_metric_points", metrics_ttl),
            (METRIC_EXEMPLARS_TABLE, metrics_ttl),
        ]
        .into_iter()
        .map(|(table, ttl)| (table.to_string(), ttl))
        .collect();
        if let Ok(families) = self.discover_metric_families().await {
            for family in families {
                // Histogram families store samples in the `_count` table used as
                // stats_table; also touch sibling `_bucket` / `_sum` when named.
                targets.push((family.stats_table.clone(), metrics_ttl));
                if let Some(base) = family.stats_table.strip_suffix("_count") {
                    targets.push((format!("{base}_bucket"), metrics_ttl));
                    targets.push((format!("{base}_sum"), metrics_ttl));
                }
            }
        }
        for (table, ttl) in targets {
            let sql = format!("ALTER TABLE {table} SET 'ttl' = '{}'", escape(ttl));
            if let Err(error) = self.sql(&sql).await {
                let text = error.to_string().to_ascii_lowercase();
                if !text.contains("not found")
                    && !text.contains("exist")
                    && !text.contains("unknown table")
                {
                    tracing::warn!("ttl reconcile for {table} failed: {error:#}");
                }
            }
        }
    }

    /// Run a batch of idempotent post-create ALTERs, swallowing the benign
    /// "already exists" / "not found" outcomes (the table may not exist yet, or
    /// the deviation may already be applied from a prior run).
    async fn try_deviations<I, S>(&self, statements: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for statement in statements {
            if let Err(error) = self.sql(statement.as_ref()).await {
                let text = error.to_string().to_ascii_lowercase();
                if !text.contains("exist")
                    && !text.contains("duplicate")
                    && !text.contains("not found")
                    && !text.contains("already")
                {
                    tracing::warn!("native deviation failed: {error:#}");
                }
            }
        }
    }

    /// Logs deviations: SKIPPING on trace_id; ADD COLUMN repair for extract-key
    /// fields. Body FULLTEXT is native-default on ≥1.1 (no ALTER).
    async fn try_logs_deviations(&self) {
        self.try_deviations([
            r#"ALTER TABLE opentelemetry_logs MODIFY COLUMN "trace_id" SET SKIPPING INDEX"#
                .to_string(),
            format!(
                "ALTER TABLE opentelemetry_logs ADD COLUMN {} STRING",
                wire_attr_ident(semconv::SERVICE_NAME)
            ),
            format!(
                "ALTER TABLE opentelemetry_logs ADD COLUMN {} STRING",
                wire_attr_ident(semconv::CLI_INVOCATION_ID)
            ),
            format!(
                "ALTER TABLE opentelemetry_logs ADD COLUMN {} STRING",
                wire_attr_ident(semconv::SESSION_ID)
            ),
            format!(
                "ALTER TABLE opentelemetry_logs MODIFY COLUMN {} SET SKIPPING INDEX",
                wire_attr_ident(semconv::CLI_INVOCATION_ID)
            ),
            format!(
                "ALTER TABLE opentelemetry_logs MODIFY COLUMN {} SET SKIPPING INDEX",
                wire_attr_ident(semconv::SESSION_ID)
            ),
            format!(
                "ALTER TABLE opentelemetry_logs ADD COLUMN {} STRING",
                wire_attr_ident(semconv::EVENT_NAME)
            ),
            format!(
                "ALTER TABLE opentelemetry_logs ADD COLUMN {} BIGINT",
                wire_attr_ident(semconv::LOG_OBSERVED_TS_NANOS)
            ),
        ])
        .await;
        let sql = format!(
            "ALTER TABLE opentelemetry_logs SET 'ttl' = '{}'",
            escape(&self.logs_ttl)
        );
        crate::outcomes::warn_error(self.sql(&sql).await, "logs TTL reconcile");
    }

    /// Reconcile native trace retention once per process after the first trace
    /// forward auto-creates `opentelemetry_traces`.
    ///
    /// Fingerprint-to-trace correlation is owned by the derived `error_events`
    /// relation (plan 125). Legacy never-populated `fingerprint` columns on
    /// native traces are dropped when present (guarded on information_schema;
    /// DROP is live-proven safe on stable 1.1 and nightly 1.2 probes).
    pub(super) async fn ensure_traces_deviations(&self) {
        if self
            .traces_deviations_done
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            let sql = format!(
                "ALTER TABLE opentelemetry_traces SET 'ttl' = '{}'",
                escape(&self.traces_ttl)
            );
            crate::outcomes::warn_error(self.sql(&sql).await, "traces TTL reconcile");
            self.drop_legacy_trace_fingerprint_column().await;
            // Pre-create the correlation/projection attribute columns the
            // invocation queries reference, so COALESCE over span + resource
            // sources never hits "column not found" on sparse emitters. The
            // greptime_trace_v1 pipeline widens the same columns on demand;
            // these ALTERs are idempotent no-ops once any emitter stamped them.
            let mut alters = Vec::new();
            for attribute in [
                semconv::CLI_INVOCATION_ID,
                semconv::SESSION_ID,
                semconv::CLI_COMMAND_NAME,
                semconv::APP_MODE,
                semconv::APP_SCREEN_ID,
                semconv::UI_ACTION_NAME,
                semconv::OUTCOME,
                semconv::BACKGROUND_CYCLE_NAME,
                semconv::JOB_ID,
                semconv::JOB_TYPE,
                semconv::GEN_AI_AGENT_NAME,
                semconv::GEN_AI_CONVERSATION_ID,
                semconv::GEN_AI_PROVIDER_NAME,
            ] {
                alters.push(format!(
                    "ALTER TABLE opentelemetry_traces ADD COLUMN {} STRING",
                    quoted_ident(&semconv::span_column(attribute))
                ));
            }
            for attribute in [semconv::CLI_INVOCATION_ID, semconv::SESSION_ID] {
                alters.push(format!(
                    "ALTER TABLE opentelemetry_traces ADD COLUMN {} STRING",
                    quoted_ident(&semconv::resource_column(attribute))
                ));
            }
            self.try_deviations(alters).await;
        }
    }

    /// Drop the legacy unpopulated native `fingerprint` column when present
    /// (plan 125). Safe on empty legacy columns; guarded so missing columns
    /// do not error (code 4002).
    async fn drop_legacy_trace_fingerprint_column(&self) {
        let present = self
            .sql(
                r#"SELECT "column_name" FROM information_schema.columns
                   WHERE "table_name" = 'opentelemetry_traces'
                     AND "column_name" = 'fingerprint'
                   LIMIT 1"#,
            )
            .await
            .ok()
            .is_some_and(|rows| !rows.is_empty());
        if !present {
            return;
        }
        if let Err(error) = self
            .sql(r#"ALTER TABLE opentelemetry_traces DROP COLUMN "fingerprint""#)
            .await
        {
            let text = error.to_string().to_ascii_lowercase();
            if !text.contains("not exist") && !text.contains("not found") {
                tracing::warn!("legacy fingerprint drop failed: {error:#}");
            }
        } else {
            tracing::info!("dropped legacy unpopulated opentelemetry_traces.fingerprint column");
        }
    }

    /// Apply the logs deviations once per process, after the first logs forward
    /// has auto-created `opentelemetry_logs`.
    pub(super) async fn ensure_logs_deviations(&self) {
        if self
            .logs_deviations_done
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.try_logs_deviations().await;
        }
    }
}
