//! Deterministic error-event fingerprinting.
//!
//! Graduated from `poc/evidence-loop/src/fingerprint.rs`: group by exception
//! type + normalized message + normalized top stack frame, with volatile tokens
//! (numbers, hex ids, UUIDs, selected producer slugs) normalized away so
//! "after 2000ms (attempt 4)" and "after 1500ms (attempt 2)" land in the same
//! group.

use regex::{Captures, Regex};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

struct Normalizers {
    uuid: Regex,
    long_hex: Regex,
    short_hex: Regex,
    container: Regex,
    uid_gid: Regex,
    digits: Regex,
    whitespace: Regex,
    frame_suffix: Regex,
    absolute_path: Regex,
}

fn normalizers() -> &'static Normalizers {
    static CELL: OnceLock<Normalizers> = OnceLock::new();
    CELL.get_or_init(|| {
        Normalizers {
            uuid: Regex::new(
                r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
            )
            .expect("static regex"),
            long_hex: Regex::new(r"\b[0-9a-fA-F]{16,}\b").expect("static regex"),
            short_hex: Regex::new(r"\b[0-9a-f]{6,15}\b").expect("static regex"),
            container: Regex::new(r"\bjk-[a-z0-9-]+\b").expect("static regex"),
            uid_gid: Regex::new(r"\b\d+:\d+\b").expect("static regex"),
            // No word boundaries: "2000ms" and "attempt4" must normalize too.
            digits: Regex::new(r"\d+").expect("static regex"),
            whitespace: Regex::new(r"\s+").expect("static regex"),
            frame_suffix: Regex::new(r":\d+(?::\d+)?$").expect("static regex"),
            absolute_path: Regex::new(r"/[^\s]+").expect("static regex"),
        }
    })
}

/// Normalize volatile tokens out of an error message before grouping.
pub fn normalize_message(message: &str) -> String {
    let normalizers = normalizers();
    let mut out = message.to_string();
    out = normalizers.uuid.replace_all(&out, "<uuid>").into_owned();
    out = normalizers.long_hex.replace_all(&out, "<hex>").into_owned();
    out = normalizers
        .short_hex
        .replace_all(&out, |caps: &Captures<'_>| {
            let token = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
            if token.chars().any(|c| c.is_ascii_digit()) {
                "<hex>".to_string()
            } else {
                token.to_string()
            }
        })
        .into_owned();
    out = normalizers
        .container
        .replace_all(&out, "<container>")
        .into_owned();
    out = normalizers.uid_gid.replace_all(&out, "<uid>").into_owned();
    out = normalizers.digits.replace_all(&out, "<n>").into_owned();
    out = normalizers.whitespace.replace_all(&out, " ").into_owned();
    out.trim().to_string()
}

fn collapse_path(path: &str) -> String {
    let trimmed = path.trim_end_matches([')', ',', ';']);
    let suffix = &path[trimmed.len()..];
    let components: Vec<&str> = trimmed.split('/').filter(|part| !part.is_empty()).collect();
    let collapsed = match components.as_slice() {
        [] => String::new(),
        [only] => (*only).to_string(),
        parts => format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1]),
    };
    format!("{collapsed}{suffix}")
}

fn normalize_frame(frame: &str) -> String {
    let normalizers = normalizers();
    let without_location = normalizers.frame_suffix.replace(frame.trim(), "");
    let collapsed_paths = normalizers
        .absolute_path
        .replace_all(&without_location, |caps: &Captures<'_>| {
            collapse_path(caps.get(0).map(|m| m.as_str()).unwrap_or_default())
        });
    normalize_message(&collapsed_paths)
}

/// First frame of a newline-separated stacktrace, or empty string.
pub fn top_frame(stacktrace: Option<&str>) -> String {
    stacktrace
        .and_then(|s| s.lines().next())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// 16-hex-char fingerprint over (type, normalized message, normalized top frame).
pub fn fingerprint(error_type: &str, message: &str, stacktrace: Option<&str>) -> String {
    fingerprint_with_operation(error_type, message, stacktrace, None)
}

/// 16-hex-char fingerprint with an optional structured operation component.
pub fn fingerprint_with_operation(
    error_type: &str,
    message: &str,
    stacktrace: Option<&str>,
    operation: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(error_type.as_bytes());
    hasher.update([0u8]);
    hasher.update(normalize_message(message).as_bytes());
    hasher.update([0u8]);
    hasher.update(normalize_frame(&top_frame(stacktrace)).as_bytes());
    if let Some(operation) = operation.map(str::trim).filter(|op| !op.is_empty()) {
        hasher.update([0u8]);
        hasher.update(operation.as_bytes());
    }
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

    #[test]
    fn frame_line_numbers_do_not_split() {
        let a = fingerprint(
            "redis::ConnectionTimeout",
            "connect timed out",
            Some("checkout::payment::authorize at /srv/app/src/payment.rs:184:9"),
        );
        let b = fingerprint(
            "redis::ConnectionTimeout",
            "connect timed out",
            Some("checkout::payment::authorize at /tmp/build/src/payment.rs:200"),
        );
        let c = fingerprint(
            "redis::ConnectionTimeout",
            "connect timed out",
            Some("checkout::payment::capture at /tmp/build/src/payment.rs:200"),
        );
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn broadened_volatile_tokens_group_together() {
        let a = fingerprint(
            "jackin::AttachFailed",
            "capsule attach failed for jk-qfrehkbv-holla-thearchitect uid 501:0 id a1b2c3d4",
            None,
        );
        let b = fingerprint(
            "jackin::AttachFailed",
            "capsule attach failed for jk-z9y8x7-holla-thearchitect uid 501:20 id de4dbeef",
            None,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn prose_hex_words_do_not_normalize() {
        assert_eq!(
            normalize_message("deadbe prose token"),
            "deadbe prose token"
        );
        assert_eq!(normalize_message("a1b2c3 token"), "<hex> token");
    }

    #[test]
    fn distinct_messages_without_volatile_tokens_do_not_group() {
        let a = fingerprint("redis::ConnectionTimeout", "connection timed out", None);
        let b = fingerprint("redis::ConnectionTimeout", "authentication failed", None);
        assert_ne!(a, b);
    }

    #[test]
    fn operation_partitions_same_error_message() {
        let a = fingerprint_with_operation(
            "jackin::AttachFailed",
            "capsule failed for jk-demo-one",
            None,
            Some("capsule.attach"),
        );
        let b = fingerprint_with_operation(
            "jackin::AttachFailed",
            "capsule failed for jk-demo-two",
            None,
            Some("capsule.attach"),
        );
        let c = fingerprint_with_operation(
            "jackin::AttachFailed",
            "capsule failed for jk-demo-two",
            None,
            Some("capsule.detach"),
        );
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
