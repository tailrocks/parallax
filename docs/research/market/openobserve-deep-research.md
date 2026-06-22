# OpenObserve Deep Research

Research date: 2026-06-22

This note gives OpenObserve ("O2") a Maple-style standalone deep-dive, consolidating
what was previously scattered across `competitor-watch.md` (the "OpenObserve AI/MCP
Enterprise Recheck" ledger), `competitive-comparison-matrix.md`, the storage evaluation,
go/no-go, and the validation ledgers. It is grounded in current primary sources
(openobserve.ai, docs, GitHub) checked June 2026; facts are flagged where a source
conflict exists.

OpenObserve is the closest open/self-hosted competitor on **storage + runtime fit**:
Rust engine, object-storage-native, self-hostable, single binary, OTLP-native, with a
shipping AI SRE/RCA and MCP surface. That overlaps Parallax's local-first, Rust, OTLP
positioning more than any other tracked tool — which is exactly why the wedge analysis
below matters.

## Sources

- [github.com/openobserve/openobserve](https://github.com/openobserve/openobserve) (+ GitHub REST API releases/languages)
- [openobserve.ai](https://openobserve.ai/), [/pricing](https://openobserve.ai/pricing/), [/enterprise-license](https://openobserve.ai/enterprise-license/)
- [docs/architecture](https://openobserve.ai/docs/architecture/), [docs/getting-started](https://openobserve.ai/docs/getting-started/), [docs/ingestion/logs/otlp](https://openobserve.ai/docs/ingestion/logs/otlp/)
- [docs/features/enterprise](https://openobserve.ai/docs/features/enterprise/), [docs/user-guide/functions](https://openobserve.ai/docs/user-guide/functions/functions-in-openobserve/), [docs/user-guide/data-exploration/rum/overview](https://openobserve.ai/docs/user-guide/data-exploration/rum/overview/)
- [/ai-assistant](https://openobserve.ai/ai-assistant/), [/ai-sre](https://openobserve.ai/ai-sre/), [/incidents](https://openobserve.ai/incidents/)
- [docs/integration/ai/mcp](https://openobserve.ai/docs/integration/ai/mcp/), [blog/mcp-monitoring](https://openobserve.ai/blog/mcp-monitoring/)
- [blog: Apache→AGPL relicense](https://openobserve.ai/blog/what-are-apache-gpl-and-agpl-licenses-and-why-openobserve-moved-from-apache-to-agpl/), [blog/series-a-announcement](https://openobserve.ai/blog/series-a-announcement/), [blog/june-25-pricing-policy-updates](https://openobserve.ai/blog/june-25-pricing-policy-updates/)
- [BusinessWire Series A](https://www.businesswire.com/news/home/20260429840147/en/), [FinSMEs](https://www.finsmes.com/2026/04/openobserve-raises-10m-in-series-a-funding.html)
- third-party read-only MCP: [mdfranz/openobserve-oss-mcp](https://github.com/mdfranz/openobserve-oss-mcp), [alilxxey/openobserve-community-mcp](https://github.com/alilxxey/openobserve-community-mcp)

## What OpenObserve Is

OpenObserve is an **open-source, cloud-native observability platform** unifying logs,
metrics, traces, frontend/RUM (with session replay), data pipelines, and LLM observability
in a **single Rust binary**. It is positioned as a Datadog / Splunk / Elasticsearch
alternative, with a headline "140x lower storage cost" claim from a Parquet-on-object-storage
architecture.

- **License — open-core / dual-license:**
  - Core: **AGPL-3.0** (relicensed from Apache-2.0 in Nov 2023).
  - Enterprise: separate **commercial Enterprise License Agreement** (non-AGPL). Enterprise-gated:
    Super Cluster/federated search, **Sensitive Data Redaction**, RBAC/SSO, BYOB, cipher keys,
    **audit trail**, and **all AI/MCP features**.
- **Primary language:** the **server/engine is Rust** (~26% by line count). GitHub's headline label
  is TypeScript (~38%) and Vue (~19%) — that is the UI, not the engine. Do not read the TS label as
  "OpenObserve is a TS project."
- **Company & funding:** OpenObserve Inc., founded 2022, Menlo Park CA. **Series A: $10M, announced
  ~2026-04-28/29**, co-led by Nexus Venture Partners + Dell Technologies Capital (same leads as the
  earlier seed). Series A coincided with the "Observability 3.0" AI-native launch. Seed amount/date
  unconfirmed.
- **Maturity:** ~19.4k stars, ~863 forks (June 2026). Latest tag **v0.91.0-rc3 (2026-06-18)** — an
  RC, though the API marks `prerelease=false`; latest clean non-RC stable **v0.90.3 (2026-05-26)**.
  Very rapid cadence (patches within days, minors every few weeks with multiple RCs).

## Architecture

- **Storage:** schema-on-read, **object-storage-native**; all telemetry written as **Apache
  Parquet**. No inverted indexes — performance from partitioning + lightweight indexing + caching.
  Ingest path: Memtable → Immutable → local Parquet → object storage, with WAL.
- **Query engine:** **Apache DataFusion** (Arrow), querying Parquet directly.
- **Components:** Router, Ingester, Compactor, Querier (LEADER/WORKER split over gRPC), AlertManager.
- **Metadata store (mode-dependent):** single-node → **SQLite**; HA/cluster → **PostgreSQL** +
  **NATS** as cluster coordinator (older versions used etcd).
- **Single-binary vs HA:** single Rust binary (SQLite + local disk or object store) vs horizontally
  scaled stateless services (Postgres + object store + NATS). Multi-region Super Cluster is
  Enterprise-only.
- **Object-storage backends:** **S3, MinIO, GCS, Azure Blob, and local disk.**

### Key Technical Choices

| Layer | Choice | Notes |
|-------|--------|-------|
| Engine | Rust | single self-contained binary |
| Storage format | Apache Parquet on object storage | schema-on-read, no inverted index |
| Query engine | Apache DataFusion (Arrow) | queries Parquet directly |
| Metadata | SQLite (single) / PostgreSQL + NATS (HA) | mode-dependent |
| Object store | S3 / MinIO / GCS / Azure / local disk | disk-only is the single-node default |
| Transform | VRL functions | ingest-time + query-time (enrichment, redaction, reduction) |

## OTLP / OTel Support

- **Native OTLP, both transports, all three signals.** OTLP/gRPC (port 5081, `ZO_GRPC_PORT`) and
  OTLP/HTTP (port 5080, per-org `/api/<org>/v1/{logs,metrics,traces}`, Basic auth). gRPC added
  single-node v0.6.4, distributed v0.7.4.
- **RUM / frontend:** Core Web Vitals, frontend error tracking with stack traces, session tracking,
  **session replay** with input masking/privacy controls, via a JS snippet. (Reported Datadog
  browser-sdk lineage is **unconfirmed** from official docs.)
- **Sentry:** **No native Sentry SDK/envelope ingestion and no documented Sentry migration path.**
  The only documented migration guide is Grafana LGTM → OpenObserve. Error data is expected over
  OTLP or the RUM JS snippet. (Absence of evidence; if a path exists it is undocumented.)
- **Query languages:** **SQL** (logs/traces), **PromQL** (metrics; also SQL-queryable), **VRL** for
  ingest/query-time transforms (not a search DSL). No proprietary search DSL.

## Local-Run Story

- **Single self-contained Rust binary** ("running in under 2 minutes"). One command:
  `ZO_ROOT_USER_EMAIL=... ZO_ROOT_USER_PASSWORD=... ./openobserve` (or equivalent Docker run with
  `-p 5080:5080 -v $PWD/data:/data`). UI at `http://localhost:5080`.
- **Disk-only mode without S3 — yes, the single-node default.** Persists to local disk with no
  object-storage config. `ZO_DATA_DIR` sets the path; `ZO_MEM_TABLE_MAX_SIZE` bounds RAM cache.
- **Footprint claims (vendor/community, not independent):** "140x lower storage" and "~1/4–1/5 the
  infrastructure" vs Elasticsearch; Rust avoids JVM heap overhead; some deployments reportedly
  <1 GB RAM. **No direct "vs ClickHouse memory" claim found** — comparisons target
  Elasticsearch/Datadog/Splunk.

This is a genuine local-first single-binary story — the closest of any tracked competitor to
Parallax's own single-binary local positioning, and unlike SigNoz's ~5-container ClickHouse stack.

## AI / MCP Surface

**All AI features are Enterprise-edition (free under the registered tier) + BYO-LLM-key — NOT in
the AGPL community build.** Master switch `O2_AI_ENABLED=true`.

- **AI Assistant (Enterprise):** NL→SQL and NL→PromQL, log summarization/anomaly detection,
  dashboard + alert generation, schema-aware.
- **AI SRE Agent / RCA (Enterprise):** three-phase per-incident pipeline (Context Assembly →
  Historical Pattern Matching against past incidents → LLM analysis → structured RCA), multi-signal
  correlation, service-dependency mapping, auto incident reports with links to supporting evidence.
  Fires on alert. Cloud price **$0.50 per AI Credit**. Providers: OpenAI, Anthropic, Gemini, Bedrock,
  DeepSeek, OpenRouter, OpenAI-compatible/self-hosted.
- **MCP server (Enterprise):** official, **Rust, built into the Enterprise binary** — there is **no
  standalone `openobserve/mcp-server` repo** (standalone repos are third-party/community and
  read-only). Exposed over **HTTP** at per-org `https://<instance>/api/{org_id}/mcp`, protocol
  `2025-11-25`, HTTP Basic auth.
  - **Tool surface: docs cite 140+ tools across 14 categories — full CRUD + admin, NOT read-only.**
    Roughly ~93 read (`SearchSQL`, `SearchAround`, `StreamList`, `StreamSchema`, `ListAlerts`,
    `ListIncidents`/`GetIncident`, `PrometheusQuery/RangeQuery`, `ListDashboards`, …); ~32
    create/update (`CreateAlert`, `TriggerAlert`, `CreateDashboard`, `CreatePipeline`, `UserSave`,
    `CreateRoles`, `CreateOrganization`, `SetKVValue`, `AssumeServiceAccount`, …); ~18
    delete/destructive ⚠️ (`DeleteAlert`, `DeleteDashboard`, `StreamDelete`, `RemoveUserFromOrg`,
    `DeleteSearchJob`, …); plus role/system-setting admin. (Counts/names vary blog vs docs; treat
    docs `Search*`/`Stream*` names as canonical, counts as approximate.)
  - **Gating — important:** **no native read-only mode; write/destructive tools are NOT disabled by
    default.** Safety relies on operator-configured **RBAC** (connect as a least-privilege user). An
    optional `O2_MCP_VALIDATION_ENABLED=true` ("hybrid validation mode") validates calls but is not a
    read-only switch. Client confirmation (e.g., Claude) is client-side, not server-enforced.
- **Evidence / audit:** uses "complete evidence chain," "verifiable evidence," "audit trail of every
  log/trace/metric during the investigation" language, plus a separate Enterprise **Audit Trail**
  (`O2_AUDIT_ENABLED=true`). **No published portable, versioned evidence-bundle schema** — these are
  in-product workflows with in-app evidence links and auto-generated reports, not a documented
  exportable/standardized bundle format.

## Pricing & Tiers

- **Cloud (pay-as-you-go):** ingestion **$0.50/GB** (current page; older June-2025 blog said
  $0.30/GB — stale), query **$0.01/GB**, extra retention **$0.02/GB per 30 days**, RUM/replay
  $1/1,000 sessions, error tracking $0.15/1,000 events, AI $0.50/credit. 14-day trial; the
  **permanent free Cloud tier was discontinued (June 2025)**. Default retention: metrics 15 months,
  logs/traces/RUM 30 days. Unlimited users.
- **Self-hosted Community:** AGPL, free forever, community support.
- **Self-hosted Enterprise:** adds SSO, RBAC, federated search/Super Cluster, BYOB, cipher keys,
  query/workload management, audit trail, extended retention, sensitive-data redaction — free under
  the registered GB/day allowance.

### The free GB/day conflict — three numbers, three different things (do not collapse)

| GB/day | What it means | Source |
|--------|---------------|--------|
| **10 GB/day** | Free, **no registration** | EULA ([enterprise-license](https://openobserve.ai/enterprise-license/)) — verified |
| **50 GB/day** | Free **with registration** (Free-Tier Key, expires every 12 months, offline activation for air-gapped) | EULA + docs + pricing — verified |
| **200 GB/day** | Older founder/marketing claim (≈$60k/yr Datadog value) — **NOT on any current authoritative page** | Techzine via search; third-party reviews |

**Current authoritative answer (June 2026):** tiered **10 GB/day unregistered → 50 GB/day
registered**. The 50 GB/day figure is the customer-facing headline; **200 GB/day is stale
marketing**, not the binding EULA limit.

## What OpenObserve Does Well

1. **Real Rust engine + object-storage/Parquet/DataFusion architecture** with a strong cost story —
   the most architecturally similar tracked competitor to Parallax's Rust, lean-storage direction.
2. **Genuine single-binary, disk-only-default local run** — S3 optional. Overlaps heavily with the
   "local-first OTLP-native" positioning and is lighter than SigNoz's container stack.
3. **OTLP-native** (gRPC + HTTP, all three signals) plus RUM/session replay and data pipelines.
4. **Broad shipping AI/MCP surface** — AI Assistant, AI SRE with three-phase RCA + historical
   pattern matching, 140+ MCP tools, BYO-LLM, multi-provider. The AI SRE "evidence chain / audit
   trail" framing is the most direct pressure on the evidence narrative of any tracked tool.
5. **Has VRL-based redaction** (Sensitive Data Redaction, Enterprise) and an Enterprise Audit Trail —
   more safety surface than most competitors, though Enterprise-gated.
6. **Funded ($10M Series A), fast cadence, ~19.4k stars.**

## What OpenObserve Does Not Do Well (vs the Parallax wedge)

1. **No portable, versioned evidence-bundle schema.** "Evidence chain" is in-app UX + auto-reports,
   not an exportable, standardized, versioned artifact with provenance/redaction-report/raw-ref/
   query-manifest/missing-evidence/outcome semantics. Strongest differentiation lane.
2. **No Sentry SDK/envelope ingestion or migration path** — clear open gap for Sentry-migration.
3. **MCP server is write/destructive-capable by default; no native read-only mode** — safety leans
   on operator RBAC + an optional validation flag. A read-only bounded evidence projection is a
   defensible contrast.
4. **AI/MCP and redaction are Enterprise-gated** (absent from the AGPL community build); AI is
   BYO-key and metered ($0.50/credit). An open, local-first AI/evidence story differs.
5. **AGPL core** may deter some adopters vs a permissive (Apache-2.0) competitor.
6. **No outcome loop** — the incident workflow ends at an auto-report, not a verified-outcome /
   accepted-rejected-reverted loop.

## Comparison: OpenObserve vs Parallax

| Dimension | OpenObserve | Parallax |
|-----------|-------------|----------|
| **Product category** | Full observability platform (logs/metrics/traces/RUM/pipelines) | Evidence context engine (not a dashboard suite) |
| **Primary language** | Rust engine (+ Vue/TS UI) | Rust (Tokio) |
| **Ingest protocols** | OTLP only (+ RUM JS snippet) | Sentry envelope + OTLP |
| **Telemetry store** | Parquet on object storage / disk, DataFusion | GreptimeDB behind storage adapter |
| **Metadata** | SQLite (single) / PostgreSQL + NATS (HA) | Turso (libSQL) |
| **Local mode** | Single Rust binary, disk-only default, S3 optional | Single binary (GreptimeDB + Turso) |
| **Agent surface** | MCP (140+ tools, read + write/destructive), Enterprise | CLI-first, HTTP underneath, MCP (read-only, gated) |
| **Evidence bundles** | "Evidence chain" UX + auto-reports, no schema | Bounded, redacted, citable, portable, versioned bundles |
| **Outcome tracking** | None (ends at auto-report) | Fix-outcome loop (accepted/rejected/reverted) |
| **Redaction** | VRL Sensitive Data Redaction (Enterprise) | Structured redaction before agent access (A6 gate) |
| **AI gating** | Enterprise + BYO-key + metered credits | Open core, local-first |
| **License** | AGPL-3.0 core / commercial Enterprise | Apache-2.0 |
| **Maturity** | v0.90.3 stable / v0.91 RC, ~19.4k stars, $10M Series A | Research/early implementation |

## Threat Assessment for Parallax

**Highest structural pressure of any tracked competitor on storage/runtime fit; still does not close
the wedge.**

OpenObserve is the closest tool to Parallax on architecture — Rust, object-storage-lean, single
binary, OTLP-native — and it has a shipping AI SRE/RCA surface with "evidence chain" and "audit
trail" language plus VRL redaction. That makes it the most serious watch target on both the
storage-fit and AI-evidence narratives.

It does **not** close the wedge because the checked sources keep these gaps open:

1. **No portable evidence-bundle schema** — auto-reports and in-app evidence links, not a versioned
   exportable artifact.
2. **No Sentry ingestion / migration path.**
3. **MCP is a broad CRUD+admin management plane, write/destructive-capable by default** — not a
   read-only bounded evidence projection.
4. **AI/MCP + redaction are Enterprise-gated and BYO-key** — not an open local-first AI/evidence
   story.
5. **No outcome loop.**

### Watch Triggers

Re-evaluate if OpenObserve:

- Publishes a **versioned, portable evidence/investigation schema** (provenance, redaction report,
  raw-refs, query manifest, missing-evidence flags, outcome rows) — would pressure A3.
- Adds **Sentry envelope ingestion** or a Sentry migration path — would close the migration lane.
- Ships a **native read-only MCP mode** or a bounded evidence-projection surface.
- Moves **AI/MCP into the AGPL community build** (un-gates the agent surface).
- Adds **fix-outcome tracking** — would close the core-thesis differentiator.

## Summary Verdict

OpenObserve is the closest open/self-hosted competitor to Parallax on **storage and runtime fit** —
a real Rust engine, Parquet-on-object-storage with DataFusion, a genuine single-binary disk-only
local mode, OTLP-native ingest, and a funded, fast-moving team. Its AI SRE/RCA surface with
"evidence chain" framing and VRL redaction makes it the most direct pressure on Parallax's
evidence-and-safety narrative of any tracked tool.

But it is a **full observability platform with an Enterprise-gated agent/AI plane**, not an evidence
context engine. Its "evidence chain" is in-product UX and auto-generated reports, not a versioned,
portable, redacted, hand-off-able bundle with an outcome loop; its MCP is a broad CRUD+admin
management plane that is write/destructive by default; and it has no Sentry-migration path. The
unoccupied ground — Sentry-ingest → OTLP correlation → versioned redacted read-only bundles →
outcome tracking, in an open local-first core — remains open, but OpenObserve is the competitor
most capable of moving into it, and should stay the top storage-fit watch target.
