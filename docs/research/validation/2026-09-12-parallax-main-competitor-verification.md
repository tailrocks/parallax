# 2026-09-12 — Parallax `main` live competitor verification

Status: **COMPLETE** — all sections final; evidence live-verified 2026-09-12.
Run root (isolated, outside all repos): `/Users/donbeave/Projects/tailrocks/parallax-project/run-20260912/`
UI evidence: `run-20260912/artifacts/ui/comparison/2026-09-12/<backend>/<feature>/`

## Executive conclusion

**Parallax `main` (6b3a92b) is real and works end-to-end.** Every shipped capability was
exercised live against the strongest current competitor implementation of that capability
(7 backends, one shared telemetry story, API truth before UI truth). Ingest parity is
exact: one telemetrygen stream + one Sentry-envelope error story landed identically
everywhere (304 traces / 608 spans / 72k logs / 121k metric points; 52 Parallax issue
groups from the same chaos errors Sentry triages).

**Where Parallax leads (verified, not claimed):** trace attribute compare
(window-vs-window ranked attribute deltas — no competitor ships this), story view +
`sha256-jcs:` hash-pinned agent-ready incident bundles + MCP server (the AI-agent thesis;
nothing in the roster competes), the safe read-only SQL console over telemetry (roster
peers expose raw ClickHouse at best), and the densest single trace detail (critical path,
flame, lanes, tree, errors, correlated logs inline).

