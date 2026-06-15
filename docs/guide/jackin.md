# Running jackin' with Parallax locally

jackin' is the first real integration target: a run of it exports traces,
logs, and process metrics to Parallax under one run id. This page is the
minimal recipe — start it, connect it, see everything.

## 1. Start Parallax

```sh
parallax serve
```

Wait for the ready banner: UI on `http://127.0.0.1:4000`, OTLP/HTTP on
`:4318`.

## 2. Build jackin' with OTLP support

Until [jackin-project/jackin#581](https://github.com/jackin-project/jackin/pull/581)
merges, build from that branch:

```sh
git clone https://github.com/jackin-project/jackin
cd jackin
git switch feature/parallax-run-telemetry
cargo build --bin jackin     # `otlp` is a default feature of the binary
```

## 3. Start jackin' connected to Parallax

Preferred: let the Parallax wrapper assign the run id, inject OTLP endpoints,
and keep the same id for UI, logs, traces, metrics, and CLI lookup:

```sh
parallax run start -- ./target/debug/jackin console --debug
```

The first line is the id to use everywhere:

```text
Parallax run id: 18b946258b86fe20
command: ./target/debug/jackin console --debug
live: parallax run watch 18b946258b86fe20
```

jackin' should not mint a separate Parallax-facing run id under the wrapper.
The Parallax id is the single lookup handle for Parallax pages and commands.

Wrapper-less mode is still useful when testing jackin' directly. The exporter
is off until an endpoint and run id are set. Point it at Parallax, provide the
same run id in OTel resource attributes, and run any command — `--debug`
exports the full firehose, without it only lifecycle/errors:

```sh
RUN_ID=18b946258b86fe20
OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317 \
OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://127.0.0.1:4317 \
OTEL_EXPORTER_OTLP_LOGS_ENDPOINT=http://127.0.0.1:4317 \
OTEL_EXPORTER_OTLP_METRICS_ENDPOINT=http://127.0.0.1:4317 \
OTEL_EXPORTER_OTLP_PROFILES_ENDPOINT=http://127.0.0.1:4317 \
OTEL_EXPORTER_OTLP_PROTOCOL=grpc \
OTEL_EXPORTER_OTLP_TRACES_PROTOCOL=grpc \
OTEL_EXPORTER_OTLP_LOGS_PROTOCOL=grpc \
OTEL_EXPORTER_OTLP_METRICS_PROTOCOL=grpc \
OTEL_EXPORTER_OTLP_PROFILES_PROTOCOL=grpc \
OTEL_RESOURCE_ATTRIBUTES="parallax.run.id=$RUN_ID" \
  ./target/debug/jackin console --debug
```

The wrapper sets the same standard OTel variables, preferring OTLP/gRPC on
`:4317` for every signal. Avoid `JACKIN_OTLP_ENDPOINT` for Parallax runs; it
is a legacy jackin-specific escape hatch and can bypass the standard
per-signal settings.

## 4. See everything under that run id

UI: open `http://127.0.0.1:4000/runs/<parallax-run-id>` — issues, stage-timing
traces, the log stream, the process-metrics card (CPU, memory, tokio
tasks), and the evidence bundle on one page. `jackin` also appears in the
service selector on the Logs/Traces/Services pages.

On the run page, **Go live** streams the run's new logs and finished spans
over SSE and repolls the metrics card every 5 s — the single observation
entrance while the run executes.

CLI (what an agent runs):

```sh
parallax run watch 18b946258b86fe20 --for 30s  # live logs + spans, then counts
parallax run inspect 18b946258b86fe20        # status, counts, issues
parallax logs --run 18b946258b86fe20         # the run's log stream
parallax traces --run 18b946258b86fe20       # the run's traces
parallax run bundle 18b946258b86fe20         # agent handoff (Markdown)
parallax sql "SELECT name, avg(value) FROM otel_metrics_points \
  WHERE run_id = '18b946258b86fe20' GROUP BY name"
```

The same id also names the local diagnostics file jackin' keeps on its own:
`~/.jackin/data/diagnostics/runs/<run-id>.jsonl`.

## Notes

- Metrics arrive every 5 s while the process lives; the first CPU sample
  is always 0 (sysinfo needs two samples), so very short runs chart CPU
  flat. Memory and tokio gauges are real from the first point.
- Deeper background: integration analysis in
  [docs/research/validation/jackin-first-integration.md](../research/validation/jackin-first-integration.md),
  jackin'-side docs in its repo under `docs/content/docs/guides/run-telemetry.mdx`.
