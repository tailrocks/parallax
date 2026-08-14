use clap::ValueEnum;

/// Output shape for agent-facing projections (bundles, agent sessions).
/// Markdown is the human default; JSON is the machine/agent contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub(crate) enum OutputFormat {
    #[default]
    Markdown,
    Json,
}
