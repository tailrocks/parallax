//! Versioned default-deny source minimization for agent-visible evidence.
//!
//! This is the typed policy seam for plan 111. Detector execution remains in
//! `bundle::redaction`; callers must not project a field before consulting this
//! policy once the bundle contract is versioned for the migration.

pub const SOURCE_POLICY_VERSION: &str = "evidence-source-v1";

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
    AllowStructural,
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
            rule_id: "source.structural.allow",
            action: SourceAction::AllowStructural,
        },
        EvidenceField::Unknown => SourceDecision {
            rule_id: "source.unknown.drop",
            action: SourceAction::Drop,
        },
    }
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
        assert_eq!(
            decide(EvidenceField::Unknown),
            SourceDecision {
                rule_id: "source.unknown.drop",
                action: SourceAction::Drop,
            }
        );
    }
}
