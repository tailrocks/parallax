//! Typed failures for engine-neutral telemetry capabilities.

use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageErrorKind {
    Transport,
    Query,
    Schema,
    Unavailable,
    Timeout,
    Internal,
}

/// Typed telemetry-storage boundary error. Detailed engine context remains in
/// the source chain and is never part of the stable client-facing mapping.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("telemetry transport failed: {source}")]
    Transport {
        #[source]
        source: anyhow::Error,
    },
    #[error("telemetry query failed: {source}")]
    Query {
        #[source]
        source: anyhow::Error,
    },
    #[error("telemetry schema operation failed: {source}")]
    Schema {
        #[source]
        source: anyhow::Error,
    },
    #[error("telemetry store unavailable: {source}")]
    Unavailable {
        #[source]
        source: anyhow::Error,
    },
    #[error("telemetry operation timed out: {source}")]
    Timeout {
        #[source]
        source: anyhow::Error,
    },
    #[error("telemetry operation failed: {source}")]
    Internal {
        #[source]
        source: anyhow::Error,
    },
}

impl StorageError {
    #[must_use]
    pub fn kind(&self) -> StorageErrorKind {
        match self {
            Self::Transport { .. } => StorageErrorKind::Transport,
            Self::Query { .. } => StorageErrorKind::Query,
            Self::Schema { .. } => StorageErrorKind::Schema,
            Self::Unavailable { .. } => StorageErrorKind::Unavailable,
            Self::Timeout { .. } => StorageErrorKind::Timeout,
            Self::Internal { .. } => StorageErrorKind::Internal,
        }
    }

    pub fn internal(source: impl Into<anyhow::Error>) -> Self {
        Self::Internal {
            source: source.into(),
        }
    }

    pub fn query(source: impl Into<anyhow::Error>) -> Self {
        Self::Query {
            source: source.into(),
        }
    }

    pub fn transport(source: impl Into<anyhow::Error>) -> Self {
        Self::Transport {
            source: source.into(),
        }
    }
}

impl From<anyhow::Error> for StorageError {
    fn from(source: anyhow::Error) -> Self {
        Self::Query { source }
    }
}

pub type StorageResult<T> = Result<T, StorageError>;

#[cfg(test)]
mod tests;
