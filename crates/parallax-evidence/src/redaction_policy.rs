//! Versioned default-deny source minimization for agent-visible evidence.
//!
//! This is the typed policy seam for plan 111. Detector execution remains in
//! `bundle::redaction`; callers must not project a field before consulting this
//! policy. Unknown fields default to drop, never pass-through.

use crate::bundle::{RedactionReport, redact};
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
            *report.redacted_counts.entry("source.unknown.drop").or_insert(0) += 1;
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
}
