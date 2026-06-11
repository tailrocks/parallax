//! Parallax domain logic.
//!
//! Graduates the mechanisms proven in `poc/evidence-loop` (error derivation,
//! fingerprinting, grouping, bundle assembly, bounding, redaction,
//! hypotheses) onto the real OTLP protocol types. Filled milestone by
//! milestone; M0 ships the crate skeleton.

pub mod bundle;
pub mod derive;
pub mod fingerprint;
pub mod normalize;