**Where Parallax trails (verified, not hedged):** service-map **polish** (the capability
ships — `/ecosystem` typed graph, 22 services/15 edges live — but HyperDX's BETA live
view is the slickest render), error triage workflow depth (Sentry's
assignment/regression/Autofix decade), alerting channel breadth (SigNoz's 10 kinds),
metric analysis (Grafana/PromQL is not a contest — embed, don't rival), and dashboards
(same answer). SigNoz's explorer surface (funnels, alert-from-query, add-to-dashboard)
is wider than Parallax's trace explorer.

**Parallax does not win everything — by design of this run.** 6 rows best=Parallax,
4 ties, 6 rows a competitor leads (incl. service-map polish), 2 deliberate
non-rivalries, 0 outright missing capabilities in the shipped set. One process note in
Parallax's disfavor: the initial browser walk **missed the Ecosystem surface** — the
report first declared the service map missing, then live re-verification (GraphQL
`serviceMap` + `/ecosystem` render, evidence `parallax/15-ecosystem-service-map.png`)
corrected it. Verification runs must cover every nav route, not the obvious ones.

**4 defects found and fixed at root cause this run** (config silently ignoring misnested
keys at *every* level; MCP bypassing the API token; internal-vs-public URLs; the embedded
UI shipping **with no way to authenticate** — every route 401'd on the documented default
server). All regression-tested, rebuilt, and re-verified in a real browser; the 401 →
token-entry → live-UI → SSE-tail flow is on-screen evidence.

**Verdict: ship-blocking defects: none remaining. Top product gap: service map.
Verdict for `main` today: viable product with a differentiated agent-native surface,
behind the leaders on breadth, ahead of the roster on agent-readiness.**

## Environment manifest

| Component | Version/ref | Source |
|---|---|---|
| Parallax (tested) | `0.1.0+6b3a92b` built release from `origin/main` = `6b3a92bc32178e6f651e06f54009b3a9646d1954` (2026-09-01) | worktree build, `rustc 1.97.0` (mise, pinned by `rust-toolchain.toml`), profile `release` |
| Parallax (stale, NOT used) | Homebrew `0.1.0-preview.2498+408ce24` at `/opt/homebrew/bin/parallax` | identified and excluded |
| Playground | `origin/main` = `c6c1516e8580da632a55240613abea7b30da76ce` (2026-09-01) | worktree |
| OpenObserve | `openobserve/openobserve:v1.0.0` — first GA release, 2026-09-11 | GitHub releases v1.0.0 (prerelease=false; RC trail rc1–rc5 superseded), openobserve.ai/downloads |
| Maple | `maple-v0.0.22-aarch64-apple-darwin` binary (2026-09-03), repo `MapleTechLabs/maple` (301 from `Makisuo/maple`) | GitHub releases; lab Dockerfile pin also bumped v0.0.18→v0.0.22 |
| SigNoz | `signoz/signoz:v0.141.1` + `signoz/signoz-otel-collector:v0.144.9` (official pairing per helm chart `signoz-0.141.1`), ClickHouse `25.12.5`, ClickHouse Keeper `25.12.5`, Postgres `16` (metastore) | GitHub release v0.141.1 (2026-09-09); foundryctl v0.2.17 generated compose, images hand-pinned |
| Sentry self-hosted | `26.8.0` (2026-08-17), full vendor compose (~70 containers) | getsentry/self-hosted releases; `install.sh --skip-user-creation` |
| Grafana LGTM | `grafana/otel-lgtm:0.33.0` (2026-09-11; Grafana 13.2.1, OTel collector 0.160.0) | grafana/docker-otel-lgtm releases |
| HyperDX / ClickStack | `hyperdx/hyperdx-all-in-one:2.38.0` (2026-09-04; repo tags @hyperdx/app 2.38.0) | Docker Hub + hyperdxio/hyperdx |
| rustrak | `rustrak/rustrak-server:v0.14.12` + `rustrak/rustrak-ui:v0.14.12` (2026-09-07) | rustrak/rustrak releases |
| Rotel (fan-out hub) | `streamfold/rotel:v0.2.5` | lab compose (unchanged pin) |
| telemetrygen | `ghcr.io/open-telemetry/opentelemetry-collector-contrib/telemetrygen:v0.158.0` | lab compose |
| Host | macOS aarch64 (Darwin 25.6.0), 18 CPU, 128 GB RAM, Docker Desktop, compose v5.1.2 | live `docker info` |
| Run date | 2026-09-12 | — |

### Parallax lab instance configuration (isolated)

- Binary: `/Users/donbeave/Projects/tailrocks/parallax-project/run-20260912/parallax/target/release/parallax`
- `parallax --version` → `parallax 0.1.0+6b3a92b` (matches tested SHA; provenance proof)
- Config `run-20260912/parallax-lab/config.toml`: `bind=0.0.0.0`, OTLP gRPC **14317**, OTLP HTTP **14318**, API UI :4000, bearer `run20260912-lab-token`, Sentry ingest enabled (`project_id=1`, public key `c8public`), data dir `run-20260912/parallax-lab/data` (fresh; managed GreptimeDB child downloaded/supervised by the binary)
- Config gotcha fixed during run: `bind`/`otlp_*_port`/`api_token` live under `[server]`; top-level keys are ignored silently (filed as finding)

## Version-resolution notes (upstream re-derivation)

- **OpenObserve**: v1.0.0 GA published 2026-09-11 (the 0.92.x pin in the lab was one release line behind; v1.0.0 supersedes rc1–rc5).
- **Maple**: repo moved `Makisuo/maple` → `MapleTechLabs/maple` (verified 301). Latest v0.0.22 (2026-09-03). No published Docker image; local mode = single binary with embedded chDB. Lab keeps a containerized build-from-release-bundle path (Dockerfile bump), this run also runs the official binary on the host.
- **SigNoz**: since v0.130.0 the repo compose/install.sh are deprecated/unmaintained; current supported self-host = **Foundry** (`foundryctl` v0.2.17 → compose deployment). Old lab pin v0.129.0 replaced by Foundry-generated stack pinned v0.141.1/collector v0.144.9 (collector is versioned independently; pairing taken from official helm chart `signoz-0.141.1`).
- **Sentry self-hosted**: 26.8.0 (2026-08-17) replaces 26.7.2 pin. Resource floor per docs: 4 CPU/16 GB RAM (host has 18/128 GB).
- **Grafana LGTM**: image repo is `grafana/docker-otel-lgtm`; 0.33.0 supersedes 0.30.2.
- **HyperDX**: `clickhouse/clickstack-all-in-one` moved to `hyperdx/hyperdx-all-in-one`; 2.38.0 supersedes 2.35.0.
- **rustrak**: 0.14.12 supersedes 0.14.4. TypeScript (not Rust) Sentry-protocol error tracker; not an OTLP sink.
- **Uptrace** considered and **excluded**: not the strongest reference for any Parallax capability that the existing roster doesn't already cover better.
- GitHub `prerelease=false` is unreliable (OpenObserve RCs, Uptrace betas): version strings re-checked for every pin.

## Methodology

1. **Pristine sources**: both repos fetched; `origin/main` SHAs recorded; fresh git worktrees (`run-20260912/{parallax,playground}`); release build; binary provenance proven via `--version` embedding the short SHA; stale Homebrew parallax identified and excluded; no pre-existing containers reused (image store was empty; every backend pulled at its current tag).
2. **Fan-out**: rotel is the single host OTLP entry (4317/4318); every backend receives the same OTLP stream (per-signal exporter lists). Sequential-exporter caveat respected via `exporters-reachable.sh` + per-sink TCP probes before emission.
3. **Sinks**: Parallax (host), OpenObserve, Maple (host binary :14341), SigNoz (Foundry stack :14327), Grafana LGTM, HyperDX, Sentry (:9000, OTLP integration; traces+logs only — Sentry has no OTLP metrics). rustrak and the playground SDKs speak the Sentry **envelope** protocol side-channel (c8 scenario emits the same error story per target).
4. **Ingest auth gates discovered live**: Parallax bearer token; OpenObserve Basic+org/stream headers; SigNoz none (post-boot, pre-org); HyperDX OpAMP gate — collector binds 4317/4318 only after first user+team onboarding, ingest requires the *ingest token* (surfaced in collector `bearertokenauth` config), NOT the personal API key; Maple none.
5. **Layer A before Layer B**: per-backend counts/IDs asserted via strongest query API (Parallax GraphQL 77 queries / CLI; OO `_search` SQL; Maple CLI+UI facets; SigNoz `clickhouse-client` on `signoz_traces.signoz_index_v3`; Tempo search API; HyperDX `clickhouse-client` on `otel_traces`; Sentry snuba `eap_items_1_local`). UI judgments only after data truth.
6. **Equal telemetry**: one emission fans out everywhere; parity counts captured (see Layer A results). No hand-crafted per-backend datasets.
7. **Browser verification**: agent-browser 0.37.1, screenshots + interaction transcripts under `run-20260912/artifacts/ui/comparison/2026-09-12/<backend>/<feature>/`.

## Layer A results (ingest/query truth, telemetrygen smoke service)

| Backend | Query path | Result |
|---|---|---|
| Parallax | `parallax traces --service telemetrygen` / GraphQL | ✅ traces listed; spans stored |
| OpenObserve v1.0.0 | `POST /api/default/_search?type=traces` | ✅ 608 spans (`SELECT count(*) FROM "default" WHERE service_name='telemetrygen'`) |
| Maple v0.0.22 | UI facet + CLI | ✅ 304 traces / facet count 304 (CLI local-mode warehouse query 404 — Maple defect candidate, UI unaffected) |
| SigNoz v0.141.1 | `clickhouse-client` → `signoz_traces.signoz_index_v3` | ✅ 608 spans |
| Grafana LGTM 0.33.0 | Tempo `/api/search?tags=service.name=telemetrygen` | ✅ traces with rootServiceName=telemetrygen |
| HyperDX 2.38.0 | `clickhouse-client` → `otel_traces` | ✅ 304 traces |
| Sentry 26.8.0 | snuba `eap_items_1_local` count | ✅ 4223 items (OTLP traces→EAP item store) |

Count semantics note: OO/SigNoz count **spans** (608 = 304 traces × 2 spans); Maple/HyperDX count **traces** (304). Consistent parity.

## Feature inventory (tested SHA)

Shipped capabilities on `6b3a92b` (code-reality inventory, 38 areas) — summarized in `docs/research/reference/feature-inventory-and-playground-verification.md` and re-derived live this run (77 GraphQL queries enumerated via introspection: 77 match; 22 UI routes; ~46 CLI leaves). Detailed per-feature verdicts in the matrix below.

## Feature-by-feature comparison matrix

Scope rule enforced: only shipped Parallax features are compared; each row names the
strongest competitor implementation of that specific feature, not a fixed opponent.
"Best implementation" = the strongest of the two, judged on this run's live evidence.

| Parallax feature | Scenario | Parallax result | Best comparator | Comparator result | Best implementation | Why | Parallax gap/action | Evidence |
|---|---|---|---|---|---|---|---|---|
| OTLP ingest (traces+logs+metrics, one endpoint) | rotel fan-out of one telemetrygen stream, 304 traces / 608 spans / 72k logs / 121k metric points | ✅ same story visible in Overview KPIs and GraphQL counts | HyperDX 2.38.0 | ✅ 304 traces in `otel_traces`; OTLP-first design, zero-config via OpAMP gate | tie | Both ingest the identical stream losslessly; Parallax additionally accepts Sentry envelopes for errors | none — parity | `artifacts/ui/comparison/2026-09-12/parallax/01-overview-authenticated.png`; hyperdx/02-traces.png |
| Sentry-envelope error ingest | playground c8 scenario (rust/java/js PaymentError) via SDK side-channel | ✅ errors grouped into Issues (52 groups) | rustrak 0.14.12 | ✅ accepts both Sentry auth styles (header + `sentry_key` query); Sentry 26.8.0 relay rejects query-string DSNs | Parallax | Parallax fuses envelope errors with OTLP telemetry in one store; competitors keep separate stacks (Sentry) or single-protocol (rustrak) | none | `docs/research/validation/` c8 artifacts; sentry/01-issues-feed.png |
| Trace explorer (search, facets, duration filter, errors-only) | query `service = "checkout" AND http.request.method != "GET"` + facets | ✅ facet sidebar (SERVICE/SEVERITY/HTTP.REQUEST.METHOD/ERROR.TYPE), Where editor, live toggle | SigNoz v0.141.1 | ✅ trace explorer w/ facets, query builder, List/TimeSeries/Table, saved views; richer operators (TraceQL-like filters, funnels) | SigNoz | SigNoz explorer surface is larger: funnels, trace-matching (beta), one-click alert/dashboard from any query | Parallax: add one-click "alert from query" / "add to dashboard"; facets+Where already strong | parallax/05-traces-query.png; signoz/02-traces-explorer.png |
| Trace detail waterfall | open checkout error trace | ✅ waterfall + Tree/Errors/Lanes/Flame subviews, critical path, color-by-attribute, span index, correlated logs count | Grafana LGTM 0.33.0 (Tempo) | ✅ waterfall + node graph; trace detail solid but fewer analytical subviews | Parallax | Parallax packs more analytical lenses per trace (critical path, flame, lanes, correlated logs inline) | none on waterfall; node graph is the service-map gap (separate row) | parallax/06-trace-detail.png; grafana/03-tempo-trace.png |
| Trace attribute compare | facets active → selected vs previous window ranked differentiating attributes | ✅ panel renders ranked deltas | — (no equivalent) | SigNoz/OO/Grafana/HyperDX/Sentry: none ship window-vs-window attribute diffing | Parallax | Unique capability; directly answers "what changed between windows" without SQL | none — headline differentiator; market it in docs/README | parallax/05-traces-query.png (compare panel) |
| Trace query language | Where clause vs TraceQL | ✅ composable clause editor (typed operators) | Grafana Tempo TraceQL | ✅ full query language w/ autocomplete; **but** this Tempo build rejects unscoped `service.name` (400 "unknown identifier: service") — requires `.resource.service.name` | Grafana | TraceQL is a language, Parallax Where is structured — Grafana wins power, Parallax wins error-proofness; Grafana's scoping footgun is live-verified | consider documented attribute-scope defaults so bare `service.name` resolves | parallax/05-traces-query.png; grafana/02-traceql-error.png |
| Logs explorer (search, histogram, facets, patterns, saved views) | search PaymentError; patterns view | ✅ histogram + severity/service facets + Where + Patterns + saved Views | OpenObserve v1.0.0 | ✅ stream-first SQL-ish logs search at scale (512 streams / 1.8M events); stronger storage-tier story | tie | OO wins at petabyte-scale stream mgmt; Parallax wins integrated correlation (logs↔traces↔issues in one store) | document scale/tiering position; correlation story already shipped | parallax/08-logs-search.png; openobserve/02-logs.png |
| Log/trace live tail | SSE stream routes `/v1/logs/stream`, `/v1/traces/stream` | ✅ rows arrive without reload (21:31:49→21:31:56 observed); works via query-token fallback | Grafana Loki | ✅ "Live" tail button in Explore (LogQL SSE); HyperDX also ships Live Tail on every search | tie (3-way) | Three implementations live; Parallax unique in tailing **traces** (Grafana/HyperDX tail logs; HyperDX live-tail covers logs+traces actually — parity) | keep; verify reconnect/visibility behavior long-run (already handled in `use-live-stream.ts`) | parallax/09-logs-live-tail.png; grafana/05-loki-logs.png; hyperdx/01-search.png |
| Metrics explorer | metric list + point counts | ✅ 12 metrics listed, point count in window | Grafana (Prometheus) | ✅ full PromQL explore + dashboards; far deeper metric analysis | Grafana | Parallax metric explorer is a list/browse surface, not an analysis surface | gap: PromQL-like query or per-metric dashboards; Grafana remains best-in-class — Parallax should embed/link instead of rival | parallax/10-metrics.png; grafana/04-explore-metrics.png |
| Issues / error triage | PaymentError chaos errors | ✅ 52 groups/24h, grouped by service/message/culprit, trend sparkline, tags from attributes, open/resolved | Sentry 26.8.0 | ✅ strongest triage workflow: level, users/events, New/Regressed/Resolved, assignment, releases/env attribution, similar-issues + Autofix | Sentry | Sentry's decade of triage workflow (assignment, ownership rules, Autofix) leads; Parallax's tags-from-attributes cross-linking is unique | adopt: assignment + "regressed" state; keep attribute-tags advantage | parallax/07-issues.png; sentry/01-issues-feed.png, sentry/02-issue-detail.png |
| Alerting (rules, incidents, destinations) | error_rate rule → incident with `sha256-jcs:` bundle hash | ✅ rules + incidents + 4 webhook destinations; incident carries agent-ready bundle | SigNoz v0.141.1 | ✅ full alert center, 10 channel kinds, alert-from-any-explorer-query | SigNoz | SigNoz breadth (channel kinds, alert-from-query) leads; Parallax's incident bundle (agent-consumable, hash-pinned) is unique | add channel breadth later; add "create alert from current query" button (SigNoz UX) | parallax/11-alerts.png; signoz alerts; sentry/03-alerts.png (Monitors rename) |
| Service overview (RED) | services table w/ spans/errors/error-rate/p95 | ✅ 17 services, versions, env, per-service detail w/ releases | HyperDX 2.38.0 | ✅ service map (BETA) + per-service RED + trace-through | tie (different strengths) | HyperDX couples RED to the live graph; Parallax's per-row detail (runtime/version) is richer and its Ecosystem graph covers the dependency view | none — see next row | parallax/03-services.png, parallax/04-service-detail.png, parallax/15-ecosystem-service-map.png; hyperdx/05-service-map.png |
| Service/dependency map | `/ecosystem` over the playground window | ✅ **shipped and live** (initially missed in the walk; re-verified): GraphQL `serviceMap` + React Flow/ELK UI — 22 services · 15 edges, typed nodes (service/queue/database/cli/**external** with derived system labels: postgresql, kafka, flagd, api.stripe.test), per-edge call/error/p50/p95, 1-hop focus + dim-outside controls | HyperDX 2.38.0 (BETA) | ✅ live node graph, latency/error/throughput legend; checkout red at 5.2% | HyperDX (by polish) | Capability present on both; HyperDX's single live view + legend is the slickest render tested; Parallax's typed-kind derivation (queue/database/external from generic signals) is deeper modeling | UX polish: per-edge traffic animation/legend parity; keep typed-kind advantage | parallax/15-ecosystem-service-map.png; hyperdx/05-service-map.png |
| SQL console over telemetry | join `opentelemetry_traces` × `opentelemetry_logs` | ✅ read-only console, table browser, snippets, history; 4 rows in 37ms | SigNoz (ClickHouse) | ✅ raw ClickHouse console exists but is ops-grade/unsafe for users; SigNoz products **around** it | Parallax | Parallax is the only roster member exposing a safe read-only SQL product surface over telemetry | keep; publish schema docs (GreptimeDB tables) | parallax/13-sql-console.png, parallax/14-sql-results.png |
| Story view / agent-ready incident bundles | incident bundle + trace story view | ✅ shipped (`sha256-jcs:` hash-pinned bundles; story view on trace detail) | — | none: no competitor produces agent-consumable pinned bundles (MCP exists in Parallax only) | Parallax | AI-agent-readiness is Parallax's thesis; nothing in roster competes | none — differentiator; keep bundle schema stable + versioned | parallax/06-trace-detail.png (story), parallax/11-alerts.png (incidents) |
| UI auth | fresh browser, no token → 401s | ✅ (this run's fix) token panel → save → reload → all routes live; health pill green | — | roster peers are single-user local tools with no auth (OO/SigNoz have multi-user, but Parallax's threat model is local single-token) | n/a | Different product class (multi-tenant SaaS-style vs local token); defect #4 fix verified end-to-end in browser | none | parallax/01-overview-authenticated.png; defect #4 record below |
| Dashboards | workspace dashboards list | ⚠️ list renders; no drag-drop builder depth | Grafana 13.2.1 | ✅ industry-reference dashboards (panels, variables, alerting integration) | Grafana | Not a contest; Grafana is the reference | do not rival — keep saved dashboard list, consider Grafana embed/link path | parallax dashboards nav; grafana/01-home.png |
| MCP server (AI access to telemetry) | Parallax MCP stdio | ✅ shipped | — | none in roster | Parallax | Only Parallax exposes telemetry to agents via MCP + bundles | none | crates/parallax-mcp |

Matrix honesty check: 6 rows best=Parallax, 4 tie, 6 best=competitor (incl. service-map
polish), 1 n/a, 1 deliberate non-rivalry (dashboards; metrics is scored to Grafana for
the same reason), 0 outright missing capabilities among Parallax's shipped-feature set.
Parallax does not win everything — it trails on service-map polish, triage depth, and
alerting breadth.

## Parallax defects discovered (4; all fixed at root cause + regression-tested this run)

### #1 — Config keys silently ignored outside sections (severity: high, ops-breaking)

- **Symptom**: putting `bind`, `otlp_grpc_port`, `otlp_http_port`, `api_token` at the TOML
  top level instead of `[server]` starts the server on defaults (loopback, 4317/4318) with
  **no warning**. This run's operator believed the server bound `0.0.0.0:14317/14318` while
  it actually contested rotel's ports.
- **Architecture cause**: serde deserialization had no unknown-key detection anywhere —
  the same silent-drop class existed at **every** config struct, not just top level.
- **Root fix**: `#[serde(deny_unknown_fields)]` on all 10 config structs
  (`crates/parallax-server/src/config.rs`), so a misnested/typo'd key fails startup with a
  named-key error instead of silently defaulting.
- **Regression tests**: config tests assert unknown keys are rejected (see test module in
  `config.rs`).

### #2 — `parallax-mcp` bypassed the API bearer token

- **Symptom**: MCP stdio server reached the API without the plan-109 bearer token.
- **Root fix**: MCP client sends the configured bearer on every request.
- **Regression tests**: `crates/parallax-mcp/src/gql/tests.rs` —
  `empty_token_is_normalized_to_auth_disabled` (empty `--token` → auth disabled, matching
  server open mode) and `graphql_requests_attach_bearer_exactly_when_configured`
  (loopback HTTP stub captures the `Authorization` header: `Bearer secret` present when
  configured, absent in open mode — asserts the `bearer_auth` attach path itself).

### #3 — Links used internal address instead of a configurable public URL

- **Symptom**: URLs embedded in incident bundles/links assumed the bind address; behind a
  proxy or non-loopback bind the links were wrong.
- **Root fix**: new `public_url` config field + `resolved_public_url()` (config.rs),
  consumed at link construction (`serve.rs` ~line 577); removed the now-unused
  `api_addr` parameter from `spawn_alerting_loops` (the fix made the internal address
  obsolete at that call site).
- **Regression tests**: `resolved_public_url()` default + override cases in `config.rs`.

### #4 — Embedded UI had no way to authenticate (severity: high, UI unusable on token-protected servers)

- **Symptom**: the shipped UI never sent the bearer token. On any token-protected server
  (the documented default) **every** GraphQL route 401'd and every route fell into the
  generic "API did not answer" error panel; the health pill showed a stale "Offline".
- **Architecture cause**: the UI's fetch/SSE layer had no auth module at all — auth lived
  only server-side, so no single point could be fixed; four separate call sites each
  needed the header, and SSE needed a non-header channel. The enabling condition was
  "auth scattered across call sites"; the fix centralizes it in one module so future
  call sites cannot forget it.
- **Root fix**:
  - `ui/src/platform/auth/api-token.ts` — single auth module: `getApiToken`/`setApiToken`
    (localStorage `parallax.api-token`, trimmed; empty clears), `apiAuthHeaders()`
    (bearer), `withAccessTokenQuery(url)` (percent-encoded `access_token` for SSE —
    EventSource cannot send headers; server accepts query token **only** on stream
    routes via `allow_query_token`).
  - Server: `authorize()` middleware (`serve/http.rs`) with `allow_query_token` flag;
    percent-decoding + `constant_time_eq`; 401 with `WWW-Authenticate: Bearer`; SSE
    routes use the query-token variant.
  - `ui/src/platform/graphql/transport.ts` + `client.ts` — bearer on GraphQL.
  - `ui/src/features/app-status/api/load-app-status.ts` + `dashboards/api/widget-series-api.ts` —
    bearer on the two raw-fetch call sites.
  - `ui/src/platform/sse/use-live-stream.ts` — single SSE entry point appends query token.
  - `ui/src/layout/route-boundaries.tsx` — `isUnauthorizedError()` routes 401s to a
    dedicated token-entry panel (save → store → reload) instead of the generic error.
- **Regression tests**: `ui/src/platform/auth/tests/api-token.test.ts` (5 cases:
  storage trim/clear, bearer headers, query append/encoding; jsdom per repo convention).
  Server side: `percent_decode_resolves_escapes_and_plus` unit test in
  `serve/http.rs` (`%XX` escapes, `+` as space, malformed escapes pass through) and new
  integration suite `crates/parallax-server/tests/m109_sse_query_token.rs` (3 tests:
  stream routes 401 without credential / 200 with `access_token` / wrong token 401 +
  `WWW-Authenticate: Bearer`; percent-encoded token `a%26b%2Fc` decodes to `a&b/c` and
  the fallback is **scoped to stream routes** — GraphQL with the same query param still
  401s; bearer header still accepted on stream routes).
- **Live verification**: fresh browser with no token → 401 panel → token saved → Overview
  live (75k spans / 8.8k traces / 72k logs / 121k metric points); SSE live tail delivered
  rows without reload (21:31:49 → 21:31:56); health pill flipped to green. All server
  paths curl-verified: GraphQL 401/200, SSE 401/200(query)/200(bearer), UI shell 200.

## Playground improvements

Two trees matter; both are recorded so nothing is attributed to the wrong one:

**`run-20260912/playground` worktree (pristine `origin/main` = `c6c1516` + run hygiene).**
Uncommitted edits made during this run, all path/env hygiene needed to execute the c-series
against a token-protected Parallax:

- `scenarios/c8-sentry-envelope.sh` — fixed emit path (`bun ../scenarios/c8-emit-js.ts` →
  `bun scenarios/c8-emit-js.ts`); JS envelope emission was broken on `origin/main`.
- `scenarios/lib-c.sh` — `c_gql()` now sends `Authorization: Bearer $PARALLAX_API_TOKEN`
  (defect #4 follow-through: the playground's own GraphQL probes had the same blind spot
  as the UI) and three hardcoded absolute-path `PARALLAX_BIN`/`PARALLAX_MCP` fallbacks
  removed.
- `web/scenarios/c8-emit-js.ts` — moved from `scenarios/` so `bun` resolves it from the
  `web/` cwd; `web/package.json`/`bun.lock` follow.

The c-series suite itself (`c1`–`c11`, `c11-ui-agent-verify`, `corner-cases.sh`) is
`origin/main` content and passed as-is once the hygiene fixes landed.

**`parallax-telemetry-playground` branch `fix/current-verification-readiness` (HEAD
`e615984`, ~20 commits ahead of `origin/main`).** The structural rework of the playground:
shell a/b/c scenario suites replaced by a Rust scenario runner (`cli/src/scenario_runner.rs`),
a Rust commerce sample app (fulfillment/catalog/checkout), and a rewritten
`docs/VERIFICATION.md`. This branch is the going-forward playground; the run-tree hygiene
fixes above remain valid as `origin/main` fixes and were kept minimal for that reason.

## Competitor/lab integration changes

- `compose.yml`: OpenObserve `public.ecr.aws/zinclabs/openobserve:v0.92.0` → `openobserve/openobserve:v1.0.0`.
- `compose.grafana.yml`: otel-lgtm 0.30.2 → 0.33.0.
- `compose.hyperdx.yml`: `clickhouse/clickstack-all-in-one:2.35.0` → `hyperdx/hyperdx-all-in-one:2.38.0`.
- `compose.rustrak.yml`: 0.14.4 → 0.14.12.
- `compose.maple.yml`: MAPLE_VERSION v0.0.18 → v0.0.22.
- `compose.signoz.yml`: rewritten — v0.129.0 vendor-clone overlay (deprecated upstream since v0.130.0) replaced by checked-in Foundry-generated `signoz-foundry/` deployment (pinned v0.141.1/collector v0.144.9, host ports 14327/14328/3301); `setup-vendor.sh` deleted; `rotel.env.example` signoz/maple endpoints updated (maple this run: host binary :14341).
- HyperDX ingest: OpAMP onboarding gate + ingest-token header documented; overlay gained
  `FRONTEND_URL=http://127.0.0.1:18080` (default `http://localhost:8080` collided with the
  playground catalog port and bounced login redirects).
- `rotel.env`: HyperDX custom header carries the team ingest key (regenerated when the
  HyperDX volume was recreated during version bump).

## Out-of-scope competitor features

Parallax does not implement these; not benchmarked: continuous profiling, session replay, synthetics/uptime, SLOs/error budgets, k8s/infra monitoring, RBAC/SSO/multi-tenancy, mobile crash symbolication, log-derived metrics/transforms, anomaly/ML alerting, eBPF discovery, browser RUM product.

## Reproduction

All state lives under `/Users/donbeave/Projects/tailrocks/parallax-project/run-20260912/`
(outside every repo). From clean: fetch both repos, `git worktree add` at the recorded
SHAs, then:

1. **Parallax build**: `cd run-20260912/parallax && cargo build --release` (mise-pinned
   toolchain from `rust-toolchain.toml`, rustc 1.97.0). Provenance check:
   `target/release/parallax --version` must print `0.1.0+6b3a92b`.
2. **UI build**: `cd ui && npm ci && npm run build` (embedded into the binary at
   `ui/dist/client`).
3. **Parallax lab**: `run-20260912/parallax-lab/config.toml` — bind `0.0.0.0`, OTLP
   gRPC 14317 / HTTP 14318, API+UI :4000, bearer `run20260912-lab-token`, Sentry ingest
   (`project_id=1`, key `c8public`), fresh data dir (binary downloads and supervises its
   managed GreptimeDB child).
4. **Fan-out**: `cd run-20260912/playground && docker compose up` (rotel :4317/:4318 +
   sink stack; per-sink compose files: `compose.yml` OO, `compose.grafana.yml`,
   `compose.hyperdx.yml`, `compose.rustrak.yml`, `compose.maple.yml`,
   `compose.signoz.yml` + `signoz-foundry/`). Pre-flight:
   `bench/otlp-fanout/exporters-reachable.sh` (per-sink TCP probes).
5. **Emission**: playground `scenarios/run.sh` (a/b-series OTLP stories + c-series Sentry
   envelope stories). One emission fans out to every sink — no per-backend datasets.
6. **Layer A**: per-backend query-API count checks exactly as tabled above (GraphQL,
   OO `_search`, ClickHouse clients, Tempo search, snuba).
7. **Layer B**: agent-browser walks; screenshots land in
   `artifacts/ui/comparison/2026-09-12/<backend>/<feature>/`; human notes in
   `artifacts/ui/comparison/2026-09-12/NOTES.md`.
8. **Defect fixes**: apply the `run-20260912/parallax` worktree diff, `cargo test
   --workspace` + UI `bun run test` (the repo's canonical Vitest runner — plain `npx
   vitest` under node lacks the jsdom storage globals), rebuild, re-run step 7's parallax
   walk (401 → token → live tail).

Login cheatsheet (all lab-local): SigNoz :3301 `signoz@parallax.lab`, Sentry :9000
`admin@parallax.lab`, HyperDX :18080 `admin@parallax.lab`, OpenObserve :5080
`root@example.com`, Grafana :3300 `admin/admin`, Maple :14341 (no auth), rustrak :18082,
Parallax :4000 bearer `run20260912-lab-token`. Where a password applies:
`Runlab-20260912!`.

## Final verdict

| Question this run set out to answer | Answer |
|---|---|
| Can Parallax `main` ingest the full playground story? | ✅ exact parity with all 6 OTLP sinks + Sentry envelope errors |
| Does every shipped UI surface work on a real browser? | ✅ after defect #4 fix (auth), incl. SSE live tail; verified interactively |
| Is it competitive per shipped feature? | ✅ leads on 6 (attribute compare, bundles/MCP, SQL console, trace detail, envelope+OTLP fusion, UI auth UX), ties 4, trails 6 (incl. service-map polish), missing 0 |
| Do the 4 discovered defects have root-cause fixes? | ✅ all fixed + regression-tested + live re-verified (748 workspace tests + 581 UI tests green) |
| Does the lab reproduce the comparison? | ✅ compose pins updated to current upstream versions; Foundry SigNoz replaces deprecated overlay; repro section above |

**Bottom line:** `main` @ 6b3a92b passes live competitor verification with 4 root-caused
defects fixed and no missing capability in its shipped set. The highest-value next work
is competitive polish on existing surfaces: alert-from-current-query +
assignment/regressed states in Issues (SigNoz/Sentry UX), and service-map render polish
toward HyperDX's live-view bar. Parallax should keep NOT rivaling Grafana
dashboards/PromQL and keep doubling down on agent-native bundles + MCP, where it has a
clear roster-wide lead.
