# Competitor Pain Points — User-Voice Evidence Corpus

Research date: 2026-08-14. Method: four parallel research passes over primary
user-voice sources (HN threads via Algolia, Reddit via PullPush, GitHub
issues ranked by reactions/comments, G2/Capterra aggregate themes, dated
practitioner surveys), plus an internal cross-check against
[competitors/](competitors/README.md) (which, before this file, contained
**zero** user-voice sources — all competitor negatives were
analyst-derived). This corpus is the base layer for pain-point-driven
Parallax improvements; the resulting work items are `plans/173`–`176` plus
recorded-direction notes below.

Bias note: several cost figures come from alternative-vendor blogs
(OneUptime, SigNoz, CubeAPM, Parseable, Bugsink) — commercially motivated;
they are retained only where directionally consistent with primary HN/
GitHub/review evidence, and flagged. Repo rename observed during research:
Maple is now `MapleTechLabs/maple` (was `Makisuo/maple`; redirects).

## The two structural roots

Nearly every recurring complaint traces to one of two enabling conditions:

1. **Per-pillar storage engines with per-unit pricing** → three-pillars
   fragmentation and 5× duplicate storage cost (Charity Majors 2024-02;
   Grafana surveys 2025/2026: complexity #1 obstacle at 39%/38%),
   cardinality anxiety (engineers self-censoring labels), sampling guilt
   (the discarded trace is the one you needed), query-language
   proliferation (PromQL+LogQL+TraceQL), local-dev observability gap.
2. **Telemetry shaped for human eyeballs, with no bounded/joinable/redacted
   evidence primitive** → alert fatigue (#1 obstacle to faster incident
   response two Grafana surveys running; 67% of engineers admit dismissing
   alerts uninvestigated, incident.io 2025), agents unable to reach
   production context (Datadog built token-trimming into its MCP server
   because raw telemetry blows context windows), deploy correlation done by
   hand (~70% of outages are change-triggered, Google SRE), LLM
   secret-leak fear (OWASP LLM02), dashboard graveyards, test-flakiness
   blindness.

Parallax's architecture (single binary, one store + one SQL surface,
bounded redacted evidence bundles, agent-first) attacks both roots
directly. The pains below are therefore mostly *validation* of the shape —
and the improvement plans target the places where Parallax has the
machinery but does not yet deliver the answer.

## Cohort 1 — Sentry + lightweight alternatives (top evidence)

