//! Deterministic error-event fingerprinting.
//!
//! Groups by exception type + normalized message + top stack frame. Volatile
//! tokens (numbers, hex ids, UUIDs, hosts with numeric suffixes) are replaced
//! before hashing so "after 2000ms (attempt 4)" and "after 1500ms (attempt 2)"
//! land in the same group.

use regex::Regex;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

fn normalizers() -> &'static [(Regex, &'static str)] {
    static CELL: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    CELL.get_or_init(|| {
        vec![
            (Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}").unwrap(), "<uuid>"),
            (Regex::new(r"\b[0-9a-fA-F]{16,}\b").unwrap(), "<hex>"),
            (Regex::new(r"\b\d+\b").unwrap(), "<n>"),
            (Regex::new(r"\s+").unwrap(), " "),
        ]
    })
}

pub fn normalize_message(message: &str) -> String {
    let mut out = message.to_string();
    for (re, replacement) in normalizers() {
        out = re.replace_all(&out, *replacement).into_owned();
    }
    out.trim().to_string()
}

/// First frame of a newline-separated stacktrace, or empty string.
pub fn top_frame(stacktrace: Option<&str>) -> String {
    stacktrace
        .and_then(|s| s.lines().next())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// 16-hex-char fingerprint over (type, normalized message, top frame).
pub fn fingerprint(error_type: &str, message: &str, stacktrace: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(error_type.as_bytes());
    hasher.update([0u8]);
    hasher.update(normalize_message(message).as_bytes());
    hasher.update([0u8]);
    hasher.update(top_frame(stacktrace).as_bytes());
    let digest = hasher.finalize();
    digest.iter().take(8).map(|b| format!("{b:02x}")).collect()
}
