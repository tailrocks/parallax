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
            wire_attr_ident(semconv::PARALLAX_RUN_ID),
            wire_attr_ident(semconv::EVENT_NAME),
            wire_attr_ident(semconv::LOG_OBSERVED_TS_NANOS),
            escape(&self.logs_ttl),
        );
        self.sql(&logs_create).await?;

        let statements = [
            format!(
                r#"CREATE TABLE IF NOT EXISTS error_events (
                   "ts" TIMESTAMP(9) NOT NULL, "service" STRING, "fingerprint" STRING,
                   "error_type" STRING, "message" STRING, "stacktrace" STRING, "source" STRING,
                   "trace_id" STRING, "span_id" STRING, "attributes" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service", "fingerprint")
                 ) WITH (ttl = '{error_events_ttl}')"#
            ),
            format!(
                r#"CREATE TABLE IF NOT EXISTS run_metric_points (
                   "ts" TIMESTAMP(9) NOT NULL, "run_id" STRING SKIPPING INDEX,
                   "service" STRING, "name" STRING, "value" DOUBLE, "attributes" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service", "name")
                 ) WITH (append_mode = 'true', ttl = '{metrics_ttl}')"#
            ),
        ];
        for statement in statements {
            self.sql(&statement).await?;
        }
        self.migrate_metric_exemplars(metrics_ttl).await?;
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
                   "run_id" STRING SKIPPING INDEX, "attributes" JSON,
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

    async fn table_primary_key(&self, table: &str) -> anyhow::Result<Vec<String>> {
        Ok(self
            .sql(&format!("DESCRIBE {}", quoted_ident(table)))
            .await?
            .iter()
            .filter(|row| str_at(row, 2) == "PRI" && str_at(row, 5) == "TAG")
            .map(|row| str_at(row, 0))
            .collect())
    }

    async fn table_count(&self, table: &str) -> anyhow::Result<u128> {
        let rows = self
            .sql(&format!("SELECT COUNT(*) FROM {}", quoted_ident(table)))
            .await?;
        Ok(rows.first().map(|row| u128_at(row, 0)).unwrap_or(0))
    }

    async fn verify_exemplar_copy(&self, source: &str, destination: &str) -> anyhow::Result<()> {
        let source_count = self.table_count(source).await?;
        let destination_count = self.table_count(destination).await?;
        anyhow::ensure!(
            source_count == destination_count,
            "metric exemplar migration row-count mismatch: {source}={source_count}, {destination}={destination_count}"
        );
        let mismatches = self
            .sql(&format!(
                r#"SELECT COUNT(*) FROM (
                       (SELECT {METRIC_EXEMPLAR_COLUMNS} FROM {source} EXCEPT
                        SELECT {METRIC_EXEMPLAR_COLUMNS} FROM {destination})
                       UNION ALL
                       (SELECT {METRIC_EXEMPLAR_COLUMNS} FROM {destination} EXCEPT
                        SELECT {METRIC_EXEMPLAR_COLUMNS} FROM {source})
                   ) AS differences"#,
                source = quoted_ident(source),
                destination = quoted_ident(destination),
            ))
            .await?;
        anyhow::ensure!(
            mismatches.first().map(|row| u128_at(row, 0)).unwrap_or(0) == 0,
            "metric exemplar migration changed values"
        );
        Ok(())
    }

    async fn migrate_metric_exemplars(&self, metrics_ttl: &str) -> anyhow::Result<()> {
        let canonical_exists = self.table_exists(METRIC_EXEMPLARS_TABLE).await?;
        let legacy_exists = self.table_exists(METRIC_EXEMPLARS_LEGACY).await?;
        let canonical_key = if canonical_exists {
            Some(self.table_primary_key(METRIC_EXEMPLARS_TABLE).await?)
        } else {
            None
        };
        let state = exemplar_migration_state(canonical_key.as_deref(), legacy_exists);

        match state {
            ExemplarMigrationState::Complete => {
                if self.table_exists(METRIC_EXEMPLARS_REPLACEMENT).await? {
                    self.sql(&format!(
                        "DROP TABLE {}",
                        quoted_ident(METRIC_EXEMPLARS_REPLACEMENT)
                    ))
                    .await?;
                }
                return Ok(());
            }
            ExemplarMigrationState::CleanupLegacy => {
                self.verify_exemplar_copy(METRIC_EXEMPLARS_LEGACY, METRIC_EXEMPLARS_TABLE)
                    .await?;
                self.sql(&format!(
                    "DROP TABLE {}",
                    quoted_ident(METRIC_EXEMPLARS_LEGACY)
                ))
                .await?;
                if self.table_exists(METRIC_EXEMPLARS_REPLACEMENT).await? {
                    self.sql(&format!(
                        "DROP TABLE {}",
                        quoted_ident(METRIC_EXEMPLARS_REPLACEMENT)
                    ))
                    .await?;
                }
                return Ok(());
            }
            ExemplarMigrationState::Fresh => {
                self.sql(&Self::metric_exemplars_ddl(
                    METRIC_EXEMPLARS_TABLE,
                    metrics_ttl,
                ))
                .await?;
                return Ok(());
            }
            ExemplarMigrationState::UnknownCanonical => {
                anyhow::bail!("metric_exemplars has an unknown primary-key shape")
            }
            ExemplarMigrationState::MigrateCanonical | ExemplarMigrationState::ResumeFromLegacy => {
            }
        }

        let source = if state == ExemplarMigrationState::MigrateCanonical {
            METRIC_EXEMPLARS_TABLE
        } else {
            METRIC_EXEMPLARS_LEGACY
        };

        if self.table_exists(METRIC_EXEMPLARS_REPLACEMENT).await? {
            self.sql(&format!(
                "DROP TABLE {}",
                quoted_ident(METRIC_EXEMPLARS_REPLACEMENT)
            ))
            .await?;
        }
        self.sql(&Self::metric_exemplars_ddl(
            METRIC_EXEMPLARS_REPLACEMENT,
            metrics_ttl,
        ))
        .await?;
        self.sql(&format!(
            "INSERT INTO {} ({METRIC_EXEMPLAR_COLUMNS}) SELECT {METRIC_EXEMPLAR_COLUMNS} FROM {}",
            quoted_ident(METRIC_EXEMPLARS_REPLACEMENT),
            quoted_ident(source)
        ))
        .await?;
        self.verify_exemplar_copy(source, METRIC_EXEMPLARS_REPLACEMENT)
            .await?;

        if source == METRIC_EXEMPLARS_TABLE {
            self.sql(&format!(
                "ALTER TABLE {} RENAME {}",
                quoted_ident(METRIC_EXEMPLARS_TABLE),
                quoted_ident(METRIC_EXEMPLARS_LEGACY)
            ))
            .await?;
        }
        self.sql(&format!(
            "ALTER TABLE {} RENAME {}",
            quoted_ident(METRIC_EXEMPLARS_REPLACEMENT),
            quoted_ident(METRIC_EXEMPLARS_TABLE)
        ))
        .await?;
        self.verify_exemplar_copy(METRIC_EXEMPLARS_LEGACY, METRIC_EXEMPLARS_TABLE)
            .await?;
        self.sql(&format!(
            "DROP TABLE {}",
            quoted_ident(METRIC_EXEMPLARS_LEGACY)
        ))
        .await?;
        Ok(())
    }

    /// Apply configured retention TTLs via `ALTER TABLE … SET 'ttl'`.
    /// Per-metric native tables are excluded (TTL rides creation hints only).
    async fn reconcile_ttls(&self, metrics_ttl: &str, error_events_ttl: &str) {
        let targets = [
            ("opentelemetry_traces", self.traces_ttl.as_str()),
            ("opentelemetry_logs", self.logs_ttl.as_str()),
            ("error_events", error_events_ttl),
            ("run_metric_points", metrics_ttl),
            (METRIC_EXEMPLARS_TABLE, metrics_ttl),
        ];
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
                wire_attr_ident(semconv::PARALLAX_RUN_ID)
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
    /// relation. Existing native nullable `fingerprint` columns are inert
    /// legacy schema: no query reads them and startup never backfills or drops
    /// a native table column without a separately live-proven migration.
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
