//! The metadata store: mutable product state (issues, runs, dashboards,
//! settings) per implementation spec §6. Turso is the engine.

use crate::model::{Dashboard, Issue, IssueQuery, IssueSortKey, RunRecord, TrendPoint};
use std::collections::BTreeMap;
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
CREATE TABLE IF NOT EXISTS issue_buckets (
  fingerprint TEXT NOT NULL,
  bucket_ts   INTEGER NOT NULL,
  count       INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (fingerprint, bucket_ts)
);
";

/// Trend rollups count occurrences per fingerprint per minute.
const BUCKET_MILLIS: i64 = 60_000;

/// Window cap for filtered issue scans; `issues_filtered`'s `total` is exact
/// up to this many matching rows.
pub const ISSUE_SCAN_CAP: usize = 1000;

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
    /// The event's attributes — merged into the issue's bounded tag cache.
    pub attributes: &'a serde_json::Value,
}

/// Bounds for the per-issue tag-values cache (`issues.tags`).
const TAGS_MAX_KEYS: usize = 16;
const TAGS_MAX_VALUES_PER_KEY: usize = 8;
const TAGS_MAX_VALUE_LEN: usize = 64;

/// Merge an event's scalar attributes into the `{key: {value: count}}` cache.
/// `exception.*` keys are the event body, not tags; nested values are skipped.
fn merge_tags(existing: &str, attributes: &serde_json::Value) -> String {
    let mut tags: BTreeMap<String, BTreeMap<String, u64>> =
        serde_json::from_str(existing).unwrap_or_default();
    if let Some(map) = attributes.as_object() {
        for (key, value) in map {
            if key.starts_with("exception.") {
                continue;
            }
            let rendered = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => continue,
            };
            if rendered.is_empty() || rendered.len() > TAGS_MAX_VALUE_LEN {
                continue;
            }
            if !tags.contains_key(key) && tags.len() >= TAGS_MAX_KEYS {
                continue;
            }
            let values = tags.entry(key.clone()).or_default();
            if !values.contains_key(&rendered) && values.len() >= TAGS_MAX_VALUES_PER_KEY {
                continue;
            }
            *values.entry(rendered).or_insert(0) += 1;
        }
    }
    serde_json::to_string(&tags).unwrap_or_else(|_| "{}".to_string())
}

