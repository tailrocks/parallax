# Parallax vs Competitors — Unbiased Comparison Program

Build and permanently maintain a single canonical folder that compares Parallax
to every relevant observability / debugging / investigation product on the
market — open source and closed source — with **no pro-Parallax bias**. The
record must reflect reality, not marketing. Where Parallax is behind, behind is
written. Where a competitor is genuinely better, better is written. Honest
comparison is the entire point; a comparison that always favors Parallax is a
failure state.

## Where this writes

Canonical home: `docs/research/market/competitors/`.

Structure:

- `README.md` — the overview. A wide **feature-presence matrix** across the
  whole ecosystem (Parallax + every competitor, open and closed source): which
  product has which capability and which does not. Kept readable, not
  exhaustive prose — the matrix is the point. Links into every per-product
  deep-dive and into the refreshed source research.
- `parallax-vs-<product>.md` — one deep, one-to-one comparison per product
  (e.g. `parallax-vs-datadog.md`, `parallax-vs-sentry.md`,
  `parallax-vs-grafana.md`, `parallax-vs-honeycomb.md`,
  `parallax-vs-new-relic.md`, `parallax-vs-signoz.md`,
  `parallax-vs-openobserve.md`, `parallax-vs-coroot.md`, and so on for every
  product in the comparison set). Each is a real left/right comparison: who has
  what, who implements it better, on what axis, with what evidence.
- `comparison-set.md` — the roster of products compared, each with a one-line
  definition (what it is, license model, primary signal focus), kept current as
  the market shifts. This is the authoritative list of what is in scope.
- `PROGRESS.md` — the living checklist / status board. Tracks, per product and
  per matrix cell: verification state (unverified / stale / verified), last
  verified date, source links, open uncertainties, missing deep-dives, and the
  next highest-value gap. This is what makes the program resumable and keeps
  unfinished work, weak evidence, and future verification needs visible across
  passes. Update it on every pass.

Existing market notes — `competitive-comparison-matrix.md`,
`observability-feature-matrix.md`, `closest-to-parallax-ranked.md`,
`alternatives-deep-analysis.md`, `landscape.md`, `competitor-watch.md`, and the
per-tool deep-dives (`sentry-deep-research.md`, `signoz-deep-research.md`, etc.)
— are **sources**, not the destination. Treat them as leads. Verify, refresh,
and migrate the still-true content into the new folder; leave a one-line link
at each source pointing into `competitors/`. Do not duplicate living data in two
places — the new folder is canonical, old notes become pointers or get pruned.

`docs/research/README.md` must link to `competitors/` as the comparison entry
point.

## What the comparison must contain

Each `parallax-vs-<product>.md` deep-dive compares across the full product
surface, at minimum:

- **Signal coverage** — logs, metrics, traces, profiles, errors, LLM/agent
  spans, test results, CI/run evidence. Which signals each side ingests,
  stores, and correlates.
- **Ingestion & transport** — OTLP support, native vs custom tables, agent
  auto-instrumentation, SDK breadth, ingestion cardinality handling, retention
  model.
- **Storage architecture** — backing store, columnar vs other, cold storage,
  indexing, cost-per-byte behavior at scale. Parallax is GreptimeDB (telemetry)
  + Turso (metadata); name the competitor's equivalent honestly.
- **Query & correlation** — query language, cross-signal joins, trace-to-log,
  metric-to-trace, AI-agent run reconstruction (trace_id / run_id / invocation
  stitching), evidence pinning.
- **Dashboards & visualization** — dashboard builder, panels/widgets, service
  maps, graph rendering, custom vs templated.
- **Alerting & on-call** — alert rules, anomaly detection, routing, noise
  reduction, incident management.
- **Profiling** — continuous profiling language coverage, flamegraphs,
  allocation/lock profiling, overhead.
- **Developer experience** — SDK ergonomics, docs quality, quickstart time,
  local dev loop, error messages, first-value-to-insight time.
- **AI-native / agent-context story** — is the product a context engine for
  autonomous agents, or a human dashboard? This is Parallax's wedge; assess
  each competitor's real position, not their press releases. Include any
  vendor AI-assisted triage, query, or root-cause features.
- **Architecture & deployment model** — self-hosted vs SaaS vs hybrid,
  single-binary vs distributed, agent/collector topology, multi-tenancy.
- **Operational footprint** — deploy complexity, resource cost, operator
  burden, day-2 operations, upgrade model.
- **Scalability & performance** — verified or cited throughput, ingestion
  rate, query latency, cardinality ceiling, known limits at scale. Prefer
  measured numbers; otherwise cite vendor-published limits with date.
- **Security** — auth (SSO/SAML/OIDC), RBAC granularity, secret management,
  audit logs, network/transport security, sandboxing.
- **Privacy & compliance** — data residency, PII scrubbing, retention/PII
  controls, GDPR/SOC2/HIPAA/ISO27001 posture where applicable, data ownership.
- **Openness, licensing & vendor lock-in** — open source / source-available /
  closed; license (Apache-2.0, ELv2, BSL, proprietary); self-host viability;
  export/migration cost; proprietary query language or data format lock-in.
- **Extensibility** — plugins, custom integrations, APIs, pipeline/processor
  model, programmable alerts, webhooks, ecosystem of integrations.
