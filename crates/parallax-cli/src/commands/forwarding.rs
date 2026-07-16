//! OTLP forwarding resolution, injected environment, and display time helpers.

use crate::client::Client;
use parallax_semconv as semconv;
use std::time::{SystemTime, UNIX_EPOCH};

/// Rotel target for `--otlp-forward rotel` and `PARALLAX_OTLP_FORWARD=rotel`.
pub(crate) const DEFAULT_ROTEL_ENDPOINT: &str = "http://localhost:4317";
pub(crate) const OTLP_GRPC_PROTOCOL: &str = "grpc";
pub(crate) const OTLP_HTTP_PROTOCOL: &str = "http";
#[derive(Debug)]
pub(crate) struct ParallaxEndpoints {
    pub(crate) grpc: String,
    pub(crate) http_traces: String,
}
/// Resolved compare-mode forwarding target for `run start`.
pub(crate) struct Forward {
    pub(crate) endpoint: String,
    pub(crate) protocol: &'static str,
    /// Whether child telemetry is forwarded through Rotel in comparison mode.
    pub(crate) compare: bool,
}
/// OTLP HTTP defaults to port 4318; treat anything else as gRPC.
pub(crate) fn protocol_for(endpoint: &str) -> &'static str {
    if endpoint.contains(":4318") {
        OTLP_HTTP_PROTOCOL
    } else {
        OTLP_GRPC_PROTOCOL
    }
}
/// Pure compare-mode precedence: flag, forwarding env, existing OTLP env, default.
pub(crate) fn resolve_forward_from(
    flag: Option<&str>,
    env_forward: Option<String>,
    env_otel: Option<String>,
    default_parallax_endpoint: &str,
) -> anyhow::Result<Forward> {
    if let Some(raw) = flag.map(str::to_owned).or(env_forward) {
        let value = raw.trim();
        let endpoint = match value.to_ascii_lowercase().as_str() {
            "off" | "parallax" => {
                return Ok(Forward {
                    endpoint: default_parallax_endpoint.to_string(),
                    protocol: OTLP_GRPC_PROTOCOL,
                    compare: false,
                });
            }
            "rotel" | "1" | "true" | "on" => DEFAULT_ROTEL_ENDPOINT.to_string(),
            _ if value.starts_with("http://") || value.starts_with("https://") => value.to_string(),
            other => {
                anyhow::bail!("invalid --otlp-forward '{other}' (use a URL, 'rotel', or 'off')")
            }
        };
        let protocol = protocol_for(&endpoint);
        return Ok(Forward {
            endpoint,
            protocol,
            compare: true,
        });
    }
    // Respect a pre-existing OTLP endpoint when no forward is explicit.
    if let Some(existing) = env_otel.filter(|v| !v.is_empty()) {
        let protocol = protocol_for(&existing);
        let compare = existing != default_parallax_endpoint;
        return Ok(Forward {
            endpoint: existing,
            protocol,
            compare,
        });
    }
    Ok(Forward {
        endpoint: default_parallax_endpoint.to_string(),
        protocol: OTLP_GRPC_PROTOCOL,
        compare: false,
    })
}
pub(crate) fn resolve_forward(
    flag: Option<&str>,
    default_parallax_endpoint: &str,
) -> anyhow::Result<Forward> {
    let env_forward = std::env::var("PARALLAX_OTLP_FORWARD")
        .ok()
        .filter(|v| !v.is_empty());
    let env_otel = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
    resolve_forward_from(flag, env_forward, env_otel, default_parallax_endpoint)
}
pub(crate) fn endpoint_from_api_url_and_port(api_url: &str, port: u16) -> anyhow::Result<String> {
    let url = reqwest::Url::parse(api_url)
        .map_err(|e| anyhow::anyhow!("invalid Parallax API URL {api_url:?}: {e}"))?;
    let scheme = url.scheme();
    let host = url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("Parallax API URL {api_url:?} has no host"))?;
    let host = if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.to_string()
    };
    Ok(format!("{scheme}://{host}:{port}"))
}

