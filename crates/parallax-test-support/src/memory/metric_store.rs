//! In-memory metric store capability.

use super::*;

#[async_trait::async_trait]
impl MetricStore for MemoryStore {
    async fn metric_names(&self, range: RangeInclusive<u128>) -> anyhow::Result<Vec<String>> {
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

    async fn metric_labels(&self, name: &str) -> anyhow::Result<Vec<String>> {
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
    ) -> anyhow::Result<Vec<String>> {
        anyhow::ensure!(
            metric_group_label_allowed(label),
            "high-cardinality identifier - filter, don't group"
        );
        let labels = self.metric_labels(name).await?;
        anyhow::ensure!(
            labels.iter().any(|known| known == label),
            "unknown metric label"
        );
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
}
