# Parallax — Vision, Storage Decision, and Restructure Brief

<!-- markdownlint-disable MD013 -->

**Objective:** finalize the Parallax product direction and the GreptimeDB-vs-ClickHouse
storage decision, and restructure the research record into a clean, navigable form,
preserving every verified finding. This brief is the statement of intent; where an existing
`docs/research/` note disagrees with it, this brief wins (reconcile the note). Parts 1–3 are
the durable vision and decision rule; Part 4 is the current restructure mission.

---

## Part 1 — The Vision (operator intent, in plain terms)

**The premise: AI already writes the features. The repository is the context that makes
that work.** The operator develops almost entirely with AI coding agents, and the agents
produce high-quality code *because the whole context lives in the repository*:

- **user-facing documentation** — how the product works (what features exist, how users
  use them);
- **specifications** — the intent behind each feature, captured when it is built, so an
  agent always knows what was wanted before and adapts code to that intent instead of
  losing it;
- **internal technical documentation** — decisions, flows, and the "why" that has no
  natural home in code, so an agent sees not just the code but how the system is meant to
  fit together;
- **roadmap, feature status, and tasks** — kept in GitHub, so the agent has the full
  picture of what to build next.

With all of this in the repo, **delivering high-quality features fast is mostly a solved
problem** — the agent never misses context.

**The unsolved problem is what happens after delivery: bugs and instability.** Even with
great features, the running system is unstable and ships bugs. Here AI is much weaker,
because **many production errors are side effects of conditions the agent cannot see** —
high CPU, memory pressure, resource exhaustion, a saturated dependency. The visible error
is a symptom; the cause is a runtime condition. Shown only the error, the agent "fixes"
its own (often wrong) interpretation, because it has no knowledge of the underlying state.

**The fix is to give the agent the runtime context too.** If the agent receives the
logs, metrics, traces, and grouped errors around a failure — and can *verify* against them
(was CPU high? was there enough memory? did this start after a deploy? is this error
frequent or rare?) — it can reason about the real cause and propose a correct fix. **Full
context = repository intent (docs + specs + technical docs + roadmap) + runtime
observability.** That combination is what makes trustworthy, near-automatic fixes
possible.

**The product is the missing observability half of that context, as one system.** Today
that context is spread across separate tools:

| Signal | Tool the operator uses today | What Parallax must absorb |
| --- | --- | --- |
| Errors | **Sentry** (the part used most: error **grouping**, organizing by frequency, stack traces, and the surrounding error context) | OTLP-first error ingest, deterministic grouping, issue/fingerprint model; future Sentry-envelope migration adapter |
| Logs | ELK / Loki / Kibana, Tempo-adjacent | OTLP logs, searchable, correlated by trace |
| Metrics | Prometheus | OTLP / Prometheus metrics, PromQL-style queries |
| Traces | Jaeger / Tempo | OTLP traces, span trees |

The operator is happy with each tool individually but wants **one self-hosted system
instead of many**, because the value is the *correlated, AI-ready context across all
signals*, not any single dashboard. Existing open projects come close but none is a clean
open-source replacement for exactly this job.

**Hard constraints the operator has stated:**

1. **Cost is a first-class design goal.** The observability layer must be cheap and
   minimal to run — as lean as possible. This is *the* reason the implementation language
   is **Rust** (low overhead) and a reason the storage decision is weighted toward cheap,
   object-storage-native economics.
2. **Self-hosted on dedicated servers** the operator controls.
3. **Rust-first**, because the operator will invest engineering in Rust (not C++) and
   wants a substrate he can extend and contribute to.

**The north-star outcome:** an AI-native observability layer that, combined with the
repository's intent, gives a coding agent enough verified context to diagnose and fix
production failures correctly — cheaply, on infrastructure the operator owns.

---

## Part 2 — The product, precisely (keep this discipline)

The repository's validated **GO** verdict is for the *narrow* product, and that
discipline must survive contact with the ambitious vision above:

> **Parallax is an open-source, Rust-first, self-hosted execution-context engine.** It
> ingests OpenTelemetry logs/traces/metrics/error events (plus CLI and coding-agent
> execution traces), groups errors deterministically, correlates signals into a typed
> evidence graph, and serves **bounded, redacted, schema-valid evidence bundles** to humans
> and coding agents over a CLI/HTTP API first, and a read-only MCP adapter after safety
> gates. Sentry-envelope ingest is future migration compatibility, not V1 scope.