pub struct MetadataStore {
    /// Turso forbids concurrent statement use on one connection; the worker
    /// upserts while the API reads, so every operation takes this lock.
    conn: tokio::sync::Mutex<turso::Connection>,
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
        Ok(Self {
            conn: tokio::sync::Mutex::new(conn),
        })
    }

    /// Record one more occurrence of a fingerprint (insert or update).
    pub async fn upsert_issue_occurrence(
        &self,
        occurrence: &IssueOccurrence<'_>,
    ) -> anyhow::Result<()> {
        let millis = nanos_to_millis(occurrence.ts_nanos);
        let conn = self.conn.lock().await;
        conn.execute(
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
        conn.execute(
            "INSERT INTO issue_buckets (fingerprint, bucket_ts, count)
             VALUES (?1, ?2, 1)
             ON CONFLICT(fingerprint, bucket_ts) DO UPDATE SET count = count + 1",
            (
                occurrence.fingerprint,
                millis / BUCKET_MILLIS * BUCKET_MILLIS,
            ),
        )
        .await?;
        // Tag cache: read-merge-write under the same connection lock. The
        // SELECT's statement must be dropped before the UPDATE — an UPDATE
        // executed while another statement is open on the same turso
        // connection reports success but does not persist.
        let existing = {
            let mut rows = conn
                .query(
                    "SELECT tags FROM issues WHERE fingerprint = ?1",
                    (occurrence.fingerprint,),
                )
                .await?;
            rows.next().await?.map(|row| text(&row, 0))
        };
        if let Some(existing) = existing {
            let merged = merge_tags(&existing, occurrence.attributes);
            conn.execute(
                "UPDATE issues SET tags = ?1 WHERE fingerprint = ?2",
                (merged, occurrence.fingerprint),
            )
            .await?;
        }
        Ok(())
    }

    /// Occurrence counts per step bucket since a timestamp, oldest first.
    /// Rollups are minute-grained; coarser steps are summed in SQL.
    pub async fn issue_trend(
        &self,
        fingerprint: &str,
        since_nanos: u128,
        step_seconds: u32,
    ) -> anyhow::Result<Vec<TrendPoint>> {
        let step_millis = i64::from(step_seconds.max(60)) * 1_000;
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT bucket_ts / ?3 * ?3 AS step_ts, SUM(count)
                 FROM issue_buckets
                 WHERE fingerprint = ?1 AND bucket_ts >= ?2
                 GROUP BY step_ts ORDER BY step_ts ASC",
                (fingerprint, nanos_to_millis(since_nanos), step_millis),
            )
            .await?;
        let mut points = Vec::new();
        while let Some(row) = rows.next().await? {
            points.push(TrendPoint {
                ts_nanos: millis_to_nanos(integer(&row, 0)),
                count: u64::try_from(integer(&row, 1)).unwrap_or(0),
            });
        }
        Ok(points)
    }

    /// The shared projection for every issue read.
    const ISSUE_COLUMNS: &'static str = "fingerprint, title, error_type, culprit, service, status,
         first_seen, last_seen, event_count, last_trace_id, tags";

    fn issue_from_row(row: &turso::Row) -> Issue {
        Issue {
            fingerprint: text(row, 0),
            title: text(row, 1),
            error_type: text(row, 2),
            culprit: opt_text(row, 3),
            service: text(row, 4),
            status: text(row, 5),
            first_seen_nanos: millis_to_nanos(integer(row, 6)),
            last_seen_nanos: millis_to_nanos(integer(row, 7)),
            event_count: u64::try_from(integer(row, 8)).unwrap_or(0),
            last_trace_id: opt_text(row, 9),
            tags: match opt_text(row, 10) {
                Some(tags) if !tags.is_empty() => tags,
                _ => "{}".to_string(),
            },
        }
    }

    pub async fn issues(&self, limit: usize) -> anyhow::Result<Vec<Issue>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {} FROM issues ORDER BY last_seen DESC LIMIT ?1",
                    Self::ISSUE_COLUMNS
                ),
                [Value::Integer(i64::try_from(limit).unwrap_or(i64::MAX))],
            )
            .await?;
        let mut issues = Vec::new();
        while let Some(row) = rows.next().await? {
            issues.push(Self::issue_from_row(&row));
        }
        Ok(issues)
    }

    pub async fn issue(&self, fingerprint: &str) -> anyhow::Result<Option<Issue>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {} FROM issues WHERE fingerprint = ?1",
                    Self::ISSUE_COLUMNS
                ),
                (fingerprint,),
            )
            .await?;
        Ok(rows.next().await?.map(|row| Self::issue_from_row(&row)))
    }

    pub async fn issues_by_fingerprints(
        &self,
        fingerprints: &[String],
    ) -> anyhow::Result<Vec<Issue>> {
        if fingerprints.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = (1..=fingerprints.len())
            .map(|i| format!("?{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let params: Vec<Value> = fingerprints
            .iter()
            .map(|f| Value::Text(f.clone()))
            .collect();
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {} FROM issues WHERE fingerprint IN ({placeholders})
                     ORDER BY last_seen DESC",
                    Self::ISSUE_COLUMNS
                ),
                params,
            )
            .await?;
        let mut issues = Vec::new();
        while let Some(row) = rows.next().await? {
            issues.push(Self::issue_from_row(&row));
        }
        Ok(issues)
    }

    /// Filtered, sorted, paged issue listing. One scan path: the SQL filters
    /// and orders, the tag filter applies in Rust, and the page is sliced from
    /// a window capped at [`ISSUE_SCAN_CAP`] rows — `total` is therefore exact
    /// up to that cap (plenty for a single developer machine).
    pub async fn issues_filtered(
        &self,
        filter: &IssueQuery,
        sort: IssueSortKey,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<(Vec<Issue>, usize)> {
        let mut clauses: Vec<String> = Vec::new();
        let mut params: Vec<Value> = Vec::new();
        let bind = |params: &mut Vec<Value>, value: Value| {
            params.push(value);
            format!("?{}", params.len())
        };
        if let Some(service) = &filter.service {
            let p = bind(&mut params, Value::Text(service.clone()));
            clauses.push(format!("service = {p}"));
        }
        if let Some(status) = &filter.status {
            let p = bind(&mut params, Value::Text(status.clone()));
            clauses.push(format!("status = {p}"));
        }
        if let Some(query) = &filter.query {
            let like = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
            let p = bind(&mut params, Value::Text(like));
            clauses.push(format!(
                "(title LIKE {p} ESCAPE '\\' OR error_type LIKE {p} ESCAPE '\\' \
                 OR fingerprint LIKE {p} ESCAPE '\\')"
            ));
        }
        if let Some(from) = filter.from_nanos {
            let p = bind(&mut params, Value::Integer(nanos_to_millis(from)));
            clauses.push(format!("last_seen >= {p}"));
        }
        if let Some(to) = filter.to_nanos {
            let p = bind(&mut params, Value::Integer(nanos_to_millis(to)));
            clauses.push(format!("last_seen <= {p}"));
        }
        let where_clause = if clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", clauses.join(" AND "))
        };
        let order = match sort {
            IssueSortKey::LastSeen => "last_seen DESC".to_string(),
            IssueSortKey::FirstSeen => "first_seen DESC".to_string(),
            IssueSortKey::Events => "event_count DESC".to_string(),
            IssueSortKey::Trend => {
                let since = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
                    .unwrap_or(0)
                    - 24 * 3_600_000;
                let p = bind(&mut params, Value::Integer(since));
                format!(
                    "(SELECT COALESCE(SUM(count), 0) FROM issue_buckets b
                      WHERE b.fingerprint = issues.fingerprint AND b.bucket_ts >= {p}) DESC"
                )
            }
        };
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                &format!(
                    "SELECT {} FROM issues{where_clause} ORDER BY {order} LIMIT {}",
                    Self::ISSUE_COLUMNS,
                    ISSUE_SCAN_CAP
                ),
                params,
            )
            .await?;
        let mut matched = Vec::new();
        while let Some(row) = rows.next().await? {
            matched.push(Self::issue_from_row(&row));
        }
        if let (Some(key), Some(value)) = (&filter.tag_key, &filter.tag_value) {
            matched.retain(|issue| {
                serde_json::from_str::<serde_json::Value>(&issue.tags)
                    .ok()
                    .and_then(|tags| tags.get(key).and_then(|values| values.get(value)).cloned())
                    .is_some()
            });
        }
        let total = matched.len();
        let page = matched
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();
        Ok((page, total))
    }

    pub async fn set_issue_status(&self, fingerprint: &str, status: &str) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
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
            .lock()
            .await
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
            .lock()
            .await
            .execute("DELETE FROM dashboards WHERE id = ?1", (id,))
            .await?;
        Ok(affected > 0)
    }

    pub async fn dashboards(&self) -> anyhow::Result<Vec<Dashboard>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, name, layout, created_at, updated_at
                 FROM dashboards ORDER BY updated_at DESC",
                (),
            )
            .await?;
        let mut dashboards = Vec::new();
        while let Some(row) = rows.next().await? {
            dashboards.push(Self::dashboard_from_row(&row));
        }
        Ok(dashboards)
    }

    fn dashboard_from_row(row: &turso::Row) -> Dashboard {
        Dashboard {
            id: text(row, 0),
            name: text(row, 1),
            layout: text(row, 2),
            created_at_nanos: millis_to_nanos(integer(row, 3)),
            updated_at_nanos: millis_to_nanos(integer(row, 4)),
        }
    }

    pub async fn dashboard(&self, id: &str) -> anyhow::Result<Option<Dashboard>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, name, layout, created_at, updated_at
                 FROM dashboards WHERE id = ?1",
                (id,),
            )
            .await?;
        Ok(rows.next().await?.map(|row| Self::dashboard_from_row(&row)))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("parallax-meta-test-{nanos}.db"))
    }

    fn occurrence<'a>(
        fingerprint: &'a str,
        service: &'a str,
        ts_nanos: u128,
        attributes: &'a serde_json::Value,
    ) -> IssueOccurrence<'a> {
        IssueOccurrence {
            fingerprint,
            title: format!("Error: {fingerprint}"),
            error_type: "Error",
            culprit: None,
            service,
            ts_nanos,
            trace_id: None,
            attributes,
        }
    }

    #[tokio::test]
    async fn tags_accumulate_bounded() {
        let store = MetadataStore::open(temp_db()).await.expect("open");
        let attrs = serde_json::json!({
            "http.route": "/checkout",
            "exception.message": "ignored",
            "nested": {"skip": true},
            "attempt": 3,
        });
        for _ in 0..2 {
            store
                .upsert_issue_occurrence(&occurrence("fp1", "svc", 1_000_000_000, &attrs))
                .await
                .expect("upsert");
        }
        let issue = store.issue("fp1").await.expect("issue").expect("present");
        let tags: serde_json::Value = serde_json::from_str(&issue.tags).expect("tags json");
        assert_eq!(tags["http.route"]["/checkout"], 2);
        assert_eq!(tags["attempt"]["3"], 2);
        assert!(tags.get("exception.message").is_none());
        assert!(tags.get("nested").is_none());
    }

    #[tokio::test]
    async fn filtered_issues_page_and_total() {
        let store = MetadataStore::open(temp_db()).await.expect("open");
        let attrs = serde_json::json!({"env": "dev"});
        for i in 0..5u128 {
            let fingerprint = format!("fp{i}");
            let service = if i % 2 == 0 { "alpha" } else { "beta" };
            store
                .upsert_issue_occurrence(&occurrence(
                    &fingerprint,
                    service,
                    (i + 1) * 60_000_000_000,
                    &attrs,
                ))
                .await
                .expect("upsert");
        }
        let (page, total) = store
            .issues_filtered(
                &IssueQuery {
                    service: Some("alpha".into()),
                    ..Default::default()
                },
                IssueSortKey::LastSeen,
                2,
                0,
            )
            .await
            .expect("filtered");
        assert_eq!(total, 3);
        assert_eq!(page.len(), 2);
        assert_eq!(page[0].fingerprint, "fp4"); // newest last_seen first

        let (tagged, tagged_total) = store
            .issues_filtered(
                &IssueQuery {
                    tag_key: Some("env".into()),
                    tag_value: Some("dev".into()),
                    ..Default::default()
                },
                IssueSortKey::Events,
                10,
                0,
            )
            .await
            .expect("tag filtered");
        assert_eq!(tagged_total, 5);
        assert_eq!(tagged.len(), 5);

        let (none, none_total) = store
            .issues_filtered(
                &IssueQuery {
                    query: Some("missing-needle".into()),
                    ..Default::default()
                },
                IssueSortKey::LastSeen,
                10,
                0,
            )
            .await
            .expect("query filtered");
        assert_eq!(none_total, 0);
        assert!(none.is_empty());
    }

    /// Regression guard for the turso pitfall the tag cache hit: an UPDATE
    /// executed while a SELECT statement is still open on the same connection
    /// reports success but does not persist. Read-merge-write paths must drop
    /// the reading statement first.
    #[tokio::test]
    async fn update_with_open_statement_is_lost() {
        let store = MetadataStore::open(temp_db()).await.expect("open");
        let conn = store.conn.lock().await;
        conn.execute("INSERT INTO settings (key, value) VALUES ('k', 'v1')", ())
            .await
            .expect("insert");

        // Open statement held across the UPDATE: the write is lost.
        let mut open_rows = conn
            .query("SELECT value FROM settings WHERE key = 'k'", ())
            .await
            .expect("open select");
        let _row = open_rows.next().await.expect("next").expect("row");
        conn.execute("UPDATE settings SET value = 'lost' WHERE key = 'k'", ())
            .await
            .expect("update during open statement");
        drop(open_rows);

        // Statement dropped first: the write persists.
        conn.execute("UPDATE settings SET value = 'v2' WHERE key = 'k'", ())
            .await
            .expect("update");
        let mut rows = conn
            .query("SELECT value FROM settings WHERE key = 'k'", ())
            .await
            .expect("select");
        let row = rows.next().await.expect("next").expect("row");
        assert_eq!(text(&row, 0), "v2");
    }
    #[tokio::test]
    async fn external_runs_register_once() {
        let store = MetadataStore::open(temp_db()).await.expect("open");
        store
            .ensure_run("jk-run-1", 5_000_000_000)
            .await
            .expect("ensure");
        store
            .ensure_run("jk-run-1", 9_000_000_000)
            .await
            .expect("ensure again");
        let run = store
            .run("jk-run-1")
            .await
            .expect("run")
            .expect("registered");
        assert_eq!(run.status, "external");
        assert_eq!(run.started_at_nanos, 5_000_000_000);

        // A wrapper-started run keeps its own record.
        store
            .start_run("run_cli", Some("cargo test"), 1_000_000_000)
            .await
            .expect("start");
        store
            .ensure_run("run_cli", 2_000_000_000)
            .await
            .expect("ensure existing");
        let cli_run = store.run("run_cli").await.expect("run").expect("present");
        assert_eq!(cli_run.status, "running");
        assert_eq!(cli_run.command.as_deref(), Some("cargo test"));
    }
}
