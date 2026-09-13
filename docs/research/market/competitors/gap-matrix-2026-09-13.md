# Workflow gap matrix — 2026-09-13 (Workstream A, GOAL.md §12)

Living comparison. Versions: [version-pins-2026-09-13.md](version-pins-2026-09-13.md).
Parallax-today grounded in `inv:`=[feature-inventory-and-playground-verification.md](../../reference/feature-inventory-and-playground-verification.md),
`09-12:`=[2026-09-12 report](../../validation/2026-09-12-parallax-main-competitor-verification.md),
`ledger:`=[code-reality-ledger.md](../../code-reality-ledger.md).
Playground refs: `product:*`/`a*`/`b*`/`c*` scenarios per `inv:L192-200`.
Status: `keep` = Parallax leads/tie, defend; `adopt` = gap to close;
`non-rivalry` = deliberate embed/not-compete; `watch` = drift risk.

## A. Errors (GOAL §4)

| Workflow / capability | Parallax today | Best implementation found | Product | Why it is better | P0/P1/P2/Reject | Backend gap | UI gap | Playground coverage | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Issue/error inbox | list + trend sparkline, open/resolve, 52 groups live (`09-12:L132`) | Issue stream w/ for-review/new/regressed tabs, saved searches, Cmd+K triage | Sentry 26.8.0 | decade of triage density: fewer clicks to verdict | P0 | regressed-state derivation | triage tabs, bulk ops | `product:issue_context`, c8 | adopt |
| Grouping + fingerprints | deterministic `fp-v1` (`inv:L68`) | server grouping + custom fingerprint rules + grouping preview | Sentry 26.8.0 | adjustable when deterministic is wrong; Parallax has no override | P1 | fingerprint rule overrides | grouping preview + merge/split | c8 | adopt |
| Occurrence timeline | per-occurrence selection + trace correlation (HEAD `de5592ae`) | event timeline w/ volume graph + per-event drill | Sentry 26.8.0 | richer per-occurrence forensics | P1 | none major | volume graph on detail | c8 | adopt |
| Regression detection | none (no regressed state) | auto-regress on resolved-issue recurrence + release suspect | Sentry 26.8.0 | answers "which release broke it" automatically | P0 | recurrence-vs-release detector | regressed badge + suspect release | a13 deploy regression | adopt |
| New/resolved states | open/resolve (`inv:L69`) | resolved-in-release, ignored/archived, auto-resolve | Sentry 26.8.0 | lifecycle matches real triage | P0 | state machine extension | state transitions | `product:issue_context` | adopt |
| Assignment + ownership | none | assignee + ownership rules (CODEOWNERS/path) + Slack assign | Sentry 26.8.0 | triage ends in an owner, not a tab | P0 | assignee store + rules | assign control | none | adopt |
| Severity | severity words + ramp (`DESIGN.md` §5) | level + issue priority (seer-ranked) | Sentry 26.8.0 | priority separates signal from level noise | P2 | priority score | priority sort | none | watch |
| Stack traces + frames | culprit frames (`inv:L114`) | frame package collapsing, in-app detection, suspect frames | Sentry 26.8.0 | faster to the guilty line | P1 | in-app classifier | frame collapse UX | c8 | adopt |
| Source context | none (no code fetch) | inline source lines + suspect commits | Sentry 26.8.0 | guilty line + guilty commit together | P1 | repo-link adapter | code frame | none | adopt |
| Exception chains | partial (derive from spans/logs) | chained exceptions w/ mechanism + thread render | Sentry 26.8.0 | async/threaded failures readable | P1 | chain model | chained render | none | adopt |
| Breadcrumbs | shipped (`inv:L115`) | breadcrumbs + touch trail + replay link | Sentry 26.8.0 | pre-crash story denser | P1 | none major | trail density | c8 partial | adopt |
| Tags/dimensions | tags-from-attributes cross-links, unique (`09-12:L132`) | tag distribution + tag facets per issue | Sentry 26.8.0 | tie: Parallax cross-linking unique, Sentry distribution UX deeper | P1 | none | distribution bars | c8 | keep |
| Users/sessions | session.id browser; user attribution partial | user tab: count, identity, affected-user trend | Sentry 26.8.0 | "how many users" is the triage question | P1 | user identity rollup | users tab | RUM scenarios partial | adopt |
| Environment | env shown | env filter + per-env release health | Sentry 26.8.0 | env-scoped verdicts | P1 | env rollup | env filter | a13 (2 versions) | adopt |
| Release/build/deploy context | release strip + GitHub deploy/CI ingest (`inv:L46-47`) | release health: adoption, crash-free, suspect commits | Sentry 26.8.0 | release verdict, not just strip | P0 | crash-free/session rollup | release health panel | a13, `product:github_ingest` | adopt |
| First/last seen, frequency | trend + counts | seen-stats + lifetime sparklines + stats API | Sentry 26.8.0 | tie-ish; polish gap only | P2 | none | stats polish | c8 | keep |
| Trace association | per-occurrence trace link (HEAD) | trace link + trace waterline on issue | Sentry 26.8.0 | tie | P1 | none | waterline embed | c8 | keep |
| Logs around error | correlated logs inline on trace; issue→logs path | logs-on-issue + "events around" | Sentry/HyperDX 2.38.0 | HyperDX log↔trace stitching slicker | P1 | none | one-click issue→surrounding-logs | c8 | adopt |
| Metrics around error | service RED nearby; no issue-scoped metric pane | metric widgets on issue (volume by tag) | Sentry 26.8.0 | quantifies blast radius in place | P1 | issue-window metric query | metric pane | none | adopt |
| Suspect spans / root cause | critical path shipped | Seer Autofix + suspect-span ranking + similar issues | Sentry 26.8.0 | proposes cause+fix, not just path | P1 | span suspiciousness score | suspect panel; no built-in fixer (thesis) | none | adopt |
| Saved investigations | case files w/ pins+notes (`inv:L128`) | — | Parallax | unique: portable pinned case + bundle | — | none | keep | `product:saved_state` | keep |
| Evidence bundles | `sha256-jcs:` hash-pinned, unique (`09-12:L137`) | — | Parallax | unique: agent-ready pinned bundle | — | none | keep | `product:issue_context` | keep |
| CLI/agent session links | agent-session projection + Claude import (`inv:L75`) | Preflight session obs (SaaS) | New Relic Preflight | Preflight deeper on coding-agent sessions; Parallax deeper on prod-link | P1 | session→issue join UX | session link on issue | `product:agent_session` | keep |
| Browser source maps | none | artifact bundles + debug-ids + Symbolicator | Sentry 26.8.0 / Honeycomb FEO GA | readable stacks from minified JS; Parallax cannot | P0 | artifact store + mapping | mapped frames | RUM error partial | adopt |
| Native macOS symbolication | none | dSYM upload + symbolication + MachO line tables (plan-102 note exists) | Sentry 26.8.0 | macOS crashes unreadable without it | P1 | symbol store + atos-class mapping | symbolicated frames | none (needs macOS host) | adopt |

