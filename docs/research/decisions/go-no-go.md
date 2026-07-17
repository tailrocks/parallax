# Parallax Go / No-Go Verdict

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25 · Restructured into a decision record 2026-05-29

> **Current stack authority (operator, 2026-06-12): GreptimeDB + Turso are
> mandatory.** ClickHouse and Postgres remain research comparators only. This GO
> verdict authorizes the product direction, not an implementation queue or an
> alternate backend. Plan 093 is closed; supported server hardening remains in
> active plan 115; all other unfinished implementation must live in `plans/`.
>
> **Implementation status (2026-07-17): the narrow product described here has
> shipped.** Parallax now has native-table OTLP gRPC/HTTP ingest, Sentry envelope
> ingest, GitHub webhooks (`deployment`/`deployment_status`/`workflow_job`), deterministic issues, bundle-v1/envelope-v2 evidence,
> the code-first GraphQL API, CLI, alerting, dashboards, investigations, SQL,
> and the TanStack UI. Local-stdio MCP (`parallax-mcp`) graduated plan 112
> (DONE); remote MCP remains deferred to Plan 109.

> **Decision record — Status: GO (narrow product).** Build the open-source, Rust-first,
> self-hosted execution-context engine; do **not** build the generic AI-RCA chatbot,
> dashboard suite, or autonomous SRE. Reverse only if a kill criterion (below) triggers.
> The storage engine is a separate decision — **GreptimeDB is mandatory** —
> see [storage-engine.md](storage-engine.md). The adversarial counterweight is
> [risks-and-bear-case.md](risks-and-bear-case.md); the synthesis + coverage map is
> [strategic-coverage.md](strategic-coverage.md). Full gate answers and evidence follow.
>
> **2026-05-29 skeptical re-assessment** ([skeptical-reassessment-2026-05.md](skeptical-reassessment-2026-05.md)):
> GO survives but narrower — the technical wedge is still unoccupied, but "Sentry migration" and
> "simpler than Sentry" are no longer differentiators, **A1 (bundle beats raw context) is now the #1
> existential gate**, and open self-hosted is structurally non-paying (plan a managed/enterprise tier).
>
> **Pass 152 (2026-07-17) — GO reaffirm against indefinite-research evidence.**
> Status remains **GO (narrow product)**. Kill criteria that would reverse GO
> were re-checked in recent passes; **none fired**.
>
> **Pass 171 (2026-07-18) — GO reaffirm** after passes **156–170**. **Still GO.**
>
> **Pass 197 (2026-07-18) — GO reaffirm** after passes **171–196**. **Still GO.**
>
> **Pass 221 (2026-07-18) — GO reaffirm** after passes **197–220**. **Still GO.**
>
> **Pass 230 (2026-07-18) — GO reaffirm** after passes **221–229**. **Still GO
> (narrow product).** Kill criteria **still unfired** (desk rechecks only;
> empirical A1/A2/A4/A6 still open, not failed):
>
> | Kill / reverse trigger | Latest recheck | Fired? |
> | --- | --- | --- |
> | A1: bundles do not beat raw context | Pass **228** — golden ok; no result ledger | **No** |
> | A2: no paying segment | Pass **222/228** — **0** interview rows; desk triangle holds (pass **216**) | **No** |
> | Full wedge closed by peer | Pass **223–225** cohort pins — Traceway/TMA1/Bugsink/etc. **not** full combo | **No** |
> | OTel commoditizes evidence-bundle schema | Pass **224** — #1185 idle; OCSF 1.8.0 | **No** |
> | Incumbent self-host Seer-class AI | Pass **226/229** — Seer closed; Grafana Assistant hybrid Cloud | **No** |
> | App-code auto-merge commodity | Pass **227** — Bits still never auto-merges | **No** |
> | A4 real messy telemetry reliability | Pass **226** — no `correlation-reliability-runs/` | **No** |
> | A6 agent-visible mixed redaction | Pass **226** — canary ok; mixed open | **No** |
> | A3 external schema adoption | Pass **226** — `$id` search total 6 in-tree | **No** |
> | Sentry OTLP metrics GA | Pass **224** — still unsupported | **No** |
>
> **Pass 242 (2026-07-18) — GO reaffirm (A2 desk only).** Monetization triangle
> primary re-scrape **holds** (Grafana / SigNoz / OpenObserve). A2 kill still
> **unfired**: **0** interview rows (open, not failed); survivors still use
> usage-Cloud + EE/AI gates. Full kill table not re-walked this pass (wedge =
> pass 241; Seer = pass 238). **Still GO (narrow product).**
>
> **Pass 243 (2026-07-18) — GO reaffirm (schema commoditization).** OTel #1185
> still open/idle; OCSF GA still 1.8.0. Kill "OTel commoditizes evidence-bundle
> schema" still **unfired**. **Still GO (narrow product).**
>
> **Pass 245 (2026-07-18) — GO reaffirm (incumbent AI + Bits).** Seer still
> closed on self-host; Grafana Assistant still hybrid Cloud backend; Bits Code
> still **never auto-merges**. Related kills **unfired**. **Still GO (narrow
> product).**
>
> **Pass 246 (2026-07-18) — GO reaffirm (A1 hygiene).** A1 still
> **`not_measured`** (open, not failed): golden ok; no comparative result
> ledger. Kill "bundles do not beat raw" **unfired** until measured. **Still
> GO (narrow product).**
>
> **Pass 247 (2026-07-18) — GO reaffirm (A4/A6 hygiene).** A4 still
> **`not_measured`** (no reliability-runs ledger). A6 split holds (canary ok;
> mixed open). Related kills **unfired** (open ≠ failed). **Still GO (narrow
> product).**
>
> **Pass 249 (2026-07-18) — GO reaffirm (wedge + Sentry OTLP).** Traceway still
> lacks full combo; Sentry still no OTLP metrics. Wedge-close and Sentry OTLP
> metrics kills **unfired**. **Still GO (narrow product).**
>
> **Pass 250 (2026-07-18) — GO reaffirm (loop hygiene).** Detect trigger ledger
> still absent; `fixer_outcome` unit **3/3** ok; live replay open. Loop-stage
> product claims still open (not failed). **Still GO (narrow product).**
>
> **Pass 251 (2026-07-18) — GO reaffirm (OPW air-gap).** Datadog OPW still
> route-to-destinations Worker, not self-hosted Bits/store. Air-gap combination
> differentiator still holds (combination claim; A1 open). **Still GO (narrow
> product).**
>
> **Pass 257 (2026-07-18) — GO composite reaffirm** after passes **242–256**.
> **Still GO (narrow product).** Kill criteria **still unfired** (desk + unit
> hygiene only; empirical A1/A2/A4/A6 mixed still **open**, not failed):
>
> | Kill / reverse trigger | Latest recheck | Fired? |
> | --- | --- | --- |
> | A1: bundles do not beat raw context | Pass **246** — golden ok; no result ledger | **No** (open) |
> | A2: no paying segment | Pass **253** — **0** interview rows; desk triangle **242** holds | **No** (open) |
> | Full wedge closed by peer | Pass **249/254–256** — Traceway/error peers/LLMOps/TMA1/Odigos **not** full combo | **No** |
> | OTel commoditizes evidence-bundle schema | Pass **243** — #1185 idle; OCSF 1.8.0 | **No** |
> | Incumbent self-host Seer-class AI | Pass **245** — Seer closed; Grafana Assistant hybrid Cloud | **No** |
> | App-code auto-merge commodity | Pass **245** — Bits still never auto-merges | **No** |
> | A4 real messy telemetry reliability | Pass **247** — no `correlation-reliability-runs/` | **No** (open) |
> | A6 agent-visible mixed redaction | Pass **247** — canary ok; mixed open | **No** (open) |
> | Sentry OTLP metrics GA | Pass **249** — still unsupported | **No** |
> | Datadog OPW = self-host Bits store | Pass **251** — still route-only Worker | **No** |
> | TMA1 prod-incident collision | Pass **256** — **23rd UNFIRED** | **No** |
>
> **Pass 261 (2026-07-18) — GO reaffirm (BYOC Logs).** Datadog BYOC Logs is
> hybrid customer log store + SaaS UI/AI — **not** offline Seer/Bits and **not**
> OPW. Air-gap combination differentiator still holds. **Still GO (narrow
> product).**
>
> **Pass 268 (2026-07-18) — GO reaffirm (schema commoditization).** OTel #1185
> still open/idle; OCSF GA still 1.8.0. Kill "OTel commoditizes evidence-bundle
> schema" still **unfired**. **Still GO (narrow product).**
>
> **Pass 269 (2026-07-18) — GO reaffirm (Bits + Sentry OTLP).** Bits Code still
> never auto-merges; Sentry still no OTLP metrics. Related kills **unfired**.
> **Still GO (narrow product).**
>
> **Pass 273 (2026-07-18) — GO reaffirm (A3 + Seer).** External evidence-bundle
> schema adoption still **none**; Seer still closed on self-host. Related kills
> **unfired** (open ≠ failed for A3). **Still GO (narrow product).**
>
> **Pass 275 (2026-07-18) — GO composite reaffirm** after passes **268–274**.
> **Still GO (narrow product).** Kill criteria **still unfired** (desk + unit
> hygiene; empirical A1/A2/A4/A6 mixed still **open**, not failed):
>
> | Kill / reverse trigger | Latest recheck | Fired? |
> | --- | --- | --- |
> | A1: bundles do not beat raw context | Pass **265** — golden ok; no result ledger | **No** (open) |
> | A2: no paying segment | Pass **271** — **0** interview rows; desk triangle holds | **No** (open) |
> | Full wedge closed by peer | Pass **270/272/274** — TMA1/error peers/Traceway **not** full combo | **No** |
> | OTel commoditizes evidence-bundle schema | Pass **268** — #1185 idle; OCSF 1.8.0 | **No** |
> | Incumbent self-host Seer-class AI | Pass **273/274** — Seer closed; Assistant hybrid Cloud | **No** |
> | App-code auto-merge commodity | Pass **269** — Bits still never auto-merges | **No** |
> | A4 real messy telemetry reliability | Pass **267** — no reliability-runs | **No** (open) |
> | A6 agent-visible mixed redaction | Pass **267** — canary ok; mixed open | **No** (open) |
> | A3 external schema adoption | Pass **273** — still none | **No** (open) |
> | Sentry OTLP metrics GA | Pass **269** — still unsupported | **No** |
> | TMA1 prod-incident collision | Pass **270** — **24th UNFIRED** | **No** |
>
> **Pass 283 (2026-07-18) — GO reaffirm (incumbent kills).** Seer closed; Bits
> never auto-merges; Sentry no OTLP metrics; self-host still 26.7.0. Related
> kills **unfired**. Pass 281 desk triangle holds; pass 282 TMA1 **25th
> UNFIRED**. **Still GO (narrow product).**
>
> **Pass 288 (2026-07-18) — GO composite reaffirm** after passes **281–287**.
> **Still GO (narrow product).** Kill criteria **still unfired**:
>
> | Kill / reverse trigger | Latest recheck | Fired? |
> | --- | --- | --- |
> | A1: bundles do not beat raw context | Pass **284** — golden ok; no result ledger | **No** (open) |
> | A2: no paying segment | Pass **286** — **0** interview rows; desk triangle **281** holds | **No** (open) |
> | Full wedge closed by peer | Pass **282/287** — Traceway/TMA1/error peers/Noz Cloud | **No** |
> | OTel commoditizes evidence-bundle schema | Pass **285** — #1185 idle; OCSF 1.8.0 | **No** |
> | Incumbent self-host Seer-class AI | Pass **283** — Seer closed; Assistant hybrid | **No** |
> | App-code auto-merge commodity | Pass **283** — Bits never auto-merges | **No** |
> | A4 real messy telemetry reliability | Pass **284** — no reliability-runs | **No** (open) |
> | A6 agent-visible mixed redaction | Pass **286** — canary ok; mixed open | **No** (open) |
> | A3 external schema adoption | Pass **284** — still none | **No** (open) |
> | Sentry OTLP metrics GA | Pass **283** — still unsupported | **No** |
> | TMA1 prod-incident collision | Pass **282** — **25th UNFIRED** | **No** |
>
> **Pass 295 (2026-07-18) — GO composite reaffirm** after passes **288–294**.
> **Still GO (narrow product).** Kill criteria **still unfired**:
>
> | Kill / reverse trigger | Latest recheck | Fired? |
> | --- | --- | --- |
> | A1: bundles do not beat raw context | Pass **290** — golden ok; no result ledger | **No** (open) |
> | A2: no paying segment | Pass **286/281** — **0** interview rows; desk triangle holds | **No** (open) |
> | Full wedge closed by peer | Pass **292/293** — Traceway/TMA1 **26th UNFIRED**/cohort | **No** |
> | OTel commoditizes evidence-bundle schema | Pass **285** — #1185 idle; OCSF 1.8.0 | **No** |
> | Incumbent self-host Seer-class AI | Pass **294** — Seer closed; Assistant hybrid (291) | **No** |
> | App-code auto-merge commodity | Pass **294** — Bits never auto-merges | **No** |
> | A4 real messy telemetry reliability | Pass **284** — no reliability-runs | **No** (open) |
> | A6 agent-visible mixed redaction | Pass **294** — canary ok; mixed open | **No** (open) |
> | A3 external schema adoption | Pass **284** — still none | **No** (open) |
> | Sentry OTLP metrics GA | Pass **294** — still unsupported | **No** |
> | TMA1 prod-incident collision | Pass **293** — **26th UNFIRED** | **No** |
> | Datadog OPW = Bits store | Pass **291** — still route-only | **No** |
>
> **Pass 303 (2026-07-18) — GO composite reaffirm** after passes **296–302**.
> Live primary re-fetch this pass on kill-adjacent watches. **Still GO
> (narrow product).** Kill criteria **still unfired** (desk + unit hygiene;
> empirical A1/A2/A4/A6 mixed still **open**, not failed):
>
> | Kill / reverse trigger | Latest recheck | Fired? |
> | --- | --- | --- |
> | A1: bundles do not beat raw context | Pass **303** — golden **ok**; no comparative result ledger | **No** (open) |
> | A2: no paying segment | Pass **301** — **0** interview rows; desk triangle holds | **No** (open) |
> | Full wedge closed by peer | Pass **303** — Traceway **1,024★**/v1.9.1; TMA1 **alpha12** **28th UNFIRED** | **No** |
> | OTel commoditizes evidence-bundle schema | Pass **303** — #1185 idle (`updated_at` 2025-10-24); OCSF **1.8.0** | **No** |
> | Incumbent self-host Seer-class AI | Pass **303** — Seer closed; Assistant hybrid Cloud backend | **No** |
> | App-code auto-merge commodity | Pass **303** — Bits still **never auto-merges** PRs/MRs | **No** |
> | A4 real messy telemetry reliability | Pass **302** — no reliability-runs ledger | **No** (open) |
> | A6 agent-visible mixed redaction | Pass **300** — canary ok; mixed open | **No** (open) |
> | A3 external schema adoption | Pass **284** — still none | **No** (open) |
> | Sentry OTLP metrics GA | Pass **303** — still unsupported | **No** |
> | TMA1 prod-incident collision | Pass **303** — **28th UNFIRED** | **No** |
> | Datadog OPW = Bits store | Pass **303** — still route-to-destinations Worker | **No** |
>
>
> **Pass 310 (2026-07-18) — GO composite reaffirm** after passes **303–309**.
> **Still GO (narrow product).** Kill criteria **still unfired** (desk + unit
> hygiene; empirical A1/A2/A4/A6 mixed still **open**, not failed):
>
> | Kill / reverse trigger | Latest recheck | Fired? |
> | --- | --- | --- |
> | A1: bundles do not beat raw context | Pass **304** — golden ok; no result ledger | **No** (open) |
> | A2: no paying segment | Pass **305** — **0** interview rows; desk triangle holds | **No** (open) |
> | Full wedge closed by peer | Pass **308** — Traceway/TMA1 **29th UNFIRED**/Bugsink | **No** |
> | OTel commoditizes evidence-bundle schema | Pass **306** — #1185 idle; OCSF 1.8.0 | **No** |
> | Incumbent self-host Seer-class AI | Pass **309** — Seer closed; Assistant hybrid | **No** |
> | App-code auto-merge commodity | Pass **309** — Bits never auto-merges | **No** |
> | A4 real messy telemetry reliability | Pass **307** — no reliability-runs | **No** (open) |
> | A6 agent-visible mixed redaction | Pass **306** — canary ok; mixed open | **No** (open) |
> | A3 external schema adoption | Pass **307** — still none | **No** (open) |
> | Sentry OTLP metrics GA | Pass **309** — still unsupported | **No** |
> | TMA1 prod-incident collision | Pass **308** — **29th UNFIRED** | **No** |
> | Datadog OPW = Bits store | Pass **303** — still route-only | **No** |
> | Detect/loop product gate | Pass **310** — no Detect ledger; `fixer_outcome` **3/3** | **No** (open) |
>
>
> **Pass 319 (2026-07-18) — GO composite reaffirm** after passes **310–318**.
> **Still GO (narrow product).** Kill criteria **still unfired**:
>
> | Kill / reverse trigger | Latest recheck | Fired? |
> | --- | --- | --- |
> | A1: bundles do not beat raw context | Pass **313** — golden ok; no result ledger | **No** (open) |
> | A2: no paying segment | Pass **315** — **0** interview rows; desk triangle **305** | **No** (open) |
> | Full wedge closed by peer | Pass **317** — Traceway/TMA1 **30th UNFIRED**/Bugsink | **No** |
> | OTel commoditizes evidence-bundle schema | Pass **316** — #1185 idle; OCSF 1.8.0 | **No** |
> | Incumbent self-host Seer-class AI | Pass **318** — Seer closed; Assistant hybrid (314) | **No** |
> | App-code auto-merge commodity | Pass **318** — Bits never auto-merges | **No** |
> | A4 real messy telemetry reliability | Pass **319** — no reliability-runs | **No** (open) |
> | A6 agent-visible mixed redaction | Pass **315** — canary ok; mixed open | **No** (open) |
> | A3 external schema adoption | Pass **307** — still none | **No** (open) |
> | Sentry OTLP metrics GA | Pass **318** — still unsupported | **No** |
> | TMA1 prod-incident collision | Pass **317** — **30th UNFIRED** | **No** |
> | Datadog OPW = Bits store | Pass **314** — still route-only | **No** |
> | Detect/loop product gate | Pass **319** — no Detect ledger; `fixer_outcome` **3/3** | **No** (open) |
>
>
> **Pass 325 (2026-07-18) — GO composite reaffirm** after passes **319–324**.
> **Still GO (narrow product).** Kill criteria **still unfired**:
>
> | Kill / reverse trigger | Latest recheck | Fired? |
> | --- | --- | --- |
> | A1: bundles do not beat raw context | Pass **321** — golden ok; no result ledger | **No** (open) |
> | A2: no paying segment | Pass **320** — **0** interviews; desk triangle holds | **No** (open) |
> | Full wedge closed by peer | Pass **323** — TMA1 **31st UNFIRED** / Traceway stable | **No** |
> | OTel commoditizes evidence-bundle schema | Pass **323** — #1185 idle | **No** |
> | Incumbent self-host Seer-class AI | Pass **324** — Seer closed; Assistant hybrid | **No** |
> | App-code auto-merge commodity | Pass **324** — Bits never auto-merges | **No** |
> | A4 real messy telemetry reliability | Pass **319** — no reliability-runs | **No** (open) |
> | A6 agent-visible mixed redaction | Pass **323** — canary ok; mixed open | **No** (open) |
> | A3 external schema adoption | Pass **321** — still none | **No** (open) |
> | Sentry OTLP metrics GA | Pass **318** — still unsupported | **No** |
> | TMA1 prod-incident collision | Pass **323** — **31st UNFIRED** | **No** |
> | Datadog OPW = Bits store | Pass **324** — still route-only | **No** |
>
> **Narrow product identity holds.** Research program continues.