**Parallax is the context engine, not the fixer.** The automatic-fix outcome from Part 1
is achieved by Parallax *feeding* bounded evidence bundles to a **separate** coding
agent / fixer — not by Parallax mutating production or opening PRs itself. Opening a PR is
not proof of a fix; outcome rows (accepted / reverted / recurred) are. Keep this boundary:

```text
Parallax stores + serves evidence
  → CLI / HTTP API (read-only MCP later)
  → separate fixer / coding agent consumes the bundle + repo intent
  → agent proposes or opens a PR
  → outcome rows decide whether the fix actually helped
```

**Do not** rebuild Parallax into a generic "AI RCA chatbot," a full dashboard suite, or an
autonomous production SRE — that space is crowded and is a feature for the incumbents. The
defensible wedge is the **open evidence-bundle schema + the cheap, self-hosted, Rust-first
capture quality + the failure/fix-outcome corpus.**

Three deployment tiers, same event/bundle contract throughout:

- **Tier 1 (tiny):** one `parallax-server` binary, local WAL, embedded metadata, managed
  local GreptimeDB on local disk, no Postgres, no cloud object storage, no queue. Must be
  **simpler to run than self-hosted Sentry** — that is the entry wedge.
- **Tier 2 (production single-node):** GreptimeDB server/standalone, Turso metadata until
  production gates fail, optional Postgres fallback, optional object storage for raw
  evidence/backups/long retention.
- **Tier 3 (durable):** split ingest/worker/API, object-storage retention, optional
  Apache Iggy stream for replay, backpressure, burst handling, and worker separation.
- **Tier 4 (scale-out):** horizontal ingest/workers, distributed columnar store, object
  storage, Postgres metadata fallback.

---

## Part 3 — The storage-engine decision (GreptimeDB vs ClickHouse)

This is the question the operator most wants answered. It is also the **single
most-researched question in this repo** (≈170 benchmark runs, full source-level teardown
of both engines under `docs/research/storage/greptimedb-vs-clickhouse/`, all load-bearing claims
re-verified, versions current as of 2026-05-29). It is **not** under-researched. It has
converged to a **conditional decision**, and the agent's job is to keep it honest, not to
re-run the whole loop.

### What the evidence actually says (do not relitigate; verify and maintain)

- **On raw speed, ClickHouse wins** heavy analytical scans, broad log search, dynamic-JSON
  attribute queries, and in-DB joins (decade-tuned C++ vectorized engine). The gap *widens*
  with scale (5M+ rows). This refutes the early "GreptimeDB is fastest" hypothesis.
- **On Parallax's actual hot path it does not matter.** Parallax's dominant query is the
  *anchored* evidence bundle (everything for one `trace_id` / `fingerprint`). Both engines
  serve that interactively (≪300 ms) at every tested scale. Engine choice is **not
  latency-bound for the workload Parallax is designed around.**
- **Two lenses reach opposite defaults, on purpose:**
  - *Fit + long-term-investment lens* → **GreptimeDB**: Rust (operator-contributable),
    object-store-native cost, metrics/PromQL-native, scale-out by design; its speed
    deficits are *closable engineering on the shared DataFusion/Parquet roadmaps, not a
    physics wall.*
  - *Parallax-as-proxy lens* (the operator's own 2026-05-25 architecture decision: Parallax
    owns OTLP ingest/routing/conversion) → **ClickHouse** as pragmatic default: because the
    proxy supplies the protocols, GreptimeDB's native-ingest edge is neutralized, leaving
    retrieval speed + build-on-top ecosystem (SigNoz/Uptrace/HyperDX/ClickStack) — both
    ClickHouse wins.

### The deciding inputs — one now resolved, the rest still open

1. **Parallax's real query mix — RESOLVED (operator, 2026-05-29): anchored-bundle-retrieval-
   dominant.** The hot path is fetching all signals for one `trace_id` / `fingerprint` /
   issue to assemble an evidence bundle, not broad ad-hoc analytics. Consequence: on the hot
   path **both engines are interactive (≪300 ms at every tested scale), so ClickHouse's
   raw-speed lead is not decisive for Parallax.** The decision therefore turns on **cost +
   Rust**, where GreptimeDB leads — not on analytical-scan speed, where ClickHouse leads.
