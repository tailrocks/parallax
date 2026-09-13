use super::*;
use crate::adapter::AttributeFilter;

impl GreptimeStore {
    pub(super) async fn span_field_columns(&self) -> anyhow::Result<Vec<SpanFieldColumn>> {
        let rows = self
            .sql(
                r#"SELECT "column_name" FROM information_schema.columns
                   WHERE "table_schema" = 'public'
                     AND "table_name" = 'opentelemetry_traces'
                     AND ("column_name" LIKE 'span_attributes.%'
                          OR "column_name" LIKE 'resource_attributes.%')
                   ORDER BY "column_name""#,
            )
            .await?;
        let mut columns = vec![SpanFieldColumn {
            key: format!("resource.{}", semconv::SERVICE_NAME),
            column: "service_name".to_string(),
            source: FieldSource::Resource,
        }];
        let service_name_key = format!("resource.{}", semconv::SERVICE_NAME);
        columns.extend(
            rows.iter()
                .filter_map(|row| span_field_column_from_name(&str_at(row, 0)))
                .filter(|column| column.key != service_name_key),
        );
        Ok(columns)
    }

    pub(super) async fn discover_span_attribute_keys(&self) -> anyhow::Result<BTreeSet<String>> {
        let rows = self
            .sql(
                r#"SELECT "column_name" FROM information_schema.columns
                   WHERE "table_schema" = 'public'
                     AND "table_name" = 'opentelemetry_traces'
                     AND "column_name" LIKE 'span_attributes.%'
                   ORDER BY "column_name""#,
            )
            .await?;
        Ok(rows
            .iter()
            .filter_map(|row| {
                str_at(row, 0)
                    .strip_prefix("span_attributes.")
                    .map(str::to_string)
            })
            .collect())
    }

    pub(super) async fn span_attribute_counts(
        &self,
        key: &str,
        range: &RangeInclusive<u128>,
        service: Option<&str>,
        error_only: bool,
    ) -> anyhow::Result<(u64, BTreeMap<String, u64>)> {
        let rows = self
            .sql_lenient(&Self::span_attribute_counts_sql(
                key, range, service, error_only,
            ))
            .await?;

        let mut total = 0;
        let mut counts = BTreeMap::new();
        for row in &rows {
            let value = str_at(row, 0);
            if !attribute_compare_value_allowed(&value) {
                continue;
            }
            let count = u128_at(row, 1) as u64;
            total += count;
            counts.insert(value, count);
        }
        Ok((total, counts))
    }

    /// Discover the base metric names from the schema: every public table that
    /// is neither a native otel table, an extension table, the metric-engine
    /// physical table, nor a system table. Histogram siblings collapse to the
    /// base name (`<name>_bucket`/`_count`/`_sum` → `<name>`), sorted unique.
    pub(super) async fn discover_metric_names(
        &self,
        range: &RangeInclusive<u128>,
    ) -> anyhow::Result<BTreeSet<String>> {
        const RESERVED: &[&str] = &[
            "opentelemetry_traces",
            "opentelemetry_traces_services",
            "opentelemetry_traces_operations",
            "opentelemetry_logs",
            "error_events",
            "invocation_metric_points",
            METRIC_EXEMPLARS_TABLE,
            EXP_HISTOGRAMS_TABLE,
            "greptime_physical_table",
        ];
        let rows = self
            .sql(
                r#"SELECT "table_name" FROM information_schema.tables
                   WHERE "table_schema" = 'public'"#,
            )
            .await?;
        let tables = rows
            .iter()
            .map(|row| str_at(row, 0))
            .filter(|table| {
                !table.is_empty()
                    && !RESERVED.contains(&table.as_str())
                    && !table.starts_with("opentelemetry_")
            })
            .collect::<Vec<_>>();
        let table_set = tables.iter().cloned().collect::<BTreeSet<_>>();
        let mut names = BTreeSet::new();
        for table in tables {
            let base = if let Some(base) = table.strip_suffix("_bucket") {
                base.to_string()
            } else if let Some(base) = table.strip_suffix("_count") {
                if table_set.contains(&format!("{base}_bucket")) {
                    base.to_string()
                } else {
                    table.clone()
                }
            } else if let Some(base) = table.strip_suffix("_sum") {
                if table_set.contains(&format!("{base}_bucket")) {
                    base.to_string()
                } else {
                    table.clone()
                }
            } else {
                table.clone()
            };
            let display = runtime_display_name(&base).unwrap_or_else(|| base.to_string());
            names.insert(canonical_metric_display_name(&display));
        }

        // Run-scoped extension rows keep the original OTLP metric name. Union
        // them so run dashboards can use dotted names even when native table
        // names are Prometheus-normalized.
        for row in self
            .sql_lenient(&format!(
                r#"SELECT DISTINCT "name" FROM invocation_metric_points
                   WHERE "name" IS NOT NULL AND "name" != ''
                     AND "ts" >= {} AND "ts" <= {}"#,
                sql_ts(*range.start()),
                sql_ts(*range.end()),
            ))
            .await?
        {
            names.insert(canonical_metric_display_name(&str_at(&row, 0)));
        }
        // Converted-exp metrics never create native tables; union them so
        // exp-only instruments are discoverable by name too.
        for row in self
            .sql_lenient(&format!(
                r#"SELECT DISTINCT "name" FROM "{EXP_HISTOGRAMS_TABLE}"
                   WHERE "name" IS NOT NULL AND "name" != ''
                     AND "ts" >= {} AND "ts" <= {}"#,
                sql_ts(*range.start()),
                sql_ts(*range.end()),
            ))
            .await?
        {
            names.insert(canonical_metric_display_name(&str_at(&row, 0)));
        }
        Ok(names)
    }

