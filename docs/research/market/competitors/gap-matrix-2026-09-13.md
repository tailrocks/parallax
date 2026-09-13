# Workflow gap matrix — 2026-09-13 pass 68 (Workstream A restamp)

Living comparison. Versions: [version-pins-2026-09-13.md](version-pins-2026-09-13.md).
PromQL: [promql-decision-2026-09-13.md](promql-decision-2026-09-13.md).
P0 freeze: finite list in scratch `p0-freeze.txt` (same 14 items; not grown).

**Parallax-today** is HEAD `544e5a3d83fd7662c9849526225880a49c7e2317`
(`goal/final-p0-hotfix`), not origin/main and not 2026-07 ledger prose. Pointers:

- GraphQL SDL `ui/graphql/schema.graphql` (77 Query / 14 Mutation, recounted
  2026-09-13 pass 68 — same count as ledger)
- Issues: `(service, fingerprint)` PK (`crates/parallax-metadata/src/turso/connection.rs`);
  UI `/issues/$service/$fingerprint`; occurrence selection + `CorrelationCard`
  (`ui/src/features/issues/`)
- Metrics: typed `metricQuery` + reset-clamped `rate`/`increase`
  (`crates/parallax-storage/src/adapter_math.rs`)
- RUM: `/rum` nav + vitals p75 via `histogramQuantile`
  (`ui/src/features/rum/`, `ui/src/shared/navigation.ts`)
- Exp histograms: ingest conversion (`docs/architecture/exp-histograms.md`)
- Playground: `scenarios/README.md` (91 proofs); macOS harness
  `parallax-telemetry-playground/macos/`

Status: `keep` = Parallax leads/tie, defend · `adopt` = gap to close this goal
if P0 else later · `remaining` = freeze P0 still open · `non-rivalry` =
deliberate not-compete · `watch` = drift.

“Why better” is a **workflow** reason.

## A. Errors (GOAL §4)