2. **Self-hosted vs managed cloud — still open.** Strictly self-hosted at scale favors
   GreptimeDB's 1× object-storage copy + compute/storage separation; if ClickHouse Cloud
   (`SharedMergeTree`) is acceptable, that erases GreptimeDB's cost-economics edge.

### Current lean, and what must close before it is settled

The storage engine is **an open research question for this brief to finalize — not yet a
settled default.** With the query mix now known to be anchored-retrieval-dominant, and given
the operator's priorities (minimal cost, Rust, self-hosted, a substrate he will extend), the
**current lean is GreptimeDB**: its two surviving wins under the proxy lens (object-store
cost economics + Rust-contributable substrate) are exactly the operator's priorities, and the
workload removes ClickHouse's speed advantage from the hot path.

> **Current lean: GreptimeDB. Not yet settled.** Keep **both engines behind the
> `StorageAdapter`**; never hard-code engine magic into the schema or bundle contract.

To **finalize** the decision, this research must still land:

1. **Sized cost numbers on a real server tier** — $/GB retained, per-signal compression, and
   **multi-replica object-storage cost** (GreptimeDB 1× shared S3 vs OSS ClickHouse N× replica
   copies). This is the operator's #1 priority and the least-measured axis.
2. **Cold-read latency at GB–TB from object storage** — the one regime that could still
   surprise even an anchored workload.
3. **The self-host-vs-managed-cloud decision** (input 2 above).
4. **A re-test on GreptimeDB v1.1 GA** (expected Q2 2026 — narrows the dynamic-JSON gap and
   may move the metrics path); re-pin and re-run the load-bearing benchmarks when it ships.

Absent a surprise in (1)–(2) or a "yes" to managed cloud in (3), the anchored workload + cost
+ Rust point at **GreptimeDB**. Honest guardrail: if the sized cost numbers come back at
parity *and* a managed path is acceptable, ClickHouse's ecosystem + speed make it the safer
pick — let the numbers, not the Rust preference, settle it.

### Standing maintenance task for this question

- Keep both engines behind one `StorageAdapter` trait; never hard-code engine magic into
  the schema or bundle contract.
- **Query mix is resolved** (anchored-bundle-retrieval-dominant, operator 2026-05-29). The
  remaining inputs that finalize the engine choice are the sized cost numbers and the
  self-host-vs-managed-cloud decision (above), not another query-shape model.
- Re-pin versions and re-verify the load-bearing speed/cost claims when either engine ships
  a new stable release (GreptimeDB v1.1 GA, expected Q2 2026, narrows the dynamic-JSON gap —
  re-test then).

---

## Part 4 — The restructure mission (this brief's primary task right now)

**When you run this brief, execute the restructure specified below.** The research findings
are sound; the *record* is messy — ~99 flat files in `docs/research/`, ~35 in the engine
sub-study, conclusions buried under "Run N / pass M" history, and many near-duplicate notes.
A newcomer (human or agent) cannot quickly find "what did we decide and why." Fix the
**record** without losing a single verified finding or primary source.

Work in **small, reviewable commits** (one topic group per commit) and push after each, per
[`COMMITS.md`](../COMMITS.md) and [`AGENTS.md`](../AGENTS.md); keep
[`PROJECT_STRUCTURE.md`](../PROJECT_STRUCTURE.md) accurate in the same commits. Use `git mv`
so history follows the files. **Merge by appending sections, never by deleting evidence** —
when notes collapse into one, fold both bodies in with a one-line provenance note; cut only
true duplication and dead cross-references.

**Principles:**

1. **Separate current truth from working history.** A small set of crisp **decision records**
   (ADR-style, conclusion first) state the current answer; long re-verification logs become
   clearly-labeled evidence/changelog appendices, not the front door.
2. **One concept, one home.** Collapse every `*-gate` + `*-ledger` pair and every
   per-competitor `*-recheck` into one consolidated note with sections.
3. **Group by topic** in subdirectories instead of one flat folder.
4. **Lead with conclusions**; detail and history follow.
5. **Preserve evidence and primary-source links** — compress prose, keep the facts.

