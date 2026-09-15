//! Developer-API host protection, optional bearer auth, and GraphQL handling.

use crate::alerting::{AdapterMeasurementSource, preview_rule};
use crate::config::LimitsConfig;
use axum::Json;
use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use parallax_api::{AlertPreviewData, AlertPreviewer, ApiContext, Schema as ParallaxSchema};
use parallax_metadata::AlertRuleRecord;
use parallax_storage::{adapter::TelemetryStore, metadata::MetadataStore};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

struct StoreAlertPreviewer {
    store: Arc<dyn TelemetryStore>,
}

impl AlertPreviewer for StoreAlertPreviewer {
    fn preview(
        &self,
        rule: AlertRuleRecord,
        window_minutes: u32,
        now_nanos: u128,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<AlertPreviewData>> + Send + '_>> {
        let source = AdapterMeasurementSource::new(self.store.clone());
        Box::pin(async move { preview_rule(&source, &rule, window_minutes, now_nanos).await })
    }
}

#[derive(Clone)]
pub(super) struct GraphQlState {
    pub(super) schema: Arc<ParallaxSchema>,
    pub(super) store: Arc<dyn TelemetryStore>,
    pub(super) metadata: Arc<dyn MetadataStore>,
    pub(super) alerts: Option<Arc<parallax_metadata::TursoMetadataStore>>,
    pub(super) pipeline: Arc<dyn parallax_api::PipelineSnapshot>,
    pub(super) otlp_grpc_port: u16,
    pub(super) otlp_http_port: u16,
    pub(super) limits: LimitsConfig,
}

#[derive(Clone)]
pub(super) struct HostGuard {
    allowed_hosts: Arc<Vec<String>>,
}

impl HostGuard {
    pub(super) fn for_listener(bind: &str, api_addr: SocketAddr, public_url: &str) -> Self {
        let mut allowed = vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "[::1]".to_string(),
        ];
        add_allowed_host(&mut allowed, bind);
        add_allowed_host(&mut allowed, &api_addr.ip().to_string());
        if let Some(host) = host_from_public_url(public_url) {
            add_allowed_host(&mut allowed, &host);
        }
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

fn host_from_public_url(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }
    let without_scheme = match url.split_once("://") {
        Some((_, rest)) => rest,
        None => url,
    };
    let hostport = without_scheme.split('/').next().unwrap_or("");
    normalize_host_header(hostport)
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

#[derive(Clone)]
pub(super) struct LoginState {
    pub(super) enabled: bool,
    pub(super) username: String,
    pub(super) token: Option<Arc<str>>,
}

#[derive(Debug, serde::Serialize)]
pub(super) struct LoginStatus {
    login_enabled: bool,
    username: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, serde::Serialize)]
pub(super) struct LoginResponse {
    token: String,
    username: String,
}

pub(super) async fn login_status_handler(State(state): State<LoginState>) -> Json<LoginStatus> {
    Json(LoginStatus {
        login_enabled: state.enabled,
        username: state
            .enabled
            .then(|| state.username.clone())
            .filter(|name| !name.is_empty()),
    })
}

pub(super) async fn login_handler(
    State(state): State<LoginState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    authenticate_login(&state, &request).map(Json)
}

fn authenticate_login(
    state: &LoginState,
    request: &LoginRequest,
) -> Result<LoginResponse, StatusCode> {
    if !state.enabled {
        return Err(StatusCode::NOT_FOUND);
    }
    let Some(expected) = state.token.as_deref() else {
        return Err(StatusCode::NOT_FOUND);
    };
    let username = request.username.trim();
    let password = request.password.trim();
    let user_ok =
        !state.username.is_empty() && username.eq_ignore_ascii_case(state.username.trim());
    let pass_ok = constant_time_eq(password.as_bytes(), expected.as_bytes());
    if user_ok && pass_ok {
        return Ok(LoginResponse {
            token: expected.to_string(),
            username: state.username.clone(),
        });
    }
    Err(StatusCode::UNAUTHORIZED)
}

/// Shared operator bearer token for protected developer API routes (plan 109).
/// `None` keeps the surface open (local-first default).
#[derive(Clone)]
pub(super) struct ApiAuth {
    token: Option<Arc<str>>,
}

impl ApiAuth {
    pub(super) fn from_token(token: Option<String>) -> Self {
        Self {
            token: token.map(|value| Arc::from(value.into_boxed_str())),
        }
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "auth-mode assertions exercise this in tests")
    )]
    fn required(&self) -> bool {
        self.token.is_some()
    }
}

pub(super) async fn api_auth_middleware(
    State(auth): State<ApiAuth>,
    request: Request,
    next: Next,
) -> Response {
    authorize(auth, request, next, false).await
}

/// Same bearer check, plus an `access_token` query fallback scoped to the SSE
/// stream routes: browsers cannot attach headers to `EventSource`, so the live
/// tail is otherwise unreachable on token-protected servers. Query strings are
/// commonly logged, which is why this fallback is opt-in per route.
pub(super) async fn api_auth_middleware_with_query_token(
    State(auth): State<ApiAuth>,
    request: Request,
    next: Next,
) -> Response {
    authorize(auth, request, next, true).await
}

async fn authorize(
    auth: ApiAuth,
    request: Request,
    next: Next,
    allow_query_token: bool,
) -> Response {
    let Some(expected) = auth.token.as_deref() else {
        return next.run(request).await;
    };
    let presented = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(extract_bearer)
        .map(str::to_string)
        .or_else(|| {
            if !allow_query_token {
                return None;
            }
            let raw = request.uri().query()?;
            raw.split('&').find_map(|pair| {
                let (key, value) = pair.split_once('=')?;
                if key != "access_token" {
                    return None;
                }
                let decoded = percent_decode(value);
                (!decoded.is_empty()).then_some(decoded)
            })
        });
    match presented.as_deref() {
        Some(candidate) if constant_time_eq(candidate.as_bytes(), expected.as_bytes()) => {
            tracing::debug!(auth.result = "ok", "api auth accepted");
            next.run(request).await
        }
        _ => {
            tracing::info!(auth.result = "deny", "api auth rejected");
            (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                "unauthorized",
            )
                .into_response()
        }
    }
}

