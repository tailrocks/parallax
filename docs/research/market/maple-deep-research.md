# Maple Deep Research

Research date: 2026-05-31

## Sources

- [maple.dev](https://maple.dev/) — landing page, all feature pages, comparison pages, pricing, roadmap, local mode
- [docs.maple.dev](https://maple.dev/docs) — introduction, sampling & throughput, OTel conventions, local mode, CLI reference
- [github.com/Makisuo/maple](https://github.com/Makisuo/maple) — README, DESIGN.md, PRODUCT.md, TODO.md, repo structure
- Integration pages — Next.js, Python, Node.js

## What Maple Is

Maple is an **open-source, OpenTelemetry-native observability platform** that provides distributed tracing, log management, metrics & dashboards, error tracking, service catalog, Kubernetes monitoring, browser session replay, and an AI/MCP agent surface. It competes directly with Datadog, Grafana Cloud, New Relic, and Dash0 as a **full-stack observability backend**.

- **License:** FSL-1.1 (Functional Source License — source-available, self-hostable, but competitive use restrictions)
- **Language:** TypeScript (95.2%), with Rust (0.9%) for the local-mode chDB bridge, Go (0.2%) for some tooling
- **Runtime:** Bun
- **Storage:** ClickHouse (hosted via Tinybird; embedded chDB for local mode)
- **Metadata:** SQLite / Turso (libSQL)
- **Auth:** Clerk (cloud) or `self_hosted` mode with root password
- **Deploy:** Cloudflare Workers + D1 (via Alchemy), Docker Compose (self-host), single binary (local)
- **Monorepo:** 854 commits, 382 GitHub stars, 25 forks, 9 releases (v0.0.11 latest)
- **SDKs:** First-party Effect SDK; standard OTLP for Node.js, Next.js, Python, Go, Rust, Java, C#, Kotlin
- **MCP tools:** 10+ tools — `system_health`, `find_errors`, `inspect_trace`, `search_logs`, `search_traces`, etc.
- **Pricing:** Starter $19/mo (50 GB, 14d retention), Startup $39/mo (100 GB + $0.30/GB overage, 30d), Enterprise custom

## Architecture

### Monorepo Layout

| Path | Purpose |
|------|---------|
| `apps/web` | TanStack Router SPA (Vite) — the dashboard |
| `apps/api` | Effect HTTP API — Tinybird proxy + MCP server |
| `apps/ingest` | OTLP ingest gateway — key auth + org enrichment + collector forwarding |
| `apps/landing` | Astro landing site (maple.dev) |
| `apps/alerting` | Alert evaluation worker |
| `apps/chat-agent` | Cloudflare Worker chat surface |
| `apps/cli` | CLI utilities (the `maple` binary) |
| `apps/mobile` | Expo mobile app |
| `packages/domain` | Shared Effect HTTP contracts and domain types |
| `packages/query-engine` | Shared query and observability logic |
| `packages/ui` | Shared UI primitives and components |
| `deploy/` | Kubernetes Helm chart (`maple-k8s-infra`) |

### Data Flow (Hosted)

1. App emits OTLP (traces, logs, metrics) via standard OTel SDK
2. `apps/ingest` authenticates the ingest key, enriches with org context, forwards to OTel Collector
3. Collector writes to Tinybird (managed ClickHouse) — the query engine
4. `apps/api` proxies queries to Tinybird, serves the dashboard and MCP tools
5. `apps/web` renders the SPA dashboard

### Data Flow (Local Mode)

1. Single Bun-compiled binary (`maple` + `libchdb`)
2. OTLP/HTTP on `POST /v1/{traces,logs,metrics}` on `:4318`
3. chDB (embedded ClickHouse) stores in `~/.maple/data`
4. `/local/query` API serves both CLI and dashboard
5. Dashboard at `local.maple.dev` (cloud-hosted, loopback) or `--offline` from binary

### Key Technical Choices

| Layer | Choice | Notes |
|-------|--------|-------|
| Backend framework | Effect (TypeScript) | Functional effect system, not a typical Express app |
| Query engine | Tinybird (hosted) / chDB (local) | ClickHouse under the hood for both |
| Metadata store | SQLite via libSQL (Turso) | Dashboards, org config, ingest keys |
| Agent surface | MCP server in `apps/api` | 10+ tools, read-oriented |
| Deployment infra | Cloudflare Workers + D1 (Alchemy) | Per-app Alchemy runs for prd/stg/pr-preview |
| Frontend | TanStack Router + Vite | SPA, not SSR |
| Landing | Astro | Static site |
| Package manager | Bun | Monorepo with Turbor |

## Feature Inventory

### Shipped (10 of 14 roadmap items)

1. **Distributed Tracing** — waterfall, flamegraph, flow view; span attributes; trace-log correlation; root span filtering
2. **Log Management** — full-text search; severity filtering; service grouping; real-time streaming; trace correlation
3. **Metrics & Dashboards** — 20+ chart types; drag-and-drop builder; custom time ranges; dashboard sharing
4. **Service Catalog & Dependency Map** — latency percentiles per service; Apdex; dependency edges with call/error rates; commit SHA tracking
5. **Error Tracking** — smart grouping by type/message; trend detection; trace linking; spam filtering; environment filtering
6. **AI Agent MCP Server** — 10+ MCP tools; auto-diagnosis chaining; Claude Code / Cursor / Windsurf support; any MCP client
7. **Alerting & Notifications** — integrated alerting (not separate component)
8. **Custom Dashboard Builder** — drag-drop; persistent layouts
9. **Browser Session Replay** — pixel-perfect replay; session timeline; console/network capture; trace correlation via shared session ID; input masking; privacy-by-default
10. **Maple Local** — single binary; embedded ClickHouse (chDB); OTLP ingest + query API + dashboard; CLI query tools; no cloud, no auth

### In Progress (1)

- **AI Chat** — conversational AI interface for querying observability data (Q1 2026)

### Planned (1)

- **One-Click Self-Hosting Setup** — Railway template, Docker Compose, automated config (Q2 2026)

### Exploring (2)

- **API-Specific Observability Dashboard** — per-route latency, error rates, request volume, payload size (Q2 2026)
- **AI Observability** — token usage, model latency, prompt/completion pairs, cost per inference (Q3 2026)

### Notable Gaps

- **No Sentry envelope ingestion.** Maple is pure OTLP. Teams using Sentry SDKs must migrate to OTel.
- **No CLI/agent session tracing.** No mention of capturing CLI invocations, coding-agent sessions, or CI runs as first-class evidence.
- **No evidence bundles or deterministic grouping.** Errors are grouped by type/message (fingerprint-style) but there is no portable, citable evidence schema or outcome/audit graph.
- **No fixer-outcome tracking.** The MCP `propose_fix` demo shows opening a PR, but there is no closed loop of accepted/rejected/reverted fix outcomes.
- **No Rust-first capture.** TypeScript/Bun runtime; the only Rust is the FFI bridge to chDB.
- **No redaction/safety layer.** No mention of redacting PII or secrets from evidence before serving to agents.
- **No streaming/message broker.** No Iggy/Kafka-style replay or processor separation.

## What Maple Does Well

### 1. Exceptional Product Polish and UX Vision

The DESIGN.md and PRODUCT.md reveal a deeply intentional design system:

- **"The Operator Terminal"** creative north star — designed for the engineer at 2am in a dim room
- Monospace body font (Geist Mono) as the default; proportional reserved for headings only
- Flat, tonal depth system — zero shadows, depth conveyed by OKLCH lightness steps
- Severity-as-meaning color system — 6-step severity ramp is the most semantically loaded color, not the brand accent
- Explicit anti-references: no Datadog chrome, no AI-startup neon-on-black, no hero-metric templates
- 16-color categorical service palette with accessibility rules (always paired with service initial/icon)
- Every design rule is documented with named principles ("The Mono-As-Body Rule", "The Flat-By-Default Rule", "The Severity-Owns-Color Rule")

This level of design intentionality is rare in open-source observability. It's closer to Linear or Vercel's polish than to typical developer tools.

### 2. Local Mode Is Outstanding

The `maple start` single-binary experience is arguably the best local observability story in the market:

- One Bun-compiled binary + libchdb (embedded ClickHouse) — no Docker, no second language
- Full OTLP ingest on `:4318` — any OTel SDK works out of the box
- Same query engine and UI as hosted Maple, running on localhost
- Rich CLI with `maple services`, `maple traces`, `maple errors`, `maple logs`, `maple query "SQL"`
- `--offline` mode serves dashboard from binary with no internet
- Data persists in `~/.maple/data` between runs

This is a genuine adoption wedge: point your OTLP exporter at localhost and explore immediately. No competitor in the Parallax watchlist offers this.

### 3. OTel Convention Depth

The OTel conventions doc is remarkably detailed:

- Specific attribute-to-fast-column mapping (materialized views at write time)
- Fallback chains for HTTP route extraction
- Attribute prominence scoring (0-100 numeric ranking for chip strip display)
- Span-kind-specific rendering rules (Client vs Server route display)
- Sampling-aware throughput estimation with UI indicators (`~` prefix, traced-rate secondary line)
- Service map edge rules requiring `peer.service` + `db.system` + span kind

This depth shows a team that understands OTel telemetry at the protocol level, not just as a transport.

### 4. MCP Agent Surface Is Real and Documented

Unlike competitors that mention MCP in a blog post, Maple ships it:

- 10+ named tools with clear inputs/outputs
- Auto-diagnosis chaining (agent calls `system_health` → `find_errors` → `inspect_trace` automatically)
- First-class support for Claude Code, Cursor, Windsurf
- The `propose_fix` demo shows the full agent workflow from detection to PR
- The TODO mentions making MCP-to-UI query linking smoother

### 5. Browser Session Replay with Trace Correlation

The session replay feature links every user action (navigation, click, input, console, network, error) to a trace ID. This is a powerful debugging loop: see what the user did → jump to the trace → see why it broke. Few observability tools combine RUM with backend tracing this cleanly.

### 6. Kubernetes Integration Is Practical

The Helm chart approach is well-thought-out:

- OTel Operator injects `k8s.pod.name`, `k8s.node.name`, `k8s.namespace.name` at admission
- Spans carry pod identity so you can drill from slow trace → exact replica
- Three first-class views: workloads, pods, nodes
- Standard kube-state-metrics via OTLP

### 7. Honest Pricing Model

- No per-seat fees (unlimited team members)
- Per-GB ingest pricing ($0.30/GB overage)
- Interactive calculator on the pricing page
- Side-by-side cost comparisons with real numbers against Datadog/Grafana/New Relic/Dash0

### 8. Cloudflare Workers Architecture

Deploying the entire platform on Cloudflare Workers + D1 is architecturally interesting:

- Per-PR preview environments
- Stage grammar (prd/stg/pr-NUMBER)
- Alchemy infrastructure-as-code
- Single Doppler token for all secrets

This is a modern, low-ops deployment story that self-hosted users can replicate.

## What Maple Does Not Do Well

### 1. No Sentry Compatibility At All

Maple is pure OTLP. Teams currently using Sentry SDKs must rip out their instrumentation and replace it with OTel. This is a hard migration for any team with Sentry embedded in their error pipeline. Parallax explicitly targets Sentry envelope ingestion as a migration path — Maple does not.

**Implication for Parallax:** This is a genuine differentiator. The Sentry-compatible ingestion wedge that Parallax plans is a migration story Maple intentionally does not serve.

### 2. TypeScript/Bun, Not Rust

Maple's entire backend is TypeScript on Bun. For a self-hosted observability tool that needs to ingest and query billions of rows, this is a risk:

- Memory safety and performance ceiling are lower than Rust
- The only Rust code is the FFI bridge to chDB for local mode
- TypeScript effect systems (Effect-TS) add abstraction but not raw throughput

**Implication for Parallax:** The Rust-first capture quality that Parallax plans is a real architectural advantage for high-throughput ingest and low-resource self-hosting. Maple's choice of TypeScript limits its floor on resource efficiency.

### 3. Tinybird Dependency for Hosted

The hosted version depends entirely on Tinybird (managed ClickHouse). This means:

- Maple does not own its storage layer
- Query engine capabilities are bounded by Tinybird's API surface
- Cost structure includes Tinybird as a pass-through dependency
- Self-hosted users must operate their own ClickHouse

**Implication for Parallax:** Parallax's plan to own the storage adapter (GreptimeDB or ClickHouse behind a trait) gives more control over query patterns, schema evolution, and cost.

### 4. No Evidence Bundles, No Outcome Graph

Maple shows traces, logs, metrics, errors, and sessions — but it does not:

- Package correlated evidence into a portable, citable bundle
- Track what agents did with the evidence (fix attempts, outcomes)
- Build an audit graph of actions and outcomes over time
- Provide a schema that could become an interchange format

The MCP `propose_fix` demo shows an agent opening a PR, but there is no feedback loop. The agent cannot learn whether its fix was accepted, rejected, or reverted.

**Implication for Parallax:** This is Parallax's core thesis — the evidence bundle + outcome graph. Maple treats agent access as a read-only query surface. Parallax treats it as a bidirectional evidence loop. This is the sharpest distinction between the two products.

### 5. No CLI/Agent Session Capture

Maple has no mechanism for capturing:

- CLI invocations (e.g., `cargo test`, `kubectl apply`)
- Coding-agent sessions (e.g., Claude Code, Cursor agent)
- CI pipeline runs
- Deploy/change events

These are first-class signal types in Parallax's architecture. Maple only captures what flows through OTLP (application telemetry).

**Implication for Parallax:** If Parallax can capture CLI and agent sessions as evidence, it covers a signal class that Maple (and most competitors) completely ignore.

### 6. No Redaction or Safety Layer

Maple's MCP tools expose raw observability data to agents. There is:

- No PII redaction before serving to agents
- No secret masking in evidence
- No bounded evidence envelope (agents see everything in the scope of their query)
- No safety gates on what agents can read

**Implication for Parallax:** The redaction/safety layer (A6 gate) is a trust differentiator. If agents can see raw logs with secrets, organizations will gate agent access. Parallax's plan to serve bounded, redacted bundles is a security posture, not just a UX choice.

### 7. Dashboard-First, Not CLI-First

Maple's primary interface is a web dashboard. The CLI exists for local mode but is secondary. Parallax plans CLI-first, HTTP underneath.

**Implication for Parallax:** The CLI-first approach serves a different workflow — automation, scripting, agent chaining — that a dashboard-first tool doesn't naturally support.

### 8. Early Maturity Signals

- v0.0.11 (9 releases) — still in rapid iteration
- 382 stars, 25 forks — small community
- Only 2 items on the TODO list (dashboard MCP fix, MCP-to-UI query linking)
- No published benchmarks for ingest throughput or query latency
- No mention of high-availability, clustering, or multi-region for self-hosted
- The self-hosting setup is "planned" (Q2 2026 roadmap) — not yet documented

## Backend & Data Flow

See [backend-and-data-flow.md](backend-and-data-flow.md) for the side-by-side. Maple summary
(all perf figures are vendor claims):

- **Engine:** ClickHouse — **Tinybird-managed** (cloud) / **embedded chDB** (local single binary).
  Metadata in **libSQL/Turso**. **No broker.**
- **Flow:** `OTel SDK ─OTLP─► apps/ingest (auth, org enrich) ─► OTel Collector ─► ClickHouse (Tinybird cloud / chDB local)`;
  query: `apps/web ─► apps/api (Effect, Tinybird proxy) ─► ClickHouse`. Metadata stays off the telemetry path.
- **Write/read:** no Maple-level WAL — batching/flush/compaction delegated to ClickHouse MergeTree; reads are
  columnar scans authored as Tinybird Pipes (cloud) / embedded engine (local). No inverted index.
- **Throughput (vendor):** "12.8B rows in 198 ms" (query latency, not ingest); no published ingest rate or compression.
- **Designed for:** ClickHouse-grade scan speed without operating ClickHouse + a genuine single-binary local story.
  **Not for:** self-controlled engine at scale or broker-buffered backpressure — the fast path is coupled to hosted Tinybird.

## Comparison: Maple vs Parallax

| Dimension | Maple | Parallax |
|-----------|-------|----------|
| **Product category** | Full observability platform (traces, logs, metrics, errors, sessions, K8s) | Evidence context engine (not a dashboard suite) |
| **Primary language** | TypeScript (Bun) | Rust (Tokio) |
| **Ingest protocols** | OTLP only | Sentry envelope + OTLP |
| **Storage** | ClickHouse (Tinybird/chDB) | GreptimeDB (lean) / ClickHouse (fallback) behind adapter |
| **Metadata** | SQLite / Turso | Turso (prototype) / Postgres (production) |
| **Agent surface** | MCP with 10+ read tools | CLI first, HTTP underneath, MCP (read-only, after safety gates) |
| **Evidence bundles** | None — raw query results | Bounded, redacted, citable, portable bundles |
| **Outcome tracking** | None | Fix-outcome loop (accepted/rejected/reverted) |
| **Session capture** | Browser session replay only | CLI runs, agent sessions, CI runs, deploy/change events |
| **Redaction** | Input masking in browser sessions only | Structured redaction before agent access (A6 gate) |
| **Dashboard** | Full SPA dashboard with service map, flamegraphs, etc. | Not a dashboard — CLI/HTTP/MCP bundle delivery |
| **Kubernetes** | First-class Helm chart + K8s views | Not planned as first-class feature |
| **Local mode** | Excellent single-binary local experience | Tiny tier (single binary, no broker) planned |
| **License** | FSL-1.1 | TBD (open-source core planned) |
| **Pricing** | $19-$39/mo SaaS; self-host free | TBD (managed cloud + enterprise planned) |
| **Maturity** | v0.0.11, shipping, has paying customers | Research stage, no code yet |

## What Parallax Should Learn From Maple

### 1. The Local Experience Is a Model

Maple Local proves that a single-binary, zero-config local observability experience is a powerful adoption wedge. Parallax's "tiny tier" should aim for the same friction: `parallax start` → point Sentry SDK or OTLP exporter → see evidence immediately. This is more important than any dashboard feature.

### 2. OTel Convention Depth Matters

Maple's detailed attribute mapping, fast-column extraction, fallback chains, and prominence scoring show that understanding the OTel data model at protocol depth is table stakes for any OTLP-native tool. Parallax should invest in equivalent convention documentation before building the storage adapter.

### 3. Design System as Product Differentiator

Maple's DESIGN.md is a competitive document. The "Operator Terminal" aesthetic — monospace body, flat surfaces, severity-as-meaning color, no shadows, no hero metrics — is deliberately different from every other observability tool. Even if Parallax is CLI-first, the design philosophy (precision, density, calm) should inform the CLI output, the bundle schema, and any future web surface.

### 4. MCP Is Table Stakes — Maple Proves It

Maple's 10+ MCP tools with auto-diagnosis chaining show what "agent-native observability" looks like in practice. Parallax's MCP surface must ship with comparable tool depth, even if the tools serve bundles instead of raw query results.

### 5. Sampling-Aware Throughput Estimation

Maple's approach to detecting probability-based sampling from `TraceState` and extrapolating throughput is clever. If Parallax ingests sampled OTLP telemetry, it needs an equivalent mechanism to avoid underestimating error rates and throughput.

## Threat Assessment for Parallax

### How Much Does Maple Pressure the Parallax Wedge?

**Moderate pressure, not existential.**

Maple pressures the "OTLP-native, self-hostable, open-source observability" positioning but does NOT pressure Parallax's specific wedge because:

1. **No Sentry path.** Teams on Sentry SDKs cannot migrate to Maple without rewriting instrumentation. Parallax's Sentry-envelope ingestion is a migration story Maple deliberately does not serve.

2. **No evidence bundles.** Maple serves raw query results to agents. It does not package correlated signals into a portable, citable bundle. This is Parallax's core thesis.

3. **No outcome loop.** Maple has no mechanism for tracking fix outcomes. Without this, agent sessions are stateless — the system cannot learn from past fixes.

4. **No CLI/agent/CI session capture.** Maple only captures application telemetry via OTLP. It does not capture CLI runs, coding-agent sessions, CI runs, or deploy events.

5. **No redaction/safety layer.** Maple exposes raw data to agents without redaction. This limits enterprise adoption of the agent surface.

### Where Maple Does Pressure Parallax

1. **"Simple self-hosted observability" positioning.** Maple is already live, shipping, and priced. Any marketing claim about "simpler than Sentry self-hosted" must now account for Maple.

2. **Local mode as adoption wedge.** Maple Local is the best local observability experience available. Parallax's tiny tier must match or exceed this friction level.

3. **MCP tool depth.** Maple ships 10+ MCP tools. If Parallax launches MCP with fewer tools or less auto-diagnosis capability, it will look behind.

4. **Design polish.** Maple's UI is best-in-class for open-source observability. Any web surface Parallax builds will be compared against it.

5. **ClickHouse-based query speed.** Maple queries "billions of rows in milliseconds." If Parallax's GreptimeDB adapter cannot match this, the evidence-bundle retrieval story weakens.

### Watch Triggers

Re-evaluate if Maple:

- Ships Sentry envelope ingestion (would close the migration-path differentiator)
- Adds evidence bundles or outcome tracking (would close the core-thesis differentiator)
- Adds CLI/agent session capture (would close the signal-class differentiator)
- Reaches v1.0 with documented self-hosting (would pressure the self-hosted positioning)
- Grows community beyond 1,000 stars (would indicate adoption velocity)

## What Parallax Should Do Differently Than Maple

1. **Start with Sentry SDK compatibility, not OTLP-only.** Maple's biggest gap is the Sentry migration wall. Parallax should make it possible to point a Sentry SDK at Parallax and get immediate value — then layer OTLP correlation on top.

2. **Build the evidence bundle schema before the MCP tools.** Maple built MCP tools that serve raw query results. Parallax should build the bundle schema first, then make every MCP tool serve bundles. This means every agent interaction is citable, auditable, and safe.

3. **Track outcomes from day one.** Every agent fix attempt, acceptance, rejection, and revert should be recorded. This is the data moat — the failure/fixer-outcome corpus that compounds with usage.

4. **Capture the signals Maple ignores.** CLI runs, agent sessions, CI runs, deploy events. These are the signals that distinguish an evidence engine from a telemetry backend.

5. **Redact before serving.** Every bundle served to an agent should be redacted. This is a security posture, not a feature — and Maple does not have it.

6. **Stay CLI-first.** Maple is dashboard-first. The CLI is secondary. Parallax should keep the CLI as the primary interface and make every bundle available via CLI, HTTP, and MCP simultaneously.

7. **Rust matters for the target persona.** Self-hosted teams running Parallax on a single VPS or alongside their application benefit from Rust's resource efficiency. TypeScript/Bun is fine for Maple's Cloudflare Workers deployment model, but it limits Maple's self-hosted ceiling.

## Summary Verdict

Maple is the most polished open-source observability platform in the market right now. Its design system, local-mode experience, OTel convention depth, and MCP tool surface are all best-in-class. It is a credible competitor to Datadog and Grafana Cloud for teams that want open-source, self-hostable observability.

However, Maple is a **full-stack observability platform**, not an evidence context engine. It does not serve Parallax's specific wedge: Sentry-compatible error ingest → OTLP correlation → portable evidence bundles → agent outcome tracking → redacted delivery. These are deliberate product-shape choices, not feature gaps — Maple has chosen to be OTLP-only, dashboard-first, and read-only-agent.

The key insight for Parallax is that **Maple validates the market for open, self-hosted, agent-native observability** while simultaneously **proving that the evidence-bundle + outcome-graph differentiator is still unoccupied**. No competitor in the current landscape — Maple included — combines Sentry-envelope ingestion, deterministic evidence graphs, bounded redacted bundles, fix-outcome tracking, and CLI/agent/CI session capture into one product.

The risk is that Maple's execution speed, design polish, and local-mode wedge give it a distribution advantage that could make "open-source OTel observability" synonymous with "Maple" before Parallax ships. The counter is that Parallax is not building an observability platform — it is building the evidence substrate underneath one.