**Target structure + file map** (`←` means "merge these sources into one note"; give every
existing note a home):

```text
docs/research/
  README.md                        # rewrite: navigable index grouped by the dirs below
  00-vision/
    thesis.md                      ← project-thesis.md
    platform-direction.md          ← future-platform-direction.md
    ai-native-observability.md     ← ai-native-observability-and-incident-intelligence.md
  decisions/                       # ADR-style, current truth first, one decision per file
    go-no-go.md                    ← verdict.md
    strategic-coverage.md          ← strategic-verdict-and-research-coverage.md
    risks-and-bear-case.md         ← risks-and-bear-case.md
    storage-engine.md              ← condensed current verdict (full record in storage/greptimedb-vs-clickhouse/)
    stack-decision.md              ← a5-stack-decision-ledger.md
    metadata-store.md              ← decision part of the metadata notes (evidence in storage/metadata/)
    agent-access-surface.md        ← agent-access-surface-cli-api-mcp.md + agent-access-surface-safety-ledger.md
    fixer-boundary.md              ← fixer-component-and-outcome-loop.md + fixer-outcome-ledger.md
  architecture/
    implementation-concept.md      ← technical-implementation-concept.md
    overview.md                    ← self-hosted-observability-architecture.md
    evidence-bundle-schema.md      ← evidence-bundle-and-schema.md
    causal-reconstruction.md       ← causal-reconstruction-and-agent-safety.md
    build-roadmap.md               ← build-roadmap-and-validation-sequence.md
  capture/                         # how each signal is collected + made safe
    rust.md                        ← rust-data-collection-and-instrumentation + rust-capture-fidelity-recheck + rust-stacktrace-grouping-and-symbolication + rust-stacktrace-grouping-ledger
    frontend.md                    ← frontend-collection-and-cross-tier-correlation + frontend-capture-safety-ledger + frontend-browser-ingest-profile-recheck + frontend-replay-sourcemap-privacy-recheck
    sentry-ingest.md               ← future sentry-compatible-ingestion + sentry-envelope-item-policy-recheck + sentry-sdk-fixture-compatibility + sentry-sdk-compatibility-ledger
    otlp.md                        ← opentelemetry-protocol-and-context-layer + otlp-transport-profile-recheck + otlp-receiver-conformance-and-collector-equivalence + otlp-conformance-ledger
    agent-cli-tracing.md           ← agent-and-cli-execution-tracing + agent-cli-otel-semconv-mapping + agent-session-tracing-real-tools + agent-session-tracing-ledger + cli-trace-overhead-and-redaction + cli-trace-safety-ledger
    deploy-change-context.md       ← deploy-change-and-issue-context + deploy-change-context-ledger
    ci-and-flaky-tests.md          ← ci-failure-context-mvp + flaky-test-investigation-and-replay
    production-db-evidence.md      ← production-database-evidence-access + production-database-evidence-ledger
    correlation.md                 ← correlation-reliability-real-telemetry-gate + a4-correlation-reliability-ledger
    redaction.md                   ← redaction-pipeline-and-secret-safety + redaction-detector-toolchain + redaction-toolchain-betterleaks-recheck + a6-synthetic-canary-fixture-corpus + a6-redaction-red-team-ledger
  storage/
    evaluation.md                  ← greptimedb-storage-evaluation
    benchmark-plan.md              ← observability-storage-benchmark-plan + storage-benchmark-prototype + storage-benchmark-artifact-interpretation
    freshness-and-latency.md       ← storage-freshness-and-bundle-latency-gate
    size-and-object-cost.md        ← storage-size-and-object-cost-gate + retention-cost-model
    metadata/                      ← metadata-store-benchmark-plan + turso-metadata-production-readiness
    streaming/                     ← messaging-and-ingestion-layer + ingest-log-replay-and-backpressure-gate
    greptimedb-vs-clickhouse/      # keep the 35-file sub-study; split verdict from history (below)
  validation/                      # A1–A7 assumption gates + ledgers, merged pairs
    a1-bundle-value/               ← bundle-value-evaluation + bundle-value-seed-corpus + bundle-value-phase0-runbook + phase0-telemetry-overlay-contract + a1-eval-result-ledger-and-model-refresh + a1-task-source-freeze-check + a1-source-drift-and-leakage-recheck + a1-huggingface-row-hash-procedure + datadog-bits-ai-eval-loop
    a2-user-demand.md              ← user-interview-and-deployment-intent-gate + a2-interview-evidence-ledger
    a3-schema-corpus.md            ← schema-adoption-and-corpus-moat-gate + a3-schema-adoption-corpus-ledger
    a7-scope.md                    ← a7-scope-discipline-ledger
    self-hosted-simplicity.md      ← self-hosted-simplicity-gate + self-hosted-deployment-baseline-inventory + self-hosted-simplicity-ledger
    business-model.md              ← business-model-and-economics + business-model-validation-ledger
    repo-intent.md                 ← repo-intent-dependence + repo-intent-value-ledger
    (A4 → capture/correlation.md; A5 → decisions/stack-decision.md; A6 → capture/redaction.md)
  market/
    landscape.md                   ← market-landscape
    competitor-watch.md            ← open-self-hosted-competitor-watch + lightweight-sentry-compatible-competitor-watch + agentic-observability-competitor-drift-ledger + ALL per-competitor rechecks (openobserve / bugsink / rustrak / traceway / gosnag / urgentry / coroot / signoz / sentry-mcp-seer / lightweight-error-tracker-mcp-boundary / mcp-power-boundary)
  reference/
    agent-observability-review.md  ← agent-observability-technical-review
```

