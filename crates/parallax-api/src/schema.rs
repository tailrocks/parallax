//! Schema construction and request execution.

use juniper::{EmptySubscription, RootNode};

use crate::{ApiContext, Mutation, Query};

pub type Schema = RootNode<Query, Mutation, EmptySubscription<ApiContext>>;

#[must_use]
pub fn build_schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::new())
}

/// Authoritative GraphQL SDL for UI codegen (Plan 152).
///
/// Normalizes line endings to `\n` and ensures exactly one trailing newline.
/// This is the only schema authority for `cargo xtask ui graphql export|check`.
#[must_use]
pub fn export_schema_sdl() -> String {
    normalize_schema_sdl(&build_schema().as_sdl())
}

/// Normalize SDL bytes for deterministic checked-in export.
#[must_use]
pub fn normalize_schema_sdl(sdl: &str) -> String {
    let normalized = sdl.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim_end_matches('\n');
    format!("{trimmed}\n")
}

/// Execute one GraphQL request against the schema — the whole integration
/// layer (the server's axum handler wraps this in ~10 lines).
pub async fn execute(
    schema: &Schema,
    context: &ApiContext,
    request: juniper::http::GraphQLRequest,
) -> juniper::http::GraphQLResponse {
    request.execute(schema, context).await
}
