# Parallax public walkthrough: CLI to browser

**Review date:** 2026-08-25

**Required channel:** latest Homebrew preview; never use the stable formula

**Repositories:** [parallax](https://github.com/tailrocks/parallax) and
[parallax-telemetry-playground](https://github.com/tailrocks/parallax-telemetry-playground)

Follow this document from top to bottom. The core story is a live,
deterministic tour. The feature maps cover every primary browser route and
every top-level CLI family without pretending that every experimental surface
is ready for a live claim.

Last verified live:

- Parallax `0.1.0-preview.2497+dd93398`; moving release tag
  [`preview`](https://github.com/tailrocks/parallax/releases/tag/preview).
- Parallax source SHA recorded in the Homebrew formula:
  `dd9339891f379723e6fa52c4daad798c57517401`.
- All 13 primary UI routes rendered with `agent-browser`.
- Checkout, orders, RUM, traces, issues, metrics, exemplars, logs, CLI
  invocations, and read-only SQL were exercised against that preview.

## Navigation

- [One-time setup](#1-one-time-setup)
- [Preflight](#2-preflight-before-every-presentation)
- [Start Parallax and the playground](#3-start-parallax-and-the-playground)
- [Core story](#4-core-story)
- [Complete feature maps](#5-complete-feature-maps)
- [Stop, reset, and known limits](#6-stop-reset-and-known-limits)

## 1. One-time setup

Requirements: macOS or Linux, Homebrew, Docker with Compose, `git`, `curl`,
`nc`, `awk`, `python3`, and `agent-browser`. The first playground build is
large and can take several minutes.

The playground publishes unauthenticated demo services and credentials on host
ports. Run it only on a trusted machine and network; do not expose those ports
to a shared or public network.

Install the preview formula by its full name. This auto-taps the repository
and grants formula-scoped Homebrew trust; it avoids the broader tap trust
needed by the short alias.

```bash
brew install tailrocks/parallax/parallax-preview
"$(brew --prefix tailrocks/parallax/parallax-preview)/bin/parallax" --version
```

The output must contain `-preview.`. Never run `brew install parallax` or
`brew install tailrocks/parallax/parallax`; those select the stable channel.

Clone both repositories into one parent directory:

```bash
mkdir -p parallax-walkthrough
cd parallax-walkthrough
git clone https://github.com/tailrocks/parallax.git
git clone https://github.com/tailrocks/parallax-telemetry-playground.git
```

Set these paths in every new terminal. Replace the parent path once. Prepending
the formula prefix makes every terminal, including `demo.sh`, use the preview:

```bash
export PARALLAX_REPO="/absolute/path/to/parallax-walkthrough/parallax"
export PLAYGROUND_DIR="/absolute/path/to/parallax-walkthrough/parallax-telemetry-playground"
export PREVIEW_PREFIX="$(brew --prefix tailrocks/parallax/parallax-preview)"
export PATH="$PREVIEW_PREFIX/bin:$PATH"
hash -r
```

## 2. Preflight before every presentation

Refresh Homebrew first. This resolves the latest published preview instead of
pinning the version recorded above.

```bash
brew update
brew upgrade tailrocks/parallax/parallax-preview

PREVIEW_PREFIX="$(brew --prefix tailrocks/parallax/parallax-preview)"
export PATH="$PREVIEW_PREFIX/bin:$PATH"
hash -r

PARALLAX_VERSION="$("$PREVIEW_PREFIX/bin/parallax" --version)"
printf '%s\n' "$PARALLAX_VERSION"
case "$PARALLAX_VERSION" in
  *-preview.*) ;;
  *) printf 'STOP: Parallax preview is not installed\n' >&2; exit 1 ;;
esac
test "$(parallax --version)" = "$PARALLAX_VERSION" || {
  printf 'STOP: another Parallax binary shadows the preview\n' >&2
  exit 1
}
```

Update both source checkouts and record their revisions with the demo notes:

```bash
git -C "$PARALLAX_REPO" pull --ff-only
git -C "$PLAYGROUND_DIR" pull --ff-only
git -C "$PARALLAX_REPO" rev-parse --short HEAD
git -C "$PLAYGROUND_DIR" rev-parse --short HEAD
```

Verify the required tools:

```bash
docker info >/dev/null
docker compose version
agent-browser --version
curl --version | head -1
```

## 3. Start Parallax and the playground

### Terminal A: Parallax

Keep this process running:

```bash
parallax serve
```

Wait for the `Parallax ready` banner. It names these surfaces:

| Surface | Address |
| --- | --- |
| Web UI and GraphQL | <http://127.0.0.1:4000> |
| OTLP/gRPC ingest | `127.0.0.1:4317` |
| OTLP/HTTP ingest | `127.0.0.1:4318` |
| Managed GreptimeDB | `127.0.0.1:24000` |

### Terminal B: health and playground

```bash
curl -fsS http://127.0.0.1:4000/health
curl -fsS http://127.0.0.1:4000/version
parallax doctor
```

Expected: health prints `ok`; `/version` prints the server package version.
It is not the Homebrew channel/build identity. `parallax --version` is the
authoritative preview identity. In `doctor`, require `api (:4000): ok` and
`greptime child (:24000): ok`; a metadata-lock note from an optional diagnostic
does not fail those gates.

Start the playground. Exporting the current revision overrides any stale local
`deploy/.env` value and records the checkout HEAD in emitted telemetry.

```bash
cd "$PLAYGROUND_DIR"
export GIT_SHA="$(git rev-parse HEAD)"
./demo.sh
```

Wait for both browser-facing services, then stop background load so each
scenario has an unambiguous result:

```bash
curl --retry 30 --retry-connrefused --retry-delay 2 -fsS \
  http://localhost:5173/ >/dev/null
for port in 8088 8089 8090 8092; do
  curl --retry 30 --retry-connrefused --retry-delay 2 -fsS \
    "http://localhost:$port/healthz"
done
curl --retry 30 --retry-connrefused --retry-delay 2 -fsS \
  -H 'content-type: application/json' \
  --data '{"query":"{ __typename }"}' \
  http://localhost:8080/graphql >/dev/null
docker compose -f deploy/docker-compose.yml --profile demo stop loadgen
```

Create one visible, isolated browser session:

```bash
export AGENT_BROWSER_SESSION="$(agent-browser session id --scope worktree --prefix parallax-public)"
agent-browser --headed open http://127.0.0.1:4000/
agent-browser wait --load networkidle
agent-browser snapshot -i -c
```

After every navigation, submit, or dynamic update, run a fresh snapshot before
using another `@ref`.

## 4. Core story

### A. Overview: one local system

Browser: <http://127.0.0.1:4000/>.

Show spans, logs, metric points, error rate, recent issues, and slow traces.
Say: **one local process receives OTLP and joins the signals used in the rest
of the tour.**

Pass condition: the page is populated; it is not a blank shell.

### B. Distributed trace: checkout to three dependencies

Terminal B:

```bash
cd "$PLAYGROUND_DIR"
./scenarios/run.sh a1

TRACE_ID=""
for attempt in 1 2 3 4 5; do
  TRACE_ID="$(parallax traces --service checkout --grep http.server.request \
    --since 2m --limit 1 | \
    awk 'length($3) == 32 && $3 ~ /^[0-9a-f]+$/ { print $3; exit }')"
  test -n "$TRACE_ID" && break
  sleep 1
done
test -n "$TRACE_ID"
printf 'TRACE_ID=%s\n' "$TRACE_ID"
parallax trace inspect "$TRACE_ID"
```

Browser:

```bash
agent-browser open "http://127.0.0.1:4000/traces/$TRACE_ID"
agent-browser wait --load networkidle
agent-browser snapshot -i -c
```

Show the waterfall and service chips: `checkout`, `pricing`, `inventory`, and
`recommendation`. Show the PostgreSQL reserve span inside inventory.

Say: **one checkout request becomes one correlated, cross-service trace.**

Pass condition: the selected trace contains all four services. Never paste a
historical trace ID into the presentation.

### C. Browser RUM: stitched request versus intentional gap

Print the scenario contract:

```bash
cd "$PLAYGROUND_DIR"
./scenarios/run.sh a28
```

Drive the current browser with semantic locators:

```bash
agent-browser open http://localhost:5173/
agent-browser wait --load networkidle
agent-browser snapshot -i -c
agent-browser find role link click --name "checkout journey"
agent-browser wait --load networkidle
agent-browser snapshot -i -c
agent-browser find role button click --name "submit checkout"
agent-browser wait --fn "document.body.innerText.includes('Success:')"
agent-browser snapshot -i -c

agent-browser open http://localhost:5173/orders
agent-browser wait --load networkidle
agent-browser snapshot -i -c
agent-browser find role button click --name "submit order"
agent-browser wait --fn "document.body.innerText.includes('Success:')"
agent-browser snapshot -i -c

agent-browser open http://localhost:5173/
agent-browser wait --load networkidle
agent-browser find role button click --name "apply promo (unresponsive)"
agent-browser find role button click --name "apply promo (unresponsive)"
agent-browser find role button click --name "apply promo (unresponsive)"
agent-browser snapshot -i -c
agent-browser find role button click --name "break (RUM error)"
agent-browser wait --fn "document.body.innerText.includes('intentional RUM error')"
agent-browser snapshot -i -c

agent-browser open 'http://localhost:5173/checkout?nopropagate=1'
agent-browser wait --load networkidle
agent-browser snapshot -i -c
agent-browser find role button click --name "submit checkout"
agent-browser wait --fn "document.body.innerText.includes('Success:')"
agent-browser snapshot -i -c
agent-browser close
```

Use the direct `?nopropagate=1` URL. Do not use the page's **open intentional
propagation-break test** link: the current playground serializes its value as
`%221%22`, so it does not enable the exact `nopropagate=1` state.

Find the current browser submits:

```bash
parallax sql "SELECT trace_id, \
  \`span_attributes.telemetry.propagation.disabled\` AS propagation_disabled \
  FROM \`opentelemetry_traces\` \
  WHERE service_name = 'web' AND span_name = 'ui.submit' \
  ORDER BY timestamp DESC LIMIT 6"
```

Reopen Parallax and open the current trace IDs from that result:

```bash
agent-browser --headed open http://127.0.0.1:4000/traces
agent-browser wait --load networkidle
agent-browser snapshot -i -c
```

Show that a `propagation_disabled=false` submit shares a trace with checkout
and its dependencies. The `true` submit is the deliberate web-only gap. Also
show the visible RUM error and web-vital/browser-route spans.

Say: **Parallax shows both successful browser-to-backend correlation and the
instrumentation gap when propagation is disabled.**

### D. Grouped issues and bounded agent evidence

Terminal B:

```bash
cd "$PLAYGROUND_DIR"
./scenarios/run.sh a31
parallax issue list --status open

ISSUE_FP=""
for attempt in 1 2 3 4 5; do
  ISSUE_FP="$(parallax issue list --status open | \
    awk '$NF == "PaymentError" { print $1; exit }')"
  test -n "$ISSUE_FP" && break
  sleep 1
done
test -n "$ISSUE_FP"
printf 'ISSUE_FP=%s\n' "$ISSUE_FP"
parallax issue context "$ISSUE_FP"
parallax issue context "$ISSUE_FP" --format json --max-tokens 1200
```

Browser:

```bash
agent-browser open "http://127.0.0.1:4000/issues/$ISSUE_FP"
agent-browser wait --load networkidle
agent-browser snapshot -i -c
agent-browser open http://127.0.0.1:4000/issues
agent-browser wait --load networkidle
agent-browser snapshot -i -c
agent-browser find text "http.server.error" click
agent-browser wait --load networkidle
agent-browser snapshot -i -c
```

Show occurrences, latest trace, nearby logs and metrics, the copied CLI handoff,
and the handled `PaymentError` versus the unhandled `http.server.error`. The
unhandled request may report HTTP `500` or connection reset `000`.

Say: **the browser and CLI expose the same bounded evidence packet.** Treat
bundle output as sensitive even though it is bounded and redacted.

### E. Metric exemplar to exact trace

Terminal B:

```bash
cd "$PLAYGROUND_DIR"
./scenarios/run.sh a2
```

Browser:

```bash
agent-browser open http://127.0.0.1:4000/metrics/catalog_product_queries_total
agent-browser wait --load networkidle
agent-browser snapshot -i -c
agent-browser click 'a[href^="/traces/"]'
agent-browser wait --load networkidle
agent-browser snapshot -i -c
```

Show the rate chart and trace-ID links below it. Click a current exemplar link
and show that it opens `/traces/<id>`.

Say: **a metric point links directly to the trace that produced it.**

Pass condition: the page contains at least one trace link. The exact metric
name verified on the current preview is `catalog_product_queries_total`.

### F. Structured logs and trace correlation

Terminal B:

```bash
cd "$PLAYGROUND_DIR"
./scenarios/run.sh a9
```

Browser:

```bash
agent-browser open \
  'http://127.0.0.1:4000/logs?where=app_screen_name%20%3D%20workspace-select'
agent-browser wait --load networkidle
agent-browser snapshot -i -c
```

Show the WARN spike, open `slow render observed`, then show its structured
fields and trace link. Switch to **Live** only after the static result is clear.

Say: **structured fields narrow a spike; the selected log still links to its
trace.**

## 5. Complete feature maps

### Browser routes

The core story covers the strongest public narrative. Use this table for a
complete route walk.

| Route | Present | Producer or honest boundary |
| --- | --- | --- |
| `/` | Volume, error rate, latency, issues, slow traces | `a1` and the core story |
| `/issues` | Grouping, occurrences, trace/log/metric context, resolve state | `a31`; verified live |
| `/tests` | Run/session, cases, flaky/failure evidence | Empty state only in the base tour; the populated acceptance path requires `mise`, Rust, and Bun and is outside this Homebrew walkthrough |
| `/traces` | Query/live modes, facets, waterfalls, story, compare | `a1`; optional `a6`, `a3`, `a23` |
| `/ecosystem` | Languages, SDKs, instrumentation inventory, dependency graph | Current data; use **Last 24h** |
| `/logs` | Query/live modes, fields, patterns, trace links | `a9`; optional `c3` |
| `/metrics` | Series, aggregation, grouping, exemplars, alert/dashboard actions | `a2`; verified live |
| `/services` | Service inventory, dependencies, runtime context, releases | `a1`; optional `b5` or `a13` |
| `/invocations` | Wrapped CLI command, exit code, linked telemetry counts, evidence bundle | `parallax invocation start -- echo parallax-demo` proves lifecycle; its telemetry counts may be zero |
| `/alerts` | Rules, destinations, incident state | Present setup controls only; current incident opening is not live-verified |
| `/dashboards` | Saved telemetry view surface | `c5` creates an empty shell only; do not present it as a populated dashboard |
| `/investigations` | Saved investigation state | `c5` creates minimal state only; do not present a full workflow claim |
| `/sql` | Read-only query against native telemetry tables | Query below; verified live |

Useful optional producers:

```bash
cd "$PLAYGROUND_DIR"

# GraphQL batching, N+1, partial errors, operation-name cardinality.
./scenarios/run.sh a6

# Async producer-to-consumer span link.
./scenarios/run.sh a3

# Rust storefront GraphQL to Java payment gRPC.
./scenarios/run.sh a23

# Saved-state shells; not populated content.
./scenarios/run.sh c5

# One bounded CLI invocation.
parallax invocation start -- echo parallax-demo

# Read-only raw telemetry; paste the same query into /sql.
parallax sql 'SELECT * FROM `opentelemetry_logs` ORDER BY timestamp DESC LIMIT 5'
```

### CLI families

This list covers every top-level command in the verified preview.

| Family | Safe presentation command | Notes |
| --- | --- | --- |
| Server | `parallax serve` | Core startup |
| Trace list/detail | `parallax traces ...`; `parallax trace inspect <trace-id>` | Core story; add `--follow --for 30s` for a bounded live tail |
| Logs | `parallax logs --service checkout --since 15m --limit 50` | Add `--follow --for 30s` for a bounded live tail |
| Issues | `parallax issue list`; `parallax issue context <fingerprint>` | `issue resolve` changes local workflow state; run only intentionally |
| Invocations | `parallax invocation start -- echo parallax-demo` | Then use `list`, `inspect`, `bundle`, or bounded `watch --for 30s` |
| Invocation metrics | `parallax metrics --invocation <invocation-id>` | Requires an instrumented invocation; the playground acceptance path supplies one |
| SQL | `parallax sql 'SELECT ...'` | Read-only SELECT-shaped statements only |
| Diagnostics | `parallax doctor` | Core preflight |
| Contexts | `parallax context list` | `add`, `use`, `show`, and `remove` manage named API targets |
| Claude import | `parallax import-claude --help` | Consent-only; requires an operator-provided stream-json NDJSON file |
| Lifecycle prune | `parallax prune` | Dry-run by default; run after stopping the server if the metadata DB is locked |
| Uninstall | `parallax uninstall --help` | Do not run in a presentation; `--purge` deletes the Parallax data directory |

Bare invocation mode also exposes `invocation finish`; `invocation agent`
requires imported agent-session evidence and should not be shown on a plain
`echo` invocation.

## 6. Stop, reset, and known limits

Close the browser and stop the playground:

```bash
agent-browser close
cd "$PLAYGROUND_DIR"
docker compose -f deploy/docker-compose.yml --profile demo down
```

Stop `parallax serve` with `Ctrl-C` in Terminal A. This preserves Parallax data
under `~/.parallax`.

For a disposable playground-only reset, remove its Docker volumes too:

```bash
docker compose -f deploy/docker-compose.yml --profile demo down -v
```

`down -v` deletes playground database/broker volumes. Do not run it when the
generated data is needed for another presenter.

Known limits on the verified preview:

- The page-generated propagation-break link is encoded incorrectly; use the
  direct `?nopropagate=1` URL documented above.
- Alert rule and destination setup are visible, but current-preview incident
  opening was not reproduced. Do not run `c4` live or claim the incident
  lifecycle until sustained-breach verification passes.
- `c5` saves an empty dashboard and minimal investigation state. It proves
  persistence plumbing, not meaningful saved content.
- The direct playground stack does not prove a live Sentry UI or flamegraph.
  Sentry envelope emission has separate coverage.
- Scalar, histogram, and exemplar metrics are supported. Exponential
  histograms and summaries are dropped.
- No clock-skew banner was observed.
- The Homebrew preview ships `parallax`, not `parallax-mcp`; MCP requires a
  source build outside this walkthrough. Its current `check` output also does
  not match CLI/GraphQL bundle JSON exactly.
- This walkthrough proves local ingest, correlation, UI navigation, CLI
  inspection, and evidence handoff. It does not prove production scale, TLS,
  multi-user isolation, hosted MCP, or competitor-replacement parity.

## Sources of truth

- Preview identity: moving GitHub `preview` release and Homebrew
  `parallax-preview` formula.
- Install/update behavior: Homebrew documentation and tap README.
- CLI syntax: installed preview `--help` and
  `crates/parallax-cli/src/main.rs`.
- Ports and startup: `parallax serve` ready banner plus playground `demo.sh`
  and `deploy/docker-compose.yml`.
- Scenarios: `parallax-telemetry-playground/scenarios/README.md` and
  `scenarios/run.sh`.
- Browser routes: `parallax/ui/src/routes/`, verified with `agent-browser`.
