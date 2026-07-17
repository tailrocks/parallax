//! In-memory trace analytics capability.

use super::*;

#[async_trait::async_trait]
impl adapter::TraceAnalyticsStore for MemoryStore {
    async fn traces_search(
        &self,
        query: &adapter::TraceQuery,
    ) -> StorageResult<adapter::TraceList> {
        trace_search::search(self, query).map_err(Into::into)
    }

    async fn trace_duration_stats(
        &self,
        query: &adapter::TraceQuery,
    ) -> StorageResult<adapter::DurationStats> {
        // Same filtered representative set as search, duration bounds and
        // paging removed (presets never feed back into themselves).
        let unbounded = adapter::TraceQuery {
            min_duration_ns: None,
            max_duration_ns: None,
            limit: usize::MAX,
            offset: 0,
            ..query.clone()
        };
        let list = trace_search::search(self, &unbounded)?;
        let mut durations: Vec<u128> = list.items.iter().map(|t| t.duration_ns).collect();
        durations.sort_unstable();
        // Exact nearest-rank percentile (small in-memory sets).
        let percentile = |p: f64| -> Option<f64> {
            if durations.is_empty() {
                return None;
            }
            let rank = (p * durations.len() as f64).ceil() as usize;
            let index = rank.clamp(1, durations.len()) - 1;
            Some(durations[index] as f64)
        };
        Ok(adapter::DurationStats {
            p50_ns: percentile(0.50),
            p95_ns: percentile(0.95),
        })
    }

