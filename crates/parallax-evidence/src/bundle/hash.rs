use super::*;

/// Sorted-key compact SHA-256 over evidence fields only; excludes the hash,
/// generator, and per-request bounding report.
pub(super) fn canonical_hash(bundle: &Bundle) -> String {
    let mut value = serde_json::to_value(bundle).unwrap_or_default();
    if let serde_json::Value::Object(map) = &mut value {
        map.remove("canonical_hash");
        map.remove("generator");
        map.remove("bounded");
    }
    fn canonical(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Object(map) => {
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                let inner: Vec<String> = keys
                    .into_iter()
                    .map(|k| {
                        format!(
                            "{}:{}",
                            serde_json::to_string(k).unwrap_or_default(),
                            canonical(&map[k])
                        )
                    })
                    .collect();
                format!("{{{}}}", inner.join(","))
            }
            serde_json::Value::Array(items) => {
                format!(
                    "[{}]",
                    items.iter().map(canonical).collect::<Vec<_>>().join(",")
                )
            }
            leaf => serde_json::to_string(leaf).unwrap_or_default(),
        }
    }
    let digest = Sha256::digest(canonical(&value).as_bytes());
    format!(
        "sha256:{}",
        digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}