## B. Logs (GOAL §4)

| Workflow / capability | Parallax today | Best implementation found | Product | Why it is better | P0/P1/P2/Reject | Backend gap | UI gap | Playground coverage | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Log explorer | histogram brush + Where chips + facet autocomplete (`inv:L118`) | Explore Logs w/ patterns-first triage | Grafana Loki 3.7.7 / HyperDX 2.38.0 | HyperDX search-to-chart-to-alert loop tighter | P1 | none major | search→chart→alert loop | log spike, patterns | adopt |
| Full-text search | search shipped | LogQL line filters + parsed-field search at scale | Grafana Loki 3.7.7 | language power over structured-only | P1 | text index depth | syntax help | log spike | adopt |
| Structured field search | Where-clause chips + columns (`inv:L118`) | Lucene/ES\|QL field search | Elastic 9.5.3 | search-engine recall on huge fields | P1 | none major | saved-field sets | structured logs | keep |
| Facets / field discovery | facets + field explorer + field stats (`inv:L101-108`) | facet sidebar + inferred fields + cardinality hints | SigNoz v0.141.1 / Datadog | Datadog facet UX is the reference density | P1 | cardinality hints | facet density | facets active | adopt |
| Query builder | typed Where editor, error-proof (`09-12:L128`) | TraceQL/LogQL/PromQL full languages | Grafana LGTM 0.33.0 | language > structured for power users | P1 | documented scope defaults (`service.name` resolves) | keep error-proofness | Where queries | keep |
| Time navigation | URL time range + histogram brush (`inv:L114-120`) | brush + compare windows + surrounding-lines jump | Grafana / HyperDX 2.38.0 | window-compare is Parallax's own trace win; missing on logs | P1 | log window-compare | port attribute-compare to logs | histogram brush | adopt |
| Severity distribution | severity floor + facets | severity histogram + level stats | SigNoz / OpenObserve v1.0.0 | tie | P2 | none | keep | severity facets | keep |
| Multiline + JSON | structured shipped; multiline partial | multiline policies + JSON auto-parse + pretty | Datadog / Elastic 9.5.3 | ingestion-time multiline rules | P1 | multiline assembly rules | JSON pretty polish | structured logs | adopt |
| Embedded stack traces | derive errors from ERROR logs (`ledger:L35`) | stack view inside log line + issue create | Sentry 26.8.0 | one-click log→issue | P1 | log→issue promotion | promote button | c8 | adopt |
| Service identity | service facets + service colors | service-name inference + catalog link | SigNoz v0.141.1 | tie | P2 | none | keep | facets | keep |
| Patterns/clustering | Drain patterns (`inv:L102`) | Log Patterns (auto-cluster + one-click filter/exclude) | Datadog / Grafana Loki | Datadog pattern-insights wired to alerts | P1 | pattern volume alerts | pattern actions | patterns view | adopt |
| Surrounding logs | around-anchor (`inv:L101`) | context view ±N + live context | Grafana Loki / HyperDX | tie | P2 | none | keep | around-anchor | keep |
| Trace/span correlation | log→trace links; correlated counts | trace-id jump + embedded span context | HyperDX 2.38.0 | HyperDX stitching slicker | P1 | none | embed mini-span | RUM stitch | adopt |
| Issue correlation | tags cross-links | "create issue from log" + issue stream link | Sentry 26.8.0 | closes log→triage loop | P1 | promotion path | promote button | none | adopt |
| Metric correlation | weak on logs surface | split-chart log↔metric + metric-from-logs | Datadog / Splunk | Datadog generates metrics from log patterns | P1 | log-derived metric defs | metric-from-log action | none | adopt |
| Saved views + history | saved views + SQL history (`inv:L108,130`) | saved queries + recent history + share links | SigNoz v0.141.1 | SigNoz recent-history deeper | P2 | query history store | history dropdown | `product:saved_state` | adopt |
| Live tail | SSE logs+traces (`09-12:L130`, 3-way tie) | Live Tail on every search | HyperDX 2.38.0 / Loki Live | tie; Parallax unique in trace tail | — | reconnect hardening | keep | `product:live_tail` | keep |
| High-volume usability | virtualized tables; cardinality partial | 1.8M-event streams + stream mgmt (OO) / sampling (Honeycomb) | OpenObserve v1.0.0 / Honeycomb | petabyte stream mgmt; Parallax single-node | P0 | stream mgmt + sampling policy | volume guardrails | cardinality stress | adopt |
| Keyboard navigation | ⌘K + keyboard zoom; row-nav partial | full keyboard triage (j/k, e, x) | Sentry / Honeycomb | Honeycomb/Sentry operable mouseless | P1 | none | row keyboard map | `product:ui_agent_verify` | adopt |
| CLI-oriented logs | `parallax logs` + `--follow` (`inv:L85`) | — | Parallax | unique agent fix-verification signal | — | none | keep | CLI runs | keep |

