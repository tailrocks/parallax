//! `bundle-v2` versioned envelope (plan 104 Option C, preliminary Step 3).
//!
//! The approved contract wraps the **untouched** `bundle-v1` dossier in a
//! versioned envelope: `{kind, id}` anchor, ISO-8601 UTC envelope timestamps,
//! a version-scoped canonical hash over JCS-style canonical JSON, a permanent
//! v1 read window, deterministic fail-closed conversion, and rejection of
//! unknown versions. This module owns the envelope model, conversion, and
//! version-dispatching reader; projections (GraphQL/CLI/MCP/Markdown) are
//! Step 4 and remain unimplemented.
//!
//! Canonicalization note: keys are sorted code-point-wise and output is
//! compact UTF-8 per RFC 8785; number rendering uses serde_json's shortest
//! round-trip (ryu) form, which matches the RFC's ES6 serialization for the
//! value ranges bundles produce. The peer executor re-verifies the RFC 8785
//! §3.2.2.3 edge cases (exponent thresholds, -0) before declaring Step 5
//! equivalence.

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::bundle::SCHEMA_VERSION as V1_SCHEMA_VERSION;

/// The envelope's own immutable version string.
pub const ENVELOPE_SCHEMA_VERSION: &str = "bundle-v2";

/// Fail-closed conversion/read errors. Unknown versions and missing required
/// fields are rejected, never coerced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeError {
    /// The input was not a JSON object.
    NotAnObject,
    /// A required field is missing or has the wrong type.
    MissingField(&'static str),
    /// The payload/document declares a version this reader does not support.
    UnsupportedVersion(String),
}

impl std::fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAnObject => write!(f, "bundle document is not a JSON object"),
            Self::MissingField(field) => write!(f, "bundle document missing field: {field}"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported bundle schema version: {version}")
            }
        }
    }
}

impl std::error::Error for EnvelopeError {}

/// Envelope anchor — the approved `{kind, id}` shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnvelopeAnchor {
    pub kind: String,
    pub id: String,
}

/// The `bundle-v2` envelope around an untouched `bundle-v1` payload.
#[derive(Debug, Serialize)]
pub struct EnvelopeV2 {
    pub schema_version: &'static str,
    /// Deterministic id derived from the payload's canonical form only, so
    /// regenerating the same evidence yields the same id.
    pub bundle_id: String,
    /// ISO-8601 UTC instant the envelope was generated.
    pub generated_at: String,
    /// Copied from the v1 payload's generator.
    pub generator: String,
    pub anchor: EnvelopeAnchor,
    /// Version of the wrapped payload — always `bundle-v1` in this window.
    pub payload_schema_version: String,
    /// The untouched v1 dossier (including its own redaction report and
    /// canonical hash).
    pub payload: serde_json::Value,
    /// Version-scoped hash over the canonical envelope minus this field.
    pub canonical_hash: String,
}

/// A successfully version-dispatched bundle document. `bundle-v1` remains
/// permanently readable per the approved compatibility window.
#[derive(Debug)]
pub enum ParsedBundle {
    /// A bare v1 dossier.
    V1(serde_json::Value),
    /// A v2 envelope; the payload inside is the untouched v1 dossier.
    V2 {
        bundle_id: String,
        generated_at: String,
        anchor: EnvelopeAnchor,
        payload: serde_json::Value,
        canonical_hash: String,
    },
}

/// JCS-style canonical JSON: object keys sorted by code point, compact
/// separators, serde_json leaf rendering (see module note on numbers).
#[must_use]
pub fn canonical_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let inner: Vec<String> = keys
                .into_iter()
                .map(|key| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key).unwrap_or_default(),
                        canonical_json(&map[key])
                    )
                })
                .collect();
            format!("{{{}}}", inner.join(","))
        }
        serde_json::Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        leaf => serde_json::to_string(leaf).unwrap_or_default(),
    }
}

fn sha256_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Version-scoped canonical hash: the version string participates in the
/// digest input so a v1 hash can never collide with a v2 hash of the same
/// bytes.
#[must_use]
pub fn version_scoped_hash(version: &str, canonical: &str) -> String {
    format!("sha256:{}", sha256_hex(&format!("{version}\n{canonical}")))
}

