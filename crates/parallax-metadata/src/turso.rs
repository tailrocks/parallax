//! The metadata store: mutable product state (issues, runs, dashboards) per
//! implementation spec §6. Turso is the engine.

use parallax_model::IssueOccurrence;
use parallax_model::{
    Dashboard, Investigation, Issue, IssueQuery, IssueSortKey, RunRecord, SavedView, TrendPoint,
};
use parallax_proto::semconv;
use std::{collections::BTreeMap, path::Path};
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
CREATE TABLE IF NOT EXISTS investigations (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  state       TEXT NOT NULL,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS saved_views (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  page        TEXT NOT NULL,
  state       TEXT NOT NULL,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);
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
const ISSUE_SCAN_CAP: usize = 1000;

/// Nanosecond timestamps are stored as INTEGER milliseconds in the metadata
/// store (SQLite-class integers are i64; nanos since 1970 overflow in 2262 as
/// i64 but UI/sorting only needs millis precision here).
fn nanos_to_millis(nanos: u128) -> i64 {
    i64::try_from(nanos / 1_000_000).unwrap_or(i64::MAX)
}

fn millis_to_nanos(millis: i64) -> u128 {
    u128::try_from(millis.max(0)).unwrap_or(0) * 1_000_000
}

/// Bounds for the per-issue tag-values cache (`issues.tags`).
const TAGS_MAX_KEYS: usize = 16;
const TAGS_MAX_VALUES_PER_KEY: usize = 8;
const TAGS_MAX_VALUE_LEN: usize = 64;

/// Merge an event's scalar attributes into the `{key: {value: count}}` cache.
/// Exception keys are the event body, not tags; nested values are skipped.
fn merge_tags(existing: &str, attributes: &serde_json::Value) -> String {
    let mut tags: BTreeMap<String, BTreeMap<String, u64>> =
        serde_json::from_str(existing).unwrap_or_default();
    if let Some(map) = attributes.as_object() {
        for (key, value) in map {
            if key.starts_with(semconv::EXCEPTION_EVENT_NAME)
                && key
                    .as_bytes()
                    .get(semconv::EXCEPTION_EVENT_NAME.len())
                    .is_some_and(|byte| *byte == b'.')
            {
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

#[derive(Debug)]
pub struct TursoMetadataStore {
    /// Turso forbids concurrent statement use on one connection; the worker
    /// upserts while the API reads, so every operation takes this lock.
    conn: tokio::sync::Mutex<turso::Connection>,
}

impl TursoMetadataStore {
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
        self.upsert_issue_occurrences(std::slice::from_ref(occurrence))
            .await
    }

    /// Record many occurrences under a single connection lock.
    ///
    /// Tag-cache read-merge-write is grouped by fingerprint: one SELECT, merge
    /// every attribute set for that fingerprint, one UPDATE. Preserves the
    /// turso constraint that the SELECT statement must drop before UPDATE
    /// (same connection reports success but does not persist otherwise).
    pub async fn upsert_issue_occurrences(
        &self,
        occurrences: &[IssueOccurrence<'_>],
    ) -> anyhow::Result<()> {
        if occurrences.is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock().await;
        // Fingerprints that received at least one insert, in first-seen order,
        // so tag merge can SELECT once per fingerprint after all inserts.
        let mut tag_order: Vec<&str> = Vec::new();
        let mut tag_attrs: BTreeMap<&str, Vec<&serde_json::Value>> = BTreeMap::new();

        for occurrence in occurrences {
            let millis = nanos_to_millis(occurrence.ts_nanos);
            conn.execute(
                "INSERT INTO issues
                       (fingerprint, title, error_type, culprit, service,
                        first_seen, last_seen, event_count, last_trace_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, 1, ?7)
                     ON CONFLICT(fingerprint) DO UPDATE SET
                       first_seen = MIN(first_seen, excluded.first_seen),
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
            if !tag_attrs.contains_key(occurrence.fingerprint) {
                tag_order.push(occurrence.fingerprint);
            }
            tag_attrs
                .entry(occurrence.fingerprint)
                .or_default()
                .push(occurrence.attributes);
        }

        for fingerprint in tag_order {
            let attrs = tag_attrs
                .remove(fingerprint)
                .ok_or_else(|| anyhow::anyhow!("tag attrs missing for ordered fingerprint"))?;
            // Tag cache: read-merge-write under the same connection lock. The
            // SELECT's statement must be dropped before the UPDATE — an UPDATE
            // executed while another statement is open on the same turso
            // connection reports success but does not persist.
            let existing = {
                let mut rows = conn
                    .query(
                        "SELECT tags FROM issues WHERE fingerprint = ?1",
                        (fingerprint,),
                    )
                    .await?;
                rows.next().await?.map(|row| text(&row, 0))
            };
            if let Some(existing) = existing {
                let mut merged = existing;
                for attributes in attrs {
                    merged = merge_tags(&merged, attributes);
                }
                conn.execute(
                    "UPDATE issues SET tags = ?1 WHERE fingerprint = ?2",
                    (merged, fingerprint),
                )
                .await?;
            }
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

impl TursoMetadataStore {
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

    pub async fn investigation_save(
        &self,
        id: &str,
        name: &str,
        state: &str,
        now_nanos: u128,
    ) -> anyhow::Result<()> {
        let millis = nanos_to_millis(now_nanos);
        self.conn
            .lock()
            .await
            .execute(
                "INSERT INTO investigations (id, name, state, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name, state = excluded.state,
                   updated_at = excluded.updated_at",
                (id, name, state, millis),
            )
            .await?;
        Ok(())
    }

    pub async fn investigation_delete(&self, id: &str) -> anyhow::Result<bool> {
        let affected = self
            .conn
            .lock()
            .await
            .execute("DELETE FROM investigations WHERE id = ?1", (id,))
            .await?;
        Ok(affected > 0)
    }

    pub async fn investigations(&self) -> anyhow::Result<Vec<Investigation>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, name, state, created_at, updated_at
                 FROM investigations ORDER BY updated_at DESC",
                (),
            )
            .await?;
        let mut investigations = Vec::new();
        while let Some(row) = rows.next().await? {
            investigations.push(Self::investigation_from_row(&row));
        }
        Ok(investigations)
    }

    fn investigation_from_row(row: &turso::Row) -> Investigation {
        Investigation {
            id: text(row, 0),
            name: text(row, 1),
            state: text(row, 2),
            created_at_nanos: millis_to_nanos(integer(row, 3)),
            updated_at_nanos: millis_to_nanos(integer(row, 4)),
        }
    }

    pub async fn investigation(&self, id: &str) -> anyhow::Result<Option<Investigation>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, name, state, created_at, updated_at
                 FROM investigations WHERE id = ?1",
                (id,),
            )
            .await?;
        Ok(rows
            .next()
            .await?
            .map(|row| Self::investigation_from_row(&row)))
    }

    pub async fn saved_view_save(
        &self,
        id: &str,
        name: &str,
        page: &str,
        state: &str,
        now_nanos: u128,
    ) -> anyhow::Result<()> {
        let millis = nanos_to_millis(now_nanos);
        self.conn
            .lock()
            .await
            .execute(
                "INSERT INTO saved_views (id, name, page, state, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name, page = excluded.page, state = excluded.state,
                   updated_at = excluded.updated_at",
                (id, name, page, state, millis),
            )
            .await?;
        Ok(())
    }

    pub async fn saved_view_delete(&self, id: &str) -> anyhow::Result<bool> {
        let affected = self
            .conn
            .lock()
            .await
            .execute("DELETE FROM saved_views WHERE id = ?1", (id,))
            .await?;
        Ok(affected > 0)
    }

    pub async fn saved_views(&self, page: Option<&str>) -> anyhow::Result<Vec<SavedView>> {
        let conn = self.conn.lock().await;
        let mut saved_views = Vec::new();
        if let Some(page) = page {
            let mut rows = conn
                .query(
                    "SELECT id, name, page, state, created_at, updated_at
                     FROM saved_views WHERE page = ?1 ORDER BY updated_at DESC",
                    (page,),
                )
                .await?;
            while let Some(row) = rows.next().await? {
                saved_views.push(Self::saved_view_from_row(&row));
            }
        } else {
            let mut rows = conn
                .query(
                    "SELECT id, name, page, state, created_at, updated_at
                     FROM saved_views ORDER BY updated_at DESC",
                    (),
                )
                .await?;
            while let Some(row) = rows.next().await? {
                saved_views.push(Self::saved_view_from_row(&row));
            }
        }
        Ok(saved_views)
    }

    fn saved_view_from_row(row: &turso::Row) -> SavedView {
        SavedView {
            id: text(row, 0),
            name: text(row, 1),
            page: text(row, 2),
            state: text(row, 3),
            created_at_nanos: millis_to_nanos(integer(row, 4)),
            updated_at_nanos: millis_to_nanos(integer(row, 5)),
        }
    }

    pub async fn saved_view(&self, id: &str) -> anyhow::Result<Option<SavedView>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT id, name, page, state, created_at, updated_at
                 FROM saved_views WHERE id = ?1",
                (id,),
            )
            .await?;
        Ok(rows
            .next()
            .await?
            .map(|row| Self::saved_view_from_row(&row)))
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
mod tests;