## C. Traces (GOAL §4)

| Workflow / capability | Parallax today | Best implementation found | Product | Why it is better | P0/P1/P2/Reject | Backend gap | UI gap | Playground coverage | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Trace search + list | search + facets + duration + errors-only (`09-12:L125`) | explorer w/ funnels, Trace Matching beta, List/TimeSeries/Table | SigNoz v0.141.1 | wider analytical surface per query | P0 | funnel queries | result-view switcher | checkout queries | adopt |
| Waterfall + span tree | waterfall/tree/errors/lanes/flame, densest tested (`09-12:L126`) | — | Parallax | densest single trace detail in roster | — | none | keep | waterfall | keep |
| Critical path | shipped (`inv:L117`) | critical-path + latency contribution rank | Datadog / Parallax | tie | P2 | none | keep | critical path | keep |
| Duration visualization | minimap + zoom | latency histogram + duration breakdown by span | Honeycomb / Jaeger 2.20.0 | Honeycomb distribution-first | P2 | duration stats API exists (`inv:L100`); surface it | duration panel | duration filter | adopt |
| Errors/status/attrs/events/links | all shipped incl. Links tab (`inv:L100-101`) | span links + baggage render | Grafana Tempo 3.0.3 | tie | P2 | none | keep | span links, baggage | keep |
| DB/external ops | typed ecosystem nodes (postgresql, stripe) (`09-12:L135`) | span-tag-driven DB dashboard + query stats | Datadog APM / New Relic | query-level aggregation, not just nodes | P1 | span-derived query stats | query panel | Postgres pathologies | adopt |
| Service boundaries | color-by-attribute + lanes | service graph overlay on trace | Dynatrace Smartscape / HyperDX | topology-in-trace | P2 | none major | overlay toggle | service boundaries | watch |
| Parent/child navigation | shipped | keyboard span walk + minimap sync | Jaeger 2.20.0 / Tempo | tie | P2 | none | keep | span index | keep |
| Cross-trace/run relations | invocation stitching + links (`inv:L87-90`) | trace-to-trace + session stitching | Honeycomb Agent Timeline (GA) | Timeline reconstructs multi-trace agent flows | P1 | multi-trace session view | session lane | links 1/1, RUM stitch | adopt |
| Compare traces | trace compare shipped | diff view w/ attribute deltas | Parallax (+attribute compare unique `09-12:L127`) | unique window-vs-window ranked diff | — | none | keep | trace compare | keep |
| Slow/anomalous spans | duration filter; no anomaly | anomaly-flagged spans + baseline compare | Dynatrace Davis / Honeycomb | automatic "weird span" flag | P2 | baseline model | anomaly badges | slow traces | adopt |
| Trace-derived RED | services RED (`inv:L122`) | RED auto-dashboards per service/endpoint | SigNoz / Grafana | tie-ish; SigNoz endpoint auto-views deeper | P1 | endpoint rollup | endpoint tab | RED | adopt |
| Trace→logs/metrics/errors | correlated logs inline; exemplars; issue links | one-click pivots all directions | HyperDX 2.38.0 / SigNoz | SigNoz alert-from-query + add-to-dashboard from any pivot | P0 | none (data present) | **alert-from-query + add-to-dashboard buttons** | correlated logs | adopt |
| Exemplars | `metricExemplars` + trace click-through (`inv:L102,117`) | exemplar dots on every chart | Grafana / Parallax | tie | P2 | none | keep | exemplars | keep |
| Frontend→backend | RUM stitch `ui.click`→checkout (`inv:L311`) | session→trace waterfall stitch | Sentry / HyperDX replay | Sentry session-linked traces deeper | P1 | session join | session lane | RUM stitch | adopt |
| CLI→backend | TRACEPARENT injection + `otlp-forward` compare (`inv:L87`) | — | Parallax | unique: CLI run as first-class trace root | — | none | keep | CLI run/cron | keep |
| macOS→backend | none | mobile SDK trace propagation + crash↔trace join | Sentry 26.8.0 (Apple SDKs) | Parallax has no native client SDK story | P1 | native SDK / OTel Swift path | client span lane | none (workstream F) | adopt |
| Sampling/tail sampling | none (head ingest) | Refinery adaptive tail sampling, donated to OTel (2026-09) | Honeycomb | cost+completeness control at volume | P2 | sampling policy + rate attribution | sampling controls | sampling gap (b-series) | adopt |

