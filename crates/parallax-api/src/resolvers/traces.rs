//! GraphQL traces facade: domain projections and query orchestration.

mod queries;
mod types;

pub(crate) use queries::*;
pub(crate) use types::*;

#[cfg(test)]
use parallax_analysis::trace_analysis;

#[cfg(test)]
mod tests;
