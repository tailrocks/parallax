//! GraphQL investigations domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;

use crate::{ApiContext, field_err, nanos_string};

use crate::{
    SAVED_VIEWS_PER_PAGE, validate_investigation_name, validate_investigation_state,
    validate_saved_view_name, validate_saved_view_page,
};

pub(crate) struct Investigation(pub(crate) model::Investigation);

#[graphql_object(context = ApiContext)]
impl Investigation {
    fn id(&self) -> &str {
        &self.0.id
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    /// Opaque V1 investigation state JSON:
    /// `{version, window, pins, notes}`.
    fn state(&self) -> &str {
        &self.0.state
    }
    fn created_at_nanos(&self) -> String {
        nanos_string(self.0.created_at_nanos)
    }
    fn updated_at_nanos(&self) -> String {
        nanos_string(self.0.updated_at_nanos)
    }
}

pub(crate) struct SavedView(pub(crate) model::SavedView);

#[graphql_object(context = ApiContext)]
impl SavedView {
    fn id(&self) -> &str {
        &self.0.id
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    fn page(&self) -> &str {
        &self.0.page
    }
    /// URL search string captured from the page state.
    fn state(&self) -> &str {
        &self.0.state
    }
    fn created_at_nanos(&self) -> String {
        nanos_string(self.0.created_at_nanos)
    }
    fn updated_at_nanos(&self) -> String {
        nanos_string(self.0.updated_at_nanos)
    }
}

pub(crate) async fn investigation(
    context: &ApiContext,
    id: String,
) -> FieldResult<Option<Investigation>> {
    Ok(context
        .metadata
        .investigation(&id)
        .await
        .map_err(field_err)?
        .map(Investigation))
}

pub(crate) async fn investigations(context: &ApiContext) -> FieldResult<Vec<Investigation>> {
    let investigations = context.metadata.investigations().await.map_err(field_err)?;
    Ok(investigations.into_iter().map(Investigation).collect())
}

pub(crate) async fn saved_views(
    context: &ApiContext,
    page: Option<String>,
) -> FieldResult<Vec<SavedView>> {
    let saved_views = context
        .metadata
        .saved_views(page.as_deref().filter(|page| !page.is_empty()))
        .await
        .map_err(field_err)?;
    Ok(saved_views.into_iter().map(SavedView).collect())
}

pub(crate) async fn investigation_save(
    context: &ApiContext,
    name: String,
    state: String,
    id: Option<String>,
) -> FieldResult<Investigation> {
    let name = validate_investigation_name(&name)?;
    validate_investigation_state(&state)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let id = id
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| format!("case_{now:x}"));
    context
        .metadata
        .investigation_save(&id, &name, &state, now)
        .await
        .map_err(field_err)?;
    context
        .metadata
        .investigation(&id)
        .await
        .map_err(field_err)?
        .map(Investigation)
        .ok_or_else(|| field_err("investigation save did not persist"))
}

pub(crate) async fn investigation_delete(context: &ApiContext, id: String) -> FieldResult<bool> {
    context
        .metadata
        .investigation_delete(&id)
        .await
        .map_err(field_err)
}

pub(crate) async fn saved_view_save(
    context: &ApiContext,
    name: String,
    page: String,
    state: String,
    id: Option<String>,
) -> FieldResult<SavedView> {
    let name = validate_saved_view_name(&name)?;
    validate_saved_view_page(&page)?;
    let existing = match id.as_deref().filter(|id| !id.is_empty()) {
        Some(id) => context.metadata.saved_view(id).await.map_err(field_err)?,
        None => None,
    };
    if existing.as_ref().is_none_or(|view| view.page != page)
        && context
            .metadata
            .saved_views(Some(&page))
            .await
            .map_err(field_err)?
            .len()
            >= SAVED_VIEWS_PER_PAGE
    {
        return Err(field_err("saved view cap reached for page"));
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let id = id
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| format!("view_{now:x}"));
    context
        .metadata
        .saved_view_save(&id, &name, &page, &state, now)
        .await
        .map_err(field_err)?;
    context
        .metadata
        .saved_view(&id)
        .await
        .map_err(field_err)?
        .map(SavedView)
        .ok_or_else(|| field_err("saved view save did not persist"))
}

pub(crate) async fn saved_view_delete(context: &ApiContext, id: String) -> FieldResult<bool> {
    context
        .metadata
        .saved_view_delete(&id)
        .await
        .map_err(field_err)
}

#[cfg(test)]
mod tests;
