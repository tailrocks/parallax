use super::*;

impl TursoMetadataStore {
    pub async fn start_invocation(
        &self,
        invocation_id: &str,
        command: Option<&str>,
        app_mode: Option<&str>,
        started_at_nanos: u128,
    ) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "INSERT OR REPLACE INTO invocations
                   (invocation_id, command, app_mode, started_at, status)
                 VALUES (?1, ?2, ?3, ?4, 'running')",
                (
                    invocation_id,
                    command.map(str::to_string),
                    app_mode.map(str::to_string),
                    nanos_to_millis(started_at_nanos),
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn finish_invocation(
        &self,
        invocation_id: &str,
        ended_at_nanos: u128,
        exit_code: i32,
        outcome: Option<&str>,
    ) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "UPDATE invocations
                 SET ended_at = ?2, exit_code = ?3, outcome = ?4, status = 'finished'
                 WHERE invocation_id = ?1",
                (
                    invocation_id,
                    nanos_to_millis(ended_at_nanos),
                    i64::from(exit_code),
                    outcome.map(str::to_string),
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn invocations(&self, limit: usize) -> anyhow::Result<Vec<InvocationRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT invocation_id, command, app_mode, started_at, ended_at, exit_code,
                        outcome, status
                 FROM invocations ORDER BY started_at DESC LIMIT ?1",
                [Value::Integer(i64::try_from(limit).unwrap_or(i64::MAX))],
            )
            .await?;
        let mut invocations = Vec::new();
        while let Some(row) = rows.next().await? {
            invocations.push(Self::invocation_from_row(&row));
        }
        Ok(invocations)
    }

    fn invocation_from_row(row: &turso::Row) -> InvocationRecord {
        InvocationRecord {
            invocation_id: text(row, 0),
            command: opt_text(row, 1),
            app_mode: opt_text(row, 2),
            started_at_nanos: millis_to_nanos(integer(row, 3)),
            ended_at_nanos: opt_integer(row, 4).map(millis_to_nanos),
            exit_code: opt_integer(row, 5).and_then(|v| i32::try_from(v).ok()),
            outcome: opt_text(row, 6),
            status: text(row, 7),
        }
    }

    pub async fn invocation(
        &self,
        invocation_id: &str,
    ) -> anyhow::Result<Option<InvocationRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT invocation_id, command, app_mode, started_at, ended_at, exit_code,
                        outcome, status
                 FROM invocations WHERE invocation_id = ?1",
                (invocation_id,),
            )
            .await?;
        Ok(rows.next().await?.map(|row| Self::invocation_from_row(&row)))
    }

    /// Auto-register an invocation id first seen in telemetry (no CLI
    /// `invocationStart`): insert with status `external` unless it exists.
    pub async fn ensure_invocation(
        &self,
        invocation_id: &str,
        first_seen_nanos: u128,
    ) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "INSERT OR IGNORE INTO invocations (invocation_id, started_at, status)
                 VALUES (?1, ?2, 'external')",
                (invocation_id, nanos_to_millis(first_seen_nanos)),
            )
            .await?;
        Ok(())
    }
}