- **Pricing & economics** — real numbers where public (per-host, per-event,
  per-GB, per-span, free tiers, committed-use discounts). If no public number
  exists, say so explicitly and reference the closest proxy with its date.

Every claim carries a source link and a research date. Re-verify on each pass.

## No-bias rules

- Default assumption: the competitor may be better until evidence says
  otherwise. Do not start from "Parallax wins."
- Parallax's own limitations (immaturity, missing signals, single-tenant
  assumptions, GreptimeDB/Turso constraints) are stated plainly, not hidden.
- "Better" is always scoped to a named axis with evidence, never a vague
  verdict.
- Marketing language from any vendor is a lead, not a fact. Confirm against
  docs, source, changelogs, pricing pages, or reproducible measurement.
- When a claim cannot be proven, mark it unproven and say what measurement
  would prove it. Never present an unproven claim as settled.

## Real numbers

Prefer hard, current, sourced numbers: pricing tiers, ingest throughput,
query latency, retention defaults, cardinality limits, feature limits. When a
number exists, cite the source and date. When no public number exists, write
that explicitly, give the best-grounded proxy, and note why a direct number is
unavailable. A referenced "we could not measure this because X" is acceptable;
an invented or stale number is not. For every derived or estimated figure,
state the method, the assumptions, the source limitations, and a confidence
level (high / medium / low) so a reader can weigh it; never present an estimate
as a measured fact.

Where a comparison depends on storage/ingest/query performance and a real
measurement is feasible, flag it as benchmark-dependent and mark it unproven
until measured by the benchmark program — do not fabricate a figure to fill the
cell.

## Comparison set

Cover the full ecosystem, open and closed source, expanding as the market
moves. Non-exhaustive starting roster — verify each still exists and still
matters on every pass, add new entrants, retire dead ones:

- Closed / commercial: Datadog, Sentry, Grafana Cloud / LGTM, Honeycomb, New
  Relic, Dynatrace, Splunk Observability, Elastic Observability, Sumo Logic,
  Chronosphere, Observe, Axiom, Mezmo, Tracelo.
- Open source: SigNoz, OpenObserve, Coroot, Highlight.io, Uptrace, HyperDX,
  Odigos; component-level (Prometheus/Mimir, Loki, Tempo/Jaeger, Vector,
  Fluent Bit).
- AI/LLM-agent observability: Langfuse, LangSmith, Arize Phoenix, PostHog,
  Helicone, Braintrust — directly relevant to Parallax's agent-context wedge.

If a meaningful product is missing, add it to `comparison-set.md` and write its
deep-dive. Scope is the whole market, not a fixed list.

## One pass

Before any change: **read the existing record first.** On the first pass read
all of `docs/research/` (market, architecture, decisions, storage, security,
00-vision, README) and all of `prompts/`; on later passes re-read the prompt,
`competitors/`, and any source notes touched that pass. Treat every existing
finding (in `competitors/`, in the legacy market notes, and in the rest of
`docs/research/`) as a hypothesis, not settled fact — many files carry an old
research date and the market has moved. Each pass:

1. Re-read this prompt, `competitors/README.md`, `comparison-set.md`, and
   `PROGRESS.md`.
2. Pick the weakest, stalest, least-sourced, most strategically important, or
   most suspicious comparison cell, product, or missing deep-dive. Use
   `PROGRESS.md` as the work queue. Prioritize: missing deep-dives for products
   in the set, then unproven cells in the overview matrix, then aging
   per-product claims, then inconsistent terminology across files.
3. Re-research it from current primary sources — vendor docs, pricing pages,
  changelogs, GitHub repos/releases, the per-tool deep-dives in
  `docs/research/market/`, and current web sources.
4. Reconsider whether the comparison still reflects reality, with the no-bias
   rules enforced.
5. Update or create the focused file (`README.md` matrix, a
   `parallax-vs-<product>.md`, or `comparison-set.md`); refresh the matching
   legacy source note(s) in `docs/research/market/` and leave a pointer into
   `competitors/`. Correct outdated or unsupported claims wherever found, not
   only in the file under focus.
6. Update `PROGRESS.md`: flip the touched cell/product to verified with today's
   date and source links, record any new uncertainty or open question, and
   queue the next gap.
7. Update `docs/research/README.md`, `PROJECT_STRUCTURE.md`, and this prompt
   when the folder shape or scope changes.
8. Commit and push, then continue to the next highest-value gap.

Depth over speed. A single cell verified against three current primary sources,
with bias actively checked, beats a full matrix of unverified inherited marks.
Spend the time. Re-check what looks settled. Re-measure what drifted. The goal
is a comparison record so thorough and so honest that it is trusted as the
external view of where Parallax actually stands.

## Prompt maintenance rule

This prompt is living operator intent. When the operator names a new competitor
to add, changes the comparison axes, tightens or loosens the no-bias rules,
redefines the folder, or changes which products matter, update this file in the
same change. Do not keep direction changes only in chat or only in generated
notes — the prompt must stay aligned so future autonomous runs continue from the
latest operator intent.

## Stop condition

Only the operator stops or replaces this program. There is always a next
highest-value gap: a missing product, an unproven cell, an aging number, a new
market entrant, a competitor release that shifts a verdict. Comparison quality,
source trustworthiness, coverage of the full ecosystem, and freedom from bias
are the success measures — and they can always be improved.