| Workflow / capability | Parallax today | Best implementation found | Product | Why it is better | P0/P1/P2/Reject | Backend gap | UI gap | Playground coverage | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Issue/error inbox | list + trend sparkline, open/resolved/regressed badge, service-scoped identity, sort LAST_SEEN/FIRST_SEEN/EVENTS/TREND; titles wrap (no `table-fixed` truncate) | Issue stream w/ for-review/new/regressed tabs, saved searches, Cmd+K triage | Sentry 26.8.0 | fewer clicks to a verdict; tabs encode lifecycle | P0 | none for derived `regressed` | triage tabs, bulk ops | `product:issue_context`, `sentry:envelopes` | keep (tabs P1) |
| Grouping + fingerprints | deterministic `fp-v1` (`parallax-analysis/src/fingerprint.rs`); grouping explanation on Issue | server grouping + custom fingerprint rules + grouping preview | Sentry 26.8.0 | adjustable when deterministic is wrong | P1 | fingerprint rule overrides | grouping preview + merge/split | `issues:burst`, `issues:multi_language` | adopt |
| Occurrence timeline | per-occurrence selection + trace correlation (HEAD) | event timeline w/ volume graph + per-event drill | Sentry 26.8.0 | richer per-occurrence forensics | P1 | none major | volume graph on detail | c8 | adopt |
| Regression detection | **shipped:** resolved issue + new occurrence → `regressed` (keep `resolved_at`); GraphQL filter; UI badge. No crash-free/suspect-release rollup | auto-regress on resolved-issue recurrence + release suspect | Sentry 26.8.0 | suspect-release answers “which release broke it” | P0 | first/last release rollup remaining | suspect-release remaining | `product:issue_regression`, `deploy:release_regression` | keep core / remaining release-health |
| New/resolved states | `issueSetStatus` open/resolve; derived `regressed` on recurrence; `resolved_at` kept | resolved-in-release, ignored/archived, auto-resolve | Sentry 26.8.0 | ignored/archived + resolved-in-release still missing | P0 | none for derived `regressed` | ignored/archived P1 | `product:issue_context` | keep |
| Assignment + ownership | none (no assignee column) | assignee + CODEOWNERS + Slack assign | Sentry 26.8.0 | triage ends in an owner | **P1** (dropped from freeze) | assignee store | assign control | none | adopt |
| Severity | severity words + ramp | level + issue priority | Sentry 26.8.0 | priority ≠ level | P2 | priority score | priority sort | none | watch |
| Stack traces + frames | culprit frames; `parseStacktrace` | frame collapsing, in-app, suspect frames | Sentry 26.8.0 | faster to the guilty line | P1 | in-app classifier | frame collapse UX | c8 | adopt |
| Source context | none (no code fetch) | inline source + suspect commits | Sentry 26.8.0 | guilty line + commit together | P1 | repo-link adapter | code frame | none | adopt |
| Exception chains | partial (derive from spans/logs) | chained exceptions + mechanism + threads | Sentry 26.8.0 | async/threaded failures readable | P1 | chain model | chained render | none | adopt |
| Breadcrumbs | shipped on Sentry envelopes | breadcrumbs + touch trail + replay | Sentry 26.8.0 | pre-crash story denser | P1 | none major | trail density | c8 | adopt |
| Tags/dimensions | tags JSON + cross-links | tag distribution facets per issue | Sentry 26.8.0 | Parallax cross-link unique; Sentry distribution deeper | P1 | none | distribution bars | c8 | keep |
| Users/sessions | `sessionId` on ErrorEvent; no user rollup | user tab: count, identity, affected-user trend | Sentry 26.8.0 | “how many users” is the triage question | P1 | user identity rollup | users tab | RUM partial | adopt |
| Environment | `environment` on ErrorEvent | env filter + per-env release health | Sentry 26.8.0 | env-scoped verdicts | P1 | env rollup | env filter | a13 (2 versions) | adopt |
| Release/build/deploy | `serviceVersion` on events; `releases()` windows; GitHub deploy ingest; service release strip | release health: adoption, crash-free, suspect commits | Sentry 26.8.0 | release *verdict*, not a strip | P0 | crash-free/session rollup | release health panel | a13, `product:github_ingest` | remaining |
| First/last seen, frequency | firstSeen/lastSeen/eventCount + trend | seen-stats + lifetime sparklines | Sentry 26.8.0 | polish gap only | P2 | none | stats polish | c8 | keep |
| Trace association | selected occurrence → `traceId`; CorrelationCard loads `trace` + `logsByTrace` | trace link + waterline on issue | Sentry 26.8.0 | tie on association; Sentry waterline denser | P0 | none | waterline embed optional | c8, `product:ui_agent_verify` | keep |
| Logs around error | CorrelationCard shows logs for selected occurrence’s trace | logs-on-issue + events-around | HyperDX 2.38.0 / Sentry | HyperDX log↔trace stitch is one gesture from any log | P0 | none (data present) | keep; also jump from log row → issue | `logs:trace_correlation`, c8 | keep |
| Metrics around error | `MetricStrip` on issue detail | metric widgets on issue (volume by tag) | Sentry 26.8.0 | blast radius in place | P1 | issue-window metric query | metric pane polish | none | adopt |
| Suspect spans / root cause | `traceCriticalPath` shipped | Seer Autofix + suspect-span ranking | Sentry 26.8.0 | proposes cause; Parallax must not become the fixer | P1 | span suspiciousness score | suspect panel; **no built-in fixer** | none | adopt |
| Saved investigations | case files w/ pins+notes | — | Parallax | portable pinned case + bundle | — | none | keep | `product:saved_state` | keep |
| Evidence bundles | `sha256-jcs:` hash-pinned | — | Parallax | agent-ready pinned bundle | — | none | keep | `product:issue_context` | keep |
| CLI/agent session links | `invocationId` on ErrorEvent; agent-session projection; Claude import | Preflight session obs | New Relic Preflight | Preflight deeper on coding-agent sessions; Parallax deeper on prod-link | P1 | session→issue join UX | session link on issue | `product:agent_session` | keep |
| Browser source maps | **none** (no artifact store in crates) | artifact bundles + debug-ids + Symbolicator | Sentry 26.8.0 / Honeycomb FEO | minified JS stacks unreadable without mapping | P0 | artifact store + mapping **or honest remaining-gap** | mapped frames | RUM error emits minified | remaining |
| Native macOS symbolication | none server-side; playground harness client-proven (dSYM UUID, atos) | dSYM upload + server symbolication | Sentry Cocoa v9 | macOS crashes unreadable without it | P1 (freeze: four-app-class macOS path is P0 *correlation*, not full dSYM product) | symbol store | symbolicated frames | `macos/` harness | remaining / specified |