## Verdict

**GO.**

Build Parallax, but only as the narrow version:

> An open-source, Rust-first, self-hostable execution context engine that accepts
> OpenTelemetry traces/logs/metrics, derives Parallax-owned error rows from
> exception spans and ERROR/FATAL logs, accepts CLI invocation traces and
> coding-agent session records from tested capture adapters, then stores and
> serves bounded evidence bundles for humans and agents. Sentry-compatible Rust
> error-event ingest is shipped as an opt-in migration adapter.

Do not build the broad version:

> A generic AI observability dashboard, AI root-cause chatbot, or autonomous SRE
> agent over every production signal.

That broad version is already a feature direction for Sentry, Datadog, Grafana,
New Relic, Dynatrace, Splunk, and other observability platforms. The buildable
Parallax product is the open, self-hosted evidence layer underneath agentic
debugging.

## Gate Answers

| Question | Verdict |
| --- | --- |
| Is the problem real? | **Yes.** The problem is not "no one has dashboards." The problem is that production debugging, CI debugging, CLI execution, and coding-agent work produce fragmented evidence that humans and agents must manually reconstruct. Public product direction from Datadog Bits AI SRE, Sentry Seer, Grafana Assistant, and others validates this pain. |
| Does Parallax solve it? | **Partially, and that is enough.** Parallax can solve context assembly, evidence retention, correlation, issue grouping, and agent-safe bundle generation. It cannot prove all root causes from telemetry alone, and it should never claim omniscient RCA. |
| Are there direct competitors? | **Yes.** Sentry Seer and Datadog Bits AI SRE are direct for production debugging. Grafana Assistant is direct for observability-agent workflows. LangSmith/Langfuse/Phoenix/Braintrust/AgentOps-style systems are adjacent for agent/LLM execution telemetry. CI/autofix products are direct for test and pipeline failures. |
| Do competitors leave room? | **Yes, narrowly.** They mostly optimize inside their own observability or LLM-app platform. Parallax can win only if it is simpler to self-host, exposes an open evidence schema, gives CLI/HTTP access from day one and read-only MCP only after projection/safety gates, stores agent and CLI side effects, and produces portable bundles rather than product-bound answers. |
| Is this just a Sentry/Grafana/Datadog feature? | **Generic AI investigation is a feature.** A low-resource, Rust-first, self-hostable context store with fixture-gated Sentry envelope error-event migration, conformance-gated OTLP ingestion, adapter-backed CLI/agent audit records, and portable evidence bundles is a product wedge. |
| Does the market make sense? | **Yes, with discipline.** AI is making software faster to write and riskier to operate without audit trails. The market is crowded, but the crowding validates the shift from dashboards to evidence-backed investigation. The opportunity is not "better AI"; it is owning the evidence contract agents use. |

