# Quickstart: install → serve → connect a Rust app → first bundle

Parallax V1 is the self-sufficient local profile: one binary, no Docker, no
auth, no cloud. Everything below happens on your machine.

## 1. Install

Preview builds install from the per-project Homebrew tap:

```sh
brew tap tailrocks/parallax
brew install parallax@preview
parallax --version
```

For local development from the repository:

```sh
cargo install --path crates/parallax-cli
parallax --version
```

## 2. Serve

```sh
parallax serve
```

First start downloads a pinned, checksum-verified GreptimeDB and supervises it
as a child process (ports `24000–24003`, data under `~/.parallax/`). Parallax
itself listens on:

| Port | What |
| --- | --- |
| `:4317` | OTLP/gRPC ingest |
| `:4318` | OTLP/HTTP ingest |
| `:4000` | GraphQL API + web UI (`http://127.0.0.1:4000`) |

No GreptimeDB? `storage.mode = "none"` in `~/.parallax/config.toml` runs the
in-memory store (degraded, still fully functional for a dev loop).

## 3. Connect a Rust app

Standard OTel crates only — there is no Parallax SDK to add:

```toml
# Cargo.toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-opentelemetry = "0.31"
opentelemetry = "0.30"
opentelemetry_sdk = "0.30"
opentelemetry-otlp = { version = "0.30", features = ["grpc-tonic", "logs"] }
```

```rust
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;

fn init_telemetry() -> anyhow::Result<()> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:4317".into());
    let resource = opentelemetry_sdk::Resource::builder()
        .with_attributes([
            KeyValue::new("service.name", env!("CARGO_PKG_NAME")),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ])
        .build();

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint.clone())
                .build()?,
        )
        .with_resource(resource.clone())
        .build();
    opentelemetry::global::set_tracer_provider(tracer_provider);

    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::LogExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .build()?,
        )
        .with_resource(resource)
        .build();
    // Bridge `tracing` into both signals; see the conventions page for the
    // panic hook and the tracing-opentelemetry layer.
    let _ = logger_provider;
    Ok(())
}
```

Full wiring for the supported stack — tonic, axum, tokio-postgres, the
`clickhouse` crate, redis, lapin, GraphQL — lives in
[rust-stack-instrumentation.md](../research/capture/rust-stack-instrumentation.md).

## 4. Run your app under a run id (optional but worth it)

```sh
parallax run start -- cargo run
```

The wrapper injects OTLP/gRPC endpoint and protocol env vars for traces,
logs, metrics, and profiles plus a `parallax.run.id` resource attribute,
then propagates your program's exit code. Everything the run emitted is now
addressable: `parallax run list`, `parallax logs --run <id>`.

## 5. First bundle

Make your app fail once (panic, error span, ERROR log — all three derive), then:

```sh
parallax issue list
parallax issue context <fingerprint>
```

`issue context` prints the evidence bundle: error identity, trace waterfall
with database query text, correlated logs, **metric windows around the event**
(CPU/memory/tokio gauges, run-scoped under the wrapper), deterministic
hypotheses — bounded to an agent-sized token budget and redacted
(redaction-lite, pre-A6). Paste it into your coding agent, or point the agent
at the command itself — that is the [agent how-to](agent-howto.md).

Two more surfaces worth knowing on day one: `parallax logs --follow`
(kubectl-style live tail, `--for 30s` to watch a window and exit with the
match count) and `parallax sql "<SELECT …>"` (raw read-only queries against
the engine's tables) — both in the [CLI reference](cli.md).