## B. Logs (GOAL §4)

| Workflow / capability | Parallax today | Best implementation found | Product | Why it is better | P0/P1/P2/Reject | Backend gap | UI gap | Playground coverage | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Log explorer | histogram brush + Where chips + facets + patterns + live tail + saved views + **Create alert** (`log_count`) | Explore Logs w/ patterns-first + search→chart→alert | Grafana Loki 3.7.7 / HyperDX 2.38.0 | tighter loop from “weird line” to alert | P0 (usable explorer, not rows-only) | none major | keep | `logs:field_spike`, patterns, live tail | keep |
| Full-text search | `query` substring on body | LogQL line filters + parsed-field search | Grafana Loki 3.7.7 | language power at scale | P1 | text index depth | syntax help | log spike | adopt |
| Structured field search | Where-clause chips + columns | Lucene/ES\|QL | Elastic 9.5.3 | search-engine recall | P1 | none major | saved-field sets | structured bodies | keep |
| Facets / field discovery | `logFacets` + field stats | facet sidebar + cardinality hints | Datadog / SigNoz v0.141.1 | Datadog facet density is the reference | P1 | cardinality hints | facet density | facets | adopt |
| Query builder | typed Where editor, error-proof | TraceQL/LogQL/PromQL | Grafana LGTM 0.33.0 | language > structured for power users | P1 | — | keep error-proofness | Where queries | keep |
| Time navigation | URL range + histogram brush | brush + compare windows + surrounding jump | Grafana / HyperDX | window-compare missing on logs | P1 | log window-compare | port attribute-compare | histogram brush | adopt |
| Severity distribution | severity floor + facets | severity histogram | SigNoz / OpenObserve v1.0.0 | tie | P2 | none | keep | severity facets | keep |
| Multiline + JSON | structured shipped; multiline partial | ingest-time multiline rules + pretty JSON | Datadog / Elastic 9.5.3 | assembly at ingest | P1 | multiline rules | JSON pretty | `logs:bodies` | adopt |
| Embedded stack traces | derive errors from ERROR logs | stack view + create issue from log | Sentry 26.8.0 | closes log→triage | P1 | log→issue promotion | promote button | c8 | adopt |
| Service identity | service facets + colors | catalog link | SigNoz | tie | P2 | none | keep | facets | keep |
| Patterns/clustering | Drain `logPatterns` | Log Patterns wired to alerts | Datadog / Loki | pattern-insights → action | P1 | pattern volume alerts | pattern actions | `logs:patterns` | adopt |
| Surrounding logs | `logsAround` API + UI context window | context ±N + live context | Grafana Loki / HyperDX | tie | P0 | none | keep | around-anchor | keep |
| Trace/span correlation | log→trace links; `logsByTrace` | trace-id jump + embedded span | HyperDX 2.38.0 | stitch without copying IDs | P0 | none | mini-span embed polish | `logs:trace_correlation` | keep |
| Issue correlation | tags cross-links | create issue from log | Sentry 26.8.0 | log→triage loop | P1 | promotion path | promote button | none | adopt |
| Metric correlation | weak on logs surface | split-chart log↔metric | Datadog | generate metrics from patterns | P1 | log-derived metric defs | metric-from-log | none | adopt |
| Saved views + history | saved views + SQL history | recent history + share | SigNoz v0.141.1 | recall beats rebuild | P2 | query history store | history dropdown | `product:saved_state` | adopt |
| Live tail | SSE logs+traces | Live Tail on every search | HyperDX / Loki | tie; Parallax unique in trace tail | — | reconnect hardening | keep | `product:live_tail` | keep |
| High-volume usability | virtualized tables; `logs:burst` 5k; **no sampling policy** | stream mgmt (OO) / Refinery (Honeycomb) | OpenObserve v1.0.0 / Honeycomb | signal not silently lost at volume | P0 | sampling policy + drop reasons | volume guardrails | `sampling:low_sample_gap` (gap demo) | remaining |
| Keyboard navigation | ⌘K + zoom; row-nav partial | j/k triage | Sentry / Honeycomb | mouseless operable | P1 | none | row keyboard map | `product:ui_agent_verify` | adopt |
| CLI-oriented logs | `parallax logs --follow`; bounded child stdout on invocations | — | Parallax | unique agent fix-verification | — | none | keep | CLI runs | keep |