/// Days-to-civil conversion (Howard Hinnant's algorithm) for dependency-free
/// ISO-8601 rendering.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    (
        year,
        u32::try_from(month).unwrap_or(1),
        u32::try_from(day).unwrap_or(1),
    )
}

/// Render Unix nanoseconds as an ISO-8601 UTC instant with second precision
/// (the envelope's approved timestamp form).
#[must_use]
pub fn iso8601_utc_from_nanos(nanos: u128) -> String {
    let total_secs = i64::try_from(nanos / 1_000_000_000).unwrap_or(i64::MAX);
    let days = total_secs.div_euclid(86_400);
    let secs_of_day = total_secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3_600;
    let minute = (secs_of_day % 3_600) / 60;
    let second = secs_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn required_str<'v>(
    object: &'v serde_json::Map<String, serde_json::Value>,
    field: &'static str,
) -> Result<&'v str, EnvelopeError> {
    object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or(EnvelopeError::MissingField(field))
}

fn v1_anchor(payload: &serde_json::Value) -> Result<EnvelopeAnchor, EnvelopeError> {
    let object = payload.as_object().ok_or(EnvelopeError::NotAnObject)?;
    let anchor = object
        .get("anchor")
        .and_then(serde_json::Value::as_object)
        .ok_or(EnvelopeError::MissingField("anchor"))?;
    Ok(EnvelopeAnchor {
        kind: required_str(anchor, "kind")?.to_string(),
        id: required_str(anchor, "id")?.to_string(),
    })
}

/// Wrap an untouched `bundle-v1` dossier in a `bundle-v2` envelope.
///
/// Fail-closed and deterministic: the payload must be a v1 object with a
/// well-formed anchor and generator; any other version is rejected. The same
/// payload and `generated_at_nanos` always produce byte-identical envelopes,
/// and `bundle_id` depends on the payload alone.
pub fn envelope_from_v1(
    payload: serde_json::Value,
    generated_at_nanos: u128,
) -> Result<EnvelopeV2, EnvelopeError> {
    let object = payload.as_object().ok_or(EnvelopeError::NotAnObject)?;
    let version = required_str(object, "schema_version")?;
    if version != V1_SCHEMA_VERSION {
        return Err(EnvelopeError::UnsupportedVersion(version.to_string()));
    }
    let generator = required_str(object, "generator")?.to_string();
    let anchor = v1_anchor(&payload)?;

    let payload_canonical = canonical_json(&payload);
    let bundle_id = format!("b2-{}", &sha256_hex(&payload_canonical)[..16]);

    let mut envelope = EnvelopeV2 {
        schema_version: ENVELOPE_SCHEMA_VERSION,
        bundle_id,
        generated_at: iso8601_utc_from_nanos(generated_at_nanos),
        generator,
        anchor,
        payload_schema_version: V1_SCHEMA_VERSION.to_string(),
        payload,
        canonical_hash: String::new(),
    };
    let mut value = serde_json::to_value(&envelope).unwrap_or_default();
    if let serde_json::Value::Object(map) = &mut value {
        map.remove("canonical_hash");
    }
    envelope.canonical_hash = version_scoped_hash(ENVELOPE_SCHEMA_VERSION, &canonical_json(&value));
    Ok(envelope)
}

