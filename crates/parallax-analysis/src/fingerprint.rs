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
    ansi: Regex,
}

#[expect(clippy::expect_used, reason = "static regex literal")]
fn static_regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static regex")
}

fn normalizers() -> &'static Normalizers {
    static CELL: OnceLock<Normalizers> = OnceLock::new();
    CELL.get_or_init(|| {
        Normalizers {
            uuid: static_regex(
                r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
            ),
            long_hex: static_regex(r"\b[0-9a-fA-F]{16,}\b"),
            short_hex: static_regex(r"\b[0-9a-f]{6,15}\b"),
            container: static_regex(r"\bjk-[a-z0-9-]+\b"),
            uid_gid: static_regex(r"\b\d+:\d+\b"),
            // No word boundaries: "2000ms" and "attempt4" must normalize too.
            digits: static_regex(r"\d+"),
            whitespace: static_regex(r"\s+"),
            frame_suffix: static_regex(r":\d+(?::\d+)?$"),
            absolute_path: static_regex(r"/[^\s]+"),
            ansi: static_regex(r"\x1b\[[0-9;]*[A-Za-z]"),
        }
    })
}

/// Strip terminal ANSI escape sequences (colored CLI output must group and
/// title identically to its plain form).
#[must_use]
pub fn strip_ansi(message: &str) -> String {
    normalizers().ansi.replace_all(message, "").into_owned()
}

/// Normalize volatile tokens out of an error message before grouping.
#[must_use]
pub fn normalize_message(message: &str) -> String {
    let normalizers = normalizers();
    let mut out = strip_ansi(message);
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
#[must_use]
pub fn top_frame(stacktrace: Option<&str>) -> String {
    stacktrace
        .and_then(|s| s.lines().next())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// 16-hex-char fingerprint over (type, normalized message, normalized top frame).
#[must_use]
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
mod tests;

#[cfg(test)]
mod property_tests {
    //! Plan-103: fingerprint determinism and normalization stability.
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Same inputs always yield the same 16-hex fingerprint (pure hash).
        #[test]
        fn fingerprint_is_deterministic(
            error_type in ".{0,64}",
            message in ".{0,256}",
            stack in prop::option::of(".{0,512}")
        ) {
            let a = fingerprint(&error_type, &message, stack.as_deref());
            let b = fingerprint(&error_type, &message, stack.as_deref());
            prop_assert_eq!(&a, &b);
            prop_assert_eq!(a.len(), 16);
            prop_assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        }

        /// Message normalization is a fixpoint after the first pass.
        #[test]
        fn normalize_message_is_idempotent(message in ".{0,512}") {
            let once = normalize_message(&message);
            let twice = normalize_message(&once);
            prop_assert_eq!(once, twice);
        }
    }
}
