# jackin' — the first real CLI on Parallax

Research date: 2026-06-12. Status: integration implemented and verified
end-to-end on the operator's machine; jackin' PR open (branch
`feature/parallax-run-telemetry` in `jackin-project/jackin`).

## Goal

jackin' is the first real product wired to Parallax — the prototype of the
"run id answers everything" loop from the vision: the operator starts a CLI
(debug mode or not), gets a run id, and that one id opens the complete
external view — all logs, all states, all performance, how long each thing
took. This is the A2-shaped proof that the local profile serves a real
development workflow, not just synthetic acceptance tests.

## What jackin' already had, and what was missing

jackin' has a serious host-observability substrate of its own: every command
mints a run id (`jk-run-<6 hex>`), streams structured events through
`tracing` into `~/.jackin/data/diagnostics/runs/<run-id>.jsonl`, tracks
per-stage wall-clock durations, and prints the run id at startup under
`--debug`. It even had a skeletal `otlp` cargo feature exporting spans when
`JACKIN_OTLP_ENDPOINT` was set.

What the skeleton could not do:

| Gap | Consequence |
| --- | --- |
| No OTLP resource attributes | `service.name` was `unknown_service`; no run id on telemetry → nothing queryable by run |
| No log export | The diagnostics event stream (the actual story) never left the machine — spans only |
| Async reqwest client on the batch thread | Exporter thread panicked "there is no reactor running" → **zero bytes ever exported**; the panic was on a background thread, so it was invisible |
| No flush on exit | Even working exporters would drop short runs' tails |
| Non-default feature | Installed binaries could never export without a rebuild |

The skeleton had plausibly never moved a byte. The netcat probe (0 bytes
captured across full runs) and the background-thread panic confirm it.

## What the PR adds (design)

One principle: **the file is the contract, OTLP is a second sink for the
same events.** No new event taxonomy, no jackin'-side Parallax client, no
GraphQL dependency — pure OTel, runtime-gated on an env var.

- `RunDiagnostics::start` mints the run id **before** installing the
  tracing subscriber, so the OTLP resource can carry it:
  `service.name=jackin`, `service.version`, `jackin.run_id`, and
  `parallax.run.id` (the latter skipped when a wrapper already injected one
  via `OTEL_RESOURCE_ATTRIBUTES` — `parallax run start -- jackin …` wins).
  Parallax promotes `parallax.run.id` to a real column, so
  `logsByRun`/`tracesByRun` answer with jackin's own printed id.
- **Logs**: `opentelemetry-appender-tracing` bridges every tracing event to
  an OTLP log record — jackin's diagnostics events (kind/stage/detail as
  attributes, message as body) plus third-party crate telemetry (bollard's
  Docker request traces show up for free).
- **Spans**: the existing `launch_stage` tracing spans export through
  `tracing-opentelemetry` with real durations — "how much each thing takes"
  is a waterfall.
- **Two-tier severity preserved**: `kind: "debug"` events emit at DEBUG; the
  export layers filter at INFO normally, DEBUG with `--debug` — identical
  contract to the file/`clog!`/`cdebug!` rule.
- **Endpoint**: `JACKIN_OTLP_ENDPOINT` first, standard
  `OTEL_EXPORTER_OTLP_ENDPOINT` fallback; bare endpoints get `/v1/traces`
  / `/v1/logs` appended.
- **Reliability fixes**: blocking reqwest client (batch processors run on
  reactor-less dedicated threads); `shutdown_otlp()` force-flush at the end
  of `app::run` (both exits); `otlp` made a default feature of the binary
  (runtime no-op without an endpoint; `--no-default-features` removes the
  deps).

## Verified end-to-end (operator machine, 2026-06-12)

Against a live `parallax serve` (managed GreptimeDB, default ports):

1. `JACKIN_OTLP_ENDPOINT=http://127.0.0.1:4318 jackin doctor --debug`
   printed `jk-run-b2788f`; Parallax `logsByRun(runId: "jk-run-b2788f")`
   returned 15 log records with bodies — including jackin's own
   `[jackin debug docker] connect context …` firehose lines and bollard's
   request traces.
2. A `launch_stage` span probe exported with a measured 30.1 ms duration,
   `stage` attribute, and code location; `tracesByRun(runId: …)` returned
   it by the same run id.
