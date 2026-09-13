# Competitor version pins — 2026-09-13 pass 68 (Workstream A restamp)

Fresh primary-source sweep for GOAL.md §3. OSS rows from GitHub Releases API
**2026-09-13T12:32Z**. Image index digests from Docker Hub v2 **same hour**.
SaaS rows: continuous + last dated first-party item checked. Earlier same-day
pin file (pass 67, HEAD `de5592ae`) is a hypothesis this pass restamps.

**Do not treat 2026-07 notes or `artifacts/research/` screenshots as current.**

## Baseline SHAs (GOAL.md §1)

| Repo | Working HEAD | origin/main (plan baseline) | Note |
| --- | --- | --- | --- |
| parallax | `1cad2a509cfaf76a10d29d320559131584943201` (`goal/final-p0-hotfix`) | `cb9a3077783cfc8f15ca356e7d8bf8d3360d371e` | P0 work landed on this branch; inspect crates/UI as truth |
| parallax-telemetry-playground | `174a7d6471284f1b0fc2b2cfb67bb4719f111f0b` | `a23b0d84aea76144cd33b4602645086e4c2f62bc` | macOS harness + `product:issue_context` service-scoped |

Sweep timestamp: **2026-09-13T12:35:00Z**.

## Pinned versions

| Product | Latest stable | Date | Primary source | Image digest (index, when Hub published) | Drift vs pass 67 |
| --- | --- | --- | --- | --- | --- |
| Sentry self-hosted | `26.8.0` | 2026-08-17 | [getsentry/self-hosted 26.8.0](https://github.com/getsentry/self-hosted/releases/tag/26.8.0) | compose-pinned in lab; GH release has no image digest | none |
| Sentry SaaS | continuous | — | sentry.io (no version) | — | — |
| Datadog platform | SaaS continuous; DASH 2026 Bits Detection/Remediation/Code/Release (many Preview); Bits AI SRE GA since 2025-12 | 2026-06-09 | [DASH 2026 roundup](https://www.datadoghq.com/blog/dash-2026-new-feature-roundup-keynote/) | — | none (SaaS) |
| Datadog Agent (proxy) | `7.83.1` | 2026-09-09 | [DataDog/datadog-agent 7.83.1](https://github.com/DataDog/datadog-agent/releases/tag/7.83.1) | — | none |
| Grafana | `v13.2.1` | 2026-09-02 | [grafana/grafana v13.2.1](https://github.com/grafana/grafana/releases/tag/v13.2.1) | `grafana/grafana:13.2.1` `sha256:f772d434e8fab0049deb2b1b30abd43342bcfca1537614aa8d36080232cf4283` | none |
| Tempo | `v3.0.3` | 2026-08-13 | [grafana/tempo v3.0.3](https://github.com/grafana/tempo/releases/tag/v3.0.3) | `grafana/tempo:3.0.3` `sha256:0296560ac66f8a3600d7fb3014a52c189d4d9c3549ad6ff441bf2409855d68d5` | none |
| Loki | `v3.7.7` | 2026-08-27 | [grafana/loki v3.7.7](https://github.com/grafana/loki/releases/tag/v3.7.7) | `grafana/loki:3.7.7` `sha256:d70e4659623f3e109af669cae76fe2a5dd5be54e2298fe8aed380d982fbc2500` | none |
| Mimir | `3.2.1` (`mimir-3.2.1`) | 2026-09-10 | [grafana/mimir mimir-3.2.1](https://github.com/grafana/mimir/releases/tag/mimir-3.2.1) | `grafana/mimir:3.2.1` `sha256:92838f113ba54230014e79bc812e57ca90bb9ebc06e665f59e4f700098c2dd04` | none |
| Pyroscope | `v2.3.1` | 2026-09-08 | [grafana/pyroscope v2.3.1](https://github.com/grafana/pyroscope/releases/tag/v2.3.1) | `grafana/pyroscope:2.3.1` `sha256:86a9ee7448487409ead8ada78789de7b78b739711b92b5d7224a1a54abf3eeb2` | none |
| otel-lgtm | `0.33.0` (tag `v0.33.0`) | 2026-09-11 | [grafana/docker-otel-lgtm v0.33.0](https://github.com/grafana/docker-otel-lgtm/releases/tag/v0.33.0) | `grafana/otel-lgtm:0.33.0` `sha256:475319e883b66594d1a2f22ef168c2459802bb94548e6f25d9782bd5f5c19a3a` | none; lab compose uses this tag |
| SigNoz | `v0.141.1` | 2026-09-09 | [SigNoz/signoz v0.141.1](https://github.com/SigNoz/signoz/releases/tag/v0.141.1) | `signoz/signoz:v0.141.1` `sha256:c10fa03e103c76bba2d67452bd26925f08a69f2a7b8d0f7982cea0e4f81ce88e` | none. **Lab drift:** `compose.signoz.override.yml` still pins `v0.140.0@sha256:7969e02e…`; Foundry compose is `v0.141.1` un-digested |
| SigNoz collector | `v0.144.9` | 2026-08-19 | [SigNoz/signoz-otel-collector v0.144.9](https://github.com/SigNoz/signoz-otel-collector/releases/tag/v0.144.9) | `signoz/signoz-otel-collector:v0.144.9` `sha256:72aa1e4c1ec529f178e962c049be35cfd7abae02fe9a397edad10b4a9cba62fa` | none (matches lab override) |
| SigNoz MCP | `v0.14.0` | 2026-09-02 | [SigNoz/signoz-mcp-server v0.14.0](https://github.com/SigNoz/signoz-mcp-server/releases/tag/v0.14.0) | — | none |
| SigNoz Foundry | `v0.2.17` | 2026-07-29 | [SigNoz/foundry v0.2.17](https://github.com/SigNoz/foundry/releases/tag/v0.2.17) | — | none |
| OpenObserve | `v1.0.0` GA | 2026-09-11 | [openobserve/openobserve v1.0.0](https://github.com/openobserve/openobserve/releases/tag/v1.0.0) | `openobserve/openobserve:v1.0.0` `sha256:d581789cb03b5f061ed56a3e864b5e4bbc86bbf6d317ec863372e98ee49b30d5` | none |
| Honeycomb | SaaS continuous; Agent Timeline GA 2026-06-18; Canvas GA May 2026; Pro plan change 2026-07-01 | 2026-06-18 / 2026-07-01 | [Agent Timeline GA](https://www.honeycomb.io/blog/agent-timeline-generally-available), [2026 Pro plan](https://docs.honeycomb.io/get-started/honeycomb/2026-pro-plan-changes) | — | **dated: Agent Timeline is GA (June), not EA** |
| New Relic | SaaS continuous; docs notes thru 2026-08-28 (GenAI OTel, eBPF AI monitoring); Notebooks GA 2026-08-11; Preflight OSS; infra agent `v1.80.1` 2026-08-31 | 2026-08-28 | [docs release notes](https://docs.newrelic.com/docs/release-notes/docs-release-notes/), [Notebooks GA](https://docs.newrelic.com/whats-new/2026/08/whats-new-08-11-notebooks/) | — | Notebooks GA newer than pass 67 |
| Elastic Stack | `9.5.3` | 2026-09-03 | [elastic/elasticsearch v9.5.3](https://github.com/elastic/elasticsearch/releases/tag/v9.5.3) | — | none |
| Dynatrace | SaaS/Managed continuous; Grail stores every OTLP metric attribute (releases 332–337, ~2026-06); dtctl CLI | 2026-06-30 | [What's New video / Grail OTLP metrics](https://www.youtube.com/watch?v=YQMGgKHZzsw) | — | restamp; no numeric product version |
| Splunk Obs Cloud | SaaS continuous; Aug 2026: Fleet Management (2026-08-31), Agent Observability (2026-08-07), APM AI Assistant on traces (2026-08-27); AI SRE GA June 2026 | 2026-08-31 | [August 2026 notes](https://help.splunk.com/en/splunk-observability-cloud/release-notes/august-2026), [overview last updated 2026-08-31](https://docs.splunk.com/observability/en/release-notes/release-notes-overview.html) | — | **Aug 2026 notes newer than pass 67 “deep-dive stands”** |
| Chronosphere | SaaS continuous (PANW-owned); AgentiX = Cortex integration **planned/early**, not Chronosphere-native GA | 2026-07-15 | [Gartner MQ note](https://chronosphere.io/learn/chronosphere-named-a-leader-in-the-gartner-magic-quadrant-for-observability-platforms-for-third-consecutive-year/) | — | AgentiX still not Chronosphere product GA |
| Observe | SaaS continuous (Snowflake); Knowledge Graph + o11y.ai agents stand | 2026-09-13 | vendor docs checked via prior deep-dive; Snowflake parent notes 2026-09 | — | no new pin |
| Axiom | SaaS continuous; Metrics GA 2026-03-27; Correlations 2026-06-19; APL/MPL Grafana DS 2026-07-02 | 2026-07-02 | [Metrics GA](https://axiom.co/changelog/metrics-mpl), [Correlations](https://axiom.co/changelog/correlations), [APL/MPL](https://axiom.co/changelog/apl-and-mpl-in-the-grafana-data-source) | — | restamp; changelog.axiom.co fetch blocked this host (SSRF/private IP) — used dated changelog URLs from search |
| Better Stack | SaaS continuous; OTel logs+traces+metrics+RUM+uptime+incidents; eBPF collector; ClickHouse; Sentry-SDK-compatible errors | 2026-09-12 | [tracing docs](https://betterstack.com/docs/logs/tracing/), [RUM](https://betterstack.com/real-user-monitoring), GH org activity 2026-09-12 | — | still no deep-dive file; roster gap filled this pass |
| HyperDX / ClickStack | `@hyperdx/app@2.38.0` | 2026-09-04 | [hyperdxio/hyperdx @hyperdx/app@2.38.0](https://github.com/hyperdxio/hyperdx/releases/tag/%40hyperdx/app%402.38.0) | `hyperdx/hyperdx-all-in-one:2.38.0` `sha256:7b3bd9eec4e61aded56f705af7ddb2e8e49c54098d21aaaa7de6fde4d7c1f267` | none |
| Coroot | `v1.26.0` | 2026-09-07 | [coroot/coroot v1.26.0](https://github.com/coroot/coroot/releases/tag/v1.26.0) | Hub tag `coroot/coroot:v1.26.0` **404** this hour — digest not published on Docker Hub under that name | none |
| Highlight | `docker-v0.5.6` | 2025-08-08 | [highlight/highlight docker-v0.5.6](https://github.com/highlight/highlight/releases/tag/docker-v0.5.6) | — | none — still wound-down |
| Uptrace | stable `v2.0.3` (2026-05-13); newest published tag `v2.1.0-beta.8` (2026-08-13, GitHub `prerelease=false` despite beta name) | — | [uptrace/uptrace](https://github.com/uptrace/uptrace/releases) | — | **policy: compare stable `v2.0.3`; do not treat beta as stable** |
| PostHog | SaaS continuous; GH tags are desktop/SDK/agent-skills (`desktop-v0.61.382` 2026-09-12; `agent-skills-v0.1296.0` 2026-09-13) not platform | 2026-09-13 | [PostHog/posthog releases](https://github.com/PostHog/posthog/releases) | self-host is date-tagged image, not a GH platform pin | none |
| Jaeger | `v2.20.0` | 2026-07-20 | [jaegertracing/jaeger v2.20.0](https://github.com/jaegertracing/jaeger/releases/tag/v2.20.0) | `jaegertracing/jaeger:2.20.0` `sha256:46a886260e04002d8f45e213fc39063fa11a50446048fdaa64786fc0840cb9f8` (not `all-in-one:2.20.0` — that tag 404) | none |
| Prometheus | **Latest (GitHub Latest + prometheus.io Download): `v3.14.0`** (2026-08-17/18). **LTS: `v3.13.3`** (2026-09-07) | 2026-08-18 / 2026-09-07 | [prometheus.io/download](https://prometheus.io/download/), [v3.14.0](https://github.com/prometheus/prometheus/releases/tag/v3.14.0), [v3.13.3](https://github.com/prometheus/prometheus/releases/tag/v3.13.3) | `prom/prometheus:v3.14.0` `sha256:5ce7540c3c00ef4ab0c9d2c995c6a5b9c421f44b4a115d97a2c7af3b1c21cbb0`; `v3.13.3` amd64 `sha256:595c907995955f2d5fda19fae66392680921d9501cad0be5270b3fee959780b1` | **correction: pass 67 pinned only 3.14.0; 3.13.3 published later as LTS. Feature-line for PromQL reference = 3.14.0 (duration expr default-on)** |
| Maple | `v0.0.22` | 2026-09-03 | [MapleTechLabs/maple v0.0.22](https://github.com/MapleTechLabs/maple/releases/tag/v0.0.22) | lab uses official bundle, no Hub digest this pass | none |
| Rustrak | `v0.14.12` | 2026-09-07 | [rustrak/rustrak v0.14.12](https://github.com/rustrak/rustrak/releases/tag/v0.14.12) | `rustrak/rustrak-server:v0.14.12` `sha256:71c776b1c816a4ecdf668fed44a0fadf37ecb0d7c86f97262cd28b27b13e9dd4` | none |
| Odigos | **stable `v1.36.0`** (2026-09-06); `v1.37.0-rc1` / `v1.38.0-pre0` are prerelease | 2026-09-06 | [odigos-io/odigos v1.36.0](https://github.com/odigos-io/odigos/releases/tag/v1.36.0) | — | none vs pass 67; comparison-set.md still said v1.31.2 |

## Watch triggers this restamp

- **Prometheus dual-line:** pin both Latest `3.14.0` and LTS `3.13.3`. Do not call 3.14.0 “the only current.”
- **SigNoz lab override** still on `v0.140.0` digest while Foundry compose is `v0.141.1`. Workstream E: restamp override digest before claiming live SigNoz 0.141.1 from that file.
- **Honeycomb Agent Timeline is GA** (2026-06-18), not Early Access.
- **Splunk August 2026** Fleet Management + Agent Observability GA items — re-read `parallax-vs-splunk.md` agent cells before quoting pass-17.
- **Better Stack** still had no deep-dive; added [parallax-vs-betterstack.md](parallax-vs-betterstack.md) this pass (uptime/RUM/incident reference, not a P0 clone).
- **Uptrace policy:** `v2.1.0-beta.8` is published with `prerelease=false`; still not the stable comparison pin.
- **Odigos** 1.37/1.38 are pre; stable remains 1.36.0.
- GitHub API 403 after first batch; OSS pins above are from the completed first fetch + Hub + vendor pages. SaaS changelog.honeycomb.io / axiom.co HTML fetch blocked (SSRF to 198.18.x); dated first-party URLs from search used instead.

## PromQL product decision (GOAL freeze item 12)

**Keep the typed error-proof metric builder. Do not embed Grafana. Do not add PromQL as the Parallax product query language.**

Recorded in [gap-matrix-2026-09-13.md](gap-matrix-2026-09-13.md) §H and [promql-decision-2026-09-13.md](promql-decision-2026-09-13.md).
