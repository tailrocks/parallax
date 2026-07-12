//! Schema construction and request execution.

use juniper::{EmptySubscription, RootNode};

use crate::{ApiContext, Mutation, Query};

pub type Schema = RootNode<Query, Mutation, EmptySubscription<ApiContext>>;

#[must_use]
pub fn build_schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::new())
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