## C. Traces (GOAL §4)

| Workflow / capability | Parallax today | Best implementation found | Product | Why it is better | P0/P1/P2/Reject | Backend gap | UI gap | Playground coverage | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Trace search + list | `tracesPage` + facets + duration + errors-only | explorer w/ funnels, List/TimeSeries/Table | SigNoz v0.141.1 | wider analytical surface | **P1** (not freeze; freeze is “understandable waterfall”) | funnel queries | result-view switcher | checkout queries | adopt |
| Waterfall + span tree | waterfall/tree/errors/lanes/flame + minimap | — | Parallax | densest single-trace detail in roster (09-12 live) | P0 | none | keep | waterfall, wide, deep | keep |
| Critical path | `traceCriticalPath` | critical-path + contribution rank | Datadog / Parallax | tie | P0 | none | keep | critical path | keep |
| Duration visualization | minimap + `traceDurationStats` | latency histogram + span breakdown | Honeycomb / Jaeger 2.20.0 | distribution-first | P2 | surface duration stats | duration panel | duration filter | adopt |
| Errors/status/attrs/events/links | all shipped; `traceEvents` persisted HEAD | span links + baggage | Grafana Tempo 3.0.3 | tie | P2 | none | keep | span events, links, baggage | keep |
| DB/external ops | typed ecosystem nodes + **`Trace.dominantDbQueries`** (normalized SQL rank) | query-level aggregation | Datadog APM / New Relic | Datadog still has explain/pool | P0 (freeze: DB dominance) | none for rank | keep | `postgres:query_pressure` | keep |
| Service boundaries | color-by-attribute + lanes | topology-in-trace | Dynatrace / HyperDX | overlay | P2 | none major | overlay toggle | service boundaries | watch |
| Parent/child navigation | shipped | keyboard span walk | Jaeger 2.20.0 / Tempo | tie | P2 | none | keep | span index | keep |
| Cross-trace/run relations | invocation stitching + `linkedTraces` | session stitching of multi-trace agent flows | Honeycomb Agent Timeline GA 2026-06-18 | reconstructs multi-trace conversations | P1 | multi-trace session view | session lane | links, RUM stitch | adopt |
| Compare traces | `traceCompare` + attribute-compare | — | Parallax | unique window-vs-window ranked diff | — | none | keep | trace compare | keep |
| Slow/anomalous spans | duration filter | anomaly-flagged spans | Dynatrace Davis / Honeycomb | automatic “weird span” | P2 | baseline model | anomaly badges | slow traces | adopt |
| Trace-derived RED | `serviceRed` | auto-dashboards per endpoint | SigNoz / Grafana | SigNoz endpoint auto-views deeper | P1 | endpoint rollup | endpoint tab | RED | adopt |
| Trace→logs/metrics/errors | correlated logs inline; exemplars; issue links; **Create alert** from traces (`error_rate`/`p95_latency`) and logs (`log_count`) | one-click pivots + **alert-from-query** | HyperDX / SigNoz | insight → monitor without leaving the pivot | P0 | none | keep | correlated logs | keep |
| Exemplars | `metricExemplars` + click-through | exemplar dots on every chart | Grafana / Parallax | tie | P2 | none | keep | `metrics:exemplars` | keep |
| Frontend→backend | RUM stitch + `/rum` journeys over `tracesPage` | session→trace waterfall | Sentry / HyperDX | Sentry session-linked traces deeper | P0 | session model | session lane polish | `browser:rum_journey` | remaining |
| CLI→backend | TRACEPARENT injection + `otlp-forward` | — | Parallax | CLI run as first-class trace root | P0 | none | keep | `cli:checkout_invocation` | keep |
| macOS→backend | playground harness injects `traceparent`; Parallax joins `trace_id` | Apple SDK crash↔trace | Sentry Cocoa | no native product SDK | P0 (four-app-class) | native OTel Swift path | client span lane | `macos/` harness | remaining |
| Sampling/tail sampling | none (head ingest) | Refinery adaptive tail sampling | Honeycomb | cost+completeness at volume | P0 (high-volume guardrails) | sampling policy + rate attribution | sampling controls | `sampling:low_sample_gap` | remaining |