**Engine sub-study (`storage/greptimedb-vs-clickhouse/`) cleanup:**

- Rewrite `verdict-which-to-choose.md` to a **one-page current verdict** (the DQ1–DQ6 answers
  as a short table + the flip rule from Part 3); move the inline "Run 1…170 / pass N" history
  into a `run-log.md`.
- Reduce that folder's `README.md` to a short, conclusion-first index of the 30+ mechanism
  notes; move its run-by-run status prose into `run-log.md`.
- Keep every mechanism note (`*-internals.md`, `query-execution-engine.md`, …),
  `four-way-version-comparison.md`, and `local-benchmark-results.md` as the evidence layer.

**Execution order (small commits, push each):**

1. Create the directory skeleton + the rewritten `docs/research/README.md` index.
2. `git mv` the one-to-one moves (`00-vision/`, `architecture/`, `reference/`, `storage/*`).
3. Merge the `*-gate` + `*-ledger` pairs in `capture/` and `validation/` (append sections + a
   provenance line each).
4. Collapse the competitor notes into `market/competitor-watch.md`.
5. Write the `decisions/` ADRs (conclusion first), each linking to its evidence note.
6. Split the engine sub-study verdict from its run-log.
7. Update root `README.md` + `PROJECT_STRUCTURE.md` to the new tree.

**Done when:**

- A reader reaches "what is Parallax, which storage engine, and why" in under five minutes
  from the README.
- Every settled decision has exactly one `decisions/` record; every still-open proof gate is
  listed in one place.
- No finding or source lost; the engine sub-study's current verdict is one short page with
  history in an appendix.

---

## Part 5 — Deliverables this brief should keep producing

All "how to build" output is a **technical product specification — not source code.**
Describe components, their responsibilities, data flows, interfaces, decisions, and
trade-offs. No code snippets; implementation code lives in the build repository, not in this
research record.

1. **Clarity on what to build:** the narrow evidence/context engine of Part 2, in three
   tiers, with the strict "Parallax is not the fixer" boundary.
2. **Clarity on how to build it (as a technical product):** the components and what each is
   responsible for — the ingest gateway (OTLP first, future Sentry-envelope adapter), the
   normalize → group → correlate → evidence-graph pipeline, the columnar telemetry store
   behind a `StorageAdapter` (current lean GreptimeDB per Part 3, still open; ClickHouse the
   fallback), the relational metadata store (Turso, Postgres fallback), the local WAL and
   optional stream, and the CLI/HTTP/MCP context surface — plus the data flow, the interfaces
   between them, and the trade-offs. Explain *what each piece does and how the pieces fit*,
   and name technology choices at the component level; do not write code.
3. **A finalized storage decision** once the sized cost numbers land (query mix already
   known: anchored).
4. **A clean, navigable research record** per Part 4.

Never declare the research "complete" unilaterally; keep improving the record and the
decisions until the operator stops or replaces this brief.
