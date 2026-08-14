# Parallax Feature Inventory and Playground Verification Focus

Research date: 2026-08-13. Purpose: a compact reference of everything Parallax
ships today, plus the current highest-priority program — verify every feature
through the [telemetry playground](https://github.com/tailrocks/parallax-telemetry-playground)
against competitor backends until all features are production-ready. This file
is the input document for a deep implementation plan (e.g. an `/improve` pass);
it states *what exists* and *what the program must achieve*, not the step-by-step
plan itself.

Canonical deep contracts: [v1-implementation-spec.md](../architecture/v1-implementation-spec.md)
(API/UI/CLI contracts), [code-reality-ledger.md](../code-reality-ledger.md)
(claim vs shipped), [competitors/README.md](../market/competitors/README.md)
(comparison method), playground design
[telemetry-playground-sample-project.md](../validation/telemetry-playground-sample-project.md)
and fan-out lab [otlp-fanout-comparison-lab.md](../validation/otlp-fanout-comparison-lab.md).

## What Parallax is

Sentry-compatible, OpenTelemetry-native, self-hosted execution-context engine:
one Rust binary (`parallax serve`) exposing OTLP ingest + GraphQL + embedded UI,
storing telemetry in a supervised GreptimeDB child (native OTLP tables) and
mutable metadata in Turso. It serves bounded, redacted evidence bundles to
humans and coding agents; it is the context engine, not the fixer.

## Feature inventory (shipped on `main`)

### Ingest

- OTLP/gRPC `:4317` and OTLP/HTTP `:4318` — traces, logs, metrics (gzip,
  size limits, validation gate). No profiles signal.
- Sentry envelope endpoint `POST /api/<project_id>/envelope/` (disabled by
  default) — normalizes to Parallax error events, no second issue model.
- GitHub webhooks `POST /webhooks/github` (disabled by default) — HMAC-verified
  deploy + Actions CI events; bounded read-only REST backfills for both.
- Claude Code session import: `parallax import-claude` (consent-only NDJSON).
- Pipeline: raw-frame spool (PSPL1, crash forensics) → per-signal workers (no
  head-of-line blocking) → normalize → GreptimeDB write → error derivation →
  issue upsert → live broadcast. `/health` reports ingest degradation (503).

### Storage and lifecycle

- GreptimeDB native tables (`opentelemetry_traces`, `opentelemetry_logs`,
  per-metric tables) + derived extension tables (`error_events`,
  `invocation_metric_points`, exemplars). Bootstrap, repair `ALTER`s, TTL
  reconcile. Managed child (checksum-verified download, ports 24000–24003,
  supervised restart) or `external` mode.
- Turso metadata: issues/occurrences, invocations, dashboards, investigations,
  saved views, alert rules/states/incidents/destinations/deliveries, test
  reporting, CI/deploy deliveries, evidence pins, fixer outcomes, prune journal.
- Retention defaults: traces/logs 7d, metrics 14d, error events 30d;
  `parallax prune` plan-first, dry-run default, pin-aware, journaled.

### Errors, issues, evidence, agent surface

- Deterministic error derivation from exception spans + ERROR/FATAL logs;
  fingerprint grouping; issue lifecycle (open/resolve), trend, correlation.
- Evidence bundles (`bundle-v1` dossier in `bundle-v2` envelope, canonical
  hash, token-bounded, redacted, hypothesis-ranked): anchors = issue
  fingerprint | invocation | trace; `missing_evidence` gap detection; evidence
  pins survive telemetry TTL.
- Redaction engine `redaction-lite-v3` (20 secret detectors, default-deny).
- Story timeline (deterministic beats), agent-session projection, fixer
  outcome records (PR ≠ success; requires review + non-recurrence).
- MCP: local-stdio read-only `parallax-mcp`, exactly 2 tools
  (`parallax_issue_context`, `parallax_agent_session_show`), wire budgets,
  projection-equivalence `check`.

### CLI (`parallax`)

- `serve`; `doctor`; `prune`; `uninstall`; `sql` (read-only); `metrics`.
- `logs` / `traces` browse with UI-equivalent filters plus `--follow` / `--for`
  bounded live tail (the agent fix-verification signal); `trace inspect`.
- Invocations (CLI apps as first-class evidence): `invocation start`
  (wrapper/bare, OTel env injection incl. `TRACEPARENT`, `--otlp-forward`
  compare mode), `finish`, `inspect`, `bundle`, `agent`, `list`, `watch`.
- Issues: `issue list|context|resolve` (`issue context` = agent handoff).
- Remote contexts: `context add|list|use|show|remove` (`~/.parallax/contexts.toml`).
- Output contract `--format table|json|md`, exit codes 0/1/2.

### API

- One canonical GraphQL surface `POST :4000/graphql` — 76 queries, 14
  mutations, 0 subscriptions; depth/complexity limits; SDL checked into
  `ui/graphql/schema.graphql` and drift-gated.
- Query families: overview/signal series; services (catalog, map, RED,
  releases, runtime snapshot); traces (search, facets, duration stats, events,
  span links, critical path, structural compare, paging); logs (filters,
  around-anchor, histogram, facets, Drain patterns); issues + trend; metrics
  (catalog, typed query with kind-legal aggregations, labels, exemplars,
  histogram quantile); invocations + observed invocations; derived projections
  (sessions, screen visits, UI actions, background cycles, jobs,
  conversations); evidence (`bundle`, `story`, `agentSession`,
  `evidenceGaps`); field stats + `attributeCompare`; test cases; dashboards /
  investigations / saved views; alerting; raw `sql`.
- Live tail is SSE: `GET /v1/logs/stream`, `/v1/traces/stream` (per-row
  predicates, broadcast lag-drop).

### UI (embedded TanStack Start SPA)

- Surfaces: Overview (stat cards, trends, brush-to-zoom, top movers), Issues
  (list/detail, stacktrace with culprit frames, breadcrumbs, resolve/reopen,
  agent-handoff card), Traces (search + live tail, field explorer; detail with
  waterfall/compact/flamegraph, color-by-attribute, minimap, keyboard zoom,
  critical path, trace compare, clock-skew banner, evidence gaps, GraphQL ops,
  RPC streams, story tab), Logs (where-clause chips with facet autocomplete,
  severity floor, columns, patterns, saved views, live tail, histogram brush,
  context-around anchor), Metrics (catalog; per-metric workbench with legal
  aggregations, group-by, step; graduate to dashboard widget or alert rule),
  Services (heat catalog; detail with RED charts, exemplar dots, release
  strip, runtime snapshot), Ecosystem (React Flow + ELK service map: focus,
  hops, dim/hide, traffic threshold), CLI Apps (invocation list + 6-tab hub
  incl. sessions/screens/UI actions/conversations/jobs/cycles), Tests
  (variant explorer, flaky states, attempt chains), Alerts (rules/incidents/
  destinations tabs, template rule dialog), Dashboards (gallery + widget
  grid), Investigations (case files: pins with notes, window, markdown notes),
  SQL workbench (schema browser, snippets, history, examples).
- Cross-cutting: URL-driven shareable filters + time range everywhere, ⌘K
  palette with id-shape jump, theme system/light/dark, virtualized tables,
  route error/pending/not-found boundaries, onboarding empty states.

### Alerting

- Rule signals: error_rate, p95/p99 latency, throughput, log_count, metric;
  comparators incl. between; hysteresis, min samples, no-data behavior,
  severity, renotify, service scoping, group-by, attribute filters.
- Evaluator with CAS claim + pure state machine + audit rows + incidents;
  outbox delivery: webhook + Slack webhook (email deferred), backoff,
  dead-letter. Module marked preliminary.

### Test reporting

- JUnit/nextest adaptation, variant identity, attempt chains, fail-then-pass
  flaky detection (replay-safe scan), UI explorer + GraphQL.

### Operations, release, engineering gates

- Self-telemetry export (`PARALLAX_SELF_OTLP`, feedback-loop filtered).
- Homebrew preview channel; deterministic packaging with SBOM/signature/
  provenance verification; embedded-UI feature; Apple native + Linux
  zigbuild/vendored-OpenSSL builds (native TLS everywhere, never rustls).
- `cargo xtask` control plane: ci/lint/test partitions, policy families
  (architecture, structural, TS strictness, UI ownership/ratchets, GraphQL
  drift, runtime boundaries), facade + semconv generation, docs link check,
  Playwright lanes (smoke/contracts/full-stack + a11y/mobile/visual/cross).

### Known gaps and unfinished work (fold into the plan)

- No profiles signal end-to-end; no GraphQL subscriptions (SSE only); no
  SLO/error-budget/burn-rate; alert email deferred; Sentry/GitHub surfaces
  disabled by default; browser-RUM sessions are CLI/desktop-shaped
  projections, not a browser session product.
- Extension-table gRPC writes still blocked on upstream rustls-free
  `greptimedb-ingester`; retire legacy spool reader still needs one stable
  raw-frame release cycle. Residual index: [`plans/README.md`](../../../plans/README.md).
- Doc drift: guides still say `run` / `parallax.run.id`; spec + code use
  `invocation` / `cli.invocation.id`. Sentry multi-SDK compatibility ledger
  unproven; A1 (bundle beats raw context) still the open existential gate.

## Highest-priority program: playground-verified, competitor-compared features

Operator intent (2026-08-13): the next work program uses the playground as the
proving ground for **every** Parallax feature, compared side-by-side with
competitor backends fed identical telemetry through the fan-out hub, from the
perspective of a real user of each product. End state: every feature above is
verified working, compared, and production-ready — zero known bugs.

### Current playground state (repo `tailrocks/parallax-telemetry-playground`)

- 12 components: 8 Rust services (axum/tonic/sqlx/Juniper: checkout, pricing,
  inventory, recommendation, orders, notifications, storefront), 3 Java Spring
  Boot 4.1 (catalog GraphQL, payment gRPC, fulfillment Kafka), TanStack
  Start/React 19 web (browser OTLP + web-vitals + session.id + SSR
  traceparent), Rust `playground` CLI (runs/cron, JUnit→OTLP bridge).
  Dual emission: OTLP + Sentry SDK envelopes. Infra: postgres:17, Redpanda,
  flagd, k6.
- ~60 scripted scenarios: a-series feature proofs (waterfall, exemplars, span
  links, reverse-language hop, RUM error, GraphQL N+1, subscriptions/stream
  cancel, log spike, baggage, CLI run/cron, deploy regression, flag flip,
  PII-redaction canary, long/wide trace, trace compare, tokio saturation,
  Postgres pathologies, cache stampede, RUM journey, business events,
  teaching up-down/cardinality, handled vs panic) +
  b-series chaos (error/latency breach, retries, OOM, GC pressure, consumer
  lag, poison message, sampling gap, rage click, …) +
  c-series product surfaces (`c1`–`c11`, coverage-matrix spine).
- Fan-out lab lives in this repo at `bench/otlp-fanout/` — Rotel hub fanning
  identical OTLP to Parallax, OpenObserve, Maple, SigNoz, Sentry (per-signal
  routing; Sentry has no OTLP metrics).
- `VERIFICATION.md` runbook + machine-checked `playground test-verify`;
  `TOUR.md`; corner-case matrix. Comparison is manual by design.

### Workstream 1 — upgrade playground examples

Bring every example/service to current ecosystem latest (Boot, OTel Java
agent, OTel Rust, JS SDKs, Sentry SDKs); refresh `postgres:17` → 18; re-run
`renovate`-missed surfaces; re-verify the dual OTLP+Sentry emission contract
after upgrades; refresh the README verified matrix (stale since 2026-06-23,
including the unresolved Java-agent→Rotel→OpenObserve delivery snag).

### Workstream 2 — latest backend versions, pinned

Pin every backend/tool at latest stable and keep pins current (research date
2026-08-13; pins applied 2026-08-14):

| Tool | Deployed today | Latest stable |
| --- | --- | --- |
| Maple (maple.dev, Makisuo/maple) | v0.0.18 | v0.0.18 |
| OpenObserve | `v0.92.0` | v0.92.0 |
| SigNoz | vendored `v0.137.0` | v0.137.0 |
| Sentry self-hosted | vendored `26.7.2` | 26.7.2 |
| Rotel hub | `streamfold/rotel:v0.2.5` | v0.2.5 |
| OTel Collector (if added as alt hub) | — | v0.158.0 |
| postgres | 18 | 18 |
| Redpanda / flagd / k6 / telemetrygen | `v26.2.1` / `v0.16.1` / `2.2.0` / `v0.158.0` | same |

SigNoz `v0.137.0` vendor pin is current-stable, but the lab overlay cannot start:
upstream removed `deploy/docker/docker-compose.yaml` (Foundry-only). Plan 162
STOP — do not invent a Foundry rewrite in this workstream.

Candidate roster additions (decide in planning; deep-dives exist under
`market/competitors/`): Grafana LGTM v13.x, HyperDX v2.x, Uptrace v2.1.
Roster changes must keep the fan-out lab docs + `comparison-set.md` in sync.

### Workstream 3 — extend playground to cover missed Parallax features

Every inventory item above needs a scenario that exercises it; known holes in
the current catalog:

- Evidence bundles/pins/story/agent handoff: **c1** (`issue context` +
  GraphQL `bundle`) + **c2** (`invocation bundle`) + **c7** (Claude import).
- Alerting end-to-end: **c4** (rule → open incident after error seed).
- Dashboards, investigations, saved views, SQL: **c5**.
- GitHub deploy/CI ingest: **c6** (HMAC deploy fixture). Claude Code import:
  **c7**. Sentry envelope ingest: **c8**.
- Live tail: **c3** (SSE). doctor/prune dry-run: **c9**.
- Redaction canary on bundle egress: **c10**. agent-browser Overview: **c11**.

### Workstream 4a — agent-browser UI verification

Operator requirement (2026-08-13): every UI surface is additionally driven
by an agent-controlled browser (`agent-browser` CLI) — functional checks per
route (filters, live tail, waterfall interactions, mutations, ⌘K palette,
theme persistence) plus responsive checks (no horizontal overflow, nav
usable) across phone/tablet/desktop viewports in light and dark themes.
Deterministic core = playground scenario `c11-ui-agent-verify.sh`;
exploratory functional pass = agent-led checklist. Failures enter the same
`DISCREPANCY:` pipeline as Workstream 5.

### Workstream 4 — run it, user-lens comparison

Run the full stack + scenario sweep; for each feature record how each backend
(Parallax, Maple, OpenObserve, SigNoz, Sentry, any roster additions) serves a
practicing user on identical data: capability present/absent, fidelity,
workflow quality, gaps. Feed results into the competitor matrix
(`market/competitors/` axes: signals, ingestion, storage/cost, agent story,
architecture, security, economics) and the playground corner-case matrix.
Honesty rule stands: a comparison that always favors Parallax is a failure
state.

### Workstream 5 — fix and verify to production-ready

Every discrepancy found becomes a fix in Parallax (root cause first, per
repository rules), a scenario in the playground, or both; re-verify after fix.
Exit criteria: every feature in this inventory exercised by at least one
scripted scenario, verified in Parallax UI/CLI/API, compared against the
roster, `VERIFICATION.md` matrix fully green at current versions, zero known
bugs or issues open against shipped features (tracked defects either fixed or
promoted to `plans/` with a blocking reason).

### Planning notes for the deep plan

- Keep mandatory constraints: GreptimeDB+Turso only, native OTLP tables,
  native TLS never rustls, Bun only, zero-copy ingest hot path, single-branch
  `main` workflow, latest-stable version policy.
- Sequence suggestion: W2 (pins) → W1 (example upgrades) → W3 (coverage) →
  W4 (run/compare) → W5 (fix/verify) with W4/W5 iterating as a loop.
- Playground changes land in the playground repo; fan-out lab, comparison
  matrices, and this inventory live here — update both sides in the same
  program step.

## W5 discrepancy list

CLOSED: not-found hydration | 170/diagnostics-auto | parallax-ui | React minified #418 pageerror on GET /this-route-does-not-exist | expected: no pageerror | FIXED 2026-08-14: splat `/$` makes unknown URLs a real child of root (same hydrate path as `/sql`); `shellComponent` keeps document/theme/shell around MatchInner not-found; `shell.spec.ts` drops #418 allow
CLOSED: where-clause reserved-word value | 168-176/unit-gate | parallax-ui | `serialize→parse` property fail seed `632427516` value `"nOt"` → `expected CONTAINS after NOT` | expected: reserved-word keys/values round-trip | FIXED 2026-08-14: parser accepts keyword tokens as keys/values in those positions (original `.text`); `needsQuoting` quotes AND/CONTAINS/NOT so serialize cannot emit a keyword in value position

Coverage (2026-08-14T15:43Z restamp, playground PR #13 + this inventory):
`docs/coverage-matrix.md` restamped — no `MAPPED`/`UNTESTED` data cells.
`c1`–`c11` EXIT 0 this serve (c4 after `?fail=1` burst; c8 after JS
`X-Sentry-Auth`). Teaching: 18-span `5e14f8c670eb1e15`, N+1 `ae13ff562135f5e6`
two `reviewsSlow`, consumer Links 1/1 `f22fbe511f04f149` → `05a35c01c4869f7c`,
RUM stitch `19ace18bd8315e84` `ui.click`→checkout, a13 5×502 **2 versions**.

CLOSED: MCP check CLI≢HTTP bundle JSON | 164/c7 | parallax-mcp | CLI omitted `maxTokens` (API default 10000) while MCP check used 4000 | FIXED 2026-08-15: CLI `--max-tokens`; check passes 4000
CLOSED: clock-skew banner | 167/display | parallax-ui | same-service parent/child used 5min threshold | FIXED 2026-08-15: all parent/child pairs use 50ms `TRACE_SKEW_THRESHOLD`
CLOSED: exemplar click-through | 167/display | parallax-ui | metric workbench never queried `metricExemplars` | FIXED 2026-08-15: `/metrics/$name` loads exemplars and links `/traces/$traceId` (`data-has-trace-link`)
CLOSED: ecosystem default 24h ServiceMap | 167/display | parallax-api | `serviceMap` joined `observed_invocations(MAX_ROWS)` on the window | FIXED 2026-08-15: join uses the same `max_traces` cap
CLOSED: invocations unbounded observedInvocations | 167/display | parallax-ui | UI omitted `limit` | FIXED 2026-08-15: UI `limit: 50`; resolver already defaulted to 50
NOTE: service detail route is `/services/$name` (not `/$name`); `/$name` is splat not-found. Runtime lanes PASS on `/services/checkout` (tokio) and `/services/catalog` (jvm).
CLOSED: JS Sentry envelope | 164/c8 | playground | first POST is `type=session` (Parallax 415); second POST `type=event` is the exception. `c8 ok rust+java+js`. Sentry Group `plat=node Error: c8-js-sdk PaymentError`. FIXED 2026-08-14: disable session-first wait; emit type=event
CLOSED: empty Tests explorer | playground | test-verify rust --acceptance | FLAKY_PASS rows now visible (`tests-teach-flaky-1440-dark.png`)
CLOSED: span-links false-PASS | playground WS5 | agent-browser | prior shot was orders producer `96339146…` Links/events 0/7 `producer_without_consumer` | expected: visible link UI | 2026-08-14 consumer `ff46e8d94be06b78` inspector Links (1) → producer span `c1e6afa8585c40e0`
CLOSED: v1-only checkout strip | playground a13 | services UI | `/services/checkout` showed 1 version v1 | expected: v1+v2 after `RELEASE=v2` | 2026-08-14 5×502 + GraphQL releases v1+v2 + badge **2 versions**
CLOSED: RUM shot was playground HTML | playground a5 | agent-browser | `web-rum-break-1440-dark.png` is producer HTML | expected: Parallax stitch | 2026-08-14 `/traces/19edbf0ad9f030364b4657dfc7f4f463` web `ui.click` → checkout