## D. Metrics (GOAL §4)

| Workflow / capability | Parallax today | Best implementation found | Product | Why it is better | P0/P1/P2/Reject | Backend gap | UI gap | Playground coverage | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Metric discovery/catalog | `metricCatalog` + point counts | Metrics Explorer w/ type + unit | Grafana Mimir 3.2.1 / Datadog | metadata-rich browse | P1 | metric metadata store | catalog polish | catalog | adopt |
| Dimensions/group-by | group-by + step + label values | multi-dim group + autocomplete | Prometheus 3.14.0 / Mimir | PromQL label power | P1 | label autocomplete index | label UX | `metrics:labels` | adopt |
| Rate/delta semantics | **shipped:** `rate`/`increase` reset-clamped (`adapter_math.rs`); kind-legal | `rate()`/`increase()` + counter-reset | Prometheus 3.14.0 | table-stakes counter math — Parallax now has the safe form | P0 | none for rate/increase | fn picker already legal-only | `metrics:shapes` (counter reset) | keep |
| Counters/gauges/histograms | histogram quantile; **exp histograms converted at ingest** (HEAD) | native + exp histograms | Prometheus 3.14.0 / Mimir | exp at precision/cost | P1 | Summary still dropped (counted) | quantile viz | exp + histogram quantile | keep (exp path) |
| Percentile viz | p50/p95/p99 | percentile bands | Datadog / Grafana | variance readable | P2 | none | band render | p95 | keep |
| Query power | typed `metricQuery`; SQL escape hatch | PromQL Explore | Grafana/Prometheus 3.14.0 | full language | P0 **decision: keep typed builder** | none | do not add PromQL box | workbench | keep (decision) |
| Grouping/comparison | attribute compare (traces); metric timeshift absent | timeshift overlays | Grafana / Datadog | “vs last week” one click | P1 | timeshift fn | compare control | none | adopt |
| Exemplar links | dots + trace click-through | exemplar on every panel | Grafana / Parallax | tie | P2 | none | keep | exemplars | keep |
| Metric→trace/log/error | exemplar→trace; RED→services; **Traces around peak** (`peakWindowFromSeries` ±60s → `/traces`) | spike window auto-surfaces related traces | Datadog / New Relic | Datadog still denser auto-surface; Parallax now has the pivot | P0 | none for peak window | keep | metric detail | keep |
| RED/service/runtime/DB | `serviceRed` + `runtimeSnapshot` | APM auto metric sets + host maps | Datadog / Coroot v1.26.0 | out-of-box breadth | P1 | metric set breadth | auto-panels | tokio/jvm | adopt |
| Frontend Web Vitals | `/rum` p75 via `histogramQuantile`; rating good/NI/poor | dedicated Web Vitals + RUM perf | Sentry / Datadog RUM / Better Stack | vitals need their own lens | P0 | vitals rollup exists as query | session-level UX still thin | `browser:rum_journey`, web-vitals | remaining |
| Cardinality visibility | field stats | cardinality explorer + top-k | Mimir 3.2.1 / Honeycomb | names the death | P1 | per-label accounting | cardinality panel | cardinality stress | adopt |
| Missing-data behavior | incomplete-bucket dashed tail (metric detail) | explicit null/gap + no-data alerts | Grafana / SigNoz | honest gaps vs fake zeros | P2 | gap policy | gap render | `metrics:shapes` gauge gap | adopt |
| Dashboard panels | widget grid; Grafana-grade depth absent | panels + variables + alerting | Grafana 13.2.1 | industry reference; not a contest | P1 | variables | **do not Grafana-embed** | `product:saved_state` | non-rivalry |
| Saved queries/defaults | saved views + snippets | default service dashboards | SigNoz / Datadog | zero-click start | P1 | default dashboard set | defaults | `product:saved_state` | adopt |