## D. Metrics (GOAL §4)

| Workflow / capability | Parallax today | Best implementation found | Product | Why it is better | P0/P1/P2/Reject | Backend gap | UI gap | Playground coverage | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Metric discovery/catalog | catalog + point counts (`09-12:L131`) | Metrics Explorer w/ metadata + type + unit | Grafana Mimir 3.2.1 / Datadog | metadata-rich browse | P1 | metric metadata store | catalog polish | 12 metrics listed | adopt |
| Dimensions/group-by | group-by + step (`inv:L121`) | multi-dim group + label autocomplete | Prometheus 3.14.0 / Mimir | PromQL label power | P1 | label autocomplete index | label UX | group-by | adopt |
| Rate/delta semantics | typed kind-legal aggs; rate/delta partial | `rate()`/`delta()`/`increase()` + counter-reset handling | Prometheus 3.14.0 / Mimir 3.2.1 | correct counter math is table stakes | P0 | counter-reset-safe rate/delta fns | fn picker | rate semantics TBD | adopt |
| Counters/gauges/histograms | histogram quantile (`inv:L102`); exp histogram unclear | native histograms + exp buckets + quantile | Prometheus 3.14.0 / Mimir | exp histograms halve cost at precision | P1 | exp-histogram ingest+store | quantile viz | histogram quantile | adopt |
| Percentile viz | p50/p95/p99 shipped | percentile bands + multi-quantile overlays | Datadog / Grafana | band viz reads variance faster | P2 | none | band render | p95 latency | keep |
| Query power | per-metric workbench, not analysis surface (`09-12:L131`) | PromQL explore + functions + recording rules | Grafana/Prometheus 3.14.0 | full language + alerting integration | P0 | **decision: PromQL-subset vs Grafana-embed** (do not rival blindly) | workbench depth or embed | workbench | adopt |
| Grouping/comparison | attribute compare (traces); metric compare partial | timeshift overlays + multi-query compare | Grafana / Datadog | "vs last week" in one click | P1 | timeshift fn | compare control | none | adopt |
| Exemplar links | dots + trace click-through (`inv:L117`) | exemplar on every panel + trace jump | Grafana / Parallax | tie | P2 | none | keep | exemplars | keep |
| Metric→trace/log/error | exemplar→trace; RED→services | metric spike → related traces/logs auto-surfaced | Datadog / New Relic | correlation without manual pivot | P1 | spike-window trace lookup | spike action | none | adopt |
| RED/service/runtime/DB metrics | services RED + runtime snapshot (`inv:L122`) | APM auto-instrumentation metric sets + host maps | Datadog / New Relic / Coroot v1.26.0 | breadth of out-of-box sets | P1 | metric set breadth | auto-panels | tokio/jvm lanes | adopt |
| Frontend Web Vitals | web-vitals emitted (playground web); no dedicated UX | Web Vitals dashboard + RUM performance view | Sentry / Datadog RUM / Better Stack | vitals need their own lens, not generic charts | P1 | vitals rollup | vitals view | web-vitals emitted | adopt |
| Cardinality visibility | field stats (`inv:L107`) | cardinality explorer + top-k offenders + limits | Mimir 3.2.1 / Honeycomb | prevents cardinality death with names | P1 | per-label cardinality accounting | cardinality panel | cardinality stress | adopt |
| Missing-data behavior | unclear | explicit null/gap policies + no-data alerts | Grafana / SigNoz | honest gaps vs fake zeros | P2 | gap policy | gap render | none | adopt |
| Dashboard panels | widget grid (`inv:L128`); Grafana-grade depth absent (`09-12:L139`) | panels + variables + alerting integration | Grafana 13.2.1 | industry reference; not a contest | P1 | variable support | panel depth or **Grafana-embed path** | `product:saved_state` | non-rivalry |
| Saved queries/defaults | saved views + snippets (`inv:L108,130`) | metric saved queries + default service dashboards | SigNoz / Datadog | zero-click starting views | P1 | default dashboard set | defaults | `product:saved_state` | adopt |

