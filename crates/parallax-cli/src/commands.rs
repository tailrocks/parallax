#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "display math"
)]
#[expect(clippy::excessive_nesting, reason = "command flow")]
#[path = "commands/implementation.rs"]
mod implementation;

pub(crate) use implementation::*;
