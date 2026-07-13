//! CLI command facade grouped by product responsibility.

mod filters;
mod forwarding;
mod issues;
mod live;
mod logs;
mod output;
mod runs;
mod sql;
mod traces;

pub(crate) use filters::{LogsFilter, TracesFilter};
pub(crate) use issues::*;
pub(crate) use live::*;
pub(crate) use logs::*;
pub(crate) use runs::*;
pub(crate) use sql::*;
pub(crate) use traces::*;

#[cfg(test)]
use crate::OutputFormat;
#[cfg(test)]
pub(crate) use forwarding::*;
#[cfg(test)]
pub(crate) use output::*;
#[cfg(test)]
mod tests;
