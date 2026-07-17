//! `bundle-v2`: the approved Option C contract — a CloudEvents-profile
//! envelope around the **immutable** v1 dossier carried verbatim as `data`
//! (decision record: `docs/research/decisions/evidence-bundle-contract.md`,
//! approved 2026-07-17). The dossier payload is never reshaped; v1 stays
//! permanently readable and hash-verifiable under its own version-scoped
//! sorted-key hash.

use super::*;

pub const SCHEMA_VERSION_V2: &str = "bundle-v2";
/// Stable schema reference carried in every v2 envelope.
pub const SCHEMA_REF_V2: &str = "parallax/evidence/bundle-v2";
/// The only access-policy label v2 ships until plan 109 opens remote scope.
pub const ACCESS_POLICY_LOCAL: &str = "local-operator";

/// The v2 envelope. Field order is irrelevant to the hash (JCS sorts keys);
/// `data` is the v1 [`Bundle`] serialized unchanged.
#[derive(Debug, Serialize)]
pub struct EnvelopeV2 {
    pub schema_version: &'static str,
    pub bundle_id: String,
    pub schema_ref: &'static str,
    /// ISO-8601 UTC (envelope-level times are ISO-8601 by decision; the
    /// dossier payload keeps its nanosecond strings).
    pub generated_at: String,
    pub generator: &'static str,
    pub project: String,
    pub window: EnvelopeWindow,
    pub access: EnvelopeAccess,
    pub data: Bundle,
    pub canonical_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EnvelopeWindow {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize)]
pub struct EnvelopeAccess {
    pub policy: &'static str,
}

/// Inputs the deterministic v1→v2 conversion requires. `project` and
/// `window` are `Option` so callers surface what they actually have; the
/// conversion **fails closed** when either is absent — no fabricated
/// envelope fields (decision: migration_behavior).
#[derive(Debug, Clone)]
pub struct EnvelopeInputs {
    pub bundle_id: String,
    pub project: Option<String>,
    /// Inclusive evidence window in nanoseconds.
    pub window_nanos: Option<(u128, u128)>,
    pub generated_at_nanos: u128,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EnvelopeError {
    /// Conversion input lacked a project identity.
    MissingProject,
    /// Conversion input lacked the evidence window.
    MissingWindow,
    /// A timestamp was outside the representable ISO-8601 range.
    InvalidTimestamp,
    /// A reader met a version it does not support (never coerced).
    UnknownVersion(String),
    /// A reader met a document without a recognizable version field.
    Malformed(&'static str),
}

impl std::fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingProject => write!(f, "envelope conversion requires a project"),
            Self::MissingWindow => write!(f, "envelope conversion requires an evidence window"),
            Self::InvalidTimestamp => write!(f, "timestamp outside the ISO-8601 range"),
            Self::UnknownVersion(version) => write!(f, "unsupported bundle version: {version}"),
            Self::Malformed(reason) => write!(f, "malformed bundle document: {reason}"),
        }
    }
}

impl std::error::Error for EnvelopeError {}

/// Deterministic v1→v2 conversion: wraps the dossier without touching it and
/// stamps the version-scoped JCS hash. Fails closed on missing inputs.
pub fn envelope_v1(bundle: Bundle, inputs: EnvelopeInputs) -> Result<EnvelopeV2, EnvelopeError> {
    let project = inputs.project.ok_or(EnvelopeError::MissingProject)?;
    let (from, to) = inputs.window_nanos.ok_or(EnvelopeError::MissingWindow)?;
    let mut envelope = EnvelopeV2 {
        schema_version: SCHEMA_VERSION_V2,
        bundle_id: inputs.bundle_id,
        schema_ref: SCHEMA_REF_V2,
        generated_at: iso8601_utc(inputs.generated_at_nanos)?,
        generator: "parallax",
        project,
        window: EnvelopeWindow {
            from: iso8601_utc(from)?,
            to: iso8601_utc(to)?,
        },
        access: EnvelopeAccess {
            policy: ACCESS_POLICY_LOCAL,
        },
        data: bundle,
        canonical_hash: None,
    };
    envelope.canonical_hash = Some(canonical_hash_v2(&envelope));
    Ok(envelope)
}

/// Version dispatch for readers: recognizes the two supported immutable
/// versions and rejects everything else explicitly (decision:
/// unknown-version behavior — never best-effort).
pub fn document_version(document: &serde_json::Value) -> Result<&'static str, EnvelopeError> {
    let version = document
        .get("schema_version")
        .ok_or(EnvelopeError::Malformed("missing schema_version"))?
        .as_str()
        .ok_or(EnvelopeError::Malformed("schema_version is not a string"))?;
    match version {
        SCHEMA_VERSION => Ok(SCHEMA_VERSION),
        SCHEMA_VERSION_V2 => Ok(SCHEMA_VERSION_V2),
        other => Err(EnvelopeError::UnknownVersion(other.to_string())),
    }
}

/// Version-scoped v2 hash: RFC 8785 (JCS)-canonicalized SHA-256 with a
/// `sha256-jcs:` prefix a v1 verifier can never confuse with the v1
/// sorted-key `sha256:` hash. Excluded from the hashed content, mirroring
/// v1 semantics: the hash itself, the envelope generator, and the
/// per-request fields inside the payload (`generator`, `canonical_hash`,
/// `bounded`).
pub(super) fn canonical_hash_v2(envelope: &EnvelopeV2) -> String {
    let mut value = serde_json::to_value(envelope).unwrap_or_default();
    if let serde_json::Value::Object(map) = &mut value {
        map.remove("canonical_hash");
        map.remove("generator");
        if let Some(serde_json::Value::Object(data)) = map.get_mut("data") {
            data.remove("canonical_hash");
            data.remove("generator");
            data.remove("bounded");
        }
    }
    let digest = Sha256::digest(jcs(&value).as_bytes());
    format!(
        "sha256-jcs:{}",
        digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}

/// RFC 8785 (JCS) serialization: lexicographically sorted object members,
/// no insignificant whitespace, serde_json string escaping. Number
/// serialization uses serde_json's shortest round-trip (ryu) formatting,
/// which matches the RFC's ES6 `Number::toString` output for the finite
/// values evidence bundles produce; exponent-form edge cases are covered by
/// plan-104 Step 5 property tests before v2 becomes the default emit.
fn jcs(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_unstable();
            let members: Vec<String> = keys
                .into_iter()
                .map(|key| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key).unwrap_or_default(),
                        jcs(&map[key])
                    )
                })
                .collect();
            format!("{{{}}}", members.join(","))
        }
        serde_json::Value::Array(items) => {
            format!("[{}]", items.iter().map(jcs).collect::<Vec<_>>().join(","))
        }
        leaf => serde_json::to_string(leaf).unwrap_or_default(),
    }
}

/// Nanoseconds since the Unix epoch to ISO-8601 (RFC 3339) UTC.
fn iso8601_utc(nanos: u128) -> Result<String, EnvelopeError> {
    let nanos = i128::try_from(nanos).map_err(|_| EnvelopeError::InvalidTimestamp)?;
    time::OffsetDateTime::from_unix_timestamp_nanos(nanos)
        .map_err(|_| EnvelopeError::InvalidTimestamp)?
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| EnvelopeError::InvalidTimestamp)
}
