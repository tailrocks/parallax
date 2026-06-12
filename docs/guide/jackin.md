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

The exporter is off until an endpoint is set. Point it at Parallax and run
any command — `--debug` exports the full firehose, without it only
lifecycle/errors:

```sh
JACKIN_OTLP_ENDPOINT=http://127.0.0.1:4318 ./target/debug/jackin console --debug
```

jackin' prints the run id to save:

```text
[jackin] debug mode — save this run id to retrieve the run later:
    jk-run-e4ba1d
```

Alternative — let the Parallax wrapper inject the endpoint and its own run
id (no env var needed):

```sh
parallax run start -- ./target/debug/jackin console --debug
```

`JACKIN_OTLP_ENDPOINT` wins over the standard
`OTEL_EXPORTER_OTLP_ENDPOINT`; the wrapper sets the standard one.

## 4. See everything under that run id

UI: open `http://127.0.0.1:4000/runs/<run-id>` — issues, stage-timing
traces, the log stream, the process-metrics card (CPU, memory, tokio
tasks), and the evidence bundle on one page. `jackin` also appears in the
service selector on the Logs/Traces/Services pages.

CLI (what an agent runs):

```sh
parallax run inspect jk-run-e4ba1d        # status, counts, issues
parallax logs --run jk-run-e4ba1d         # the run's log stream
parallax logs --service jackin --follow   # live tail while it runs
parallax traces --service jackin --since 1h
parallax run bundle jk-run-e4ba1d         # agent handoff (Markdown)
parallax sql "SELECT name, avg(value) FROM otel_metrics_points \
  WHERE run_id = 'jk-run-e4ba1d' GROUP BY name"
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