## E. Adjacent capabilities (GOAL §5)

| Workflow / capability | Parallax today | Best implementation found | Product | Why it is better | P0/P1/P2/Reject | Backend gap | UI gap | Playground coverage | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Dashboards | gallery + widget grid, brush-to-zoom (`inv:L128,114`) | dashboards + variables + folders + alerting | Grafana 13.2.1 | reference depth; embed beats rival | P1 | variables | builder depth or embed | `product:saved_state` | non-rivalry |
| Alerting rules | error_rate/p95/p99/throughput/log_count/metric + hysteresis (`inv:L137`) | alert-from-any-query + multi-condition + anomaly | SigNoz v0.141.1 | rule creation at point of insight | P0 | anomaly conditions | alert-from-query | `product:alerting` | adopt |
| Alert channels | webhook + Slack; email deferred; module preliminary (`inv:L142-144`) | 10 channel kinds incl. email/PagerDuty/Opsgenie | SigNoz v0.141.1 | on-call reach; Parallax cannot page | P0 | email + 2-3 core channels | destination UX | `product:alerting` | adopt |
| Incidents | incidents + bundle hash, unique (`09-12:L133`) | incident timeline + tasks + postmortem | Datadog / Sentry / Better Stack | collaboration depth (timeline, tasks) | P1 | incident activity model | timeline UX | `product:alerting` | keep |
| On-call rotations | none | rotations + escalations + schedules | Better Stack / PagerDuty-class | Parallax stops at incident creation | P2 | rotation model | schedule UX | none | adopt |
| SLOs/error budgets | none (`inv:L164`) | SLO + burn-rate alerts + budget policy | Datadog / Grafana / Sentry | burn-rate is the alerting unit teams use | P1 | SLI/SLO store + burn calc | SLO view | none | adopt |
| Service catalog | heat catalog + RED + runtime (`inv:L123`) | catalog + entity graph + ownership + docs | New Relic / Datadog / Backstage-class | ownership+docs turn catalog into map | P1 | ownership metadata | owner/docs fields | heat catalog | adopt |
| Service/dependency maps | typed six-kind graph with system labels; UI preserves kinds; edge p50/p95/error rate, log-scaled width, low/medium/high dash flow, and investigation legend (HEAD `goal/next-investigation-slice`) | live map + legend + red-node highlight (slickest) | HyperDX 2.38.0 BETA | render parity reached; HyperDX keeps slicker live interaction | P1 | none (model deeper) | edge-scoped issue/log/metric pane; interactive edge polish | `ecosystem:service_map` | keep |
| Deploy/release markers | release strip + GitHub deploy ingest (`inv:L46`) | deploy markers on every chart + release compare | Sentry / Datadog / Grafana annotations | markers where eyes already are | P0 | annotation store | markers on all charts | a13, `product:github_ingest` | adopt |
| CI/test context | JUnit/nextest + flaky + explorer (`inv:L146`) | CI Test Optimization + flaky quarantine | Datadog / Currents-class | quarantine + owner routing | P1 | quarantine state | quarantine UX | flaky detection | keep |
| Database monitoring | derived nodes + wrapper-span conventions | query stats + explain + pool pressure | Datadog DBM / New Relic | query-level, not span-level | P1 | query aggregation | query view | Postgres pathologies | adopt |
| Infra/runtime telemetry | runtime snapshot tokio/jvm (`inv:L123`) | host/container/K8s maps + eBPF network | Datadog / Coroot v1.26.0 | Coroot eBPF zero-instrument breadth | P1 | host inventory model | infra view | tokio saturation | adopt |
| Continuous profiling | none (`inv:L164`) | eBPF always-on + flame + diff | Pyroscope 2.3.1 / Coroot / Datadog Profiler | profiles explain the "why slow" traces can't | P1 | OTLP profiles ingest + store | flame + diff | none | adopt |
| Frontend RUM | web-vitals + session.id + SSR traceparent; projections CLI-shaped (`inv:L166`) | RUM sessions + Web Vitals + frustration signals | Sentry / Datadog RUM / Better Stack | real session product vs projections | P0 | session model + vitals rollup | session view | RUM journey/scenarios | adopt |
| Browser/network requests | RUM stitch only | resource waterfall per page view + network errors | Sentry / Datadog RUM | per-view network truth | P1 | resource timing ingest | resource table | RUM journey | adopt |
| Session replay | none | OSS replay + error-linked replay | HyperDX 2.38.0 / PostHog (Highlight dead) | see-what-user-saw; no substitute | P2 | replay ingest + store (heavy) | player | none | adopt |
| Uptime checks | none | monitors + status pages + multi-region | Better Stack / Sentry Monitors | uptime is entry-level obs; Parallax blind | P1 | check runner + results | monitors view | none | adopt |
| Synthetics | none | scripted browser/API tests | Datadog / New Relic / Better Stack | proactive vs reactive | P2 | script runner | results view | none | adopt |
| Anomaly detection | none | Watchdog/auto-anomaly + seasonal baselines | Datadog / Dynatrace Davis | no-threshold alerting | P2 | baseline engine | anomaly badges | none | adopt |
| Log/metric pattern detection | Drain log patterns; metric none | pattern-insights + metric anomaly correlation | Datadog | patterns wired to alerts+metrics | P1 | metric pattern scan | pattern alerts | patterns view | adopt |
| Query history | SQL history; trace/log history partial | full recent-history + starred | SigNoz / Grafana | recall beats rebuild | P2 | history store | history UX | `product:saved_state` | adopt |
| Sharing/permalinks | URL-driven filters everywhere (`inv:L131`) | — | Parallax | share-by-URL is reference-grade | — | none | keep | all routes | keep |
| Annotations | release strip only; chart annotations absent | annotations on every chart + API | Grafana / Datadog | deploys/incidents visible in data | P0 | annotation store + API | annotation render | none | adopt |
| Feature-flag correlation | playground flagd + flip scenarios; product join unclear | flag evals on timeline + flag-triggered rollback | Sentry / Honeycomb / Better Stack | "flag flip caused it" in one view | P2 | flag-eval ingest join | flag lane | flag flip | adopt |
| Cost/cardinality visibility | field stats; self-host no-metering (thesis) | usage metering + cardinality guardrails | Mimir / Honeycomb / Datadog | metering irrelevant self-host; guardrails matter | P2 | cardinality guardrails (no billing) | guardrail UX | cardinality stress | adopt |
| Retention controls | TTLs + pin-aware prune (`inv:L63-64`) | tiered retention + downsampling + archive | Grafana / Elastic / OpenObserve | cold-tier economics at scale | P1 | object-store tier + downsample | retention UX | `product:lifecycle_ops` | adopt |
| Pipeline health | /health 503 degradation + self-OTLP (`inv:L51,153`) | pipeline observability + lag/drop dashboards | Mezmo / Datadog Pipelines | per-pipeline-stage truth | P1 | stage-level counters | health view | ingest health | adopt |
| Dropped-data diagnostics | loss counters (`README` durability); partial diagnostics | dropped-span accounting + reason codes | OTel Collector / Honeycomb Refinery | named reasons, not silent gaps | P1 | drop reason codes | drop panel | sampling gap | adopt |
| eBPF zero-instrumentation | none | eBPF auto-instrument → any backend | Odigos v1.36.0 / Coroot v1.26.0 | zero-code-change adoption | P2 | none (integrate, don't build) | source badges | none | watch |
| Sentry SDK compatibility | envelope ingest shipped; multi-SDK ledger unproven (`inv:L172`) | native protocol | Sentry 26.8.0 / rustrak v0.14.12 (dual-auth) | compat breadth decides migration | P0 | SDK matrix harness + dual-auth parity | compat ledger | `sentry:envelopes` (rust/java/js) | adopt |
| MCP/agent breadth | 2 read-only tools (`inv:L77`) | 41+ tools (SigNoz) / 140+ (OO EE) / 7 read-only (TMA1) | SigNoz MCP v0.14.0 | breadth vs safety: Parallax safest, narrowest | P1 | more read-only tools (trace/log/metric/evidence) | none (API surface) | MCP checks | adopt |
| AI triage/RCA | none (thesis: context engine, not fixer) | Seer Autofix / Auto-investigations / Davis / Bits SRE GA | Sentry / Honeycomb / Dynatrace / Datadog | incumbents ship investigators; Parallax feeds them | P1 | bundle quality (A1), not a fixer | "send to agent" depth | none | non-rivalry |
| Status pages | none | hosted status pages + subscribers | Better Stack | comms product, not debugging | Reject | — | — | — | — |
| Compliance certs (SOC2/HIPAA) | none | best-in-class attestations | Datadog | enterprise procurement gate, not dev UX | Reject | — | — | — | — |
| Distributed/HA/multi-region | single binary (thesis) | sharded multi-region SaaS | Datadog / Grafana Cloud | scale Parallax explicitly defers | Reject | — | — | — | — |
| Built-in fixer LLM | none (thesis) | HolmesGPT / Causely / Bits SRE | fixer layer | Parallax feeds fixers via bundle+MCP | Reject | — | — | — | — |
| SSO/RBAC/multi-tenancy | planned (`ledger:L86`) | SSO/SAML/OIDC + fine RBAC | Datadog / Sentry / SigNoz EE | team-adoption gate | P1 | authZ model | login/role UX | none | adopt |

## F. P0 rollup (credible-replacement blockers)

1. Triage lifecycle: regressed state, assignment/ownership, release-health attribution (Sentry ref).
2. Source maps for browser stacks (Sentry/Honeycomb FEO ref).
3. RUM sessions + Web Vitals UX (not projections).
4. Alert-from-any-query + email/core channels.
5. Deploy markers + chart annotations everywhere.
6. SigNoz-class explorer breadth: funnels, result views, metric compare.
7. PromQL decision: subset implementation or Grafana-embed (explicit, no drift).
8. Rate/delta counter-correct semantics.
9. Sentry multi-SDK compatibility ledger proven.
10. High-volume usability: sampling policy + stream guardrails.

## G. Differentiators to defend (GOAL §13)

Attribute-compare · hash-pinned evidence bundles · read-only SQL console ·
CLI runs as first-class traces · investigations case files · MCP read-only safety ·
URL-shareability · single-binary self-host · Apache-2.0 · Sentry+OTLP dual ingest ·
Rust-first GreptimeDB+Turso stack. None of the P0–P2 adoptions may regress these.
