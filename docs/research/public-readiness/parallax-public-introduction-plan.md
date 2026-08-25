# Parallax public introduction: verified local walkthrough

**Review date:** 2026-08-25  
**Verified binary:** `parallax 0.1.0-preview.2496+2c5a2c7`  
**Repositories:** [parallax](https://github.com/tailrocks/parallax) and [parallax-telemetry-playground](https://github.com/tailrocks/parallax-telemetry-playground)

This is the public-facing walkthrough. It is intentionally smaller than the
readiness review: copy the commands, open the listed browser page, and inspect
one clear result at a time.

## What Parallax is

Parallax is a local-first, self-hosted OTLP observability engine for developer
workflows. It receives traces, logs, and metrics, correlates them, derives
grouped issues, and emits bounded evidence for a human or coding agent.

The playground is the workload. It is not an observability backend. Its Rust,
Java, browser, GraphQL, gRPC, Kafka, database, and failure scenarios send
telemetry to Parallax.

This walkthrough proves a local preview demo. It does not prove production
scale, authentication, TLS, multi-user isolation, or replacement parity with
Sentry, Grafana, or Kibana.

## 1. Prerequisites

Use macOS/Linux with:

- Homebrew
- Docker Desktop, running
- `git`, `curl`, `nc`, and `python3`
- roughly 10 GB free disk space as a planning estimate for playground images and
  local engine data; measure your own Docker/cache footprint

Use the preview formula. Do not install the stable `parallax` formula for this
walkthrough.

```bash
brew tap tailrocks/parallax
brew update
brew install parallax@preview
```

If preview is already installed, update it instead:

```bash
brew upgrade parallax@preview
```

Verify the binary before continuing. The exact version changes as preview
builds are published; keep the output in the demo notes.

```bash
parallax --version
```

## 2. Start Parallax

Open Terminal A in any directory:

```bash
parallax serve
```

Wait for the `Parallax ready` banner. It names every surface:

| Surface | URL/port |
| --- | --- |
| Web UI | <http://127.0.0.1:4000> |
| GraphQL | <http://127.0.0.1:4000/graphql> |
| OTLP/gRPC ingest | `127.0.0.1:4317` |
| OTLP/HTTP ingest | `127.0.0.1:4318` |
| Managed GreptimeDB | `127.0.0.1:24000` |

In Terminal B, verify readiness:

```bash
curl -fsS http://127.0.0.1:4000/health
curl -fsS http://127.0.0.1:4000/version
parallax doctor
```

Keep Terminal A running for the entire walkthrough.

## 3. Start the telemetry playground

Open Terminal B. Set `PLAYGROUND_DIR` to the local playground checkout.

```bash
export PLAYGROUND_DIR="/path/to/parallax-telemetry-playground"
cd "$PLAYGROUND_DIR"
export GIT_SHA="$(git rev-parse HEAD)"
./demo.sh
```

`demo.sh` builds and starts the playground containers. It requires Parallax's
OTLP/gRPC listener on `127.0.0.1:4317`. The web app is then available at:

- <http://localhost:5173/> — home
- <http://localhost:5173/checkout> — browser checkout journey
- <http://localhost:5173/orders> — browser orders journey

The demo profile also generates background traffic. Wait until the compose
command reports the services are running before firing scenarios.

Quick checks:

```bash
curl -fsS http://localhost:5173/
curl -fsS http://localhost:8088/healthz
```

## 4. The short feature tour

Run the rows in order. Every row has one producer command and one browser
inspection.

| Step | Run in Terminal B | Then show in Parallax browser |
| --- | --- | --- |
| Distributed trace | `scenarios/run.sh a1` | `/traces`: newest `checkout` trace; open the waterfall and show pricing, inventory, and recommendation children. |
| Browser-to-backend trace | `scenarios/run.sh a28`, then perform the manual browser steps below | `/traces`: browser `web` spans stitched to checkout; compare with the intentionally broken propagation trace. |
| Grouped error | `scenarios/run.sh a31` | `/issues`: handled `PaymentError` occurrence versus the unhandled failure. Open the issue and follow its linked trace/log evidence. |
| Metrics | `scenarios/run.sh a2` | `/metrics`: open the newest catalog metric and show its trace-linked exemplar when present. |
| Structured logs | `scenarios/run.sh a9` | `/logs`: filter the recent window and show the structured field spike. |
| GraphQL shape | `scenarios/run.sh a6` | `/traces`: compare `batchedReviews` with `slowReviews` and show the N+1 child-span shape. |
| Async topology | `scenarios/run.sh a3`; optionally `a4` or `a8` | `/traces`: open the consumer trace and show the span link and downstream service hop. |
| Rust-to-Java gRPC | `scenarios/run.sh a23` | `/traces`: show the storefront GraphQL resolver followed by the Java payment hop. |
| SQL | no producer command required | `/sql`: run the read-only query shown in section 6. |

The demo profile emits background traffic. Treat “newest” as a navigation aid,
not an assertion that the row belongs to the scenario you just ran. For a
reproducible walkthrough, copy the scenario's trace ID or filter by its service,
time window, and known attributes before making a claim.

### Browser-to-backend step (`a28`)

`a28` checks the page endpoints and prints the manual journey. Follow its
instructions in the browser at <http://localhost:5173>:

1. Open **checkout journey** and submit the default checkout.
2. Open **orders journey** and submit an order.
3. Return home and click **apply promo (unresponsive)** several times.
4. Click **break (RUM error)**.
5. Open <http://localhost:5173/checkout?nopropagate=1> and submit again.
6. Background or close the tab so browser telemetry flushes.

The normal checkout should share a trace with the backend. The `nopropagate`
checkout intentionally produces disconnected browser and backend traces. This
is a teaching comparison, not a bug in the walkthrough.

## 5. CLI inspection: the same evidence without the UI

Use Terminal C, or run these commands in Terminal B after a scenario. All
commands target the local default context.

```bash
# Newest traces, including failures.
parallax traces --errors --since 30m --limit 20

# Recent error logs.
parallax logs --level error --since 30m --limit 50

# Grouped issues. Copy one fingerprint from this output.
parallax issue list --status open
parallax issue context <fingerprint>

# Machine-readable evidence for an agent or script.
parallax issue context <fingerprint> --format json

# Read-only SQL against GreptimeDB native telemetry tables.
parallax sql "SELECT * FROM opentelemetry_logs ORDER BY timestamp DESC LIMIT 10"
```

For a specific trace, copy its trace ID from `/traces` or CLI output:

```bash
parallax trace inspect <trace-id>
parallax logs --trace <trace-id>
```

The issue context command is the agent handoff in the preview. Treat its
output as sensitive: bundle-path redaction is bounded (`redaction-lite`/pre-A6),
not a complete safety guarantee.

## 6. Agent handoff: evidence, not autonomy

Give a coding agent the JSON projection after inspecting the issue manually:

```bash
parallax issue context <fingerprint> --format json
```

The local repository-built MCP surface can expose read-only issue context and
agent-session projections. It is optional, local-stdio only, and not a remote
hosted MCP service; parity checking still has a known discrepancy. The agent
proposes and verifies code changes; Parallax does not autonomously modify the
repository.

The agent story is therefore: bounded evidence, correlated telemetry,
deterministic hypotheses, and session context. Bundle-vs-raw fix quality,
remote MCP, and autonomous fixing remain unproven.

## 7. Browser route map

Use this map when presenting the product. Start at `/`, then move left to right
through the evidence path. Do not open every route during the first five-minute
demo.

| Route | Present it as |
| --- | --- |
| `/` | Overview: is data arriving? |
| `/issues` → `/issues/<fingerprint>` | Grouped error → occurrence and evidence |
| `/traces` → `/traces/<trace-id>` | Distributed request → waterfall |
| `/logs` | Correlated structured logs and live tail |
| `/metrics` → `/metrics/<metric-name>` | Metric series, windows, and exemplars |
| `/services` → `/services/<service>` | Service inventory and dependencies |
| `/invocations` → `/invocations/<id>` | Bounded CLI execution units |
| `/tests` → `/tests/<case-key>` | Test evidence when a test session is imported |
| `/sql` | Read-only engine query |
| `/dashboards` | Saved metric/log/trace views |
| `/investigations` | Saved investigation state |
| `/alerts` | Alert rules and incidents |
| `/ecosystem` | Instrumentation/ecosystem inventory |

The first demo should present only these five pages: `/`, `/traces`, `/logs`,
`/metrics`, and `/issues`. Return to the route map for the remaining product
surfaces.

## 8. Stop and reset

Stop the playground from Terminal B:

```bash
docker compose -f deploy/docker-compose.yml --profile demo down
```

Stop Parallax with `Ctrl-C` in Terminal A. This preserves the local data under
`~/.parallax` for the next run.

To remove only the playground containers and volumes after a disposable demo:

```bash
docker compose -f deploy/docker-compose.yml --profile demo down -v
```

Do not run `down -v` in a demo that needs to preserve generated data.

## 9. Honest boundaries

- Preview builds are local-only. API bearer-token configuration exists, but the
  default demo runs auth off and current OTLP routes must stay on loopback.
- The playground compose stack contains unauthenticated local services and a
  demo database password. Do not expose it to a shared network.
- The `a28` browser journey needs real browser interaction; a shell command
  alone cannot prove browser UX.
- This audit environment had no connected in-app Browser session, so navigation
  intuitiveness and visual UX remain unverified.
- `a31` may report an unhandled failure as HTTP `500` or connection reset
  (`000`), depending on server/client timing.
- Metrics support is intentionally bounded; exponential histograms and
  summaries are not the public demo's proof point.
- `parallax issue context` provides bounded evidence and hypotheses, not an
  autonomous fix or a correctness guarantee.
- The preview Homebrew formula installs the `parallax` CLI. Local-stdio MCP is
  an optional repository-built surface, not a remote hosted MCP claim in this
  walkthrough.

## 10. Competitor boundary

Parallax is a basic local observability/context alternative, not a replacement
for these mature products:

| Tool | Stronger today | Parallax's narrower angle |
| --- | --- | --- |
| Sentry | Issue lifecycle, SDK ecosystem, ownership, Seer/fix workflows | Local issue context, correlated OTLP evidence, bounded Sentry-envelope ingest |
| Grafana | Dashboards, alerting, ecosystem, maturity, and scale | Simpler local traces/metrics/logs workflow with native issue correlation |
| Kibana/Elastic | Full-text log search, ES|QL, Discover, SIEM/security operations | Structured telemetry logs tied to traces, issues, and evidence bundles |
| OpenObserve, SigNoz, Coroot, and agent-native tools | Direct open/self-hosted overlap and existing investigation/agent surfaces | A local-first execution-context hypothesis that still needs comparative validation |

Do not claim replacement parity, cheaper-at-scale economics, unique MCP
ownership, or proven superior AI-agent outcomes. Canonical comparison sources:
`docs/research/market/competitors/README.md`,
`docs/research/market/landscape.md`, and
`docs/research/market/competitors/comparison-set.md`.

## 11. Source of truth

- Preview install and server ports: `docs/guide/quickstart.md`
- CLI behavior: `parallax --help` and `crates/parallax-cli/src/main.rs`
- Playground startup: `parallax-telemetry-playground/demo.sh`
- Playground browser journey: `parallax-telemetry-playground/scenarios/a28-rum-journey.sh`
- Scenario catalog: `parallax-telemetry-playground/scenarios/README.md`
- UI route implementation: `ui/src/routes/`

This document is a runnable presentation plan, not a production-readiness
approval. Security, scale, and compatibility claims need separate evidence.