## E. Adjacent capabilities (GOAL §5) — every candidate classified

| Workflow / capability | Parallax today | Best implementation found | Product | Why it is better | P0/P1/P2/Reject | Backend gap | UI gap | Playground coverage | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Dashboards | gallery + widget grid + brush-to-zoom; graduate from metric | dashboards + variables + folders + alerting | Grafana 13.2.1 | reference depth; embed beats rival | P1 | variables | builder depth **not** Grafana-embed | `product:saved_state` | non-rivalry |
| Alerting rules | error_rate/p95/p99/throughput/log_count/metric + hysteresis; **Create alert** from metric, logs (`log_count`), traces (`error_rate`/`p95_latency`) | alert-from-any-query | SigNoz v0.141.1 | SigNoz still graduates any query shape | P0 | none for freeze path | keep | `product:alerting`, error-rate/p95 breach | keep |
| Alert channels | webhook + Slack shipped; **email deferred** (`delivery_worker.rs`); schema comment still lists `email` | 10 channel kinds | SigNoz v0.141.1 | on-call reach | P0 freeze: “existing webhook/Slack; extend if missing” → **webhook/Slack is the core path**; email = P1 | email worker | destination UX | `product:alerting` | keep core / email P1 |
| Incidents | incidents + bundle hash | incident timeline + tasks + postmortem | Datadog / Sentry / Better Stack | collaboration depth | P1 | activity model | timeline UX | `product:alerting` | keep |
| On-call rotations | none | rotations + escalations | Better Stack / PagerDuty | Parallax stops at incident | P2 | rotation model | schedule UX | none | adopt |
| SLOs/error budgets | none | SLO + burn-rate alerts | Datadog / Grafana / Sentry | burn-rate is the alerting unit | P1 | SLI/SLO store | SLO view | none | adopt |
| Service catalog | heat catalog + RED + runtime | catalog + ownership + docs | New Relic / Datadog | ownership turns catalog into map | P1 | ownership metadata | owner/docs | heat catalog | adopt |
| Service/dependency maps | Ecosystem typed graph | live map + legend + animation | HyperDX 2.38.0 BETA | polish | P1 | none (model deeper) | legend + traffic animation | 17-node / `ecosystem:full` | adopt |
| Deploy/release markers | release strip + GitHub deploy ingest + `chartAnnotations` overlay on metric detail | markers on **every chart** | Sentry / Datadog / Grafana annotations | markers where eyes already are | P0 | none for release windows | other-chart overlays P1 | a13, `product:github_ingest` | keep (metric) |
| CI/test context | JUnit/nextest + flaky explorer | CI Test Optimization + quarantine | Datadog | quarantine + owner routing | P1 | quarantine state | quarantine UX | flaky detection | keep |
| Database monitoring | derived nodes + wrapper spans | query stats + explain + pool | Datadog DBM / New Relic | query-level | P1 | query aggregation | query view | Postgres pathologies | adopt |
| Infra/runtime telemetry | runtime snapshot tokio/jvm | host/container/K8s + eBPF | Datadog / Coroot v1.26.0 | zero-instrument breadth | P1 | host inventory | infra view | tokio saturation | adopt |
| Continuous profiling | none | eBPF always-on + flame + diff | Pyroscope 2.3.1 / Coroot / Datadog | profiles explain “why slow” traces cannot | **Reject this freeze** (P1-next) | OTLP profiles | flame | none | reject-now |
| Frontend RUM | `/rum` projection: journeys=`tracesPage`, vitals=`histogramQuantile`, errors=`issues` | RUM sessions + Web Vitals + frustration | Sentry / Datadog / Better Stack | real session product vs query remix | P0 | session model | session view depth | `browser:rum_journey`, `browser:rum_error` | remaining |
| Browser/network requests | RUM stitch only | resource waterfall per page | Sentry / Datadog RUM | per-view network truth | P1 | resource timing ingest | resource table | RUM journey | adopt |
| Session replay | none | OSS replay + error-linked | HyperDX 2.38.0 / PostHog (Highlight dead) | see-what-user-saw | **Reject this freeze** | replay store | player | none | reject-now |
| Uptime checks | none | monitors + status pages | Better Stack / Sentry Monitors | entry-level obs | P1 | check runner | monitors view | none | adopt |
| Synthetics | none | scripted browser/API tests | Datadog / Better Stack | proactive | **Reject this freeze** | script runner | results | none | reject-now |
| Anomaly detection | none | Watchdog / Davis | Datadog / Dynatrace | no-threshold alerting | P2 | baseline engine | anomaly badges | none | adopt |
| Log/metric pattern detection | Drain logs; metric none | pattern-insights + metric anomaly | Datadog | patterns wired to alerts | P1 | metric pattern scan | pattern alerts | patterns | adopt |
| Query history | SQL history; others partial | full recent + starred | SigNoz / Grafana | recall | P2 | history store | history UX | `product:saved_state` | adopt |
| Sharing/permalinks | URL-driven filters everywhere | — | Parallax | share-by-URL is reference-grade | — | none | keep | all routes | keep |
| Annotations | GraphQL `chartAnnotations` (service optional) + metric `ReferenceLine` overlay + release strip | annotations on every chart | Grafana / Datadog | Grafana still annotates every panel type | P0 | none for release markers | overlay on metric detail; other charts P1 | a13 | keep (metric) |
| Feature-flag correlation | playground flagd; product join unclear | flag evals on timeline | Sentry / Honeycomb | “flag flip caused it” | P2 | flag-eval join | flag lane | `feature_flags:checkout_variants` | adopt |
| Cost/cardinality visibility | field stats; self-host no metering | usage metering + guardrails | Mimir / Honeycomb / Datadog | metering irrelevant; guardrails matter | P2 | cardinality guardrails | guardrail UX | cardinality | adopt |
| Retention controls | TTLs + pin-aware prune | tiered retention + downsample | Grafana / Elastic / OpenObserve | cold-tier economics | P1 | object-store tier | retention UX | `product:lifecycle_ops` | adopt |
| Pipeline health | `/health` 503 + self-OTLP; loss JSON | per-stage lag/drop dashboards | Mezmo / Datadog Pipelines | per-stage truth | P1 | stage counters | health view | ingest health | adopt |
| Dropped-data diagnostics | loss counters (`unsupported_metric`, exp drop counted) | named drop reasons | OTel Collector / Honeycomb Refinery | named reasons, not silent gaps | P0 (pairs with sampling) | drop reason codes | drop panel | sampling gap | remaining |
| eBPF zero-instrumentation | none | eBPF → any backend | Odigos v1.36.0 / Coroot v1.26.0 | zero-code adoption | **Reject building**; watch integrate | none | source badges | none | reject-now |
| Sentry SDK compatibility | envelope ingest shipped; playground Rust/Java/JS envelopes; **public multi-SDK ledger unproven** | native protocol | Sentry 26.8.0 / rustrak v0.14.12 | compat breadth decides migration | P0 | SDK matrix harness + dual-auth | compat ledger | `sentry:envelopes` | remaining |
| MCP/agent breadth | **2 read-only tools** (`parallax_issue_context`, `parallax_agent_session_show`); issue tool now requires `service` | 41+ (SigNoz MCP v0.14.0) / 140+ OO EE / 56 mutating (Rustrak) | SigNoz MCP | breadth vs safety: Parallax safest, narrowest | P1 | more **read-only** tools | none | MCP checks | adopt |
| AI triage/RCA | none (context engine, not fixer) | Seer / Auto-investigations / Davis / Bits SRE | Sentry / Honeycomb / Dynatrace / Datadog | incumbents ship investigators; Parallax feeds them | P1 | bundle quality (A1) | “send to agent” | none | non-rivalry |
| Status pages | none | hosted status + subscribers | Better Stack | comms product, not debugging | **Reject** | — | — | — | — |
| Compliance certs (SOC2/HIPAA) | none | attestations | Datadog | procurement gate, not dev UX | **Reject** | — | — | — | — |
| Distributed/HA/multi-region | single binary | sharded multi-region SaaS | Datadog / Grafana Cloud | scale Parallax defers | **Reject** | — | — | — | — |
| Built-in fixer LLM | none | HolmesGPT / Causely / Bits | fixer layer | Parallax feeds fixers via bundle+MCP | **Reject** | — | — | — | — |
| SSO/RBAC/multi-tenancy | planned | SSO/SAML/OIDC + RBAC | Datadog / Sentry / SigNoz EE | team-adoption gate | P1 | authZ model | login/role UX | none | adopt |
| Windows/Linux desktop native | none | native desktop agents | various | freeze: macOS only | **Reject** | — | — | — | — |

