//! Tier-1 redaction engine and default-deny source policy.
//!
//! Extracted from `parallax-evidence` so lower-tier crates (metadata
//! persistence, evidence projection) share one detector engine without an
//! inverted dependency. Pure text in, redacted text plus hit counts out.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use regex::Regex;
use serde::Serialize;

/// Detector policy identifier stamped into every report.
pub const REDACTION_POLICY_V1: &str = "redaction-lite-v3";

/// Per-detector hit counts for one projection pass.
#[derive(Debug, Serialize)]
pub struct RedactionReport {
    pub policy: &'static str,
    pub redacted_counts: BTreeMap<&'static str, u64>,
}

impl Default for RedactionReport {
    fn default() -> Self {
        Self {
            policy: REDACTION_POLICY_V1,
            redacted_counts: BTreeMap::new(),
        }
    }
}

#[expect(clippy::expect_used, reason = "static regex literal")]
fn static_regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static regex")
}

#[expect(
    clippy::too_many_lines,
    reason = "static detector table; keep rules co-located"
)]
fn redaction_rules() -> &'static [(&'static str, Regex, &'static str)] {
    static CELL: OnceLock<Vec<(&'static str, Regex, &'static str)>> = OnceLock::new();
    CELL.get_or_init(|| {
        vec![
            (
                "dsn_userinfo",
                static_regex(r"://[^/\s:@]+:[^/\s@]+@"),
                "://[REDACTED:dsn_userinfo]@",
            ),
            (
                "private_key_block",
                static_regex(
                    r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----",
                ),
                "[REDACTED:private_key_block]",
            ),
            (
                "github_token",
                static_regex(r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{20,}\b"),
                "[REDACTED:github_token]",
            ),
            (
                "github_pat",
                static_regex(r"\bgithub_pat_[A-Za-z0-9_]{20,}\b"),
                "[REDACTED:github_pat]",
            ),
            (
                "slack_token",
                static_regex(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b"),
                "[REDACTED:slack_token]",
            ),
            (
                "jwt",
                static_regex(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b"),
                "[REDACTED:jwt]",
            ),
            (
                "aws_access_key_id",
                static_regex(r"\bAKIA[0-9A-Z]{16}\b"),
                "[REDACTED:aws_access_key_id]",
            ),
            (
                "aws_secret_access_key",
                static_regex(r"(?i)\baws[_.-]?secret[_.-]?access[_.-]?key\b\s*[=:]\s*\S+"),
                "[REDACTED:aws_secret_access_key]",
            ),
            (
                "bearer_token",
                static_regex(r"Bearer\s+[A-Za-z0-9._\-]{8,}"),
                "[REDACTED:bearer_token]",
            ),
            (
                "stripe_live_key",
                static_regex(r"\bsk_live_[A-Za-z0-9_-]{10,}\b"),
                "[REDACTED:stripe_live_key]",
            ),
            (
                "stripe_test_key",
                static_regex(r"\bsk_test_[A-Za-z0-9_-]{10,}\b"),
                "[REDACTED:stripe_test_key]",
            ),
            (
                "anthropic_api_key",
                static_regex(r"\bsk-ant-[A-Za-z0-9_-]{10,}\b"),
                "[REDACTED:anthropic_api_key]",
            ),
            (
                "openai_api_key",
                static_regex(r"\bsk-[A-Za-z0-9]{20,}\b"),
                "[REDACTED:openai_api_key]",
            ),
            (
                "google_api_key",
                static_regex(r"\bAIza[0-9A-Za-z_-]{10,}\b"),
                "[REDACTED:google_api_key]",
            ),
            (
                "gitlab_pat",
                static_regex(r"\bglpat-[A-Za-z0-9_-]{10,}\b"),
                "[REDACTED:gitlab_pat]",
            ),
            (
                "npm_token",
                static_regex(r"\bnpm_[A-Za-z0-9_-]{10,}\b"),
                "[REDACTED:npm_token]",
            ),
            (
                "basic_auth",
                // Padding ends in a non-word character, so a trailing word
                // boundary would miss ordinary Base64 credentials.
                static_regex(r"(?i)\bBasic\s+[A-Za-z0-9+/]{8,}={0,2}"),
                "[REDACTED:basic_auth]",
            ),
            (
                "password_assignment",
                static_regex(r"(?i)password\s*[=:]\s*\S+"),
                "[REDACTED:password_assignment]",
            ),
            (
                "generic_secret_assignment",
                // Exclude bare `auth` (collides with `auth=Bearer …` after the
                // bearer rule rewrites the token). Values must not start with
                // `[` so already-redacted markers are not re-matched.
                static_regex(
                    r#"(?i)\b(?:api[_-]?key|apikey|secret|token|passwd|pwd|access[_-]?key)\b\s*[=:]\s*[^\s"'\[\]]{6,}"#,
                ),
                "[REDACTED:generic_secret_assignment]",
            ),
            (
                "email_address",
                static_regex(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b"),
                "[REDACTED:email_address]",
            ),
        ]
    })
}

/// Apply every detector rule plus control-character stripping, counting
/// hits into `report`.
///
/// Control characters (except `\n`/`\r`/`\t`) are stripped **before** detector
/// rules. Stripping after detectors broke the `sanitize_text` fixpoint: a
/// control character could mask a detector match on pass 1, then after strip
/// the same secret pattern fired on pass 2 (fuzz crash
/// `C@\u{6}2.srea@T` → pass1 `C@2.srea@T`, pass2 email redaction).
pub fn redact(text: &str, report: &mut RedactionReport) -> String {
    let mut out = text.to_string();
    let control_characters = out
        .chars()
        .filter(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
        .count() as u64;
    if control_characters > 0 {
        out.retain(|character| !character.is_control() || matches!(character, '\n' | '\r' | '\t'));
        *report
            .redacted_counts
            .entry("control_character")
            .or_insert(0) += control_characters;
    }
    for (name, rule, replacement) in redaction_rules() {
        let hits = rule.find_iter(&out).count() as u64;
        if hits > 0 {
            out = rule.replace_all(&out, *replacement).into_owned();
            *report.redacted_counts.entry(name).or_insert(0) += hits;
        }
    }
    out
}

// Versioned default-deny source minimization for agent-visible evidence.
//
// This is the typed policy seam for plan 111. Detector execution remains in
// `bundle::redaction`; callers must not project a field before consulting this
// policy. Unknown fields default to drop, never pass-through.

use std::panic::{AssertUnwindSafe, catch_unwind};

pub const SOURCE_POLICY_VERSION: &str = "evidence-source-v1";
pub const DETECTOR_POLICY_VERSION: &str = "detectors-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceField {
    AnchorId,
    IssueTitle,
    IssueErrorType,
    IssueCulprit,
    ServiceName,
    InvocationCommand,
    InvocationMode,
    InvocationOutcome,
    EventMessage,
    EventStacktrace,
    SpanName,
    SpanKind,
    SpanStatus,
    DatabaseQueryText,
    LogBody,
    Timestamp,
    TraceId,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceAction {
    ValidateStructural,
    RedactText,
    Drop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceDecision {
    pub rule_id: &'static str,
    pub action: SourceAction,
}

#[must_use]
pub const fn decide(field: EvidenceField) -> SourceDecision {
    match field {
        EvidenceField::IssueTitle
        | EvidenceField::IssueCulprit
        | EvidenceField::InvocationCommand
        | EvidenceField::EventMessage
        | EvidenceField::EventStacktrace
        | EvidenceField::SpanName
        | EvidenceField::DatabaseQueryText
        | EvidenceField::LogBody => SourceDecision {
            rule_id: "source.text.redact",
            action: SourceAction::RedactText,
        },
        EvidenceField::AnchorId
        | EvidenceField::IssueErrorType
        | EvidenceField::ServiceName
        | EvidenceField::InvocationMode
        | EvidenceField::InvocationOutcome
        | EvidenceField::SpanKind
        | EvidenceField::SpanStatus
        | EvidenceField::Timestamp
        | EvidenceField::TraceId => SourceDecision {
            rule_id: "source.structural.validate",
            action: SourceAction::ValidateStructural,
        },
        EvidenceField::Unknown => SourceDecision {
            rule_id: "source.unknown.drop",
            action: SourceAction::Drop,
        },
    }
}

/// Apply source policy then detectors. `None` means the field is stripped
/// (unknown / drop / detector failure). Never returns raw unknown fields.
#[must_use]
pub fn project_text(
    field: EvidenceField,
    value: &str,
    report: &mut RedactionReport,
) -> Option<String> {
    match decide(field).action {
        SourceAction::Drop => {
            *report
                .redacted_counts
                .entry("source.unknown.drop")
                .or_insert(0) += 1;
            None
        }
        SourceAction::ValidateStructural => {
            // Structural identifiers must not smuggle secrets. If detectors
            // fire, keep the sanitized form rather than the raw input.
            match safe_redact(value, report) {
                Ok(cleaned) => Some(cleaned),
                Err(()) => {
                    *report
                        .redacted_counts
                        .entry("detector_failure")
                        .or_insert(0) += 1;
                    None
                }
            }
        }
        SourceAction::RedactText => match safe_redact(value, report) {
            Ok(cleaned) => Some(cleaned),
            Err(()) => {
                *report
                    .redacted_counts
                    .entry("detector_failure")
                    .or_insert(0) += 1;
                None
            }
        },
    }
}

/// Sanitize free-form text for durable storage (issue title/culprit).
/// Fail-closed: detector panic yields a structural placeholder.
#[must_use]
pub fn sanitize_text(value: &str) -> String {
    let mut report = RedactionReport {
        policy: SOURCE_POLICY_VERSION,
        ..Default::default()
    };
    match safe_redact(value, &mut report) {
        Ok(cleaned) => cleaned,
        Err(()) => "[REDACTED:detector_failure]".to_string(),
    }
}

fn safe_redact(value: &str, report: &mut RedactionReport) -> Result<String, ()> {
    catch_unwind(AssertUnwindSafe(|| redact(value, report))).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostile_text_fields_always_route_through_detectors() {
        for field in [
            EvidenceField::IssueTitle,
            EvidenceField::IssueCulprit,
            EvidenceField::InvocationCommand,
            EvidenceField::EventMessage,
            EvidenceField::EventStacktrace,
            EvidenceField::SpanName,
            EvidenceField::DatabaseQueryText,
            EvidenceField::LogBody,
        ] {
            assert_eq!(decide(field).action, SourceAction::RedactText, "{field:?}");
        }
    }

    #[test]
    fn unknown_field_is_dropped_not_passed_through() {
        assert_eq!(SOURCE_POLICY_VERSION, "evidence-source-v1");
        assert_eq!(DETECTOR_POLICY_VERSION, "detectors-v1");
        assert_eq!(
            decide(EvidenceField::Unknown),
            SourceDecision {
                rule_id: "source.unknown.drop",
                action: SourceAction::Drop,
            }
        );
        let mut report = RedactionReport {
            policy: SOURCE_POLICY_VERSION,
            ..Default::default()
        };
        assert_eq!(
            project_text(EvidenceField::Unknown, "anything-secret", &mut report),
            None
        );
        assert_eq!(report.redacted_counts.get("source.unknown.drop"), Some(&1));
    }

    #[test]
    fn structural_strings_require_validation_not_raw_passthrough() {
        for field in [
            EvidenceField::AnchorId,
            EvidenceField::IssueErrorType,
            EvidenceField::ServiceName,
            EvidenceField::InvocationMode,
            EvidenceField::InvocationOutcome,
            EvidenceField::SpanKind,
            EvidenceField::SpanStatus,
            EvidenceField::Timestamp,
            EvidenceField::TraceId,
        ] {
            assert_eq!(
                decide(field).action,
                SourceAction::ValidateStructural,
                "{field:?}"
            );
        }
    }

    #[test]
    fn a6_public_safe_canaries_are_not_projected_raw_by_detectors() {
        // Public-safe canaries only — never real secrets / provider-shaped live keys.
        let canaries = [
            "Bearer ghp_0123456789ABCDEFGHIJKLMNOPQRST",
            "stripe=sk_live_XXXXXXXXXXXXXXXXXXXX",
            "postgres://user:p@ssw0rd@db.example:5432/app",
            "-----BEGIN PRIVATE KEY-----\nMIIE\n-----END PRIVATE KEY-----",
            "Authorization: Basic dXNlcjpwYXNzd29yZHh4eHg=",
            "api_key=supersecretvalue",
        ];
        for sample in canaries {
            let mut report = RedactionReport {
                policy: SOURCE_POLICY_VERSION,
                ..Default::default()
            };
            let out = project_text(EvidenceField::EventMessage, sample, &mut report)
                .expect("hostile text projects to a redacted string");
            assert!(
                !out.contains("ghp_0123456789")
                    && !out.contains("sk_live_XXXXXXXXXXXXXXXXXXXX")
                    && !out.contains("p@ssw0rd")
                    && !out.contains("BEGIN PRIVATE KEY")
                    && !out.contains("dXNlcjpwYXNzd29yZHh4eHg=")
                    && !out.contains("supersecretvalue"),
                "canary leaked through redaction: {sample:?} -> {out:?}"
            );
            assert!(
                report.redacted_counts.values().any(|c| *c > 0) || out.contains("REDACTED"),
                "expected redaction counts for {sample:?}, got {out:?}"
            );
        }
    }

    #[test]
    fn sanitize_text_masks_title_like_canaries_for_storage() {
        let title = "timeout postgres://admin:s3cr3t@db/app";
        let out = sanitize_text(title);
        assert!(!out.contains("s3cr3t"), "{out}");
        assert!(out.contains("[REDACTED:dsn_userinfo]"), "{out}");
    }

    mod property_tests {
        //! Plan-103: redaction idempotence (docs/research/testing/property-invariants.md).
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// `sanitize_text` is a fixpoint: re-running never rewrites output.
            #[test]
            fn sanitize_text_is_idempotent(input in ".*") {
                prop_assume!(input.len() <= 4_096);
                let once = sanitize_text(&input);
                let twice = sanitize_text(&once);
                prop_assert_eq!(once, twice);
            }
        }

        /// Minimized corpus from scheduled-measurement run 29582532812
        /// (`fuzz/artifacts/redaction_text/crash-6839e8e7…`).
        #[test]
        fn fuzz_crash_control_char_masks_email_then_fires() {
            let input = "C@\u{6}2.srea@T";
            let once = sanitize_text(input);
            let twice = sanitize_text(&once);
            assert_eq!(once, twice, "once={once:?} twice={twice:?}");
            // Control strip first so the email detector fires on pass 1.
            assert_eq!(once, "[REDACTED:email_address]@T");
        }
    }
}

#[cfg(test)]
mod engine_tests {
    use super::*;

    #[test]
    fn basic_authorization_with_base64_padding_is_redacted() {
        let mut report = RedactionReport {
            policy: "test",
            ..Default::default()
        };
        let output = redact("Authorization: Basic dXNlcjpwYXNzd29yZHh4eHg=", &mut report);
        assert_eq!(output, "Authorization: [REDACTED:basic_auth]");
        assert_eq!(report.redacted_counts.get("basic_auth"), Some(&1));
    }

    #[test]
    fn terminal_controls_are_removed_but_text_whitespace_is_preserved() {
        let mut report = RedactionReport {
            policy: "test",
            ..Default::default()
        };
        let output = redact("safe\n\t\u{1b}[31mred\u{0}\u{7f}\rtext", &mut report);

        assert_eq!(output, "safe\n\t[31mred\rtext");
        assert_eq!(report.redacted_counts.get("control_character"), Some(&3));
    }

    /// Public-safe canaries only. Never live provider-shaped secrets.
    const DETECTOR_ROWS: &[(&str, &str, &str)] = &[
        (
            "dsn_userinfo",
            "postgres://alice:s3cret@db.example:5432/app",
            "postgres host db.example has no userinfo",
        ),
        (
            "private_key_block",
            "-----BEGIN PRIVATE KEY-----\nMIIE\n-----END PRIVATE KEY-----",
            "no private key here, just a comment",
        ),
        ("github_token", "ghp_0123456789ABCDEFGHIJKLMN", "ghp short"),
        (
            "github_pat",
            "github_pat_0123456789ABCDEFGHIJ",
            "github_pat_short",
        ),
        ("slack_token", "xoxb-1234567890-abcdefghij", "xoxb-short"),
        (
            "jwt",
            "eyJhbGciOiJub25lIn0.eyJzdWIiOiJ4eHh4eHh4eHgifQ.xxxxxxxxxxxxxxxxxxxx",
            "eyJnot.a.jwt",
        ),
        ("aws_access_key_id", "AKIAIOSFODNN7EXAMPLE", "AKIASHORT"),
        (
            "aws_secret_access_key",
            "aws_secret_access_key=wJalrXUtnFEMIEXAMPLEKEY",
            "aws_access_key_id looks similar but is not the secret assignment",
        ),
        (
            "bearer_token",
            "Bearer abcdefghijklmnop",
            "bearer token without the scheme prefix",
        ),
        (
            "stripe_live_key",
            "sk_live_XXXXXXXXXXXXXXXXXXXX",
            "sk_live_short",
        ),
        (
            "stripe_test_key",
            "sk_test_XXXXXXXXXXXXXXXXXXXX",
            "sk_test_short",
        ),
        (
            "anthropic_api_key",
            "sk-ant-api03-XXXXXXXXXX",
            "sk-ant-short",
        ),
        (
            "openai_api_key",
            "sk-abcdefghijklmnopqrstuvwxyz12",
            "sk-tooshort",
        ),
        ("google_api_key", "AIzaSyDummyKeyValueHere", "AIzaShort"),
        ("gitlab_pat", "glpat-xxxxxxxxxxxxxxxxxxxx", "glpat-short"),
        ("npm_token", "npm_xxxxxxxxxxxxxxxxxxxx", "npm_short"),
        (
            "basic_auth",
            "Basic dXNlcjpwYXNzd29yZHh4eHg=",
            "Basic short",
        ),
        (
            "password_assignment",
            "password=s3cretvalue",
            "password is mentioned but not assigned",
        ),
        (
            "generic_secret_assignment",
            "api_key=supersecretvalue",
            "api_key is mentioned but not assigned",
        ),
        (
            "email_address",
            "user@example.com",
            "user at example dot com",
        ),
    ];

    #[test]
    fn every_detector_has_a_positive_canary_and_benign_negative() {
        assert_eq!(DETECTOR_ROWS.len(), 20);
        for (detector, positive, benign) in DETECTOR_ROWS {
            let mut hit = RedactionReport {
                policy: "test",
                ..Default::default()
            };
            drop(redact(positive, &mut hit));
            assert_eq!(
                hit.redacted_counts.keys().copied().collect::<Vec<_>>(),
                [*detector],
                "positive {positive:?} keys={:?}",
                hit.redacted_counts
            );
            assert_eq!(hit.redacted_counts.get(detector), Some(&1));

            let mut clean = RedactionReport {
                policy: "test",
                ..Default::default()
            };
            let out = redact(benign, &mut clean);
            assert!(
                clean.redacted_counts.is_empty(),
                "benign {benign:?} leaked {:?} -> {out:?}",
                clean.redacted_counts
            );
        }
    }

    #[test]
    fn anthropic_canary_is_not_classified_as_openai() {
        let mut report = RedactionReport {
            policy: "test",
            ..Default::default()
        };
        drop(redact("sk-ant-api03-XXXXXXXXXX", &mut report));
        assert_eq!(report.redacted_counts.get("anthropic_api_key"), Some(&1));
        assert_eq!(report.redacted_counts.get("openai_api_key"), None);
    }

    #[test]
    fn bearer_canary_without_github_token_fires_bearer_token() {
        let mut report = RedactionReport {
            policy: "test",
            ..Default::default()
        };
        drop(redact("Bearer abcdefghijklmnop", &mut report));
        assert_eq!(report.redacted_counts.get("bearer_token"), Some(&1));
        assert_eq!(report.redacted_counts.get("github_token"), None);
    }
}
