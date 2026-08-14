import { useState } from "react"

import { CopyButton } from "@/shared/console/copy-button"
import { cn } from "@/lib/utils"

export const SNIPPET_TABS = [
  {
    id: "rust",
    label: "Rust",
    code: `use opentelemetry::KeyValue;
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
}`,
  },
  {
    id: "java",
    label: "Java",
    code: `# Download the upstream agent (do not swap in a vendor-only agent).
curl -fsSL -o otel-agent.jar \\
  https://repo1.maven.org/maven2/io/opentelemetry/javaagent/opentelemetry-javaagent/2.29.0/opentelemetry-javaagent-2.29.0.jar

export JAVA_TOOL_OPTIONS="-javaagent:./otel-agent.jar"
export OTEL_SERVICE_NAME="catalog"
export OTEL_RESOURCE_ATTRIBUTES="service.version=0.1.0,deployment.environment.name=dev"
export OTEL_EXPORTER_OTLP_ENDPOINT="http://127.0.0.1:4318"
export OTEL_EXPORTER_OTLP_PROTOCOL="http/protobuf"
export OTEL_TRACES_EXPORTER="otlp"
export OTEL_METRICS_EXPORTER="otlp"
export OTEL_LOGS_EXPORTER="otlp"
export OTEL_PROPAGATORS="tracecontext,baggage"

java -jar app.jar`,
  },
  {
    id: "js",
    label: "JS",
    code: `import { WebTracerProvider, BatchSpanProcessor } from "@opentelemetry/sdk-trace-web"
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
provider.register()`,
  },
] as const

export type SnippetTabId = (typeof SNIPPET_TABS)[number]["id"]

export function snippetFor(id: SnippetTabId): string {
  const tab = SNIPPET_TABS.find((item) => item.id === id)
  return tab?.code ?? SNIPPET_TABS[0].code
}

function SnippetCode({ code }: { code: string }) {
  return (
    <pre tabIndex={0} className="max-h-72 min-w-0 flex-1 overflow-auto font-mono text-xs leading-5">
      <code>{code}</code>
    </pre>
  )
}

export function SnippetTabs({ className }: { className?: string }) {
  const [active, setActive] = useState<SnippetTabId>("rust")
  const activeIndex = SNIPPET_TABS.findIndex((tab) => tab.id === active)
  const visible = snippetFor(active)

  return (
    <div className={cn("grid gap-3", className)} data-testid="instrument-snippet-tabs">
      <div
        role="tablist"
        aria-label="Instrumentation language"
        className="relative inline-flex w-fit rounded-full bg-muted p-0.5"
      >
        <span
          aria-hidden="true"
          className="pointer-events-none absolute top-0.5 bottom-0.5 rounded-full bg-background shadow-(--elevation-1) transition-transform duration-100 ease-out motion-reduce:transition-none"
          style={{
            width: `calc((100% - 4px) / ${SNIPPET_TABS.length})`,
            transform: `translateX(${activeIndex * 100}%)`,
          }}
        />
        {SNIPPET_TABS.map((tab) => {
          const selected = tab.id === active
          return (
            <button
              key={tab.id}
              type="button"
              role="tab"
              id={`snippet-tab-${tab.id}`}
              aria-selected={selected}
              aria-controls={`snippet-panel-${tab.id}`}
              tabIndex={selected ? 0 : -1}
              className={cn(
                "relative z-10 min-w-16 rounded-full px-3 py-1 text-xs font-medium transition-colors",
                selected ? "text-foreground" : "text-muted-foreground hover:text-foreground"
              )}
              onClick={() => setActive(tab.id)}
            >
              {tab.label}
            </button>
          )
        })}
      </div>

      <div className="grid">
        {SNIPPET_TABS.map((tab) => {
          const selected = tab.id === active
          return (
            <div
              key={tab.id}
              role="tabpanel"
              id={`snippet-panel-${tab.id}`}
              aria-labelledby={`snippet-tab-${tab.id}`}
              hidden={!selected}
              className={cn(
                "col-start-1 row-start-1 min-h-48",
                selected ? "opacity-100" : "pointer-events-none opacity-0",
                "transition-opacity duration-100 ease-out motion-reduce:transition-none"
              )}
            >
              <div className="flex items-start gap-2 rounded-lg border border-dashed border-border/70 bg-background/60 p-3">
                <SnippetCode code={tab.code} />
                {selected ? <CopyButton value={visible} /> : null}
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
