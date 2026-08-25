# Parallax public walkthrough: CLI → browser

**Review date:** 2026-08-25  
**Channel:** latest Homebrew preview, resolved at install time
**Repositories:** [parallax](https://github.com/tailrocks/parallax) ·
[parallax-telemetry-playground](https://github.com/tailrocks/parallax-telemetry-playground)

Use this document in order. The five-minute path proves the product story;
the feature map covers the rest of the UI without forcing the presenter to
open every page.

## Navigation

- [Five-minute path](#five-minute-path)
- [Full feature map](#full-feature-map)
- [CLI checkpoints](#cli-checkpoints)
- [Stop and reset](#stop-and-reset)
- [Verified boundaries](#verified-boundaries)

## 1. Install the current preview CLI

Requirements: macOS or Linux, Homebrew, Docker Desktop running, `git`, `curl`,
`nc`, `python3`, and `agent-browser`. The playground builds several
containers; leave about 10 GB free as a planning estimate.

First install:

```bash
brew tap tailrocks/parallax
brew update
brew install parallax@preview
```

Later runs:

```bash
brew update
brew upgrade parallax@preview
```

Verify before presenting:

```bash
parallax --version
```

The version printed by `parallax --version` is the preview resolved by Homebrew
at install time. Always refresh and resolve the latest preview before a run;
record that output with the demo notes. Never pin a preview build or switch to
the stable `parallax` formula.

Set the playground path once. Replace the path with the checkout on your
machine:

```bash
export PLAYGROUND_DIR="/path/to/parallax-telemetry-playground"
```

Use a dedicated `agent-browser` session for every browser check:

```bash
export AGENT_BROWSER_SESSION="$(agent-browser session id --scope worktree --prefix parallax-public)"
```

## 2. Start Parallax

Terminal A — keep this running:

```bash
parallax serve
```

Wait for the `Parallax ready` banner. It gives the same endpoints used below:

| Surface | Address |
| --- | --- |
| Web UI + GraphQL | <http://127.0.0.1:4000> |
| OTLP/gRPC ingest | `127.0.0.1:4317` |
| OTLP/HTTP ingest | `127.0.0.1:4318` |
| Managed GreptimeDB | `127.0.0.1:24000` |

Terminal B — verify readiness:

```bash
curl -fsS http://127.0.0.1:4000/health
curl -fsS http://127.0.0.1:4000/version
```

Expected: `ok`, then API schema version `0.1.0`. That endpoint is not the
release/channel identifier; the build identity is the CLI output from
`parallax --version` above.

Optional diagnostics:

```bash
parallax doctor
```

The important gates are `api (:4000): ok` and `greptime child (:24000): ok`.
On this verification, a live server also printed a non-fatal metadata lock note
for deploy-context deliveries; do not present that optional line as a demo
feature.

## 3. Start the telemetry playground

Terminal B:

```bash
cd "$PLAYGROUND_DIR"
./demo.sh
```

`demo.sh` checks Parallax's OTLP/gRPC port, builds the images, starts the demo
profile, and starts background traffic. Verify the two browser-facing services:

```bash
curl -fsS http://localhost:5173/
curl -fsS http://localhost:8088/healthz
```

Open these during the presentation:

| Page | Purpose |
| --- | --- |
| <http://localhost:5173/> | Playground home and browser actions |
| <http://localhost:5173/checkout> | Browser → backend checkout |
| <http://localhost:5173/orders> | Browser order journey |
| <http://127.0.0.1:4000/> | Parallax overview |

## 4. Five-minute path

Run one command, then show one result. Use the same Parallax browser tab and
open each route directly when listed.

### A. Distributed trace — `a1`

Terminal B:

```bash
cd "$PLAYGROUND_DIR"
./scenarios/run.sh a1
```

Then in Parallax:

1. Open <http://127.0.0.1:4000/traces>.
2. Open a recent `[checkout] http.server.request` row.
3. Show the waterfall: checkout → pricing, inventory/Postgres, and
   recommendation.

Use the current run's result when precision matters; do not record a fixed
trace ID because `a1` generates new IDs on each run:

```bash
parallax traces --service checkout --grep http.server.request --since 2m --limit 5
parallax trace inspect <trace-id>
```

Present this sentence: **one checkout request becomes one correlated,
cross-service trace.**

### B. Browser RUM stitch — `a28`

Terminal B:

```bash
cd "$PLAYGROUND_DIR"
./scenarios/run.sh a28
```

Use `agent-browser` for the printed browser steps at <http://localhost:5173/>.
After every navigation or dynamic update, refresh the accessibility snapshot
and use its current `@ref` values:

```bash
agent-browser open http://localhost:5173/
agent-browser wait --load networkidle
agent-browser snapshot -i
```

Then:

1. Click **checkout journey** and submit the default checkout.
2. Open **orders** and submit an order.
3. Return home and click **apply promo (unresponsive)** several times.
4. Click **break (RUM error)**.
5. Open <http://localhost:5173/checkout?nopropagate=1> directly and submit
   again. Do not use the page's propagation-break link; its current encoding
   is incorrect.
6. Background or close the tab so browser telemetry flushes.

Then in Parallax `/traces`, find the recent `web` traces. Show that the normal
checkout shares a trace with the backend; the `nopropagate` variant is the
intentional disconnected comparison.

### C. Grouped issues — `a31`

Terminal B:

```bash
cd "$PLAYGROUND_DIR"
./scenarios/run.sh a31
```

Then in Parallax:

1. Open <http://127.0.0.1:4000/issues>.
2. Open `PaymentError` and show its occurrences and linked trace.
3. Compare the handled `502` with the unhandled panic, which may appear as
   `500` or a connection reset (`000`).

CLI handoff:

```bash
parallax issue list --status open
parallax issue context <fingerprint>
parallax issue context <fingerprint> --format json
```

Present this sentence: **the CLI and browser show the same bounded evidence
for the failure.** Treat the bundle as sensitive even with bounded redaction.

### D. Metrics and exemplars — `a2`

Terminal B:

```bash
cd "$PLAYGROUND_DIR"
./scenarios/run.sh a2
```

Open `/metrics`, discover the recent `catalog_product_queries_total` series,
and show a trace-linked exemplar when present. If the UI presents a semantic
alias, confirm it resolves to that emitted metric name; do not assume a fixed
series exists in every dataset.

### E. Structured logs — `a9`

Terminal B:

```bash
cd "$PLAYGROUND_DIR"
./scenarios/run.sh a9
```

Open `/logs`, use the recent time window, and show that
The earlier browser spike report said `app_screen_name=workspace-select`
dominated the result, but that claim was not reproduced in the current
dataset. Query the field and report it only if the current run contains it;
otherwise state that it was not observed. Use **Live** only after the static
result is clear.

## 5. Full feature map

This is the complete presentation map. The first five rows above are the
recommended story; the remaining rows are short optional stops.

| Parallax route | Show | Producer/checkpoint |
| --- | --- | --- |
| `/` | Overview: telemetry volume, error rate, latency, recent issues, slow traces | Background traffic or `a1` |
| `/issues` | Grouped failures, occurrences, linked traces, resolve state | `a31`; CLI `parallax issue list` |
| `/tests` | Test run/session evidence | Acceptance run in [playground README](https://github.com/tailrocks/parallax-telemetry-playground#test-telemetry-conventions) |
| `/traces` | Waterfalls, filters, field facets, live tail | `a1`, `a6`, `a3`, `a23` |
| `/ecosystem` | Detected languages, SDKs, and instrumentation inventory | Background traffic |
| `/logs` | Structured fields, trace links, SQL mode, live tail | `a9`; optional `./scenarios/run.sh c3` |
| `/metrics` | Series, windows, finite samples, exemplars | `a2` |
| `/services` | Service inventory and dependency/runtime context | `a1`; optional `b5` |
| `/invocations` (sidebar: **CLI Apps**) | Bounded command execution, exit code, traces, issues | `parallax invocation start -- echo parallax-demo` |
| `/alerts` | Alert rules, destinations, and incident state | `./scenarios/run.sh c4` (rule setup verified; incident opening currently unverified) |
| `/dashboards` | Saved telemetry views | `./scenarios/run.sh c5` |
| `/investigations` | Saved investigation state | `./scenarios/run.sh c5` |
| `/sql` | Read-only query against GreptimeDB telemetry tables | Query below |

Optional deep-dive commands:

```bash
# GraphQL batching vs N+1, partial errors, operation-name cardinality.
cd "$PLAYGROUND_DIR"
./scenarios/run.sh a6

# Async producer → consumer span link.
./scenarios/run.sh a3

# Rust storefront GraphQL → Java payment gRPC.
./scenarios/run.sh a23

# Capture one bounded CLI invocation; copy the printed ID.
cd "$PLAYGROUND_DIR"
parallax invocation start -- echo parallax-demo
parallax invocation inspect <invocation-id>
parallax invocation bundle <invocation-id>

# Read-only raw telemetry query. Paste the same query into Parallax /sql.
parallax sql 'SELECT * FROM `opentelemetry_logs` ORDER BY timestamp DESC LIMIT 5'
```

For live tails, use a bounded window so the presentation ends:

```bash
parallax logs --follow --grep "checkout" --for 30s
parallax traces --follow --errors --service checkout --for 30s
```

## 6. CLI checkpoints

Use these after any scenario. Copy IDs from output; background load makes “most
recent” a navigation hint, not proof that a row came from the last command.

```bash
# Traces.
parallax traces --service checkout --since 15m --limit 20
parallax traces --errors --since 15m --limit 20
parallax trace inspect <trace-id>

# Logs.
parallax logs --service checkout --since 15m --limit 50
parallax logs --trace <trace-id>

# Issues and bounded agent context.
parallax issue list --status open
parallax issue context <fingerprint> --format json

# Invocations.
parallax invocation list
parallax invocation inspect <invocation-id>

# Read-only SQL.
parallax sql 'SELECT * FROM `opentelemetry_logs` ORDER BY timestamp DESC LIMIT 5'
```

## Stop and reset

Stop the playground in Terminal B:

```bash
cd "$PLAYGROUND_DIR"
docker compose -f deploy/docker-compose.yml --profile demo down
```

Stop Parallax with `Ctrl-C` in Terminal A. This preserves Parallax data under
`~/.parallax`.

For a disposable playground reset only, remove its volumes too:

```bash
docker compose -f deploy/docker-compose.yml --profile demo down -v
```

Do not use `down -v` when the generated data is needed for the next presenter.

## Verified boundaries

- The walkthrough uses the Homebrew preview CLI, not a source build or stable
  formula.
- The demo is local-only. Default auth is off; OTLP listeners are on loopback.
- Playground containers are unauthenticated demo services with demo credentials;
  do not expose them to a shared network.
- `a28` requires real browser interaction through `agent-browser`. A shell
  smoke check cannot prove RUM navigation or propagation.
- Browser verification used `agent-browser` on the latest locally resolved
  preview: Overview, trace detail, Issues, Logs, Metrics, CLI Apps, and SQL
  loaded; the playground home, checkout, and exact `nopropagate=1` route also
  worked. Keep route-specific visual claims limited to pages opened in the
  presentation session.
- `c4` created the alert rule and destinations, but the current preview did not
  open an incident during its 180-second poll. Treat incident creation as an
  open product gap; reproduce with sustained breach traffic before presenting.
- Background load can create newer rows while presenting. Pin a trace ID or use
  service/time filters before making a claim.
- This proves local ingest, correlation, UI navigation, CLI inspection, and the
  evidence handoff. It does not prove production scale, TLS, multi-user
  isolation, hosted MCP, autonomous fixing, or replacement parity with mature
  observability products.

## Sources of truth

- Install and ports: Homebrew formula plus `parallax serve --help`
- CLI behavior: `parallax --help` and `crates/parallax-cli/src/main.rs`
- Playground startup: `parallax-telemetry-playground/demo.sh`
- Browser journey: `parallax-telemetry-playground/scenarios/a28-rum-journey.sh`
- Scenario catalog: `parallax-telemetry-playground/scenarios/README.md`
- UI routes: `ui/src/routes/`