## Why This Is A GO

### 1. The Pain Is Already Market-Validated

Datadog documents Bits AI SRE as an investigation loop that forms hypotheses,
queries telemetry, validates evidence, and returns either an evidence-backed
conclusion or an inconclusive result. It uses metrics, APM traces, logs, events,
change tracking, GitHub source code, Watchdog, RUM, network, database, profiler,
and preview third-party integrations.

Sentry documents Seer as an AI debugging agent using issue details, tracing,
logs, profiles, and code context. The Seer Issue Fix API can stop at root cause,
solution, code changes, or opening a pull request.

Grafana Assistant exposes observability workflows through UI, CLI, API, Slack,
Teams, and MCP-related integrations. Its CLI can query telemetry, run
investigations, and connect local projects with a tunnel.

These are not weak signals. The incumbents are building exactly because the
manual debugging loop is painful.

Sources:

- [Datadog Bits AI SRE investigation docs](https://docs.datadoghq.com/bits_ai/bits_ai_sre/investigate_issues/)
- [Datadog Bits AI SRE eval platform](https://www.datadoghq.com/blog/engineering/bits-ai-eval-platform/)
- [Datadog Bits AI eval loop note](../validation/a1-bundle-value/datadog-bits-ai-eval-loop.md)
- [Sentry Seer docs](https://docs.sentry.io/product/ai-in-sentry/seer/)
- [Sentry Seer Issue Fix API](https://docs.sentry.io/api/seer/start-seer-issue-fix/)
- [Grafana Assistant CLI docs](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/guides/cli/)
- [Grafana Assistant MCP servers docs](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/configure/mcp-servers/)

### 2. The Existing Products Also Prove The Trap

The trap is building "AI root cause analysis" as a headline. That is no longer
a differentiated product.

The incumbent pattern is:

```text
telemetry + topology + changes + source context
  -> hypothesis loop
  -> evidence-backed conclusion or inconclusive result
  -> action, ticket, PR, or recommendation
```

That pattern is now table stakes. Parallax should not compete with the broad
suite. It should compete on the evidence substrate:

- open schema;
- self-hosted and low-resource operation;
- Rust-first capture quality;
- Sentry envelope error-event migration path with SDK fixture gates;
- OpenTelemetry-based correlation with OTLP conformance gates;
- first-class CLI invocation traces;
- coding-agent session records only where tested adapters preserve source,
  projection, and lossiness provenance;
- portable JSON/Markdown evidence bundles;
- read-only CLI/HTTP tools first, with MCP only after the access-surface gate.

If Parallax cannot win on those dimensions, it should not be built.

### 3. The Technical Substrate Exists

The architecture is plausible with current open-source components:

| Layer | Gate decision | Evidence |
| --- | --- | --- |
| Error compatibility | The bounded Sentry envelope `event` path is implemented, not the whole Sentry product. | The shipped adapter parses and normalizes bounded envelopes into the existing spool, issue, redaction, and evidence paths. Compatibility claims remain limited to fixture-proven SDK/version coverage. |
| Telemetry standard | Use OpenTelemetry as the native telemetry protocol. | OTLP `1.10.0` is stable for traces, metrics, and logs, and gives shared `trace_id`, `span_id`, resource, and semantic-convention context. This proves the wire substrate, not agent readiness: public OTLP claims require the conformance ledger, canonical bundle/projection checks, and MCP structured-output validation. |
| Observability store | GreptimeDB only, on native OTLP tables; see [native-otel-tables.md](native-otel-tables.md). | Historical comparison found ClickHouse faster on heavy analytical scans while GreptimeDB fit the anchored workload and Rust strategy. ClickHouse remains comparator evidence, not a fallback. |
| Stream | Current local WAL/spool only; external streams remain research until explicitly planned. | Iggy is a useful comparator, but this verdict does not authorize a durable external-stream profile. |
| Metadata | Turso Database only. | Turso's maturity keeps crash, backup/restore, concurrency, and migration claims gated; a failure requires fix-forward work, not Postgres. |
| Agent surface | CLI and HTTP first; read-only MCP only after the access-surface safety gate. | Coding agents can call CLIs today, but MCP has become the standard tool discovery/invocation surface and has explicit auth/security requirements. Do not claim first-class agent-native access until MCP projects the same canonical bundle as CLI/API and passes read-only, redaction, output-budget, and audit fixtures. |

Sources:

- [Sentry envelopes](https://develop.sentry.dev/sdk/foundations/envelopes/)
- [Sentry Relay repository](https://github.com/getsentry/relay)
- [OpenTelemetry OTLP specification](https://opentelemetry.io/docs/specs/otlp/)
- [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/)
- [GreptimeDB docs](https://docs.greptime.com/)
- [GreptimeDB v1.1.3 release](https://github.com/GreptimeTeam/greptimedb/releases/tag/v1.1.3) (historical cites may still list v1.0.2)
- [GreptimeDB trace read/write docs](https://docs.greptime.com/user-guide/traces/read-write/)
- [Apache Iggy docs](https://iggy.apache.org/docs/)
- [Turso Database repository](https://github.com/tursodatabase/turso)
- [Turso Database v0.6.1 release](https://github.com/tursodatabase/turso/releases/tag/v0.6.1)
- [Turso Database v0.7.0-pre.3 release](https://github.com/tursodatabase/turso/releases/tag/v0.7.0-pre.3)
- [MCP specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25)
- [MCP authorization specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization)
- [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices)
- [MCP power boundary competitor check](../market/competitor-watch.md)

## What Parallax Actually Solves

Parallax solves these concrete jobs; outcome-corpus and product-MCP claims remain gated:

1. Preserve runtime evidence cheaply enough that teams do not fear diagnostic
   cost spikes.
2. Group errors deterministically before AI touches them.
3. Join Sentry-style errors with OTLP traces, logs, metrics, releases, deploys,
   CI runs, CLI invocations, and agent sessions.
4. Build an evidence graph with typed edge strengths, not a loose text blob.
5. Serve bounded context bundles through CLI, HTTP, and graduated local-stdio MCP
   (`parallax-mcp`: two read-only tools; plan 112 DONE; remote deferred).
6. Record what an agent saw, queried, changed, tested, proposed, and shipped.
7. Say "inconclusive" when evidence is missing instead of inventing certainty.

This is a real product surface. It is also much smaller than an observability
suite.

## What Parallax Does Not Solve

Parallax does not solve:

- every production root cause;
- missing instrumentation;
- sampled-away spans;
- unstructured logs with no trace context;
- cross-service causality without topology or span links;
- business-rule failures not represented in telemetry;
- safe autonomous production mutation;
- trust in a generated patch without tests, evidence, and human-review policy.

The right claim is:

> Parallax reconstructs the best available evidence-backed lifecycle and ranks
> hypotheses. It does not prove every root cause.

That honesty is a strength, not a limitation.

## Direct Competitor Read

| Competitor | What they prove | Where they fall short for Parallax's goal |
| --- | --- | --- |
| Sentry Seer | Production error AI debugging and PR generation are real workflows. | Hosted Seer/Autofix is closed-source, subscription/add-on, and cloud-GitHub-oriented. Current self-hosted Sentry docs explicitly list Seer and other AI/ML features as unavailable because those components are closed source, and sentry-mcp says Seer-like skills may be unavailable on self-hosted instances. Treat the self-hosted exclusion as current-source evidence, not a permanent guarantee. |
| Sentry MCP | Coding-agent MCP access over Sentry data is now a first-party Sentry surface, including remote service, Claude Code plugin/subagent path, and stdio transport for self-hosted Sentry. | The current checked release is `sentry-mcp` `0.35.0`; its README calls stdio a work-in-progress path, AI-powered search tools require OpenAI or Anthropic provider configuration, and self-hosted instances may need unsupported Seer skills disabled. The README setup path lists write scopes, while the stdio testing guide documents read-only testing scopes. This is not hosted Seer parity and makes MCP table stakes, not a moat. |
| Datadog Bits AI SRE / Dev Agent | Hypothesis-driven investigations, incident-to-code handoff, flaky-test autofix, and agent eval infrastructure are the enterprise direction. Datadog's eval-platform write-up is the clearest incumbent proof that world snapshots, noisy labels, score history, `pass@k`, model-refresh checks, and full-set regression runs are required to trust an SRE agent. | Closed, SaaS-only, and tied to Datadog data gravity. Dev Agent is still Preview in current docs, has product-specific enablement, and does not publish an open raw-dump-vs-bundle eval, self-hosted result ledger, or portable evidence-bundle standard. |
| Grafana Assistant | Agent access through CLI/API/MCP surfaces is now normal. | Self-managed Grafana OSS/Enterprise can use Assistant only by connecting to a Grafana Cloud stack; the backend, usage limits, and billing stay in Cloud, and current on-prem docs exclude investigations, investigation memory, CLI auth tokens, and Grafana Cloud MCP connections. CLI is public preview and can read local files through a tunnel. This is not air-gapped, and it is dashboard/assistant-first rather than portable evidence bundles. |
| OpenObserve "Observability 3.0" (late Apr 2026) | An open, Rust, single-binary, object-storage observability store *with* an AI SRE agent + MCP is now real and self-hostable, and its AI SRE page now pressures the evidence-bundle story with evidence-chain/audit-trail language. | The closest thing to a wedge-killer on storage/runtime fit, saved by current gaps: AI SRE/MCP require Enterprise edition/license while public pages conflict on the free Self-Hosted Enterprise allowance, the MCP surface is broad and write-capable rather than a bounded read-only evidence bundle, checked ingestion docs show OTLP rather than a Sentry-envelope path, and no versioned/exportable evidence schema was found. |
| SigNoz agent-native (May 2026) | Open, self-hostable MCP server + trace-ID RCA shipping in OSS validates the agent-native direction loudly; current docs also show a postmortem evidence-pack workflow. | Go + ClickHouse (fails the runtime filter and carries the heavy store Parallax escapes), a query/management interface rather than a checked deterministic evidence graph / portable bundle, an open-investigation-format claim and evidence-pack workflow with no checked versioned schema, validator, replayable export, or portable artifact, and **no Sentry envelope error-event ingest path**. |
| Dynatrace / New Relic / Splunk | Topology-aware RCA is enterprise table stakes. | Enterprise suite gravity, not open small-team self-hosting or agent-readable bundle portability. |
| LangSmith / Langfuse / Phoenix / Braintrust / AgentOps / similar | Agent and LLM traces are important. | They usually observe LLM app execution, not the full chain from production error to deploy, CLI side effect, coding-agent patch, CI validation, and outcome. |
| CI autofix and flaky-test tools | Failure bundles and PR automation are valuable. | They usually start at CI/test evidence, not production Sentry/OTLP context plus runtime evidence graph. |

## Competitive Window (2026-05 update)

This is the finding that moves the posture from "comfortable GO" to "GO, move
now." Between the earlier market pass and 2026-05-25, agent-native observability
went from emerging to table stakes, and two open, non-incumbent projects moved
toward Parallax's exact space: OpenObserve shipped an AI SRE agent + MCP on a
Rust, object-storage, AGPL-self-hostable base, and SigNoz shipped an open,
self-hostable agent-native MCP server.
Sentry's first-party MCP server adds pressure from the incumbent side too: even
self-hosted Sentry users can expose Sentry data to coding agents through a
work-in-progress stdio path, although the checked tool/scopes/provider shape is
not Parallax's bounded read-only evidence-bundle contract.

Neither closes the wedge today — OpenObserve's AI SRE/MCP surfaces require
Enterprise edition/license, its source-conflicted Self-Hosted Enterprise
allowance weakens a simple paywall claim, its AI SRE evidence-chain language is
not yet a versioned/exportable schema, and checked docs still show no Sentry
ingest; SigNoz is Go/ClickHouse with no Sentry ingest and no checked
evidence-graph/bundle abstraction behind its "open investigation format" claim
or postmortem evidence-pack workflow.
But both could close their gap inside 6–12 months. The consequence: **the moat
cannot be any single feature.** It must be the assets that compound with usage
and are hard to copy from a standing start —

1. the failure/fixer-outcome corpus, if outcome rows prove more than PR
   creation;
2. the open evidence schema and portable bundle format as a standard others
   build on;
3. runtime-plus-repo-intent linkage;
4. Rust-first capture quality.

The strategic instruction that follows: ship the narrow tiny tier fast, get the
schema and bundle format adopted, and start accumulating the corpus before the
category fully commoditizes. If an open competitor ships the full combination
(open + self-hosted + Rust-light + Sentry-compatible + evidence bundles) before
Parallax has adoption and a corpus, revisit this verdict — that is the live path
to NO-GO.

Current source checks for this competitive-window claim:

- [OpenObserve pricing](https://openobserve.ai/pricing/)
- [OpenObserve homepage](https://openobserve.ai/)
- [OpenObserve license and pricing docs](https://openobserve.ai/docs/enterprise-setup/license-and-pricing/)
- [OpenObserve SRE Agent setup](https://openobserve.ai/docs/enterprise-setup/sre-agent/)
- [OpenObserve AI SRE product page](https://openobserve.ai/ai-sre/)
- [OpenObserve MCP docs](https://openobserve.ai/docs/integration/ai/mcp/)
- [OpenObserve OTLP ingestion docs](https://openobserve.ai/docs/ingestion/logs/otlp/)
- [OpenObserve AI/MCP Enterprise recheck](../market/competitor-watch.md)
- [Self-hosted Sentry docs](https://develop.sentry.dev/self-hosted/)
- [Sentry MCP service](https://mcp.sentry.dev/)
- [Sentry MCP repository](https://github.com/getsentry/sentry-mcp)
- [Sentry MCP 0.35.0 release](https://github.com/getsentry/sentry-mcp/releases/tag/0.35.0)
- [Sentry MCP and Seer self-hosted recheck](../market/competitor-watch.md)
- [SigNoz agent-native observability](https://signoz.io/agent-native-observability/)
- [SigNoz Postmortem Evidence Pack](https://signoz.io/docs/ai/use-cases/postmortem-evidence-pack/)
- [SigNoz MCP server](https://signoz.io/docs/ai/signoz-mcp-server/)

## Market Verdict

The market is crowded, but not closed.

It is closed for:

- generic AI RCA;
- generic dashboard assistant;
- "Sentry plus AI";
- "Datadog but open source";
- LLM log summarization;
- flaky-test detection alone.

It is open enough for:

- open-source evidence bundle format;
- low-resource self-hosted deployment;
- Sentry envelope error-event migration after SDK fixture gates pass;
- OTLP-backed correlation only after conformance and projection gates pass;
- Rust-first capture and stacktrace quality;
- CLI and adapter-proven agent-session observability;
- safe CLI/HTTP context retrieval first, with MCP only after the access-surface
  gate;
- fixer outcome feedback loop after review, merge/revert, and recurrence rows
  exist.

## Historical Phase 2 Gate And Current Ownership

The GO originally led to the implementation blueprint below. It is now design
history, not an active plan. Current work is authoritative only in `plans/`.

The blueprint must keep the boundary strict:

```text
Parallax stores and serves evidence
  -> CLI / HTTP API expose bounded context first
  -> read-only MCP projects the same context after safety gates
  -> separate fixer component pulls Parallax + repository context
  -> coding agent proposes or opens a PR
  -> fixer writes outcome rows back; PR creation is not proof of fix
```

Parallax itself must not become the fixer. It is the context engine.

## Kill Criteria

Reverse this GO if prototype evidence shows any of the following:

1. Sentry envelope event ingestion cannot work without recreating Relay,
   Kafka, Snuba, and the operational burden Parallax exists to avoid.
2. GreptimeDB cannot answer evidence-bundle queries with
   acceptable freshness, latency, and storage cost.
3. Agent bundles do not improve diagnosis or patch quality over raw Sentry/CI
   context in controlled tests. The experiment that decides this is designed in
   [Bundle-value evaluation](../validation/a1-bundle-value/bundle-value-evaluation.md) — note its raw-telemetry-dump
   control: the bundle must beat a raw dump, not just repo-only context.
4. CLI and agent-session capture produces too much sensitive data to redact
   safely, or tested adapters cannot preserve source/projection/lossiness
   provenance.
5. MCP/API access cannot be made least-privilege, auditable, and read-only by
   default.
6. The first deployment fails the
   [self-hosted simplicity gate](../validation/self-hosted-simplicity.md), cannot pass
   the [self-hosted simplicity ledger](../validation/self-hosted-simplicity.md), and is
   not meaningfully simpler than self-hosted Sentry.

Until those kill criteria trigger, the correct decision is **GO**.

For the maintained adversarial counterweight to this verdict — the steelmanned
NO-GO case, the load-bearing-assumption register, and a full risk matrix — see
[Risks and the bear case](risks-and-bear-case.md). The bear case argues the real
danger is distribution and monetization, not feasibility, and names the market
assumptions (bundle value, real users, schema adoption) to validate before the
comfortable engineering work.