/// Minimal percent-decoding for the `access_token` query fallback: resolves
/// `%XX` escapes (case-insensitive hex) and `+` as space; every other byte
/// passes through untouched.
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' => {
                if let Some(hex) = bytes.get(index + 1..index + 3)
                    && let Ok(byte) = u8::from_str_radix(&String::from_utf8_lossy(hex), 16)
                {
                    decoded.push(byte);
                    index += 3;
                } else {
                    decoded.push(b'%');
                    index += 1;
                }
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn extract_bearer(header: &str) -> Option<&str> {
    let header = header.trim();
    let token = header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))?
        .trim();
    if token.is_empty() { None } else { Some(token) }
}

/// Constant-time equality for equal-length secrets; unequal lengths always fail.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (left, right) in a.iter().zip(b.iter()) {
        diff |= left ^ right;
    }
    diff == 0
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
            alerts: state.alerts.clone(),
            alert_previewer: Some(Arc::new(StoreAlertPreviewer {
                store: state.store.clone(),
            })),
            pipeline: Some(state.pipeline.clone()),
            otlp_grpc_port: state.otlp_grpc_port,
            otlp_http_port: state.otlp_http_port,
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

#[cfg(test)]
mod auth_tests {
    use super::{
        ApiAuth, HostGuard, constant_time_eq, extract_bearer, host_from_public_url, percent_decode,
    };

    #[test]
    fn bearer_extraction_is_strict() {
        assert_eq!(
            extract_bearer("Bearer secret-token-value"),
            Some("secret-token-value")
        );
        assert_eq!(
            extract_bearer("bearer secret-token-value"),
            Some("secret-token-value")
        );
        assert_eq!(extract_bearer("Basic secret"), None);
        assert_eq!(extract_bearer("Bearer "), None);
        assert_eq!(extract_bearer(""), None);
    }

    #[test]
    fn constant_time_compare_rejects_length_mismatch() {
        assert!(constant_time_eq(b"abcdefghijklmnop", b"abcdefghijklmnop"));
        assert!(!constant_time_eq(b"abcdefghijklmnop", b"abcdefghijklmnoq"));
        assert!(!constant_time_eq(b"short", b"abcdefghijklmnop"));
    }

    #[test]
    fn open_mode_when_token_absent() {
        assert!(!ApiAuth::from_token(None).required());
        assert!(ApiAuth::from_token(Some("a".repeat(16))).required());
    }

    #[test]
    fn public_url_host_is_allowed_and_foreign_hosts_are_not() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};
        let guard = HostGuard::for_listener(
            "0.0.0.0",
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4000),
            "https://parallax.chainargos.com",
        );
        assert!(guard.allows("parallax.chainargos.com"));
        assert!(guard.allows("parallax.chainargos.com:443"));
        assert!(guard.allows("127.0.0.1"));
        assert!(!guard.allows("evil.example.com"));
    }

    #[test]
    fn login_disabled_is_not_found_even_with_valid_credentials() {
        let state = super::LoginState {
            enabled: false,
            username: "operator@example.com".into(),
            token: Some(std::sync::Arc::from("sixteen-chars-ok")),
        };
        let request = super::LoginRequest {
            username: "operator@example.com".into(),
            password: "sixteen-chars-ok".into(),
        };
        assert_eq!(
            super::authenticate_login(&state, &request).unwrap_err(),
            axum::http::StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn login_accepts_operator_and_rejects_bad_password() {
        let state = super::LoginState {
            enabled: true,
            username: "operator@example.com".into(),
            token: Some(std::sync::Arc::from("sixteen-chars-ok")),
        };
        let ok = super::authenticate_login(
            &state,
            &super::LoginRequest {
                username: "Operator@example.com".into(),
                password: "sixteen-chars-ok".into(),
            },
        )
        .expect("valid operator login");
        assert_eq!(ok.token, "sixteen-chars-ok");
        assert_eq!(ok.username, "operator@example.com");
        assert_eq!(
            super::authenticate_login(
                &state,
                &super::LoginRequest {
                    username: "operator@example.com".into(),
                    password: "wrong-password-ok".into(),
                },
            )
            .unwrap_err(),
            axum::http::StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn host_from_public_url_strips_scheme_path_and_port() {
        assert_eq!(
            host_from_public_url("https://parallax.example.com/app"),
            Some("parallax.example.com".into())
        );
        assert_eq!(
            host_from_public_url("https://parallax.example.com:8443/"),
            Some("parallax.example.com".into())
        );
        assert_eq!(host_from_public_url(""), None);
    }

    /// The `access_token` query fallback (defect #4) must decode the standard
    /// escapes a browser's `URLSearchParams`/`encodeURIComponent` produces.
    #[test]
    fn percent_decode_resolves_escapes_and_plus() {
        assert_eq!(percent_decode("plain"), "plain");
        assert_eq!(percent_decode("a%26b%2Fc"), "a&b/c");
        assert_eq!(percent_decode("%41%42"), "AB");
        assert_eq!(
            percent_decode("a+b"),
            "a b",
            "+ is a space in query encoding"
        );
        // Malformed escapes pass through instead of panicking or dropping data.
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%G1"), "%G1");
        assert_eq!(percent_decode("%2"), "%2");
    }
}
