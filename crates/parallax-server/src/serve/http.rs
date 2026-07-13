//! Developer-API host protection and GraphQL request handling.

use crate::config::LimitsConfig;
use axum::Json;
use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;
use parallax_api::{ApiContext, Schema as ParallaxSchema};
use parallax_storage::{adapter::TelemetryStore, metadata::MetadataStore};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct GraphQlState {
    pub(super) schema: Arc<ParallaxSchema>,
    pub(super) store: Arc<dyn TelemetryStore>,
    pub(super) metadata: Arc<dyn MetadataStore>,
    pub(super) otlp_grpc_port: u16,
    pub(super) limits: LimitsConfig,
}

#[derive(Clone)]
pub(super) struct HostGuard {
    allowed_hosts: Arc<Vec<String>>,
}

impl HostGuard {
    pub(super) fn for_listener(bind: &str, api_addr: SocketAddr) -> Self {
        let mut allowed = vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "[::1]".to_string(),
        ];
        add_allowed_host(&mut allowed, bind);
        add_allowed_host(&mut allowed, &api_addr.ip().to_string());
        allowed.sort();
        allowed.dedup();
        Self {
            allowed_hosts: Arc::new(allowed),
        }
    }

    fn allows(&self, host: &str) -> bool {
        normalize_host_header(host)
            .is_some_and(|host| self.allowed_hosts.iter().any(|allowed| allowed == &host))
    }
}

fn add_allowed_host(allowed: &mut Vec<String>, host: &str) {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() {
        return;
    }
    if host.contains(':') && !host.starts_with('[') {
        allowed.push(format!("[{host}]"));
    } else {
        allowed.push(host);
    }
}

fn normalize_host_header(host: &str) -> Option<String> {
    let host = host.trim().trim_end_matches('.');
    if host.is_empty() {
        return None;
    }
    if host.starts_with('[') {
        let end = host.find(']')?;
        let rest = &host[end + 1..];
        if rest.is_empty()
            || rest
                .strip_prefix(':')
                .is_some_and(|port| !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()))
        {
            return Some(host[..=end].to_ascii_lowercase());
        }
        return None;
    }
    if host.matches(':').count() > 1 {
        return None;
    }
    let bare = match host.rsplit_once(':') {
        Some((name, port))
            if !name.is_empty() && !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) =>
        {
            name
        }
        Some(_) => return None,
        None => host,
    };
    Some(bare.to_ascii_lowercase())
}

pub(super) async fn host_guard_middleware(
    State(guard): State<HostGuard>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let allowed = request
        .headers()
        .get(header::HOST)
        .and_then(|host| host.to_str().ok())
        .is_some_and(|host| guard.allows(host));
    if allowed {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

/// The hand-rolled Juniper-over-axum handler (spec §2 note). Wrapped in a
/// `graphql.request` span so self-telemetry (when enabled) emits Parallax's own
/// API activity — this is the recurring signal that fans out to the lab.
pub(super) async fn graphql_handler(
    State(state): State<GraphQlState>,
    Json(request): Json<juniper::http::GraphQLRequest>,
) -> Json<juniper::http::GraphQLResponse> {
    use tracing::Instrument;
    let operation = request
        .operation_name
        .clone()
        .unwrap_or_else(|| "anonymous".to_string());
    async move {
        // Fresh ApiContext per request so RequestMemo is request-scoped and
        // sibling resolvers share one spans_by_trace / logs_by_trace fetch.
        let context = ApiContext {
            store: state.store.clone(),
            metadata: state.metadata.clone(),
            otlp_grpc_port: state.otlp_grpc_port,
            memo: parallax_api::RequestMemo::default(),
        };
        let response = match parallax_api::check_query_limits(
            &state.schema,
            &request.query,
            request.operation_name.as_deref(),
            state.limits.graphql_max_depth,
            state.limits.graphql_max_complexity,
        ) {
            Ok(()) => request.execute(&state.schema, &context).await,
            Err(message) => juniper::http::GraphQLResponse::error(juniper::FieldError::new(
                message,
                juniper::Value::null(),
            )),
        };
        tracing::info!(ok = response.is_ok(), "graphql request");
        Json(response)
    }
    .instrument(tracing::info_span!("graphql.request", otel.name = %operation))
    .await
}
