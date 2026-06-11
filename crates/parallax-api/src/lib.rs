//! Parallax GraphQL API.
//!
//! Implements the SDL from `docs/research/architecture/v1-implementation-spec.md`
//! §8. M0 ships health-level scaffolding only; resolvers land in M2 against the
//! storage adapters.

use async_graphql::{EmptySubscription, Object, Schema};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// API liveness probe (M0). Real queries land in M2.
    async fn health(&self) -> &'static str {
        "ok"
    }

    async fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Placeholder until M2 mutations (issueSetStatus, dashboardSave) land.
    async fn noop(&self) -> bool {
        true
    }
}

pub type ParallaxSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn build_schema() -> ParallaxSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
}