| Pain | Evidence (primary) | Signal |
| --- | --- | --- |
| Self-host ops burden: 20–40 containers, Kafka/ClickHouse/Snuba, 16GB min | [HN "I gave up on self-hosted Sentry"](https://news.ycombinator.com/item?id=43725815) 2025-04, 186pts/150c; founder admission ["100% a valid complaint"](https://news.ycombinator.com/item?id=43730831); [r/selfhosted 74pts](https://reddit.com/r/selfhosted/comments/1jf5st7/why_i_gave_up_on_selfhosted_sentry/) | very high |
| Upgrade fragility: top-reacted issues in getsentry/self-hosted are migration failures 2020→2026 | [self-hosted#4286](https://github.com/getsentry/self-hosted/issues/4286), [#3346](https://github.com/getsentry/self-hosted/issues/3346), [#468](https://github.com/getsentry/self-hosted/issues/468), [#1585 ARM64 41👍](https://github.com/getsentry/self-hosted/issues/1585) | high |
| Quota bill explosions + **silent event dropping at quota, unrecoverable** | [HN 48851745](https://news.ycombinator.com/item?id=48851745) 2026-07; [Sentry help center documents drop-forever](https://sentry.zendesk.com/hc/en-us/articles/25328193800731) | high |
| SDK weight + runtime overhead ("package bigger than React" 144👍; Python init p50 2→40ms; PHP +10–100ms/req) | [sentry-javascript#2707](https://github.com/getsentry/sentry-javascript/issues/2707); [sentry-python#2116](https://github.com/getsentry/sentry-python/issues/2116) | very high |
| Source-map/Debug-ID friction (dozens of issues; dedicated troubleshooting doc) | [sentry-cli#2126](https://github.com/getsentry/sentry-cli/issues/2126), [sentry-javascript#19213](https://github.com/getsentry/sentry-javascript/issues/19213) | high |
| Issue grouping opaque: over/under-grouping, `<uuid>` defeats manual fingerprints | [sentry#64354](https://github.com/getsentry/sentry/issues/64354), [#71630](https://github.com/getsentry/sentry/issues/71630), [discussion 66319](https://github.com/getsentry/sentry/discussions/66319) | high recurrence |
| Alert/notification fatigue (2016→2026; vendor's own blog concedes) | [sentry#2673 25👍](https://github.com/getsentry/sentry/issues/2673); [Sentry blog](https://blog.sentry.io/top-3-issue-alert-tips-to-stop-noisy-notifications/) | high |
| License distrust BSL→FSL | [HN 2019 348pts](https://news.ycombinator.com/item?id=21466967); [HN 2023 75pts](https://news.ycombinator.com/item?id=38306320) | high at events |
| EU data-sovereignty demand; 90-day SaaS retention cap; relocation moves config only | [r/BuyFromEU 65pts](https://reddit.com/r/BuyFromEU/comments/1kn7plt/sentry_alternative_in_the_european_union/) 2025-05; [migration docs](https://docs.sentry.io/concepts/migration/) | rising |
| Alternatives fragile too: Highlight killed 2026-02-28 (LaunchDarkly); GlitchTip UI bugs, tracing unowned; PostHog mobile-error gaps | [HN 43774155](https://news.ycombinator.com/item?id=43774155); [HN 43728899](https://news.ycombinator.com/item?id=43728899); [posthog#31117 76👍](https://github.com/PostHog/posthog/issues/31117) | definitive |

Thin flags: 3.2× bill-spike and replay-overcount figures (CubeAPM only);
GlitchTip "may remove tracing" (competitor-authored only).

## Cohort 2 — Incumbents (Datadog / New Relic / Dynatrace / Splunk / Grafana Cloud)

| Pain | Evidence (primary) | Signal |
| --- | --- | --- |
| Cardinality/custom-metric bill shock ($65M Coinbase; one `product_id` label ≈ $997K/mo modeled) | [HN 35837330](https://news.ycombinator.com/item?id=35837330) 213c; [HN 44426399](https://news.ycombinator.com/item?id=44426399); [OneUptime teardown](https://oneuptime.com/blog/post/2026-03-13-how-datadog-pricing-actually-works/view) (vendor blog, flagged) | critical |
| Ingest-metered pricing ruinous (Splunk 600GB/day >$1M/yr; NR $900→$8,000 one month) | [CloudZero](https://www.cloudzero.com/blog/splunk-cost-optimization/); [SigNoz NR guide](https://signoz.io/guides/new-relic-pricing/) (vendor blogs, directionally corroborated on HN) | critical |
| Cost-forced sampling → the error trace is lost, MTTR up | [Pedro Dias APM costs](https://itspedrodias.com/posts/datadogapmcost/); [groundcover sampling guide](https://www.groundcover.com/guides/what-is-sampling-in-observability) | high |
| Lock-in: proprietary agents/SPL/DQL; "Datadog snakes its way far into your codebase" | [HN 44426399](https://news.ycombinator.com/item?id=44426399); [Parseable Splunk alternatives](https://www.parseable.com/blog/splunk-alternatives) | high |
| Rehydration tax: searching your own archives costs full indexing again | [Datadog docs](https://docs.datadoghq.com/logs/log_configuration/rehydrating/) | high |
| LGTM self-assembly: 4–5 stateful systems, 3 query languages, Loki queries timing out | [GH loki#19134](https://github.com/grafana/loki/issues/19134); [CubeAPM LGTM comparison](https://cubeapm.com/blog/clickstack-vs-grafana-lgtm-stack/) (flagged) | high |
| No true self-host from incumbents (Datadog CloudPrem = logs only) | [CubeAPM FAQ](https://cubeapm.com/faqs/datadog-self-hosted-alternatives/) (flagged, structurally verifiable) | structural |
| Agent resource overhead causing outages | [datadog-agent#3793](https://github.com/DataDog/datadog-agent/issues/3793), [dd-trace-dotnet#634](https://github.com/DataDog/dd-trace-dotnet/issues/634) | recurring |

## Cohort 3 — OSS challengers (the rough edges Parallax must not repeat)

| Pain | Worst offenders + evidence | Signal |
| --- | --- | --- |
| Backing-store ops burden (forced ClickHouse cluster + 800MB idle ZooKeeper single-node; Mongo dependency) | SigNoz [#8784](https://github.com/SigNoz/signoz/issues/8784), [#7002](https://github.com/SigNoz/signoz/issues/7002); HyperDX [#1037](https://github.com/hyperdxio/hyperdx/issues/1037); [HN 45293788](https://news.ycombinator.com/item?id=45293788) | high |
| **Data loss / corruption** (compact-merge metadata loss, corrupt Parquet footers, dropped fields; destructive local-store recovery) | OpenObserve [#732](https://github.com/openobserve/openobserve/issues/732), [#8112](https://github.com/openobserve/openobserve/issues/8112), [#5082](https://github.com/openobserve/openobserve/issues/5082); Maple [#113](https://github.com/MapleTechLabs/maple/issues/113), [#297](https://github.com/MapleTechLabs/maple/issues/297) | critical |
| Resource hunger / OOM (collector >9GB, OOMKills, UI-open saturates ClickHouse) | SigNoz [#6128](https://github.com/SigNoz/signoz/issues/6128), [#9306](https://github.com/SigNoz/signoz/issues/9306), [#10590](https://github.com/SigNoz/signoz/issues/10590); Coroot [#18](https://github.com/coroot/coroot/issues/18) | high |
| Upgrade breakage (dashboards broken, schema-version mismatch blocks start, no lossless local upgrade) | SigNoz [#6304](https://github.com/SigNoz/signoz/issues/6304); OpenObserve [#11099](https://github.com/openobserve/openobserve/issues/11099), [#6826](https://github.com/openobserve/openobserve/issues/6826); Uptrace [#551](https://github.com/uptrace/uptrace/issues/551) | high |
| Auth/SSO/RBAC gated or absent in OSS tier (OIDC top-begged everywhere; SigNoz #1188 open 4 years) | SigNoz [#1188 26👍](https://github.com/SigNoz/signoz/issues/1188); HyperDX [#1140](https://github.com/hyperdxio/hyperdx/issues/1140); OpenObserve [#5373 35c](https://github.com/openobserve/openobserve/issues/5373) | high |
| Alerting unreliability (re-notify spam every evaluation; email alert bugs) | HyperDX [#2464](https://github.com/hyperdxio/hyperdx/issues/2464); OpenObserve [#6555](https://github.com/openobserve/openobserve/issues/6555) | med-high |
| **Silent metric-math bugs** (cumulative→delta ~5× rate inflation; stock dashboards using rate-of-sum) | Uptrace [#609](https://github.com/uptrace/uptrace/issues/609), [#605](https://github.com/uptrace/uptrace/issues/605) | correctness-class |
| GitOps/declarative config absent (UI-only config begged across projects) | SigNoz [#7964](https://github.com/SigNoz/signoz/issues/7964); Coroot [#489](https://github.com/coroot/coroot/issues/489); HyperDX [#1329](https://github.com/hyperdxio/hyperdx/issues/1329) | medium |
| License friction (AGPL "matters in some shops"; FSL) | [HN 44247020](https://news.ycombinator.com/item?id=44247020) | medium |

Thin flags: TMA1 (5 issues total, no social corpus); Maple social presence
(tracker-only); Coroot complaint corpus is tracker-only.

## Cohort 4 — Practitioner themes (vendor-independent), ranked

1. Three-pillars fragmentation + 5× duplicate cost — Charity Majors
   [Cost Crisis](https://charity.wtf/2024/02/09/the-cost-crisis-in-observability-tooling/);
   [Grafana surveys 2025/2026](https://grafana.com/observability-survey/2025/).
2. Alert fatigue — #1 obstacle both Grafana surveys; 67% dismiss alerts
   ([incident.io 2025](https://incident.io/blog/alert-fatigue-solutions-for-dev-ops-teams-in-2025-what-works));
   2–5% of ~50 alerts/week actionable (PagerDuty 2025).
3. OTel onboarding complexity — [HN "why was it so complicated"](https://news.ycombinator.com/item?id=42655102);
   OTel's own [stabilization proposal](https://opentelemetry.io/blog/2025/stability-proposal-announcement/).
4. Agents can't reach production context — Datadog MCP token-trimming;
   [Greptime: agents becoming primary queriers](https://www.greptime.com/blogs/2026-08-11-observability-three-pillars-history) 2026-08.
5. Cardinality anxiety (self-censored labels) — [Prometheus cardinality posts](https://kaidalov.com/posts/2025/09/prometheus-optimization/).
6. Sampling guilt — head sampling discards the trace you needed
   ([groundcover](https://www.groundcover.com/guides/what-is-sampling-in-observability); [Elastic labs](https://www.elastic.co/observability-labs/blog/how-we-fixed-head-based-sampling-in-opentelemetry)).
7. Query-language proliferation — [urgentry comparison](https://urgentry.com/guides/observability/discover-query-languages/).
8. Deploy correlation by hand — ~70% of outages change-triggered
   ([Google SRE](https://sre.google/workbook/canarying-releases/); New Relic change-tracking posts).
9. Secret/PII fear piping telemetry to LLMs — OWASP LLM02; whole
   pre-LLM-redaction product category (Kong/Arthur/Prediction Guard).
10. Dashboard graveyards vs investigation workflows — Grafana 2026: 91–92%
    want AI investigation, not more panels.
11. Local-dev observability gap — Aspire-dashboard demand proves it.
12. Test-flakiness blindness — flaky-encounter rate 10%→26% 2022→2025;
    reruns destroy the evidence.

## Mapping: pain → Parallax state → action

| Pain (root) | Parallax today (verified) | Action |
| --- | --- | --- |
| Alert fatigue; alerts carry questions not answers (root 2) | Alerting ships webhook/Slack payloads (`crates/parallax-server/src/alerting/delivery.rs:72,114`) with rule/state fields; **no evidence bundle attached at fire time** — the bundle machinery (anchors, hypotheses, `deploy_adjacency`) exists but alerting never invokes it | **plans/173** evidence-carrying alerts |
| "What changed?" hunted by hand (root 2) | Bundles already carry linkage-only `deploy_adjacency` statements (`crates/parallax-evidence/src/bundle/assembly.rs:32`); GitHub deploy/CI ingest shipped but **disabled by default**; alert payloads and issue UI don't surface change adjacency | **plans/173** (bundle-in-alert carries it); enable-posture note below |
| Upgrade breakage industry-wide; competitors' top-reacted issues are migration failures | `PRAGMA user_version` + fail-closed future schema; preview-harness + always-run lossless tests (`docs/guide/upgrade-and-durability.md`) | **plans/174** delivered |
| Silent event dropping at quota (Sentry); data loss/corruption (OpenObserve/Maple) | `/ingest/loss` counters + `/health` degrade on queue-full / terminal drop / spool fail | **plans/174** delivered |
| Resource hunger / idle footprint (SigNoz ZK 800MB idle; 9GB collectors) | Measured 2026-08-13: idle ~24 MiB Parallax + ~139 MiB Greptime (`docs/guide/footprint.md`); CI warn-only until 2026-08-28 | **plans/175** delivered |
| Grouping opacity (Sentry over/under-grouping, fingerprint fights) | Deterministic fingerprinting (`crates/parallax-analysis/src/fingerprint.rs:122`) — but the UI/CLI never EXPLAIN why events grouped, and there is no user-controlled re-fingerprint rule surface | **plans/176** grouping transparency |
| Sampling guilt (root 1) | Local full retention within TTL windows; evidence pins survive TTL | Covered: verified via playground (plans 164/165); market it |
| Cardinality anxiety (root 1) | GreptimeDB columnar native tables; no per-series billing | Covered structurally; benchmark program owns proof |
| OTel onboarding complexity; local-dev gap | Single binary local-first; onboarding snippets = plan 171 feature 4 + plan 172 empty states | Covered by existing plans |
| Query-language proliferation | One SQL surface (`sql` query + workbench) | Covered; market it |
| LLM secret-leak fear | redaction-lite-v3 at bundle build, inside the binary; egress canary = plan 164 c10; detector tests = plan 168 | Covered by existing plans |
| Alert-delivery re-notify spam (HyperDX) | Renotify interval + hysteresis exist in the rule model; **delivery dedup proven only single-actor** — concurrency test = plan 168 Step 10 | Covered by 168 |
| Silent metric-math bugs (Uptrace 5× inflation) | Kind-legal aggregation gating in `metricQuery`; exp-histogram silent drop is Parallax's own instance of this class | Fix owner: plan 166 (decision) + 168 Step 6 characterization |
| OSS-tier auth gating (OIDC begged everywhere) | V2 auth contract decision exists; V1 is local-loopback | Recorded direction: when V2 server profile ships, OIDC + basic RBAC belong in the free self-host tier — competitors' most-repeated ask; owner = `decisions/v2-auth-and-context-contract.md` |
| GitOps/everything-as-config | Dashboards/alerts are API/UI state in Turso | Recorded direction: declarative export/import fits agent-first; revisit after plans 173/176 land (their contracts shape the config schema) |
| Test-flakiness blindness | Test reporting + flaky detection shipped — **a differentiator none of the researched competitors has** | Covered; market it |
| License friction (AGPL/FSL/BSL churn) | Whole repo Apache-2.0 | Covered; market it loudly |

## Enable-posture note (correctness, not ROI)

Sentry envelope ingest and GitHub deploy/CI ingest are shipped but disabled
by default. Given the corpus (deploy correlation = the first question in
~70% of incidents; Sentry-compat = the migration wedge), the default-off
posture deserves a deliberate decision: local-first V1 has no auth, so
default-on webhooks are wrong for exposed hosts, but the *quickstart path*
should make enabling them a one-line documented step, and the evidence
bundle should treat absent deploy data as `missing_evidence` (it already
models gaps). No plan needed; recorded for the V2 auth/profile decisions.

## What this corpus deliberately does not conclude

- No benchmark claims (footprint/perf numbers become claims only via
  plans/175 and the existing benchmark program).
- No new roster additions; the do-not-repropose list in `plans/README.md`
  stands.
- A1 (bundle value vs raw context) remains the open existential gate — this
  corpus strengthens the *demand* signal (Datadog token-trimming, Grafana
  91–92% AI-investigation demand) but proves nothing about bundle efficacy.
