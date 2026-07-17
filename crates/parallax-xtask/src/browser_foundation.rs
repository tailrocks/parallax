//! Minimal foundation browser-server lifecycle for plan 132.
//!
//! Serves the already-built UI dist, exposes `/health`, stubs `/graphql` with
//! empty product-shaped data, publishes a sanitized runtime manifest, and exits
//! when the Playwright webServer supervisor terminates the process. Owns no
//! browser locator or assertion logic.

use std::{
    fs,
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use serde_json::json;

const DEFAULT_PORT: u16 = 4173;
const BIND: &str = "127.0.0.1";

/// Empty product-shaped GraphQL payload so foundation shell smoke can load
/// without a full Parallax/GreptimeDB stack. Plan 144 owns real fixtures.
const GRAPHQL_STUB: &str = r#"{
  "data": {
    "health": "ok",
    "overview": {
      "spanCount": "0",
      "traceCount": "0",
      "logCount": "0",
      "metricPointCount": "0",
      "errorCount": "0",
      "errorRate": 0,
      "activeServices": 0
    },
    "previousOverview": {
      "spanCount": "0",
      "traceCount": "0",
      "logCount": "0",
      "metricPointCount": "0",
      "errorCount": "0",
      "errorRate": 0,
      "activeServices": 0
    },
    "spansSeries": [],
    "errorsSeries": [],
    "red": {
      "rate": [],
      "errorRate": [],
      "p50": [],
      "p95": [],
      "p99": []
    },
    "previousRed": {
      "rate": [],
      "errorRate": [],
      "p50": [],
      "p95": [],
      "p99": []
    },
    "servicesNow": [],
    "servicesPrev": [],
    "issues": { "items": [] },
    "tracesPage": { "items": [] },
    "dashboards": []
  }
}"#;

pub(crate) fn run(root: &Path) -> Result<()> {
    let port = std::env::var("PARALLAX_BROWSER_FOUNDATION_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let ui_dist = root.join("ui/dist/client");
    let shell = ui_dist.join("_shell.html");
    if !shell.is_file() {
        bail!(
            "foundation server requires built UI at {} — run `cd ui && bun run build` first",
            shell.display()
        );
    }

    let listener = TcpListener::bind((BIND, port)).with_context(|| {
        format!("bind foundation server on {BIND}:{port} (is the port occupied?)")
    })?;

    let manifest_path = root.join("ui/test-results/foundation-runtime.json");
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let pid = std::process::id();
    let manifest = json!({
        "schema_version": 1,
        "bind": BIND,
        "port": port,
        "health_url": format!("http://{BIND}:{port}/health"),
        "ui_dist": ui_dist.display().to_string(),
        "pid": pid,
    });
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)
        .with_context(|| format!("write {}", manifest_path.display()))?;

    println!("==> browser foundation server starting");
    println!("    bind: {BIND}:{port}");
    println!("    ui:   {}", ui_dist.display());
    println!("    manifest: {}", manifest_path.display());
    println!("    health: http://{BIND}:{port}/health");
    println!("Parallax browser foundation ready — Ctrl-C / webServer stop to exit");

    accept_loop(listener, ui_dist)
}

fn accept_loop(listener: TcpListener, ui_dist: PathBuf) -> Result<()> {
    for connection in listener.incoming() {
        let stream = connection.context("accept foundation connection")?;
        let ui_dist = ui_dist.clone();
        thread::spawn(move || serve_or_log(stream, &ui_dist));
    }
    Ok(())
}

fn serve_or_log(stream: TcpStream, ui_dist: &Path) {
    if let Err(error) = handle_client(stream, ui_dist) {
        eprintln!("foundation server request error: {error:#}");
    }
}

fn handle_client(mut stream: TcpStream, ui_dist: &Path) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut buffer = [0_u8; 16_384];
    let read = stream.read(&mut buffer)?;
    if read == 0 {
        return Ok(());
    }
    let request = String::from_utf8_lossy(&buffer[..read]);
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/");
    let path = path.split('?').next().unwrap_or(path);

    if method == "GET" && path == "/health" {
        return write_response(&mut stream, 200, "text/plain; charset=utf-8", b"ok");
    }

    if method == "POST" && path == "/graphql" {
        return write_response(
            &mut stream,
            200,
            "application/json; charset=utf-8",
            GRAPHQL_STUB.as_bytes(),
        );
    }

    if method != "GET" && method != "HEAD" {
        return write_response(
            &mut stream,
            405,
            "text/plain; charset=utf-8",
            b"method not allowed",
        );
    }

    let relative = path.trim_start_matches('/');
    let candidate = if relative.is_empty() {
        Some(ui_dist.join("_shell.html"))
    } else {
        sanitize_join(ui_dist, relative)
    };
    let file = match candidate {
        Some(path) if path.is_file() => path,
        _ => ui_dist.join("_shell.html"),
    };
    let body = fs::read(&file).with_context(|| format!("read {}", file.display()))?;
    let content_type = content_type_for(&file);
    if method == "HEAD" {
        write_headers(&mut stream, 200, content_type, body.len())?;
    } else {
        write_response(&mut stream, 200, content_type, &body)?;
    }
    drop(stream.shutdown(Shutdown::Both));
    Ok(())
}

fn sanitize_join(root: &Path, relative: &str) -> Option<PathBuf> {
    let mut path = root.to_path_buf();
    for component in Path::new(relative).components() {
        match component {
            std::path::Component::Normal(part) => path.push(part),
            std::path::Component::CurDir => {}
            _ => return None,
        }
    }
    Some(path)
}

fn content_type_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "map" => "application/json",
        _ => "application/octet-stream",
    }
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<()> {
    write_headers(stream, status, content_type, body.len())?;
    stream.write_all(body)?;
    Ok(())
}

fn write_headers(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    length: usize,
) -> Result<()> {
    let reason = match status {
        200 => "OK",
        405 => "Method Not Allowed",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {length}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n"
    );
    stream.write_all(header.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_types_cover_ui_assets() {
        assert_eq!(
            content_type_for(Path::new("a.html")),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            content_type_for(Path::new("a.js")),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(
            content_type_for(Path::new("a.css")),
            "text/css; charset=utf-8"
        );
    }

    #[test]
    fn graphql_stub_is_valid_json() {
        let value: serde_json::Value =
            serde_json::from_str(GRAPHQL_STUB).expect("graphql stub json");
        assert_eq!(value["data"]["health"], "ok");
        assert!(value["data"]["dashboards"].as_array().unwrap().is_empty());
    }

    #[test]
    fn sanitize_join_rejects_parent_segments() {
        assert!(sanitize_join(Path::new("/tmp/ui"), "../secret").is_none());
        assert_eq!(
            sanitize_join(Path::new("/tmp/ui"), "assets/app.js"),
            Some(PathBuf::from("/tmp/ui/assets/app.js"))
        );
    }
}
