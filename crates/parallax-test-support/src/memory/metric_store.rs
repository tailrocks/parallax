//! In-memory metric store capability.

use super::*;

#[async_trait::async_trait]
impl MetricStore for MemoryStore {
    async fn metric_names(&self, range: RangeInclusive<u128>) -> StorageResult<Vec<String>> {
        let inner = self.lock();
        let mut names: Vec<String> = inner
            .metric_points
            .iter()
            .filter(|p| range.contains(&p.ts_nanos))
            .map(|p| p.name.clone())
            .chain(
                inner
                    .histograms
                    .iter()
                    .filter(|h| range.contains(&h.ts_nanos))
                    .map(|h| h.name.clone()),
            )
            .collect();
        names.sort();
        names.dedup();
        Ok(names)
    }

    async fn metric_labels(&self, name: &str) -> StorageResult<Vec<String>> {
        let inner = self.lock();
        let mut labels = BTreeSet::new();
        for attributes in inner
            .metric_points
            .iter()
            .filter(|point| point.name == name)
            .map(|point| &point.attributes)
            .chain(
                inner
                    .histograms
                    .iter()
                    .filter(|row| row.name == name)
                    .map(|row| &row.attributes),
            )
        {
            if let Some(object) = attributes.as_object() {
                for (key, value) in object {
                    if metric_group_label_allowed(key)
                        && matches!(
                            value,
                            serde_json::Value::String(_)
                                | serde_json::Value::Bool(_)
                                | serde_json::Value::Number(_)
                        )
                    {
                        labels.insert(key.clone());
                    }
                }
            }
        }
        Ok(labels.into_iter().collect())
    }

    async fn metric_label_values(
        &self,
        name: &str,
        label: &str,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<String>> {
        if !metric_group_label_allowed(label) {
            return Err(adapter::StorageError::query(anyhow::anyhow!(
                "high-cardinality identifier - filter, don't group"
            )));
        }
        let labels = self.metric_labels(name).await?;
        if !labels.iter().any(|known| known == label) {
            return Err(adapter::StorageError::query(anyhow::anyhow!(
                "unknown metric label"
            )));
        }
        let inner = self.lock();
        let mut values = BTreeSet::new();
        for attributes in inner
            .metric_points
            .iter()
            .filter(|point| point.name == name && range.contains(&point.ts_nanos))
            .map(|point| &point.attributes)
            .chain(
                inner
                    .histograms
                    .iter()
                    .filter(|row| row.name == name && range.contains(&row.ts_nanos))
                    .map(|row| &row.attributes),
            )
        {
            if let Some(value) = scalar_attribute_value(attributes, label) {
                values.insert(value);
                if values.len() >= 100 {
                    break;
                }
            }
        }
        Ok(values.into_iter().collect())
    }

    async fn metric_catalog(
        &self,
        range: RangeInclusive<u128>,
        q: Option<&str>,
        kind: Option<MetricKind>,
        limit: usize,
    ) -> StorageResult<Vec<MetricCatalogEntry>> {
        struct Acc {
            kind: MetricKind,
            services: BTreeSet<String>,
            last: u128,
            count: u64,
        }
        let needle = q.map(str::to_ascii_lowercase);
        let inner = self.lock();
        let mut entries: BTreeMap<String, Acc> = BTreeMap::new();
        // Finite gauge/sum samples count once each (metric-summary contract);
        // kind comes from OTLP monotonicity, which memory ingestion preserves.
        for point in inner
            .metric_points
            .iter()
            .filter(|p| range.contains(&p.ts_nanos) && p.value.is_finite())
        {
            let point_kind = if point.is_monotonic {
                MetricKind::Sum
            } else {
                MetricKind::Gauge
            };
            let acc = entries.entry(point.name.clone()).or_insert(Acc {
                kind: point_kind,
                services: BTreeSet::new(),
                last: 0,
                count: 0,
            });
            if !point.service.is_empty() {
                acc.services.insert(point.service.clone());
            }
            acc.last = acc.last.max(point.ts_nanos);
            acc.count += 1;
        }
        // One histogram export counts once, per its `_count` identity.
        for row in inner
            .histograms
            .iter()
            .filter(|h| range.contains(&h.ts_nanos))
        {
            let acc = entries.entry(row.name.clone()).or_insert(Acc {
                kind: MetricKind::Histogram,
                services: BTreeSet::new(),
                last: 0,
                count: 0,
            });
            acc.kind = MetricKind::Histogram;
            if !row.service.is_empty() {
                acc.services.insert(row.service.clone());
            }
            acc.last = acc.last.max(row.ts_nanos);
            acc.count += 1;
        }
        Ok(entries
            .into_iter()
            .filter(|(name, acc)| {
                needle
                    .as_deref()
                    .is_none_or(|n| name.to_ascii_lowercase().contains(n))
                    && kind.is_none_or(|k| k == acc.kind)
                    && acc.count > 0
            })
            .take(limit)
            .map(|(name, acc)| MetricCatalogEntry {
                name,
                kind: acc.kind,
                unit: None,
                services: acc.services.into_iter().collect(),
                last_datapoint_nanos: acc.last,
                point_count: acc.count,
            })
            .collect())
    }
}
