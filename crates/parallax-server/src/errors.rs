//! Classified configuration and server-startup failures.

use std::path::PathBuf;

/// Stable classification for configuration failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigErrorKind {
    Read,
    Parse,
    Invalid,
}

/// A configuration failure with its original source chain intact.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read configuration {}: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse configuration: {source}")]
    Parse {
        #[from]
        source: toml::de::Error,
    },
    #[error("{0}")]
    Invalid(String),
}

impl ConfigError {
    #[must_use]
    pub const fn kind(&self) -> ConfigErrorKind {
        match self {
            Self::Read { .. } => ConfigErrorKind::Read,
            Self::Parse { .. } => ConfigErrorKind::Parse,
            Self::Invalid(_) => ConfigErrorKind::Invalid,
        }
    }
}

pub type ConfigResult<T> = Result<T, ConfigError>;

/// Stable classification for server startup failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerErrorKind {
    Configuration,
    Filesystem,
    Storage,
    Metadata,
    Spool,
    Bind,
    Lifecycle,
}

/// A startup failure. Listener task failures after startup are logged by their
/// owning task and do not cross this boundary.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error(transparent)]
    Configuration(#[from] ConfigError),
    #[error("filesystem setup failed: {source}")]
    Filesystem {
        #[source]
        source: std::io::Error,
    },
    #[error("telemetry storage startup failed: {source}")]
    Storage {
        #[source]
        source: anyhow::Error,
    },
    #[error("metadata startup failed: {source}")]
    Metadata {
        #[source]
        source: anyhow::Error,
    },
    #[error("ingest spool startup failed: {source}")]
    Spool {
        #[source]
        source: anyhow::Error,
    },
    #[error("failed to bind {surface}: {source}")]
    Bind {
        surface: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("managed engine lifecycle failed: {source}")]
    Lifecycle {
        #[source]
        source: anyhow::Error,
    },
}

impl ServerError {
    #[must_use]
    pub const fn kind(&self) -> ServerErrorKind {
        match self {
            Self::Configuration(_) => ServerErrorKind::Configuration,
            Self::Filesystem { .. } => ServerErrorKind::Filesystem,
            Self::Storage { .. } => ServerErrorKind::Storage,
            Self::Metadata { .. } => ServerErrorKind::Metadata,
            Self::Spool { .. } => ServerErrorKind::Spool,
            Self::Bind { .. } => ServerErrorKind::Bind,
            Self::Lifecycle { .. } => ServerErrorKind::Lifecycle,
        }
    }

    pub(crate) fn storage(source: impl Into<anyhow::Error>) -> Self {
        Self::Storage {
            source: source.into(),
        }
    }

    pub(crate) fn lifecycle(source: impl Into<anyhow::Error>) -> Self {
        Self::Lifecycle {
            source: source.into(),
        }
    }
}

pub type ServerResult<T> = Result<T, ServerError>;

#[cfg(test)]
mod tests;
