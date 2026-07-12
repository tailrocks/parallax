//! Turso-backed mutable metadata adapter and migrations.

mod port;
#[expect(clippy::excessive_nesting, reason = "transaction flow")]
mod turso;

pub use turso::TursoMetadataStore;