## F. Frozen P0 (finite — do not grow)

Seed freeze validated against live pins + HEAD code. Later discoveries stay P1 / P0-next.

1. **Error triage lifecycle** — identity, occurrence, open/resolve, **derived `regressed` shipped**. Remaining: crash-free / first-last release rollup.
2. **Cross-signal navigation** — issue↔trace↔logs↔invocation↔exemplar↔peak-window traces **shipped**. Remaining: RUM session model (not a remix).
3. **Logs** — surrounding + span/trace correlation **shipped**; explorer is not rows-only; **alert-from-log shipped** (`log_count`).
4. **Traces** — waterfall/tree **keep**; **`dominantDbQueries` shipped**; **alert-from-trace shipped** (`error_rate`/`p95_latency`).
5. **Metrics** — rate/increase **shipped**; exemplars **shipped**; **spike → traces around peak shipped**.
6. **Alert-from-query + core notify** — metric/logs/traces Create alert **shipped**; webhook/Slack **shipped**; email deferred (P1, not freeze).
7. **Deploy/release markers + chart annotations** — release strip **shipped**; GraphQL `chartAnnotations` + metric overlay **shipped**.
8. **RUM sessions + Web Vitals UX** — `/rum` projection **shipped**. Remaining: real session model, not remix.
9. **Browser source maps** — **honest remaining-gap** (no artifact store).
10. **Sentry multi-SDK** — playground rust/java/js envelopes exist; **public ledger unproven**.
11. **High-volume guardrails / sampling** — **honest remaining-gap**.
12. **PromQL** — **decided: keep typed builder** (see §H). Implementation = existing `metricQuery` + shipped rate/increase. No Grafana-embed.
13. **Four app classes** — CLI reconstructable **shipped**; backend HTTP/gRPC/DB/cache/messaging **playground-covered**; browser FE→BE **partial** (`/rum`); native macOS **harness proven**, product symbolication remaining.
14. **Named GOAL §7 questions** — checkout fail / logs-around / DB dominance / spike traces / release markers answerable from shipped UI/API. Remaining: source maps, sampling, RUM sessions, Sentry SDK ledger, release-health.

