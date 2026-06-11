//! Deterministic error-event fingerprinting.
//!
//! Graduated from `poc/evidence-loop/src/fingerprint.rs` with identical
//! semantics: group by exception type + normalized message + top stack frame,
//! with volatile tokens (numbers, hex ids, UUIDs) normalized away so
//! "after 2000ms (attempt 4)" and "after 1500ms (attempt 2)" land in the same
//! group.

use regex::Regex;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

fn normalizers() -> &'static [(Regex, &'static str)] {
    static CELL: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    CELL.get_or_init(|| {
        vec![
            (
                Regex::new(
                    r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
                )
                .expect("static regex"),
                "<uuid>",
            ),
            (
                Regex::new(r"\b[0-9a-fA-F]{16,}\b").expect("static regex"),
                "<hex>",
            ),
            // No word boundaries: "2000ms" and "attempt4" must normalize too.
            (Regex::new(r"\d+").expect("static regex"), "<n>"),
            (Regex::new(r"\s+").expect("static regex"), " "),
        ]
    })
}

/// Normalize volatile tokens out of an error message before grouping.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volatile_tokens_group_together() {
        let a = fingerprint(
            "redis::ConnectionTimeout",
            "timed out connecting to redis://cache-7:6379 after 2000ms (attempt 4)",
            Some("checkout::payment::authorize at src/payment.rs:184"),
        );
        let b = fingerprint(
            "redis::ConnectionTimeout",
            "timed out connecting to redis://cache-9:6379 after 1500ms (attempt 2)",
            Some("checkout::payment::authorize at src/payment.rs:184"),
        );
        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn different_types_do_not_group() {
        let a = fingerprint("TypeA", "same message", None);
        let b = fingerprint("TypeB", "same message", None);
        assert_ne!(a, b);
    }
}