pub(crate) fn http_traces_endpoint(forward: &Forward, parallax_http: &str) -> String {
    if forward.protocol == OTLP_HTTP_PROTOCOL {
        return format!("{}/v1/traces", forward.endpoint.trim_end_matches('/'));
    }
    if forward.compare
        && let Ok(mut url) = reqwest::Url::parse(&forward.endpoint)
        && url.port() == Some(4317)
        && url.set_port(Some(4318)).is_ok()
    {
        return format!("{}/v1/traces", url.as_str().trim_end_matches('/'));
    }
    parallax_http.to_string()
}
pub(crate) async fn parallax_endpoints_from_server(
    client: &Client,
) -> anyhow::Result<ParallaxEndpoints> {
    let response = client.graphql(r#"{ otlpGrpcPort otlpHttpPort }"#).await?;
    let port = |field: &str, transport: &str| {
        response
        .get("data")
        .and_then(|data| data.get(field))
        .and_then(serde_json::Value::as_u64)
        .and_then(|port| u16::try_from(port).ok())
        .filter(|port| *port != 0)
        .ok_or_else(|| anyhow::anyhow!(
            "Parallax server did not report a valid OTLP/{transport} port; cannot inject OTLP env"
        ))
    };
    let grpc = endpoint_from_api_url_and_port(client.base_url(), port("otlpGrpcPort", "gRPC")?)?;
    let http = endpoint_from_api_url_and_port(client.base_url(), port("otlpHttpPort", "HTTP")?)?;
    Ok(ParallaxEndpoints {
        grpc,
        http_traces: format!("{http}/v1/traces"),
    })
}

/// Child resource attributes: invocation id plus comparison labels when forwarding.
pub(crate) fn forward_resource_attrs(invocation_id: &str, compare: bool) -> String {
    let mut attrs = format!("{}={invocation_id}", semconv::CLI_INVOCATION_ID);
    if compare {
        let env = std::env::var("PARALLAX_ENV")
            .ok()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "lab".to_string());
        attrs.push_str(&format!(
            ",{}=1,{}={env}",
            semconv::PARALLAX_LAB,
            semconv::DEPLOYMENT_ENVIRONMENT_NAME
        ));
    }
    attrs
}

/// The full standard OTel env block (all signals + protocols + resource attrs),
/// pointed at `endpoint`. Used identically for wrapper, bare, and dry-run modes.
pub(crate) fn otel_env_pairs(
    endpoint: &str,
    protocol: &str,
    attrs: &str,
) -> Vec<(&'static str, String)> {
    vec![
        ("OTEL_EXPORTER_OTLP_ENDPOINT", endpoint.to_string()),
        ("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", endpoint.to_string()),
        ("OTEL_EXPORTER_OTLP_LOGS_ENDPOINT", endpoint.to_string()),
        ("OTEL_EXPORTER_OTLP_METRICS_ENDPOINT", endpoint.to_string()),
        ("OTEL_EXPORTER_OTLP_PROFILES_ENDPOINT", endpoint.to_string()),
        ("OTEL_EXPORTER_OTLP_PROTOCOL", protocol.to_string()),
        ("OTEL_EXPORTER_OTLP_TRACES_PROTOCOL", protocol.to_string()),
        ("OTEL_EXPORTER_OTLP_LOGS_PROTOCOL", protocol.to_string()),
        ("OTEL_EXPORTER_OTLP_METRICS_PROTOCOL", protocol.to_string()),
        ("OTEL_EXPORTER_OTLP_PROFILES_PROTOCOL", protocol.to_string()),
        ("OTEL_RESOURCE_ATTRIBUTES", attrs.to_string()),
    ]
}

/// Best-effort reachability check for compare mode: warn (never fail) if the
/// collector isn't accepting connections — a dead Rotel means nothing shows in
/// any backend, including Parallax.
pub(crate) async fn preflight_warn(endpoint: &str) {
    let host_port = endpoint
        .split("://")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .unwrap_or(endpoint);
    let connect = tokio::net::TcpStream::connect(host_port.to_string());
    let reachable = matches!(
        tokio::time::timeout(std::time::Duration::from_millis(500), connect).await,
        Ok(Ok(_))
    );
    if !reachable {
        eprintln!("⚠ {endpoint} not reachable — telemetry may be dropped");
    }
}

pub(crate) fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

pub(crate) fn new_invocation_id() -> String {
    // Opaque UUIDv4 per top-level CLI process (unified CLI contract).
    uuid::Uuid::new_v4().to_string()
}

pub(crate) fn relative(nanos_str: &str) -> String {
    let nanos: u128 = nanos_str.parse().unwrap_or(0);
    let now = now_nanos();
    let secs = now.saturating_sub(nanos) / 1_000_000_000;
    match secs {
        0..=59 => format!("{secs}s ago"),
        60..=3599 => format!("{}m ago", secs / 60),
        3600..=86_399 => format!("{}h ago", secs / 3600),
        _ => format!("{}d ago", secs / 86_400),
    }
}