    /// Classify native metric tables into catalog families (plan 168): an
    /// explicit-histogram family collapses only when the complete
    /// `_bucket`/`_count`/`_sum` triple exists (metric-summary contract);
    /// remaining scalar tables classify Sum on the Prometheus `_total`
    /// convention, else Gauge. `stats_table` is the physical table window
    /// stats read from (`_count` sibling for histograms so one export counts
    /// once).
    pub(super) async fn discover_metric_families(&self) -> anyhow::Result<Vec<MetricFamily>> {
        const RESERVED: &[&str] = &[
            "opentelemetry_traces",
            "opentelemetry_traces_services",
            "opentelemetry_traces_operations",
            "opentelemetry_logs",
            "error_events",
            "invocation_metric_points",
            METRIC_EXEMPLARS_TABLE,
            EXP_HISTOGRAMS_TABLE,
            "greptime_physical_table",
        ];
        let rows = self
            .sql(
                r#"SELECT "table_name" FROM information_schema.tables
                   WHERE "table_schema" = 'public'"#,
            )
            .await?;
        let tables: BTreeSet<String> = rows
            .iter()
            .map(|row| str_at(row, 0))
            .filter(|table| {
                !table.is_empty()
                    && !RESERVED.contains(&table.as_str())
                    && !table.starts_with("opentelemetry_")
            })
            .collect();
        let mut families = Vec::new();
        let mut consumed: BTreeSet<String> = BTreeSet::new();
        for table in &tables {
            let Some(base) = table.strip_suffix("_bucket") else {
                continue;
            };
            let count_table = format!("{base}_count");
            let sum_table = format!("{base}_sum");
            if tables.contains(&count_table) && tables.contains(&sum_table) {
                consumed.insert(table.clone());
                consumed.insert(count_table.clone());
                consumed.insert(sum_table);
                let display = runtime_display_name(base).unwrap_or_else(|| base.to_string());
                families.push(MetricFamily {
                    display: canonical_metric_display_name(&display),
                    stats_table: count_table,
                    kind: MetricKind::Histogram,
                    exp_name: None,
                });
            }
        }
        for table in &tables {
            if consumed.contains(table) {
                continue;
            }
            let kind = if table.ends_with("_total") {
                MetricKind::Sum
            } else {
                MetricKind::Gauge
            };
            let display = runtime_display_name(table).unwrap_or_else(|| table.to_string());
            families.push(MetricFamily {
                display: canonical_metric_display_name(&display),
                stats_table: table.clone(),
                kind,
                exp_name: None,
            });
        }
        // Converted-exp families: present only when no native family shadows
        // the same display name (native explicit is authoritative on mixed
        // encodings — the same precedence the quantile path applies).
        let mut seen: BTreeSet<String> = families
            .iter()
            .map(|family| family.display.clone())
            .collect();
        for row in self
            .sql_lenient(&format!(
                r#"SELECT DISTINCT "name" FROM "{EXP_HISTOGRAMS_TABLE}" WHERE "name" IS NOT NULL"#
            ))
            .await?
        {
            let raw = str_at(&row, 0);
            if raw.is_empty() {
                continue;
            }
            let display = runtime_display_name(&raw).unwrap_or_else(|| raw.clone());
            let display = canonical_metric_display_name(&display);
            if seen.insert(display.clone()) {
                families.push(MetricFamily {
                    display,
                    stats_table: EXP_HISTOGRAMS_TABLE.to_string(),
                    kind: MetricKind::Histogram,
                    exp_name: Some(raw),
                });
            }
        }
        families.sort_by(|a, b| a.display.cmp(&b.display));
        Ok(families)
    }
}