## G. Differentiators to defend (GOAL §13)

Attribute-compare · hash-pinned evidence bundles · read-only SQL · CLI runs as
first-class traces · investigations case files · MCP read-only safety (2 tools)
· URL-shareability · single-binary self-host · Apache-2.0 · Sentry+OTLP dual
ingest · Rust-first GreptimeDB+Turso · typed kind-legal metrics (now including
reset-clamped rate/increase) · service-scoped issue identity.

None of the P0–P2 adoptions may regress these.

## H. PromQL decision (freeze item 12)

**Keep typed error-proof builder. Reject Grafana-embed. Reject PromQL-as-product-language.**

Full write-up: [promql-decision-2026-09-13.md](promql-decision-2026-09-13.md).

Workflow reason: developers get correct counter math without PromQL footguns;
power users already have SQL; Grafana-embed would split navigation and break
single-binary self-host.

## I. Thesis change (GOAL §1 / §8)

Operator intent is a **complete developer observability replacement** for
errors, logs, traces, metrics, CLI, backend, frontend, macOS, alerts, and
agent context — not a narrow “execution-context engine.”

Vision docs restamped this pass (`00-vision/thesis.md`,
`problem-audience-product-shape.md`, `platform-direction.md`). Evidence bundles
+ CLI runs + read-only MCP remain the *differentiator*, not the *scope cap*.

Still Reject: status pages, SOC2/HIPAA, HA multi-region, built-in fixer LLM.
