use super::*;

impl TursoMetadataStore {
    pub async fn start_run(
        &self,
        run_id: &str,
        command: Option<&str>,
        started_at_nanos: u128,
    ) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "INSERT OR REPLACE INTO runs (run_id, command, started_at, status)
                 VALUES (?1, ?2, ?3, 'running')",
                (
                    run_id,
                    command.map(str::to_string),
                    nanos_to_millis(started_at_nanos),
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn finish_run(
        &self,
        run_id: &str,
        ended_at_nanos: u128,
        exit_code: i32,
    ) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "UPDATE runs SET ended_at = ?2, exit_code = ?3, status = 'finished'
                 WHERE run_id = ?1",
                (
                    run_id,
                    nanos_to_millis(ended_at_nanos),
                    i64::from(exit_code),
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn runs(&self, limit: usize) -> anyhow::Result<Vec<RunRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT run_id, command, started_at, ended_at, exit_code, status
                 FROM runs ORDER BY started_at DESC LIMIT ?1",
                [Value::Integer(i64::try_from(limit).unwrap_or(i64::MAX))],
            )
            .await?;
        let mut runs = Vec::new();
        while let Some(row) = rows.next().await? {
            runs.push(Self::run_from_row(&row));
        }
        Ok(runs)
    }

    fn run_from_row(row: &turso::Row) -> RunRecord {
        RunRecord {
            run_id: text(row, 0),
            command: opt_text(row, 1),
            started_at_nanos: millis_to_nanos(integer(row, 2)),
            ended_at_nanos: opt_integer(row, 3).map(millis_to_nanos),
            exit_code: opt_integer(row, 4).and_then(|v| i32::try_from(v).ok()),
            status: text(row, 5),
        }
    }

    pub async fn run(&self, run_id: &str) -> anyhow::Result<Option<RunRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT run_id, command, started_at, ended_at, exit_code, status
                 FROM runs WHERE run_id = ?1",
                (run_id,),
            )
            .await?;
        Ok(rows.next().await?.map(|row| Self::run_from_row(&row)))
    }

    /// Auto-register a run id first seen in telemetry (no CLI `runStart`):
    /// insert with status `external` unless the run already exists.
    pub async fn ensure_run(&self, run_id: &str, first_seen_nanos: u128) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "INSERT OR IGNORE INTO runs (run_id, started_at, status)
                 VALUES (?1, ?2, 'external')",
                (run_id, nanos_to_millis(first_seen_nanos)),
            )
            .await?;
        Ok(())
    }
}
