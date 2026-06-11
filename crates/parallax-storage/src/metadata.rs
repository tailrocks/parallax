//! The metadata store: mutable product state (issues, runs, dashboards,
//! settings) per implementation spec §6. Turso is the engine.

use crate::model::{Dashboard, Issue, RunRecord};
use std::path::Path;
use turso::Value;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS issues (
  fingerprint   TEXT PRIMARY KEY,
  title         TEXT NOT NULL,
  error_type    TEXT NOT NULL,
  culprit       TEXT,
  service       TEXT NOT NULL,
  status        TEXT NOT NULL DEFAULT 'open',
  first_seen    INTEGER NOT NULL,
  last_seen     INTEGER NOT NULL,
  event_count   INTEGER NOT NULL DEFAULT 0,
  last_trace_id TEXT,
  tags          TEXT NOT NULL DEFAULT '{}'
);
CREATE TABLE IF NOT EXISTS runs (
  run_id      TEXT PRIMARY KEY,
  command     TEXT,
  started_at  INTEGER NOT NULL,
  ended_at    INTEGER,
  exit_code   INTEGER,
  status      TEXT NOT NULL DEFAULT 'running'
);
CREATE TABLE IF NOT EXISTS dashboards (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  layout      TEXT NOT NULL,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS settings ( key TEXT PRIMARY KEY, value TEXT NOT NULL );
";

/// Nanosecond timestamps are stored as INTEGER milliseconds in the metadata
/// store (SQLite-class integers are i64; nanos since 1970 overflow in 2262 as
/// i64 but UI/sorting only needs millis precision here).
fn nanos_to_millis(nanos: u128) -> i64 {
    i64::try_from(nanos / 1_000_000).unwrap_or(i64::MAX)
}

fn millis_to_nanos(millis: i64) -> u128 {
    u128::try_from(millis.max(0)).unwrap_or(0) * 1_000_000
}

/// One derived error occurrence, ready for issue upsert.
pub struct IssueOccurrence<'a> {
    pub fingerprint: &'a str,
    pub title: String,
    pub error_type: &'a str,
    pub culprit: Option<String>,
    pub service: &'a str,
    pub ts_nanos: u128,
    pub trace_id: Option<&'a str>,
}

pub struct MetadataStore {
    conn: turso::Connection,
}

