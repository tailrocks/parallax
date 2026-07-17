use super::*;

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
            });
        }
        families.sort_by(|a, b| a.display.cmp(&b.display));
        Ok(families)
    }
}

/// One classified native metric family for the explorer catalog.
pub(super) struct MetricFamily {
    pub(super) display: String,
    pub(super) stats_table: String,
    pub(super) kind: MetricKind,
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
        attributes: json_at(row, 9),
    }
}