/// Version-dispatching reader: accepts bare `bundle-v1` dossiers (permanent
/// read window) and `bundle-v2` envelopes whose payload is v1; rejects
/// everything else.
pub fn parse_bundle_json(value: serde_json::Value) -> Result<ParsedBundle, EnvelopeError> {
    let object = value.as_object().ok_or(EnvelopeError::NotAnObject)?;
    let version = required_str(object, "schema_version")?;
    match version {
        v if v == V1_SCHEMA_VERSION => Ok(ParsedBundle::V1(value)),
        v if v == ENVELOPE_SCHEMA_VERSION => {
            let bundle_id = required_str(object, "bundle_id")?.to_string();
            let generated_at = required_str(object, "generated_at")?.to_string();
            let canonical_hash = required_str(object, "canonical_hash")?.to_string();
            let anchor_object = object
                .get("anchor")
                .and_then(serde_json::Value::as_object)
                .ok_or(EnvelopeError::MissingField("anchor"))?;
            let anchor = EnvelopeAnchor {
                kind: required_str(anchor_object, "kind")?.to_string(),
                id: required_str(anchor_object, "id")?.to_string(),
            };
            let payload = object
                .get("payload")
                .cloned()
                .ok_or(EnvelopeError::MissingField("payload"))?;
            let payload_version = required_str(object, "payload_schema_version")?;
            if payload_version != V1_SCHEMA_VERSION {
                return Err(EnvelopeError::UnsupportedVersion(
                    payload_version.to_string(),
                ));
            }
            Ok(ParsedBundle::V2 {
                bundle_id,
                generated_at,
                anchor,
                payload,
                canonical_hash,
            })
        }
        other => Err(EnvelopeError::UnsupportedVersion(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn v1_payload() -> serde_json::Value {
        json!({
            "schema_version": "bundle-v1",
            "generator": "parallax",
            "anchor": { "kind": "issue", "id": "fp-1" },
            "issue": null,
            "logs": ["line"],
            "hypotheses": [],
            "missing_evidence": [],
            "redaction": { "policy": "default", "redacted_counts": {} },
            "bounded": { "max_tokens": 4000, "estimated_tokens": 10,
                          "dropped_log_lines": 0, "truncated_stacktrace": false },
            "canonical_hash": "sha256:abc"
        })
    }

    const NANOS_2026: u128 = 1_768_953_600_000_000_000; // 2026-01-21T00:00:00Z

    #[test]
    fn conversion_is_deterministic_and_payload_untouched() {
        let a = envelope_from_v1(v1_payload(), NANOS_2026).expect("envelope");
        let b = envelope_from_v1(v1_payload(), NANOS_2026).expect("envelope");
        assert_eq!(
            serde_json::to_string(&a).expect("json"),
            serde_json::to_string(&b).expect("json")
        );
        assert_eq!(a.payload, v1_payload());
        assert_eq!(a.schema_version, "bundle-v2");
        assert_eq!(a.payload_schema_version, "bundle-v1");
        assert_eq!(a.anchor.kind, "issue");
        assert_eq!(a.anchor.id, "fp-1");
        assert!(a.bundle_id.starts_with("b2-"));
        assert!(a.canonical_hash.starts_with("sha256:"));
    }

    #[test]
    fn bundle_id_depends_on_payload_only() {
        let a = envelope_from_v1(v1_payload(), NANOS_2026).expect("envelope");
        let b = envelope_from_v1(v1_payload(), NANOS_2026 + 60_000_000_000).expect("envelope");
        assert_eq!(a.bundle_id, b.bundle_id);
        // …but the envelope hash covers generated_at, so it differs.
        assert_ne!(a.canonical_hash, b.canonical_hash);
    }

    #[test]
    fn unknown_versions_are_rejected_fail_closed() {
        let mut wrong = v1_payload();
        wrong["schema_version"] = json!("bundle-v3");
        assert!(matches!(
            envelope_from_v1(wrong.clone(), NANOS_2026),
            Err(EnvelopeError::UnsupportedVersion(v)) if v == "bundle-v3"
        ));
        assert!(matches!(
            parse_bundle_json(wrong),
            Err(EnvelopeError::UnsupportedVersion(_))
        ));
        assert!(matches!(
            parse_bundle_json(json!([1, 2])),
            Err(EnvelopeError::NotAnObject)
        ));
    }

    #[test]
    fn missing_required_fields_fail_closed() {
        let mut no_anchor = v1_payload();
        no_anchor.as_object_mut().expect("object").remove("anchor");
        assert!(matches!(
            envelope_from_v1(no_anchor, NANOS_2026),
            Err(EnvelopeError::MissingField("anchor"))
        ));
        let mut no_generator = v1_payload();
        no_generator
            .as_object_mut()
            .expect("object")
            .remove("generator");
        assert!(matches!(
            envelope_from_v1(no_generator, NANOS_2026),
            Err(EnvelopeError::MissingField("generator"))
        ));
    }

    #[test]
    fn v1_documents_remain_permanently_readable() {
        match parse_bundle_json(v1_payload()).expect("parse") {
            ParsedBundle::V1(value) => assert_eq!(value, v1_payload()),
            ParsedBundle::V2 { .. } => panic!("v1 must parse as V1"),
        }
    }

    #[test]
    fn v2_round_trips_through_the_reader() {
        let envelope = envelope_from_v1(v1_payload(), NANOS_2026).expect("envelope");
        let value = serde_json::to_value(&envelope).expect("value");
        match parse_bundle_json(value).expect("parse") {
            ParsedBundle::V2 {
                bundle_id,
                anchor,
                payload,
                canonical_hash,
                generated_at,
            } => {
                assert_eq!(bundle_id, envelope.bundle_id);
                assert_eq!(anchor, envelope.anchor);
                assert_eq!(payload, v1_payload());
                assert_eq!(canonical_hash, envelope.canonical_hash);
                assert_eq!(generated_at, "2026-01-21T00:00:00Z");
            }
            ParsedBundle::V1(_) => panic!("v2 must parse as V2"),
        }
    }

    #[test]
    fn hash_excludes_itself_and_is_version_scoped() {
        let envelope = envelope_from_v1(v1_payload(), NANOS_2026).expect("envelope");
        let mut value = serde_json::to_value(&envelope).expect("value");
        value
            .as_object_mut()
            .expect("object")
            .remove("canonical_hash");
        let recomputed = version_scoped_hash(ENVELOPE_SCHEMA_VERSION, &canonical_json(&value));
        assert_eq!(recomputed, envelope.canonical_hash);
        let other_scope = version_scoped_hash("bundle-v1", &canonical_json(&value));
        assert_ne!(other_scope, envelope.canonical_hash);
    }

    #[test]
    fn iso8601_rendering_known_instants() {
        assert_eq!(iso8601_utc_from_nanos(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso8601_utc_from_nanos(NANOS_2026), "2026-01-21T00:00:00Z");
        // Leap-year boundary: 2024-02-29T23:59:59Z.
        assert_eq!(
            iso8601_utc_from_nanos(1_709_251_199_000_000_000),
            "2024-02-29T23:59:59Z"
        );
    }

    #[test]
    fn canonical_json_sorts_keys_and_is_compact() {
        let value = json!({ "b": 1, "a": { "d": [2, 3], "c": "x" } });
        assert_eq!(canonical_json(&value), r#"{"a":{"c":"x","d":[2,3]},"b":1}"#);
    }

    mod property_tests {
        //! Plan-103: canonical JSON is a fixpoint; version-scoped hashes are stable.
        use super::*;
        use proptest::collection::{btree_map, vec};
        use proptest::prelude::*;

        fn arb_leaf() -> impl Strategy<Value = serde_json::Value> {
            prop_oneof![
                Just(serde_json::Value::Null),
                any::<bool>().prop_map(serde_json::Value::Bool),
                (-1_000_000i64..1_000_000i64).prop_map(|n| json!(n)),
                ".*".prop_map(serde_json::Value::String),
            ]
        }

        fn arb_json(depth: u32) -> BoxedStrategy<serde_json::Value> {
            if depth == 0 {
                return arb_leaf().boxed();
            }
            prop_oneof![
                arb_leaf(),
                vec(arb_json(depth - 1), 0..4).prop_map(serde_json::Value::Array),
                btree_map(".{0,12}", arb_json(depth - 1), 0..4)
                    .prop_map(|map| { serde_json::Value::Object(map.into_iter().collect()) }),
            ]
            .boxed()
        }

        proptest! {
            /// Re-canonicalizing the parse of `canonical_json` yields the same bytes.
            #[test]
            fn canonical_json_is_a_fixpoint(value in arb_json(3)) {
                let once = canonical_json(&value);
                let parsed: serde_json::Value =
                    serde_json::from_str(&once).expect("canonical JSON must re-parse");
                prop_assert_eq!(canonical_json(&parsed), once.clone());
                let hash_a = version_scoped_hash("bundle-v2", &once);
                let hash_b = version_scoped_hash("bundle-v2", &canonical_json(&parsed));
                prop_assert_eq!(hash_a, hash_b);
            }
        }
    }
}
