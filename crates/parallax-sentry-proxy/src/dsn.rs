//! Sentry DSN parsing and envelope auth/DSN rewrite.

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDsn {
    pub scheme: String,
    pub public_key: String,
    pub host: String,
    pub project_id: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DsnError {
    #[error("empty DSN")]
    Empty,
    #[error("malformed DSN")]
    Malformed,
    #[error("missing envelope header line")]
    MissingHeader,
    #[error("malformed envelope header JSON")]
    MalformedHeader,
}

impl ParsedDsn {
    pub fn parse(raw: &str) -> Result<Self, DsnError> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err(DsnError::Empty);
        }
        let (scheme, rest) = raw.split_once("://").ok_or(DsnError::Malformed)?;
        let (credentials, path) = rest.split_once('/').ok_or(DsnError::Malformed)?;
        let public_key = credentials
            .split_once('@')
            .map(|(key, _host)| key.to_string())
            .ok_or(DsnError::Malformed)?;
        let host = credentials
            .split_once('@')
            .map(|(_key, host)| host.to_string())
            .ok_or(DsnError::Malformed)?;
        let project_id = path.trim_end_matches('/').to_string();
        if public_key.is_empty() || host.is_empty() || project_id.is_empty() {
            return Err(DsnError::Malformed);
        }
        Ok(Self {
            scheme: scheme.to_string(),
            public_key,
            host,
            project_id,
        })
    }

    #[must_use]
    pub fn to_dsn_string(&self) -> String {
        format!(
            "{}://{}@{}/{}",
            self.scheme, self.public_key, self.host, self.project_id
        )
    }

    #[must_use]
    pub fn envelope_url(&self) -> String {
        format!(
            "{}://{}/api/{}/envelope/",
            self.scheme, self.host, self.project_id
        )
    }

    #[must_use]
    pub fn store_url(&self) -> String {
        format!(
            "{}://{}/api/{}/store/",
            self.scheme, self.host, self.project_id
        )
    }
}

/// Rewrite the first envelope header line's `dsn` field; preserve remaining bytes.
pub fn rewrite_envelope_dsn(envelope: &[u8], destination: &ParsedDsn) -> Result<Vec<u8>, DsnError> {
    let Some(newline) = envelope.iter().position(|&b| b == b'\n') else {
        return Err(DsnError::MissingHeader);
    };
    let header_line = &envelope[..newline];
    let mut header: serde_json::Value =
        serde_json::from_slice(header_line).map_err(|_| DsnError::MalformedHeader)?;
    let Some(obj) = header.as_object_mut() else {
        return Err(DsnError::MalformedHeader);
    };
    obj.insert(
        "dsn".into(),
        serde_json::Value::String(destination.to_dsn_string()),
    );
    let mut out = serde_json::to_vec(&header).map_err(|_| DsnError::MalformedHeader)?;
    out.push(b'\n');
    out.extend_from_slice(&envelope[newline + 1..]);
    Ok(out)
}

/// Replace `sentry_key` in an inbound `X-Sentry-Auth` / `Authorization: Sentry …` value.
#[must_use]
pub fn rewrite_sentry_auth(raw: &str, public_key: &str) -> String {
    let stripped = raw
        .trim()
        .strip_prefix("Sentry ")
        .or_else(|| raw.trim().strip_prefix("sentry "))
        .unwrap_or(raw.trim());

    let mut parts: Vec<String> = stripped
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            if part.starts_with("sentry_key=") {
                format!("sentry_key={public_key}")
            } else {
                part.to_string()
            }
        })
        .collect();
    if !parts.iter().any(|part| part.starts_with("sentry_key=")) {
        parts.push(format!("sentry_key={public_key}"));
    }
    format!("Sentry {}", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_dsn() {
        let dsn = ParsedDsn::parse("https://abc123@ingest.example.com/42").expect("parse");
        assert_eq!(dsn.scheme, "https");
        assert_eq!(dsn.public_key, "abc123");
        assert_eq!(dsn.host, "ingest.example.com");
        assert_eq!(dsn.project_id, "42");
        assert_eq!(
            dsn.envelope_url(),
            "https://ingest.example.com/api/42/envelope/"
        );
    }

    #[test]
    fn rewrites_envelope_header_dsn() {
        let envelope =
            br#"{"event_id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","dsn":"https://old@proxy.example/1"}
{"type":"event","length":2}
{}
"#;
        let dest = ParsedDsn::parse("https://newkey@dest.example/9").unwrap();
        let rewritten = rewrite_envelope_dsn(envelope, &dest).unwrap();
        let first_line = rewritten.split(|&b| b == b'\n').next().unwrap();
        let header: serde_json::Value = serde_json::from_slice(first_line).unwrap();
        assert_eq!(header["dsn"], "https://newkey@dest.example/9");
        assert!(rewritten.ends_with(b"\n{}\n"));
    }

    #[test]
    fn rewrite_sentry_auth_replaces_key() {
        let raw = "Sentry sentry_version=7, sentry_client=test/1.0, sentry_key=old";
        let out = rewrite_sentry_auth(raw, "newkey");
        assert!(out.contains("sentry_key=newkey"));
        assert!(!out.contains("sentry_key=old"));
        assert!(out.contains("sentry_version=7"));
    }

    #[test]
    fn rewrite_sentry_auth_adds_missing_key() {
        let out = rewrite_sentry_auth("Sentry sentry_version=7", "k");
        assert!(out.contains("sentry_key=k"));
    }

    #[test]
    fn rewrite_sentry_auth_replaces_key_when_only_segment_has_sentry_prefix() {
        let out = rewrite_sentry_auth("Sentry sentry_key=proxy-key", "dest-key");
        assert_eq!(out, "Sentry sentry_key=dest-key");
    }
}
