# Instrument snippets

Copy-paste OTLP setup for a local Parallax (`parallax serve`). Default receivers
are gRPC `:4317` and HTTP `:4318`. Required resource: `service.name`. Also send
`service.version` and `deployment.environment.name` when you have them
([conventions.md](conventions.md)).

Plan 172's empty-state tabs consume this file. Do not copy Maple or any
FSL-licensed source — these snippets are re-derived from the playground's
Apache-2.0 checkout.

## Rust (tracing + opentelemetry-otlp)

Verified 2026-08-14 against `libs/playground-telemetry/src/lib.rs` @ `171d87a`
in `tailrocks/parallax-telemetry-playground`.

```rust
use opentelemetry::KeyValue;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn init_tracing(service_name: &'static str) -> anyhow::Result<SdkTracerProvider> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:4318".into());
    let exporter = SpanExporter::builder()
        .with_http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
        .with_endpoint(format!("{endpoint}/v1/traces"))
        .build()?;
    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", service_name),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    KeyValue::new("deployment.environment.name", "dev"),
                ])
                .build(),
        )
        .with_batch_exporter(exporter)
        .build();
    tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(provider.tracer(service_name)))
        .init();
    Ok(provider)
}
```

Point `OTEL_EXPORTER_OTLP_ENDPOINT` at `http://127.0.0.1:4317` and switch the
exporter to gRPC when you prefer the gRPC port.

## Java (OpenTelemetry javaagent)

Verified 2026-08-25 against the sibling playground's `deploy/Dockerfile.java`
(upstream Java agent 2.30.0) and the catalog/payment/fulfillment
`OTEL_*` blocks in `deploy/docker-compose.yml` at `171d87a`.

```bash
# Download the upstream agent (do not swap in a vendor-only agent).
curl -fsSL -o otel-agent.jar \
  https://repo1.maven.org/maven2/io/opentelemetry/javaagent/opentelemetry-javaagent/2.30.0/opentelemetry-javaagent-2.30.0.jar

export JAVA_TOOL_OPTIONS="-javaagent:./otel-agent.jar"
export OTEL_SERVICE_NAME="catalog"
export OTEL_RESOURCE_ATTRIBUTES="service.version=0.1.0,deployment.environment.name=dev"
export OTEL_EXPORTER_OTLP_ENDPOINT="http://127.0.0.1:4317"
export OTEL_EXPORTER_OTLP_PROTOCOL="grpc"
export OTEL_TRACES_EXPORTER="otlp"
export OTEL_METRICS_EXPORTER="otlp"
export OTEL_LOGS_EXPORTER="otlp"
export OTEL_PROPAGATORS="tracecontext,baggage"

java -jar app.jar
```

The sibling playground verifies OTLP/gRPC to `:4317` with
`OTEL_EXPORTER_OTLP_PROTOCOL=grpc`. It does not verify this standalone
copy-paste launch, HTTP/protobuf to `:4318`, or a Java application outside that
playground deployment; treat those as unverified here.

## JS / browser (sdk-trace-web)

Verified 2026-08-14 against `web/src/telemetry.ts` @ `171d87a` in
`tailrocks/parallax-telemetry-playground`.

```ts
import { WebTracerProvider, BatchSpanProcessor } from "@opentelemetry/sdk-trace-web"
import { OTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-proto"
import {
  defaultResource,
  resourceFromAttributes,
} from "@opentelemetry/resources"
import {
  ATTR_SERVICE_NAME,
  ATTR_SERVICE_VERSION,
} from "@opentelemetry/semantic-conventions"

const provider = new WebTracerProvider({
  resource: defaultResource().merge(
    resourceFromAttributes({
      [ATTR_SERVICE_NAME]: "web",
      [ATTR_SERVICE_VERSION]: "0.1.0",
      "deployment.environment.name": "dev",
    })
  ),
  spanProcessors: [
    new BatchSpanProcessor(
      new OTLPTraceExporter({
        url: "http://127.0.0.1:4318/v1/traces",
      })
    ),
  ],
})
provider.register()
```

Browser pages usually cannot speak gRPC; keep HTTP `:4318`. Same-origin proxy
(`/v1/traces`) is what the playground web app uses in production-shaped deploys.
