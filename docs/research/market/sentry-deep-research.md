# Sentry Deep Research

Research date: 2026-06-22

Sentry is the **incumbent error-tracking + issue-workflow product** that Parallax plans to
be **Sentry-envelope-compatible** with. It is not a like-for-like "run it locally instead of
Parallax" OSS competitor — it is the ecosystem Parallax wants to absorb the SDK fleet from.
This note answers the two questions the operator asked directly — (A) does Sentry support
OTLP, and (B) how to run it locally — then gives the full dossier. Grounded in current
primary sources (docs.sentry.io, develop.sentry.dev, GitHub) checked June 2026.

## Sources

- OTLP: [docs.sentry.io/concepts/otlp](https://docs.sentry.io/concepts/otlp/), [collector pipeline](https://docs.sentry.io/concepts/otlp/forwarding/pipelines/collector/), [develop OTLP](https://develop.sentry.dev/sdk/telemetry/traces/otlp/), [OTLP open-beta discussion #85902](https://github.com/getsentry/sentry/discussions/85902)
- OTel-in-SDK: [develop OpenTelemetry](https://develop.sentry.dev/sdk/telemetry/traces/opentelemetry/), [structured logging + OTel](https://blog.sentry.io/structured-logging-opentelemetry/)
- Self-host: [github.com/getsentry/self-hosted](https://github.com/getsentry/self-hosted), [self-hosted OTLP issue #3830](https://github.com/getsentry/self-hosted/issues/3830), [develop self-hosted](https://develop.sentry.dev/self-hosted/), [architecture](https://develop.sentry.dev/application/architecture/), [Snuba overview](https://getsentry.github.io/snuba/architecture/overview.html)
- Local dev: [spotlightjs.com](https://spotlightjs.com/), [Sentry for development](https://blog.sentry.io/sentry-for-development/), [devservices](https://develop.sentry.dev/development-infrastructure/devservices/)
- License: [open.sentry.io/licensing](https://open.sentry.io/licensing/), [FSL announcement](https://blog.sentry.io/introducing-the-functional-source-license-freedom-without-free-riding/)
- AI/MCP: [Seer autofix](https://docs.sentry.io/product/ai-in-sentry/seer/autofix/), [seer product](https://sentry.io/product/seer/), [mcp.sentry.dev](https://mcp.sentry.dev/), [github.com/getsentry/sentry-mcp](https://github.com/getsentry/sentry-mcp)
- Pricing: [sentry.io/pricing](https://sentry.io/pricing/)

## A) Does Sentry support OTLP?

**Yes — partially, HTTP-only, open beta — but OTLP is a bolt-on, not the native tongue.** Two
distinct things must be separated:

1. **Sentry SDKs interoperating WITH OpenTelemetry (mature, the main story).** Sentry's own SDKs
   can consume OTel instrumentation/spans internally, propagate trace context, and follow OTel
   semantic conventions. Data still leaves the app as Sentry's **envelope** format to a **DSN**,
   not as OTLP. This is "Sentry uses OTel inside the SDK," not "Sentry is an OTLP receiver."

2. **Sentry as a native OTLP RECEIVER (newer, open beta).** This now exists — a standard OTel SDK
   or OTel Collector can export **directly** to Sentry with no Sentry SDK/exporter:
   - Traces: `https://o<orgId>.ingest.sentry.io/api/<projectId>/integration/otlp/v1/traces`
   - Logs: `https://o<orgId>.ingest.sentry.io/api/<projectId>/integration/otlp/v1/logs`
   - **HTTP only** (`otlphttp` exporter, `proto` encoding, gzip). **No documented OTLP gRPC receiver.**
   - Auth via header `x-sentry-auth: sentry sentry_key=<DSN public key>` — reuses DSN identity.
   - **Traces = open beta**, **Logs = open beta**, **Metrics = NOT supported via OTLP**
     ("Sentry does not support OTLP metrics at this time").
   - Announced open beta **2025-02-25**; still open beta as of June 2026, **not GA**.
   - Self-hosted OTLP is **in progress** (tracking issue getsentry/self-hosted #3830), not confirmed GA.

**Native protocol (the real default):** the **Sentry envelope** — clients POST envelopes to **Relay**
at `/api/<project_id>/envelope/` (older `/store/`), authenticated by a **DSN**. The envelope wraps
typed items (error event, transaction, attachment, replay, feedback). Error-event-centric with rich
exception/stacktrace/grouping payloads — structurally different from OTLP span/log/metric records.
**This envelope is what Parallax's "Sentry-envelope compatibility" targets, not the OTLP endpoints.**

**Bottom line:** OTLP ingest in Sentry is a recent HTTP-only beta (traces + logs, no metrics) layered
onto an envelope/DSN-native, error-first pipeline. Parallax's OTLP-native positioning is a genuine
differentiator — in Sentry, OTLP is the guest, not the native tongue.

## B) Running Sentry locally to verify it

Three options, very different weights:

1. **`getsentry/self-hosted` (full backend, docker-compose) — heavy.**
   - Latest release **26.6.0 (2026-06-16)**; monthly CalVer (`YY.M.minor`).
   - Requirements: **4 CPU cores, 16 GB RAM + 16 GB swap, 20 GB disk** (32 GB RAM recommended), high IOPS.
   - Install: clone release → `./install.sh` (~5–30 min) → `docker compose up --wait` → `http://127.0.0.1:9000`.
   - Runs **~20–40 containers** feature-complete (Relay, Kafka, ClickHouse, Snuba consumers, Postgres,
     Redis, Symbolicator, Vroom/profiling, web, workers, cron/beat, taskbroker). An **errors-only mode**
     trims to ~10 services. Multi-database distributed system — the sharpest contrast with Parallax local-first.

2. **`devservices`** — Sentry's dev-infra CLI for hacking on Sentry itself; still backend-heavy, for contributors.

3. **Spotlight (`spotlightjs.com`) — lightweight, backend-free.** "Sentry for development": a local
   sidecar/overlay; SDKs run **without a DSN/backend** and pipe events to Spotlight for local-only
   debugging. Closest to a local-first feel, but a **dev-time debug overlay, not a persistent queryable
   backend** — no issue lifecycle, no retention, no team workflow.

**Minimal "send an error and see it" loop:** (a) full self-hosted → create project → DSN → init SDK →
throw → view issue at `:9000`; or (b) Spotlight + SDK (no DSN) → throw → see it in the local overlay.
Full-fidelity product loop requires the heavy stack.

## Identity, License, Company

- **Sentry** (Functional Software, Inc.), San Francisco; Series E; ~$217M raised; ~$3B valuation (2022);
  ~439 employees (2026). ~13-year-old incumbent.
- **License: Functional Source License (FSL)** — Sentry's own non-compete source-available license,
  converting to **Apache-2.0 or MIT after 2 years** (FSL-1.1-Apache-2.0 / FSL-1.1-MIT). **Source-available,
  not OSI open-source**, during the 2-year window. (Compare: Parallax is Apache-2.0 from day one.)
- **Languages:** server **Python/Django**; **Relay is Rust**; **Snuba** is Python over ClickHouse;
  **Symbolicator/Vroom** Rust/Go; 30+ SDKs.

## Architecture

SDK → **Relay (Rust)** receives envelopes at `/api/<id>/envelope/`, validates DSN/project (config cached
in **Redis**), normalizes, rate-limits → **Kafka** → ingest consumers preprocess (symbolicate via
**Symbolicator**, store payload in **nodestore**) → **Snuba** consumers write to **ClickHouse** (errors +
transactions/spans). **Postgres** holds relational metadata (orgs, projects, issues, users). **Vroom**
handles profiling. Reads/search go through Snuba → ClickHouse. Kafka-centric, multi-store distributed pipeline.

## Feature Inventory

Error tracking with automatic exception capture + **issue grouping** and full lifecycle
(**resolve / regress / ignore(archive) / assign**, ownership rules); distributed **tracing & performance**;
**session replay**; **profiling** (continuous); **cron monitors**; **uptime monitoring**; **logs** (incl.
OTLP logs beta); **metrics**; **dashboards**; **alerts** (Slack/Discord/PagerDuty/…); **user feedback**;
**Seer** AI. The grouping + triage workflow is the mature core and the part Parallax must interoperate with.

## AI / MCP

- **Seer** = Sentry's AI agent. **Autofix** 3-step flow (Root-Cause Analysis → Solution → Code Generation;
  can open a PR or hand off to a coding agent). Root cause ~2 min, full autofix-to-PR ~6 min. **Seer pricing:
  $40 per active contributor/month** add-on. Seer may be unavailable on self-hosted (disableable via
  `--disable-skills=seer`).
- **Official Sentry MCP server** at **`mcp.sentry.dev`** — remote (HTTP/SSE, **OAuth 2.0**, nothing to
  install) + **local stdio** (`npx @sentry/mcp-server@latest`). ~v0.33.0 (2026-04-26), ~20 tools
  (`search_events`, `search_issues`, `use_sentry`, …), ~85K weekly npm downloads. Clients: Claude Code
  (also a plugin), Cursor, Codex, Windsurf, VS Code/Copilot. MCP can invoke Seer for root-cause/fix.
- **No portable, versioned, redacted evidence-bundle schema** — Seer produces in-product RCA + PRs, not a
  hand-off-able evidence artifact.

## Pricing

- **Developer (free):** 5K errors + 10K perf units/mo, 1 user, 30-day retention.
- **Team:** from **$26/mo** (annual), 50K errors, then usage-based.
- **Business:** from **~$80/mo**; Enterprise custom. **Seer +$40/active contributor/mo.**
- **Self-hosted: free** and feature-complete — you pay in ops (~20–40 containers, 16–32 GB RAM).

## Strengths and Gaps (vs the Parallax wedge)

**Strengths:** dominant brand + envelope ecosystem (30+ SDKs); best-in-class **issue grouping + triage
lifecycle**; Seer autofix + official MCP; broad signal coverage; mature alerting/dashboards.

**Gaps Parallax exploits:**
1. **Not OTLP-native** — OTLP is a beta, HTTP-only, traces+logs (no metrics) bolt-on; envelope/DSN is the
   real protocol.
2. **Not local-first** — verification needs a 16–32 GB, 20–40-container Kafka/ClickHouse/Postgres/Redis
   stack; Spotlight is only a dev overlay. A single Rust binary is a sharp contrast.
3. **Source-available, not open** (FSL non-compete for 2 years) vs Parallax Apache-2.0.
4. **Error-centric**, no native evidence-bundle / redaction-at-ingest story.
5. **No fix-outcome loop** (accepted/rejected/reverted) — Seer opens a PR but does not track outcomes.

**Where Parallax must interoperate, not beat:** the Sentry **envelope is the de-facto error-ingest
standard**. Parallax's planned Sentry-envelope compatibility is the wedge to absorb Sentry's SDK fleet
while offering OTLP-native + local-first + redaction + outcome tracking on top.

## Backend & Data Flow

See [backend-and-data-flow.md](backend-and-data-flow.md) for the side-by-side. Sentry is a Django monolith +
Rust ingestion edge (Relay) + ClickHouse analytics (Snuba), the whole pipeline a **Kafka eventstream**:

- **Engines:** searchable event/telemetry → **ClickHouse via Snuba**; relational metadata → **Postgres 14**
  (+ PgBouncer); cache/quotas/rate-limits → **Redis** (+ Memcached); broker → **Kafka** (dozens of topics);
  **nodestore** (full raw payload) → Postgres self-host / Bigtable SaaS; profiling → object storage via **Vroom**;
  symbols → **Symbolicator** (Rust).
- **Flow:** `SDK gzip envelope ─► /api/<id>/envelope/ ─► RELAY (Rust: validate DSN, normalize, PII-scrub, rate-limit)
  ─► KAFKA ingest-* ─► consumers (errors→Symbolicator→grouping→nodestore; profiles→Vroom; metrics→metrics consumers)
  ─► KAFKA eventstream ─► SNUBA CONSUMER (Rust/arroyo) ─batched INSERT─► CLICKHOUSE ─► post-process-forwarder (alerts)`.
  Read: `Django ─► { Postgres (issues/metadata) | Snuba SnQL/MQL→ClickHouse | nodestore (full body) }`.
- **Write/read:** Kafka partitioned **by project ID** for ordering; Snuba batches Kafka → big ClickHouse INSERTs
  (at-least-once, ReplacingMergeTree dedup → eventually consistent). Spans now in the EAP wide-column store
  (`spans_v3`), claimed up to 62× faster OLAP.
- **Throughput (vendor):** Relay "hundreds of thousands req/sec" (fleet aggregate); Snuba Rust consumer ~20× vs
  Python. No ingest-to-queryable SLA — visible lag seconds, grows under backlog.
- **Footprint:** **the famous heaviness** — official min 4 CPU / 16 GB RAM + 16 GB swap / ≥20 GB disk (32 GB rec.);
  **~45–50 containers** in self-hosted compose (~23 Snuba services + ~15 consumers + infra + Relay/Symbolicator/Vroom).
- **Designed for:** high-cardinality multi-signal *application* observability with a smart edge (Relay normalizes/
  scrubs/quotas at the boundary) + best-in-class error grouping/symbolication. **Not for:** lightweight/single-node —
  a distributed many-service system assuming horizontal scale + dedicated ops.

## Comparison: Sentry vs Parallax

| Dimension | Sentry | Parallax |
|-----------|--------|----------|
| **Product category** | Error-tracking + APM incumbent (issue workflow) | Evidence context engine |
| **Primary language** | Python + Rust (Relay) | Rust (Tokio) |
| **Native protocol** | Sentry envelope / DSN | Sentry envelope + OTLP, equal citizens |
| **OTLP ingest** | Beta, HTTP-only, traces+logs, no metrics | Native, all signals |
| **Error model** | Best-in-class grouping + lifecycle (resolve/regress/ignore/assign) | Derived error events → fingerprint → bundle (interop with Sentry grouping) |
| **Telemetry store** | ClickHouse + Kafka + Postgres + Redis | GreptimeDB + Turso |
| **Local mode** | ~20–40 container stack, 16–32 GB RAM (or Spotlight dev overlay) | Single binary |
| **Agent surface** | Seer autofix + official MCP (~20 tools) | CLI-first + read-only MCP after gates |
| **Evidence bundles** | None (in-product RCA + PR) | Versioned, redacted, portable |
| **Outcome tracking** | None | Fix-outcome loop |
| **Redaction** | Server-side scrubbing / data-scrubbing rules | Structured redaction before agent access (A6) |
| **License** | FSL (→ Apache/MIT after 2 yrs) | Apache-2.0 |
| **Maturity** | Incumbent, self-hosted 26.6.0 | Research/early implementation |

## Threat Assessment for Parallax

**Sentry is the incumbent to interoperate with, not an OSS local-runnable substitute.** It pressures
the error-tracking workflow (grouping, triage, Seer autofix) and has the brand + SDK distribution. But:

- OTLP is a beta bolt-on (no metrics, HTTP-only) — Parallax OTLP-native stands.
- Local verification is a 20–40-container stack — Parallax single-binary stands.
- No portable evidence bundle, no outcome loop, no redaction-at-ingest as an artifact — the core thesis stands.

### Watch Triggers

Re-evaluate if Sentry: makes **OTLP ingest GA on self-hosted** (incl. metrics + gRPC); ships a **portable
evidence/RCA artifact** with provenance/redaction/outcome semantics; adds a **fix-outcome loop** to Seer;
or ships a genuinely **light single-binary local backend** (beyond the Spotlight dev overlay).

## Summary Verdict

Sentry is the category-defining error-tracking incumbent: unmatched issue grouping + triage lifecycle, a
huge SDK fleet on the envelope/DSN protocol, Seer autofix, and an official MCP. It now has **beta OTLP
ingest (traces + logs, HTTP-only, no metrics)** — real but not native, and partial on self-hosted.

For Parallax, Sentry is the **interoperability target, not the OSS substitute**: Parallax plans to speak
the envelope to absorb Sentry's SDKs, then add what Sentry structurally lacks — OTLP-native multi-signal
ingest, single-binary local-first deployment, versioned redacted portable evidence bundles, and a
fix-outcome loop. Sentry's heavy distributed local stack and source-available license make its self-hosted
story an operational burden Parallax's single Apache-2.0 binary is designed to undercut.