    async fn trace_facets(
        &self,
        query: &adapter::TraceQuery,
    ) -> StorageResult<Vec<adapter::Facet>> {
        // Same filtered trace set as search, paging removed; counts are
        // DISTINCT traces per dimension value, empty values skipped.
        let unbounded = adapter::TraceQuery {
            limit: usize::MAX,
            offset: 0,
            ..query.clone()
        };
        let matching: HashSet<String> = trace_search::search(self, &unbounded)?
            .items
            .into_iter()
            .map(|t| t.trace_id)
            .collect();
        let inner = self.lock();
        let in_window = |ts: u128| {
            query.from_nanos.is_none_or(|from| ts >= from)
                && query.to_nanos.is_none_or(|to| ts <= to)
        };
        let mut facets = Vec::new();
        for dimension in adapter::TRACE_FACET_DIMENSIONS {
            let mut per_value: BTreeMap<String, HashSet<&str>> = BTreeMap::new();
            for span in inner
                .spans
                .iter()
                .filter(|s| in_window(s.ts_nanos) && matching.contains(&s.trace_id))
            {
                let Some(value) = trace_search::filter_observed_value(span, dimension) else {
                    continue;
                };
                if value.is_empty() {
                    continue;
                }
                per_value
                    .entry(value)
                    .or_default()
                    .insert(span.trace_id.as_str());
            }
            let mut values: Vec<FieldValueCount> = per_value
                .into_iter()
                .map(|(value, traces)| FieldValueCount {
                    value,
                    count: traces.len() as u64,
                })
                .collect();
            values.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.value.cmp(&b.value)));
            values.truncate(adapter::FACET_VALUES_CAP);
            facets.push(adapter::Facet {
                dimension: (*dimension).to_string(),
                values,
            });
        }
        Ok(facets)
    }

    async fn attribute_compare(
        &self,
        selected: RangeInclusive<u128>,
        baseline: RangeInclusive<u128>,
        service: Option<&str>,
        error_only: bool,
        keys: &[String],
        top_n: usize,
    ) -> StorageResult<Vec<AttributeCompareRow>> {
        let limit = top_n.min(ATTRIBUTE_COMPARE_TOP_N_CAP);
        if limit == 0 {
            return Ok(Vec::new());
        }

        let spans = self.lock().spans.clone();
        let candidate_keys: Vec<String> = if keys.is_empty() {
            let mut discovered = BTreeSet::new();
            for span in spans.iter().filter(|span| {
                (span_matches_compare(span, &selected, service, error_only)
                    || span_matches_compare(span, &baseline, service, error_only))
                    && span.attributes.is_object()
            }) {
                if let Some(attributes) = span.attributes.as_object() {
                    for key in attributes.keys() {
                        if attribute_compare_key_allowed(key)
                            && scalar_attribute_value(&span.attributes, key).is_some()
                        {
                            discovered.insert(key.clone());
                        }
                    }
                }
            }
            discovered
                .into_iter()
                .take(ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT)
                .collect()
        } else {
            keys.iter()
                .map(|key| key.trim())
                .filter(|key| attribute_compare_key_allowed(key))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .take(ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT)
                .map(str::to_string)
                .collect()
        };

        let mut rows = Vec::new();
        for key in candidate_keys {
            let mut selected_counts: BTreeMap<String, u64> = BTreeMap::new();
            let mut baseline_counts: BTreeMap<String, u64> = BTreeMap::new();
            let mut selected_total = 0;
            let mut baseline_total = 0;

            for span in &spans {
                if span_matches_compare(span, &selected, service, error_only)
                    && let Some(value) = scalar_attribute_value(&span.attributes, &key)
                {
                    selected_total += 1;
                    *selected_counts.entry(value).or_default() += 1;
                }
                if span_matches_compare(span, &baseline, service, error_only)
                    && let Some(value) = scalar_attribute_value(&span.attributes, &key)
                {
                    baseline_total += 1;
                    *baseline_counts.entry(value).or_default() += 1;
                }
            }

            for (value, selected_count) in selected_counts {
                let baseline_count = baseline_counts.get(&value).copied().unwrap_or(0);
                let score = attribute_compare_score(
                    selected_count,
                    selected_total,
                    baseline_count,
                    baseline_total,
                );
                if score > 0.0 {
                    rows.push(AttributeCompareRow {
                        key: key.clone(),
                        value,
                        selected_count,
                        selected_total,
                        baseline_count,
                        baseline_total,
                        score,
                    });
                }
            }
        }

        rows.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| b.selected_count.cmp(&a.selected_count))
                .then_with(|| a.key.cmp(&b.key))
                .then_with(|| a.value.cmp(&b.value))
        });
        rows.truncate(limit);
        Ok(rows)
    }

    async fn span_field_keys(&self, range: RangeInclusive<u128>) -> StorageResult<Vec<FieldKey>> {
        let spans = self.lock().spans.clone();
        let window: Vec<SpanRow> = spans
            .into_iter()
            .filter(|span| range.contains(&span.ts_nanos))
            .collect();
        let row_count = window.len() as u64;
        let mut counts: BTreeMap<String, (FieldSource, u64)> = BTreeMap::new();

        for span in &window {
            if !span.service.trim().is_empty() {
                counts
                    .entry(format!("resource.{}", semconv::SERVICE_NAME))
                    .and_modify(|(_, count)| *count += 1)
                    .or_insert((FieldSource::Resource, 1));
            }
            if let Some(attributes) = span.attributes.as_object() {
                for key in attributes.keys() {
                    if !span_field_key_allowed(key) {
                        continue;
                    }
                    if field_scalar_value(&span.attributes, key).is_some() {
                        counts
                            .entry(key.clone())
                            .and_modify(|(_, count)| *count += 1)
                            .or_insert((FieldSource::Span, 1));
                    }
                }
            }
            if let Some(resource) = span.resource.as_object() {
                for key in resource.keys() {
                    if key == semconv::SERVICE_NAME {
                        continue;
                    }
                    let exposed = format!("resource.{key}");
                    if !span_field_key_allowed(&exposed) {
                        continue;
                    }
                    if field_scalar_value(&span.resource, key).is_some() {
                        counts
                            .entry(exposed)
                            .and_modify(|(_, count)| *count += 1)
                            .or_insert((FieldSource::Resource, 1));
                    }
                }
            }
        }

        Ok(counts
            .into_iter()
            .take(FIELD_KEYS_CAP)
            .map(|(key, (source, non_null_count))| FieldKey {
                namespace: field_key_namespace(&key),
                coverage: if row_count == 0 {
                    0.0
                } else {
                    non_null_count as f64 / row_count as f64
                },
                is_identifier: field_key_identifier_like(&key),
                key,
                source,
                row_count,
                non_null_count,
            })
            .collect())
    }

    async fn span_field_stats(
        &self,
        key: &str,
        range: RangeInclusive<u128>,
        service: Option<&str>,
    ) -> StorageResult<FieldStats> {
        if !span_field_key_allowed(key) {
            return Err(adapter::StorageError::query(anyhow::anyhow!(
                "invalid field key"
            )));
        }
        let discovered = self.span_field_keys(range.clone()).await?;
        let Some(discovered_key) = discovered.iter().find(|field| field.key == key) else {
            return Err(adapter::StorageError::query(anyhow::anyhow!(
                "unknown span field key"
            )));
        };
        let (source, raw_key) = match key.strip_prefix("resource.") {
            Some(raw) => (FieldSource::Resource, raw),
            None => (FieldSource::Span, key),
        };

        let spans = self.lock().spans.clone();
        let window: Vec<SpanRow> = spans
            .into_iter()
            .filter(|span| {
                range.contains(&span.ts_nanos) && service.is_none_or(|svc| span.service == svc)
            })
            .collect();
        let row_count = window.len() as u64;
        let mut values = Vec::new();
        for span in &window {
            let value = match source {
                FieldSource::Span => field_scalar_value(&span.attributes, raw_key),
                FieldSource::Resource if raw_key == semconv::SERVICE_NAME => {
                    field_value_display(&span.service)
                }
                FieldSource::Resource => field_scalar_value(&span.resource, raw_key),
            };
            if let Some(value) = value {
                values.push(value);
            }
        }

        let non_null_count = values.len() as u64;
        let capped = values.len() > MAX_ROWS;
        let mut counts: BTreeMap<String, u64> = BTreeMap::new();
        let mut distinct = BTreeSet::new();
        for value in values.into_iter().take(MAX_ROWS) {
            distinct.insert(value.clone());
            *counts.entry(value).or_default() += 1;
        }

        let mut top_values: Vec<FieldValueCount> = counts
            .into_iter()
            .map(|(value, count)| FieldValueCount { value, count })
            .collect();
        top_values.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.value.cmp(&b.value)));
        top_values.truncate(FIELD_TOP_VALUES_CAP);
        let sample_count = non_null_count.min(MAX_ROWS as u64);
        let is_identifier = discovered_key.is_identifier
            || (sample_count >= 20 && distinct.len() as u64 >= sample_count.saturating_sub(1));

        Ok(FieldStats {
            key: key.to_string(),
            namespace: field_key_namespace(key),
            source,
            row_count,
            non_null_count,
            distinct_count: distinct.len() as u64,
            coverage: if row_count == 0 {
                0.0
            } else {
                non_null_count as f64 / row_count as f64
            },
            capped,
            is_identifier,
            top_values,
        })
    }

    async fn service_map(
        &self,
        range: RangeInclusive<u128>,
        max_traces: usize,
    ) -> StorageResult<Vec<ServiceEdge>> {
        let trace_limit = max_traces.min(SERVICE_MAP_TRACE_CAP);
        if trace_limit == 0 {
            return Ok(Vec::new());
        }

        let spans = self.lock().spans.clone();
        let mut trace_last_seen: BTreeMap<String, u128> = BTreeMap::new();
        for span in spans.iter().filter(|span| range.contains(&span.ts_nanos)) {
            trace_last_seen
                .entry(span.trace_id.clone())
                .and_modify(|last| *last = (*last).max(span.ts_nanos))
                .or_insert(span.ts_nanos);
        }
        let mut traces: Vec<_> = trace_last_seen.into_iter().collect();
        traces.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let trace_ids: BTreeSet<String> = traces
            .into_iter()
            .take(trace_limit)
            .map(|(trace_id, _)| trace_id)
            .collect();

        let by_trace_span: HashMap<(String, String), &SpanRow> = spans
            .iter()
            .filter(|span| trace_ids.contains(&span.trace_id))
            .map(|span| ((span.trace_id.clone(), span.span_id.clone()), span))
            .collect();
        let mut grouped: BTreeMap<(String, String), (u64, u64, Vec<u128>)> = BTreeMap::new();
        for span in spans.iter().filter(|span| {
            trace_ids.contains(&span.trace_id)
                && range.contains(&span.ts_nanos)
                && span.kind == "SPAN_KIND_SERVER"
        }) {
            let Some(parent_id) = span.parent_span_id.as_deref().filter(|id| !id.is_empty()) else {
                continue;
            };
            let Some(parent) = by_trace_span.get(&(span.trace_id.clone(), parent_id.to_string()))
            else {
                continue;
            };
            if parent.service == span.service {
                continue;
            }
            let entry = grouped
                .entry((parent.service.clone(), span.service.clone()))
                .or_default();
            entry.0 += 1;
            if span.status_code == "STATUS_CODE_ERROR" {
                entry.1 += 1;
            }
            entry.2.push(span.duration_ns);
        }

        Ok(grouped
            .into_iter()
            .map(
                |((source, target), (call_count, error_count, mut durations))| {
                    let p50_ms = duration_quantile_ms(&mut durations, 0.5);
                    let p95_ms = duration_quantile_ms(&mut durations, 0.95);
                    ServiceEdge {
                        source,
                        target,
                        call_count,
                        error_count,
                        p50_ms,
                        p95_ms,
                    }
                },
            )
            .collect())
    }

    async fn error_events_by_traces(
        &self,
        trace_ids: &[String],
        limit: usize,
    ) -> StorageResult<Vec<ErrorEventRow>> {
        if trace_ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut events: Vec<ErrorEventRow> = self
            .lock()
            .error_events
            .iter()
            .filter(|e| trace_ids.contains(&e.trace_id))
            .cloned()
            .collect();
        events.sort_by_key(|e| std::cmp::Reverse(e.ts_nanos));
        events.truncate(limit);
        Ok(events)
    }
}
