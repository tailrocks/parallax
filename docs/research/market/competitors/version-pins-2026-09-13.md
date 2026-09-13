# Competitor version pins — 2026-09-13 (Workstream A)

Fresh primary-source sweep for GOAL.md §3. Every OSS row re-pulled from
GitHub Releases API on 2026-09-13. SaaS rows have no pinnable version by
nature: recorded as continuous + last dated primary item checked.

## Baseline SHAs (GOAL.md §1)

| Repo | Local HEAD | origin/main | Note |
| --- | --- | --- | --- |
| parallax | `de5592ae58ddccda6a09d025d92c2b2af597c187` | `5fc6b15beee42e5cbbf7fbe1b789220cb48af9cf` | branch `goal/error-investigation-identity`; HEAD moved `a3a240f3`→`de5592ae` mid-run (parallel workstream); tree has uncommitted changes from other workstreams |
| parallax-telemetry-playground | `f05c8838a567968fc2ab9e18d307bfebbd60c121` | `f05c8838a567968fc2ab9e18d307bfebbd60c121` | in sync |

## Pinned versions

| Product | Latest stable | Date | Primary source | Drift vs repo notes |
| --- | --- | --- | --- | --- |
| Sentry self-hosted | `26.8.0` | 2026-08-17 | `getsentry/self-hosted` releases | none (lab 09-12 current) |
| Sentry SaaS | continuous | — | sentry.io (no version) | — |
| Datadog platform | SaaS continuous; DASH 2026: 170+ capabilities, Bits AI SRE agent GA | 2026-06 | datadoghq.com DASH 2026 keynote/reports | Bits GA newer than deep-dive |
| Datadog Agent (proxy) | `7.83.1` | 2026-09-09 | `DataDog/datadog-agent` releases | new pin |
| Grafana | `v13.2.1` | 2026-09-02 | `grafana/grafana` releases | none |
| Tempo | `v3.0.3` | 2026-08-13 | `grafana/tempo` releases | +patch vs v3.0.2 in notes |
| Loki | `v3.7.7` | 2026-08-27 | `grafana/loki` releases | +patch vs v3.7.3 in notes |
| Mimir | `3.2.1` (tag `mimir-3.2.1`) | 2026-09-10 | `grafana/mimir` releases | minor drift vs 3.1.3 in notes |
| Pyroscope | `v2.3.1` | 2026-09-08 | `grafana/pyroscope` releases | minor drift vs 2.1.1 in notes |
| otel-lgtm | `v0.33.0` | 2026-09-11 | `grafana/docker-otel-lgtm` releases (repo renamed) | none (lab current) |
| SigNoz | `v0.141.1` | 2026-09-09 | `SigNoz/signoz` releases | none |
| SigNoz collector | `v0.144.9` | 2026-08-19 | `SigNoz/signoz-otel-collector` releases | none (official helm pairing) |
| SigNoz MCP | `v0.14.0` | 2026-09-02 | `SigNoz/signoz-mcp-server` releases | **drift: was v0.8.0 in notes** |
| SigNoz Foundry | `v0.2.17` | 2026-07-29 | `SigNoz/foundry` releases | none |
| OpenObserve | `v1.0.0` GA | 2026-09-11 | `openobserve/openobserve` releases | none (lab current) |
| Honeycomb | SaaS continuous; Agent Timeline GA, new Canvas, Errors-for-Frontend GA (Symbolicator) | 2026-08/09 | changelog.honeycomb.io, checked 2026-09-13 | newer than deep-dive (May launch) |
| New Relic | SaaS continuous; docs notes thru 2026-08-28 (GenAI OTel, eBPF AI monitoring); Preflight OSS active | 2026-08-28 | docs.newrelic.com/docs/release-notes, checked 2026-09-13 | newer than deep-dive |
| Elastic Stack | `9.5.3` | 2026-09-03 | `elastic/elasticsearch` + `elastic/apm-server` tags | **drift: was 9.4.3 in notes** |
| Dynatrace | SaaS/Managed continuous | — | docs.dynatrace.com release notes (checked 2026-09-13; deep-dive items stand) | no new pin; re-verify pending |
| Splunk Obs Cloud | SaaS continuous | — | docs.splunk.com/observability (checked 2026-09-13; deep-dive items stand) | no new pin; re-verify pending |
| Chronosphere | SaaS continuous | — | vendor changelog (checked 2026-09-13; AgentiX still planned per notes) | no new pin; re-verify pending |
| Observe | SaaS continuous | — | vendor changelog (checked 2026-09-13; deep-dive items stand) | no new pin; re-verify pending |
| Axiom | SaaS continuous | — | axiom.co/docs+changelog (checked 2026-09-13; deep-dive items stand) | no new pin; re-verify pending |
| Better Stack | SaaS continuous; full platform (logs/traces/metrics/errors/RUM/uptime/incidents, ClickHouse, eBPF collector) | — | betterstack.com/community+docs, checked 2026-09-13 | **no deep-dive in repo — roster gap** |
| HyperDX/ClickStack | `@hyperdx/app@2.38.0`, image `hyperdx/hyperdx-all-in-one:2.38.0` | 2026-09-04 | `hyperdxio/hyperdx` releases | none (lab current) |
| Coroot | `v1.26.0` | 2026-09-07 | `coroot/coroot` releases | **drift: was v1.23.3 (v1.24.5, v1.25.0 since)** |
| Highlight | `docker-v0.5.6` | 2025-08-08 | `highlight/highlight` releases | none — confirms wound-down |
| Uptrace | stable `v2.0.3` (2026-05-13); beta line `v2.1.0-beta.8` (2026-08-13, published prerelease=false) | — | `uptrace/uptrace` releases | **beta line newer; stable lags** |
| PostHog | SaaS continuous; self-host via date-tagged image; GH releases are desktop/SDKs (`desktop-v0.61.382`, 2026-09-12) not the platform | — | posthog.com/docs + `posthog/posthog` releases, checked 2026-09-13 | no platform pin exists |
| Jaeger | `v2.20.0` | 2026-07-20 | `jaegertracing/jaeger` releases | new pin (was component-level) |
| Prometheus | `v3.14.0` | 2026-08-18 | `prometheus/prometheus` releases | new pin (was component-level) |
| Maple | `v0.0.22` | 2026-09-03 | `MapleTechLabs/maple` releases | none (lab current) |
| Rustrak | `v0.14.12` | 2026-09-07 | `rustrak/rustrak` releases | none (lab current) |
| Odigos | `v1.36.0` | 2026-09-06 | `odigos-io/odigos` releases | **drift: was v1.31.2 in notes** |

## Watch triggers fired by this sweep

- SigNoz MCP `v0.8.0`→`v0.14.0`: re-check tool count/safety claims in `parallax-vs-signoz.md`.
- Coroot `v1.23.3`→`v1.26.0`: re-check eBPF→app-errors watch + Standard $1/core pricing.
- Odigos `v1.31.2`→`v1.36.0`: re-check own-store watch + Enterprise pricing.
- Elastic `9.4.3`→`9.5.3`: re-check Serverless rates.
- Better Stack has no deep-dive: add `parallax-vs-betterstack.md` (uptime/RUM/incident reference).
- Uptrace v2.1 beta line: decide stable-vs-beta comparison policy.
