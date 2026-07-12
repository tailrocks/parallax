//! GraphQL dashboards domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;

use crate::{ApiContext, field_err, nanos_string};

pub(crate) struct Dashboard(pub(crate) model::Dashboard);

#[graphql_object(context = ApiContext)]
impl Dashboard {
    fn id(&self) -> &str {
        &self.0.id
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    /// Widget layout as a JSON string:
    /// [{metric, agg, chart, title, quantile?}].
    fn layout(&self) -> &str {
        &self.0.layout
    }
    fn updated_at_nanos(&self) -> String {
        nanos_string(self.0.updated_at_nanos)
    }
}

pub(crate) async fn dashboard(context: &ApiContext, id: String) -> FieldResult<Option<Dashboard>> {
    Ok(context
        .metadata
        .dashboard(&id)
        .await
        .map_err(field_err)?
        .map(Dashboard))
}

pub(crate) async fn dashboards(context: &ApiContext) -> FieldResult<Vec<Dashboard>> {
    let dashboards = context.metadata.dashboards().await.map_err(field_err)?;
    Ok(dashboards.into_iter().map(Dashboard).collect())
}

pub(crate) async fn dashboard_save(
    context: &ApiContext,
    name: String,
    layout: String,
    id: Option<String>,
) -> FieldResult<Dashboard> {
    // Layout must at least be valid JSON; widget semantics are the UI's.
    if serde_json::from_str::<serde_json::Value>(&layout).is_err() {
        return Err(field_err("layout must be valid JSON"));
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let id = id.unwrap_or_else(|| format!("dash_{now:x}"));
    context
        .metadata
        .dashboard_save(&id, &name, &layout, now)
        .await
        .map_err(field_err)?;
    context
        .metadata
        .dashboard(&id)
        .await
        .map_err(field_err)?
        .map(Dashboard)
        .ok_or_else(|| field_err("dashboard save did not persist"))
}

pub(crate) async fn dashboard_delete(context: &ApiContext, id: String) -> FieldResult<bool> {
    context
        .metadata
        .dashboard_delete(&id)
        .await
        .map_err(field_err)
}