impl MetadataStore {
    pub async fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let db = turso::Builder::new_local(path.as_ref().to_string_lossy().as_ref())
            .build()
            .await?;
        let conn = db.connect()?;
        for statement in SCHEMA.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            conn.execute(statement, ()).await?;
        }
        Ok(Self { conn })
    }

    /// Record one more occurrence of a fingerprint (insert or update).
    pub async fn upsert_issue_occurrence(
        &self,
        occurrence: &IssueOccurrence<'_>,
    ) -> anyhow::Result<()> {
        let millis = nanos_to_millis(occurrence.ts_nanos);
        self.conn
            .execute(
                "INSERT INTO issues
                   (fingerprint, title, error_type, culprit, service,
                    first_seen, last_seen, event_count, last_trace_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, 1, ?7)
                 ON CONFLICT(fingerprint) DO UPDATE SET
                   last_seen = MAX(last_seen, excluded.last_seen),
                   event_count = event_count + 1,
                   last_trace_id = COALESCE(excluded.last_trace_id, last_trace_id)",
                (
                    occurrence.fingerprint,
                    occurrence.title.as_str(),
                    occurrence.error_type,
                    occurrence.culprit.clone(),
                    occurrence.service,
                    millis,
                    occurrence.trace_id.map(str::to_string),
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn issues(&self, limit: usize) -> anyhow::Result<Vec<Issue>> {
        let mut rows = self
            .conn
            .query(
                "SELECT fingerprint, title, error_type, culprit, service, status,
                        first_seen, last_seen, event_count, last_trace_id
                 FROM issues ORDER BY last_seen DESC LIMIT ?1",
                [Value::Integer(i64::try_from(limit).unwrap_or(i64::MAX))],
            )
            .await?;
        let mut issues = Vec::new();
        while let Some(row) = rows.next().await? {
            issues.push(Issue {
                fingerprint: text(&row, 0),
                title: text(&row, 1),
                error_type: text(&row, 2),
                culprit: opt_text(&row, 3),
                service: text(&row, 4),
                status: text(&row, 5),
                first_seen_nanos: millis_to_nanos(integer(&row, 6)),
                last_seen_nanos: millis_to_nanos(integer(&row, 7)),
                event_count: u64::try_from(integer(&row, 8)).unwrap_or(0),
                last_trace_id: opt_text(&row, 9),
            });
        }
        Ok(issues)
    }

    pub async fn set_issue_status(&self, fingerprint: &str, status: &str) -> anyhow::Result<()> {
        self.conn
            .execute(
                "UPDATE issues SET status = ?2 WHERE fingerprint = ?1",
                (fingerprint, status),
            )
            .await?;
        Ok(())
    }

    pub async fn start_run(
        &self,
        run_id: &str,
        command: Option<&str>,
        started_at_nanos: u128,
    ) -> anyhow::Result<()> {
        self.conn
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
        let mut rows = self
            .conn
            .query(
                "SELECT run_id, command, started_at, ended_at, exit_code, status
                 FROM runs ORDER BY started_at DESC LIMIT ?1",
                [Value::Integer(i64::try_from(limit).unwrap_or(i64::MAX))],
            )
            .await?;
        let mut runs = Vec::new();
        while let Some(row) = rows.next().await? {
            runs.push(RunRecord {
                run_id: text(&row, 0),
                command: opt_text(&row, 1),
                started_at_nanos: millis_to_nanos(integer(&row, 2)),
                ended_at_nanos: opt_integer(&row, 3).map(millis_to_nanos),
                exit_code: opt_integer(&row, 4).and_then(|v| i32::try_from(v).ok()),
                status: text(&row, 5),
            });
        }
        Ok(runs)
    }
}

impl MetadataStore {
    pub async fn dashboard_save(
        &self,
        id: &str,
        name: &str,
        layout: &str,
        now_nanos: u128,
    ) -> anyhow::Result<()> {
        let millis = nanos_to_millis(now_nanos);
        self.conn
            .execute(
                "INSERT INTO dashboards (id, name, layout, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name, layout = excluded.layout,
                   updated_at = excluded.updated_at",
                (id, name, layout, millis),
            )
            .await?;
        Ok(())
    }

    pub async fn dashboard_delete(&self, id: &str) -> anyhow::Result<bool> {
        let affected = self
            .conn
            .execute("DELETE FROM dashboards WHERE id = ?1", (id,))
            .await?;
        Ok(affected > 0)
    }

    pub async fn dashboards(&self) -> anyhow::Result<Vec<Dashboard>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, name, layout, created_at, updated_at
                 FROM dashboards ORDER BY updated_at DESC",
                (),
            )
            .await?;
        let mut dashboards = Vec::new();
        while let Some(row) = rows.next().await? {
            dashboards.push(Dashboard {
                id: text(&row, 0),
                name: text(&row, 1),
                layout: text(&row, 2),
                created_at_nanos: millis_to_nanos(integer(&row, 3)),
                updated_at_nanos: millis_to_nanos(integer(&row, 4)),
            });
        }
        Ok(dashboards)
    }
}

fn text(row: &turso::Row, index: usize) -> String {
    match row.get_value(index) {
        Ok(Value::Text(s)) => s,
        _ => String::new(),
    }
}

fn opt_text(row: &turso::Row, index: usize) -> Option<String> {
    match row.get_value(index) {
        Ok(Value::Text(s)) => Some(s),
        _ => None,
    }
}

fn integer(row: &turso::Row, index: usize) -> i64 {
    match row.get_value(index) {
        Ok(Value::Integer(v)) => v,
        _ => 0,
    }
}

fn opt_integer(row: &turso::Row, index: usize) -> Option<i64> {
    match row.get_value(index) {
        Ok(Value::Integer(v)) => Some(v),
        _ => None,
    }
}