/// One classified native metric family for the explorer catalog.
/// Per-bucket, per-service finite-sample counts across every native metric
/// family (metric-summary contract: finite scalar samples; one count per
/// histogram export via the `_count` stats table). One schema scan plus a
/// chunked UNION ALL — bounded round trips, never per-metric fan-out.
impl GreptimeStore {
    pub(super) async fn metric_point_buckets(
        &self,
        range: &RangeInclusive<u128>,
        service: Option<&str>,
        step_nanos: u128,
    ) -> StorageResult<Vec<(u128, String, u64)>> {
        const FINITE: &str = r#""greptime_value" >= -1.7976931348623157e308
                                AND "greptime_value" <= 1.7976931348623157e308"#;
        const CHUNK: usize = 24;
        let families = self.discover_metric_families().await?;
        if families.is_empty() {
            return Ok(Vec::new());
        }
        // Greptime INTERVAL seconds is not unbounded; clamp multi-day/open-ended
        // discovery steps so date_bin never receives multi-epoch second counts.
        const MAX_STEP_SECS: u128 = 86_400;
        let step_secs = (step_nanos / 1_000_000_000).clamp(1, MAX_STEP_SECS);
        let service_clause = service
            .map(|svc| format!(r#" AND "service_name" = '{}'"#, escape(svc)))
            .unwrap_or_default();
        let from_ms = range.start() / 1_000_000;
        let to_ms = range.end() / 1_000_000;
        let arms: Vec<String> = families
            .iter()
            .map(|family| {
                if let Some(exp_name) = family.exp_name.as_deref() {
                    return exp_point_buckets_arm(exp_name, step_secs, range, service);
                }
                format!(
                    r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                              AS "bucket_ms", CAST("service_name" AS STRING) AS "service",
                              COUNT("greptime_value") AS "n"
                       FROM "{}"
                       WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}
                         AND {FINITE}{service_clause}
                       GROUP BY "bucket_ms", "service""#,
                    escape_ident(&family.stats_table),
                    sql_ts(from_ms),
                    sql_ts(to_ms),
                )
            })
            .collect();
        let mut out = Vec::new();
        for chunk in arms.chunks(CHUNK) {
            let rows = self.sql_arrow_lenient(&chunk.join("\nUNION ALL\n")).await?;
            out.extend(rows.iter().map(|row| {
                (
                    u128_at(row, 0) * 1_000_000,
                    str_at(row, 1),
                    u128_at(row, 2) as u64,
                )
            }));
        }
        Ok(out)
    }
}

/// Point-bucket arm over the converted-exp extension table: one export counts
/// once (the histogram half of the metric-summary contract). The extension
/// `ts` is nanos while the fold expects ms buckets, hence the `/1000000`
/// (integer division truncates either way the engine types it).
pub(super) fn exp_point_buckets_arm(
    exp_name: &str,
    step_secs: u128,
    range: &RangeInclusive<u128>,
    service: Option<&str>,
) -> String {
    let service_clause = service
        .map(|svc| format!(r#" AND "service" = '{}'"#, escape(svc)))
        .unwrap_or_default();
    let name_filter = metric_name_sql_filter(r#""name""#, exp_name);
    format!(
        r#"SELECT CAST(CAST(date_bin(INTERVAL '{step_secs} seconds', "ts") AS BIGINT) / 1000000 AS BIGINT)
                  AS "bucket_ms", CAST("service" AS STRING) AS "service",
                  COUNT(*) AS "n"
           FROM "{EXP_HISTOGRAMS_TABLE}"
           WHERE "ts" >= {} AND "ts" <= {} AND {name_filter}{service_clause}
           GROUP BY "bucket_ms", "service""#,
        sql_ts(*range.start()),
        sql_ts(*range.end()),
    )
}

pub(super) struct MetricFamily {
    pub(super) display: String,
    pub(super) stats_table: String,
    pub(super) kind: MetricKind,
    /// Raw OTLP name for converted-exp families (stats live in the extension
    /// table, not a native table). `None` for native families.
    pub(super) exp_name: Option<String>,
}

/// A total-ordering wrapper for histogram bucket bounds (`le`), so they can key
/// a `BTreeMap`. NaN sorts last; bounds are well-formed finite values or +inf.
#[derive(PartialEq)]
pub(super) struct OrderedF64(pub(super) f64);

impl Eq for OrderedF64 {}

impl PartialOrd for OrderedF64 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Greater)
    }
}

/// SQL aggregate expression for one metric bucket. `last` needs the time
/// column to pick the latest sample; rate/increase aggregate the raw counter
/// with `sum` and post-process the bucketed series client-side.
pub(super) fn metric_agg_expr(agg: MetricAgg, value_col: &str, ts_col: &str) -> String {
    match agg {
        MetricAgg::Avg => format!(r#"avg("{value_col}")"#),
        MetricAgg::Min => format!(r#"min("{value_col}")"#),
        MetricAgg::Max => format!(r#"max("{value_col}")"#),
        MetricAgg::Sum | MetricAgg::Rate | MetricAgg::Increase => format!(r#"sum("{value_col}")"#),
        MetricAgg::Last => format!(r#"last_value("{value_col}" ORDER BY "{ts_col}")"#),
    }
}

/// Linear-interpolated quantile from native cumulative `le`-bucket counts
/// (`bound → cumulative count ≤ bound`, ascending). Mirrors the explicit-bucket
/// math the in-memory store uses, adapted to native cumulative buckets.
pub(super) fn quantile_from_cumulative(bounds: &BTreeMap<OrderedF64, f64>, q: f64) -> f64 {
    let Some((_, &total)) = bounds.iter().next_back() else {
        return 0.0;
    };
    if total <= 0.0 {
        return 0.0;
    }
    let target = q.clamp(0.0, 1.0) * total;
    let mut prev_bound = 0.0;
    let mut prev_cumulative = 0.0;
    for (OrderedF64(bound), &cumulative) in bounds {
        if cumulative >= target {
            let upper = if bound.is_finite() {
                *bound
            } else {
                prev_bound
            };
            let span = cumulative - prev_cumulative;
            let within = if span <= 0.0 {
                0.0
            } else {
                (target - prev_cumulative) / span
            };
            return prev_bound + (upper - prev_bound) * within;
        }
        prev_bound = if bound.is_finite() {
            *bound
        } else {
            prev_bound
        };
        prev_cumulative = cumulative;
    }
    prev_bound
}

/// Shared row → `ErrorEventRow` projection (fingerprint + trace-set reads).
/// Column order is the shared error-event projection: identity columns sit
/// between the span ids and the attributes.
pub(super) const ERROR_EVENT_PROJECTION: &str = r#"CAST("ts" AS BIGINT) AS "ts_nanos", "service",
                          "fingerprint", "error_type", "message", "stacktrace", "source",
                          "trace_id", "span_id", "invocation_id", "session_id",
                          "service_version", "environment", json_to_string("attributes")"#;

pub(super) fn error_event_from_row(row: &[serde_json::Value]) -> ErrorEventRow {
    ErrorEventRow {
        ts_nanos: u128_at(row, 0),
        service: str_at(row, 1),
        fingerprint: str_at(row, 2),
        error_type: str_at(row, 3),
        message: str_at(row, 4),
        stacktrace: opt_str_at(row, 5),
        source: serde_json::from_value(serde_json::Value::String(str_at(row, 6)))
            .unwrap_or(ErrorSource::LogRecord),
        trace_id: str_at(row, 7),
        span_id: str_at(row, 8),
        invocation_id: opt_str_at(row, 9),
        session_id: opt_str_at(row, 10),
        service_version: opt_str_at(row, 11),
        environment: opt_str_at(row, 12),
        attributes: json_at(row, 13),
    }
}

/// Decode one extension-table row. The bucket JSON arrays are ingest-written
/// and well-formed; anything else decodes to empty (no mass → skipped by the
/// series builders, never an error).
pub(super) fn exp_histogram_from_row(row: &[serde_json::Value]) -> HistogramRow {
    HistogramRow {
        ts_nanos: u128_at(row, 0),
        service: str_at(row, 1),
        name: str_at(row, 2),
        count: u64::try_from(u128_at(row, 3)).unwrap_or(u64::MAX),
        sum: row.get(4).and_then(|v| v.as_f64()).unwrap_or(0.0),
        bucket_counts: json_at(row, 5)
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_u64()).collect())
            .unwrap_or_default(),
        bounds: json_at(row, 6)
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_f64()).collect())
            .unwrap_or_default(),
        attributes: json_at(row, 7),
    }
}

/// Where-filter predicate over one converted row. Mirrors the memory store's
/// matcher: `service`/`service.name` reads the row service, other keys read
/// the attribute object (absent values satisfy only negative operators).
pub(super) fn exp_row_matches(
    filters: &[AttributeFilter],
    service: &str,
    attributes: &serde_json::Value,
) -> bool {
    filters.iter().all(|filter| {
        let key = filter.key.trim();
        let observed: Option<String> = if key == "service" || key == "service.name" {
            Some(service.to_string())
        } else {
            attributes.get(key).map(|value| match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
        };
        filter.matches(observed.as_deref())
    })
}

/// Latest export per step window (the native MAX-merge equivalent), then the
/// shared explicit-bucket interpolation per requested quantile.
pub(super) fn exp_quantiles_from_rows(
    rows: &[HistogramRow],
    step_nanos: u128,
    quantiles: &[f64],
) -> Vec<Vec<SeriesPoint>> {
    let latest = latest_exp_per_window(rows, step_nanos);
    quantiles
        .iter()
        .map(|q| {
            latest
                .iter()
                .map(|(ts_nanos, row)| SeriesPoint {
                    ts_nanos: *ts_nanos,
                    value: crate::adapter::explicit_bucket_quantile(
                        &row.bounds,
                        &row.bucket_counts,
                        *q,
                    ),
                })
                .collect()
        })
        .collect()
}

/// Latest cumulative export per window, then shared Δsum/Δcount math — the
/// same shape the native `_sum`/`_count` stat tables produce.
pub(super) fn exp_avg_from_rows(rows: &[HistogramRow], step_nanos: u128) -> Vec<SeriesPoint> {
    let latest = latest_exp_per_window(rows, step_nanos);
    let sums: Vec<SeriesPoint> = latest
        .iter()
        .map(|(ts_nanos, row)| SeriesPoint {
            ts_nanos: *ts_nanos,
            value: row.sum,
        })
        .collect();
    let counts: Vec<SeriesPoint> = latest
        .iter()
        .map(|(ts_nanos, row)| SeriesPoint {
            ts_nanos: *ts_nanos,
            value: row.count as f64,
        })
        .collect();
    crate::adapter::histogram_avg_from_cumulative(&sums, &counts)
}

/// Per-window new samples from cumulative exports: latest count per
/// (series, window), summed to a per-window stock, then the shared
/// reset-clamped delta (first window has no baseline and is omitted — the
/// same contract as `increase` on scalar counters).
pub(super) fn exp_counts_from_rows(rows: &[HistogramRow], step_nanos: u128) -> Vec<SeriesPoint> {
    let step = step_nanos.max(1);
    let mut latest: BTreeMap<(String, u128), (u128, f64)> = BTreeMap::new();
    for row in rows {
        let window = (row.ts_nanos / step) * step;
        // Cumulative exports are per-series; the attribute rendering is the
        // series identity (insertion-ordered JSON from one normalizer).
        let key = (row.attributes.to_string(), window);
        match latest.get(&key) {
            Some((ts, _)) if *ts >= row.ts_nanos => {}
            _ => {
                latest.insert(key, (row.ts_nanos, row.count as f64));
            }
        }
    }
    let mut stock: BTreeMap<u128, f64> = BTreeMap::new();
    for ((_, window), (_, count)) in latest {
        *stock.entry(window).or_default() += count;
    }
    let stock: Vec<SeriesPoint> = stock
        .into_iter()
        .map(|(ts_nanos, value)| SeriesPoint { ts_nanos, value })
        .collect();
    crate::adapter::increase_from_buckets(&stock)
}

fn latest_exp_per_window(rows: &[HistogramRow], step_nanos: u128) -> BTreeMap<u128, &HistogramRow> {
    let step = step_nanos.max(1);
    let mut latest: BTreeMap<u128, &HistogramRow> = BTreeMap::new();
    for row in rows {
        let window = (row.ts_nanos / step) * step;
        match latest.get(&window) {
            Some(cur) if cur.ts_nanos >= row.ts_nanos => {}
            _ => {
                latest.insert(window, row);
            }
        }
    }
    latest
}