3. Root-caused two real defects while wiring: the async-client
   background-thread panic (silent total export failure) and empty log
   bodies (no format message on the tracing events). Both fixed in the PR.
4. jackin's quality gates: `cargo fmt --check`, strict clippy
   (all-targets/all-features), diagnostics suite 21/21, docs build +
   repo-link check + tsc + bun tests green. One unrelated pre-existing test
   failure on this machine (`/private/var` vs `/var` TMPDIR canonicalization
   in a kimi auth-dialog test) fails identically on pristine `main`.

## What this proves for Parallax

- **The wrapper-less path works.** A real CLI with its own run concept maps
  onto Parallax with one resource attribute — no `parallax run start`
  required. The `parallax.run.id` promotion to a column is the load-bearing
  feature.
- **The logs pipeline carries a real firehose** — third-party crate noise,
  hex-encoded unix-socket URLs, multi-KB debug lines — not just curated
  fixtures.
- **Gap found (UI):** run-id lookup in the Parallax UI exists only for
  wrapper-registered runs (runs page) — jackin's runs are queryable via CLI
  (`parallax logs --run …`) and GraphQL but don't appear in the runs list,
  and there is no UI search box for a pasted run id. Follow-up: a run-id
  lookup input (mirroring the trace-id lookup page) or auto-registering
  externally-seen run ids from ingest.
  **Closed 2026-06-12:** the ingest worker now auto-registers every run id
  first seen in telemetry (status `external`, spec §6), the runs list shows
  them, `/runs/$runId` is a full detail page (issues, trace summaries, logs,
  run-anchored bundle), and the traces page gained a run-id lookup.
- **Gap found (metrics):** jackin' exports no OTLP metrics yet — stage
  durations arrive as spans and the run summary as a log record, which
  covers the ask, but counter/gauge export (cache hits, event counts as
  real metrics) is a natural follow-up once jackin' wants charts.
  **Closed 2026-06-12 (PR commit `97dd79b4`):** jackin' now exports gauges
  every 5 s while the process runs — `process.cpu.utilization` (0..1
  fraction of all cores via sysinfo), `process.memory.usage` (resident
  bytes), and the stable tokio runtime counters
  `tokio.runtime.{workers,alive_tasks,global_queue_depth}` — all tagged
  with the same run-id resource attributes as the spans and logs. On the
  Parallax side `otel_metrics_points` gained a promoted `run_id` column
  (ALTER-migrated on existing installs), `metricSeries` takes `runId:`,
  the services list unions all three signal tables (a logs-only or
  metrics-only service now appears), and `/runs/$runId` shows a
  run-scoped Process metrics card (CPU, memory, alive tasks) beside the
  run's issues, traces, and logs — the cross-analytics view. Verified
  live with `jackin doctor --debug` → run `jk-run-e4ba1d`: 5 gauge
  series ingested run-tagged, `services` returned `jackin`, the run page
  rendered all three charts (memory ≈ 25 MiB; CPU's first sysinfo sample
  is 0 by design, so short runs chart flat — real values appear from the
  second 5 s tick on longer runs).

## How the operator verifies (jackin' rule: every command with `--debug`)

```sh
# 1. Parallax up (one binary; engine auto-downloads on first run)
parallax serve

# 2. Any jackin' command from the PR branch, exporting:
cd ~/Projects/jackin-project/jackin
git fetch origin pull/<PR>/head:pr-parallax && git checkout pr-parallax
JACKIN_OTLP_ENDPOINT=http://127.0.0.1:4318 \
  cargo run --bin jackin -- console --debug
#    → note the printed run id, e.g. jk-run-8b4766

# 3. The external view, by that id:
parallax logs --run jk-run-8b4766                 # all logs
parallax logs --run jk-run-8b4766 --grep docker   # filtered
#    traces (stage timings) via GraphQL or the UI:
#    { tracesByRun(runId: "jk-run-8b4766") { traceId spans { name durationNs } } }
#    UI: http://127.0.0.1:4000 → services → jackin; traces → paste a trace id

# 4. Non-debug run: same id mechanics, compact INFO-level export
JACKIN_OTLP_ENDPOINT=http://127.0.0.1:4318 cargo run --bin jackin -- doctor --debug
```

The diagnostics file keeps working unchanged in all cases —
`~/.jackin/data/diagnostics/runs/<run-id>.jsonl` is written whether or not
the endpoint is set.
