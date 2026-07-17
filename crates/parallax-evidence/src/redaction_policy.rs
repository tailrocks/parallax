//! Versioned default-deny source minimization (extracted to
//! `parallax-redaction`; this module re-exports the stable seam).

pub use parallax_redaction::{
    DETECTOR_POLICY_VERSION, EvidenceField, SOURCE_POLICY_VERSION, SourceAction, SourceDecision,
    decide, project_text, sanitize_text,
};
