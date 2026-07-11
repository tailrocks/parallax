//! GraphQL investigations domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;

use crate::{ApiContext, field_err, nanos_string};

use crate::{
    SAVED_VIEWS_PER_PAGE, validate_investigation_name, validate_investigation_state,
    validate_saved_view_name, validate_saved_view_page,
};

pub struct Investigation(pub(crate) model::Investigation);

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

pub struct SavedView(pub(crate) model::SavedView);

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
        .map(|d| d.as_nanos())
        .unwrap_or(0);
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
        .map(|d| d.as_nanos())
        .unwrap_or(0);
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
mod tests {
    use super::*;
    use crate::INVESTIGATION_PIN_CAP;
    use crate::resolvers::test_support::*;
    use crate::{build_schema, execute};

    use parallax_storage::memory::MemoryStore;

    use std::sync::Arc;

    #[tokio::test]
    async fn saved_view_resolvers_round_trip_filter_delete_and_cap() {
        let schema = build_schema();
        let context = context_with_memory(Arc::new(MemoryStore::new())).await;
        let save = juniper::http::GraphQLRequest::new(
            r#"
            mutation {
              savedViewSave(name: "Errors", page: "/logs", state: "?sev=17&cols=trace") {
                id name page state
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, save).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "savedViewSave: {json}");
        let id = json
            .pointer("/data/savedViewSave/id")
            .and_then(|value| value.as_str())
            .unwrap()
            .to_string();
        assert_eq!(
            json.pointer("/data/savedViewSave/state"),
            Some(&serde_json::json!("?sev=17&cols=trace"))
        );

        let list = juniper::http::GraphQLRequest::new(
            r#"{ savedViews(page: "/logs") { id name page state } }"#.into(),
            None,
            None,
        );
        let response = execute(&schema, &context, list).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "savedViews: {json}");
        assert_eq!(
            json.pointer("/data/savedViews/0/id"),
            Some(&serde_json::json!(id.as_str()))
        );

        let delete = juniper::http::GraphQLRequest::new(
            format!(r#"mutation {{ savedViewDelete(id: "{id}") }}"#),
            None,
            None,
        );
        let response = execute(&schema, &context, delete).await;
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(
            json.pointer("/data/savedViewDelete"),
            Some(&serde_json::json!(true))
        );

        for index in 0..SAVED_VIEWS_PER_PAGE {
            context
                .metadata
                .saved_view_save(
                    &format!("view-{index}"),
                    "View",
                    "/logs",
                    "?q=x",
                    index as u128,
                )
                .await
                .unwrap();
        }
        let capped = juniper::http::GraphQLRequest::new(
            r#"mutation { savedViewSave(name: "Too many", page: "/logs", state: "?q=y") { id } }"#
                .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, capped).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("saved view cap")),
            "saved view cap enforced: {json}"
        );
    }

    #[tokio::test]
    async fn investigation_resolvers_round_trip_and_validate_state() {
        let schema = build_schema();
        let context = context_with_memory(Arc::new(MemoryStore::new())).await;
        let state = r#"{"version":1,"window":{"range":"24h"},"pins":[{"kind":"trace","ref":"/traces/t1","label":"trace"}],"notes":"triage"}"#;
        let save = juniper::http::GraphQLRequest::new(
            format!(
                r#"
                mutation {{
                  investigationSave(name: "Checkout case", state: "{}") {{
                    id name state
                  }}
                }}
                "#,
                state.replace('"', "\\\"")
            ),
            None,
            None,
        );
        let response = execute(&schema, &context, save).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json).is_empty(),
            "investigationSave: {json}"
        );
        let id = json
            .pointer("/data/investigationSave/id")
            .and_then(|value| value.as_str())
            .unwrap()
            .to_string();
        assert_eq!(
            json.pointer("/data/investigationSave/name"),
            Some(&serde_json::json!("Checkout case"))
        );

        let list = juniper::http::GraphQLRequest::new(
            r#"{ investigations { id name state updatedAtNanos } }"#.into(),
            None,
            None,
        );
        let response = execute(&schema, &context, list).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "investigations: {json}");
        assert_eq!(
            json.pointer("/data/investigations/0/id"),
            Some(&serde_json::json!(id.as_str()))
        );

        let get = juniper::http::GraphQLRequest::new(
            format!(r#"{{ investigation(id: "{id}") {{ id name state }} }}"#),
            None,
            None,
        );
        let response = execute(&schema, &context, get).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "investigation: {json}");
        assert_eq!(
            json.pointer("/data/investigation/id"),
            Some(&serde_json::json!(id.as_str()))
        );

        let delete = juniper::http::GraphQLRequest::new(
            format!(r#"mutation {{ investigationDelete(id: "{id}") }}"#),
            None,
            None,
        );
        let response = execute(&schema, &context, delete).await;
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(
            json.pointer("/data/investigationDelete"),
            Some(&serde_json::json!(true))
        );

        let bad_json = juniper::http::GraphQLRequest::new(
            r#"mutation { investigationSave(name: "Bad", state: "{bad json") { id } }"#.into(),
            None,
            None,
        );
        let response = execute(&schema, &context, bad_json).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("state must be valid JSON")),
            "bad JSON rejected: {json}"
        );

        let pins = (0..=INVESTIGATION_PIN_CAP)
            .map(|index| {
                serde_json::json!({
                    "kind": "trace",
                    "ref": format!("/traces/{index}"),
                    "label": format!("trace {index}")
                })
            })
            .collect::<Vec<_>>();
        let capped_state = serde_json::json!({
            "version": 1,
            "window": {"range": "24h"},
            "pins": pins,
            "notes": ""
        })
        .to_string();
        let capped = juniper::http::GraphQLRequest::new(
            format!(
                r#"mutation {{ investigationSave(name: "Too many", state: "{}") {{ id }} }}"#,
                capped_state.replace('"', "\\\"")
            ),
            None,
            None,
        );
        let response = execute(&schema, &context, capped).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("pin cap")),
            "pin cap enforced: {json}"
        );
    }
}
