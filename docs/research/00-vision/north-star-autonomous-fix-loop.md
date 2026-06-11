# North Star: The Autonomous Fix Loop and the Impossible Triangle

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-11. Operator vision statement recorded 2026-06-11.

> **North star.** Parallax exists so that, eventually, most production bugs are fixed by AI without
> a human gathering context — and in routine cases without a human being asked at all. A failure is
> detected from telemetry, root-caused from evidence, patched by a coding agent, validated against
> CI, canary, and recurrence data, and folded back into the system as outcome knowledge that makes
> the next fix safer. Parallax itself remains the **context and evidence substrate, not the fixer**
> (the [fixer boundary](../decisions/fixer-boundary.md) stands), but everything Parallax stores,
> schemas, and serves is designed so the full closed loop is *reachable* — and so autonomy is
> **earned from measured outcomes, level by level**, instead of claimed from model capability.

This document names the moonshot explicitly. The narrow wedge in
[go-no-go.md](../decisions/go-no-go.md) and the staged emergence in
[platform-direction.md](platform-direction.md) are unchanged and still govern what gets built
next; this note records *why the wedge is worth winning* and which design requirements the
ceiling imposes on the floor today.

## 1. The operator vision statement (2026-06-11)

Recorded as durable intent, condensed from the operator's own words:

1. **The bottleneck moved.** Building code with agents is fast. Deploying is fast. What is slow is
   figuring out what went wrong in production and fixing it. "Magically AI fixing all your bugs,"
   mostly without asking, is the hardest unclaimed problem in the industry — and the goal.
2. **OpenTelemetry as far as it goes.** Collect everything an application can emit through the
   standard protocol. Extend with Sentry-protocol compatibility only where OTLP genuinely lacks
   data, and only as a future adapter ([capture/sentry-ingest.md](../capture/sentry-ingest.md)).
3. **Pre-aggregate, pre-process, structure.** Telemetry is shaped on the way in, then lands in
   pluggable storage: the columnar evidence store (GreptimeDB lean, ClickHouse fallback, both
   behind `StorageAdapter`) plus a relational metadata store (Turso-first, Postgres fallback).
4. **GreptimeDB conviction.** The operator leans GreptimeDB harder than before: it is Rust, it is
   already designed around the concepts ClickHouse proves (columnar, object-storage-native), and —
   because AI contributes best to Rust codebases — what it still misses versus ClickHouse can be
   added over time, upstream or in-fork, until one engine serves everything in one place. See the
   reality-checked version of this strategy in [storage-engine.md](../decisions/storage-engine.md).
5. **Open source, Rust, deliberately.** In the AI era an open Rust codebase compounds faster:
   agents contribute well, the compiler gates quality, anyone can extend it.
6. **The assumed world is a context-rich mono-repo.** Code, business documentation, user
   documentation, and internal developer documentation live in one agent-navigable repository.
   The repo explains *why the code is the way it is*; Parallax explains *what happened at
   runtime*. Together an agent has a near-complete picture. (Parallax must still work without the
   repo half — see [repo-intent.md](../validation/repo-intent.md); the floor is runtime evidence,
   intent is the multiplier.)
7. **Three goals at once, no conceptual trade-offs:** top analytics performance, lowest possible
   cost, and complete evidence. The "impossible triangle" below. The instruction is explicit: do
   not accept pairwise compromises; find the architecture that bends the third constraint.
8. **Best of three worlds, three surfaces, local-dev first** (statement #2, same day): combine
   OpenTelemetry's collection standard, Sentry's issue organization, and Grafana's cross-signal
   understanding in one agent-first platform; ship CLI + API + UI where everything is a client of
   one canonical API (kubectl-style remote CLI contexts); the first user is a developer on a dev
   machine, scaling to big companies by topology, not rewrite. Full framing:
   [problem-audience-product-shape.md](problem-audience-product-shape.md).

## 2. Why the summit is unclaimed (market evidence, June 2026)

Every serious incumbent now generates fixes. **None of them closes the loop for application
code.** Checked 2026-06-11:

| Player | How far they go | The stated stopping point |
| --- | --- | --- |
| Sentry Seer | Root cause → solution → code changes → **opens PR** (auto-PR by actionability threshold); June 8, 2026 changelog frames new APIs for "self-healing workflows" | No auto-merge; human merges ([Seer docs](https://docs.sentry.io/product/ai-in-sentry/seer/), [changelog](https://sentry.io/changelog/)) |
| Datadog Bits AI | Bits Code **GA** (fix generation everywhere a problem surfaces); Bits Remediation and Bits Infrastructure Operations in preview (DASH, June 9–10, 2026) | "Never merges or deploys without human intervention" ([Bits AI Dev](https://www.datadoghq.com/blog/bits-ai-dev-agent/), [DASH roundup](https://www.datadoghq.com/blog/dash-2026-new-feature-roundup-keynote/)) |
| GitLab Duo | "Ready-to-merge" AI fix MRs | Humans decide the merge ([GitLab blog](https://about.gitlab.com/blog/automate-remediation-with-ready-to-merge-ai-code-fixes/)) |
| GitHub Copilot coding agent | Issue → draft PR via Agent Tasks | Normal review required; Actions gated ([docs](https://docs.github.com/en/copilot/concepts/about-copilot-coding-agent)) |
| AI-SRE agents (Resolve.ai, NeuBird, Bits Infra Ops) | Closed-loop **execution exists only for pre-approved infra/runbook actions** (restarts, rollbacks, config) | Not application-code fixes |

Two readings of this table:

- **The pessimistic reading:** PR generation is a commodity; the giants own the L2/L3 race
  (per the [fixer boundary](../decisions/fixer-boundary.md), opening PRs is not a moat).
- **The reading this project bets on:** everyone is stuck at the same ceiling, and the reason is
  not model capability — it is **evidence and trust**. Nobody can responsibly auto-merge because
  nobody has (a) complete, redacted, citable runtime evidence for the failure, (b) a measured
  per-failure-class track record that says *this class of fix, with this evidence strength, has
  earned this autonomy level*, and (c) a validation/recurrence feedback loop that catches the
  fix that made things worse. That substrate — portable bundles, outcome records, recurrence
  watches, autonomy budgets — is exactly what Parallax's schemas already define and what no
  incumbent ships as an open, self-hosted, inspectable artifact.

The moonshot, precisely: **be the system that makes earned autonomy possible** — the evidence
floor under L4/L5 — rather than another PR generator. If the loop works, the productivity story
is the same shape as CI/CD's: a thing nobody trusted ("deploy on every commit?!") became the
obvious default once the verification substrate existed.

## 3. Autonomy is earned, not claimed

The ladder is already defined in the [fixer boundary](../decisions/fixer-boundary.md)
(L0 observe → L5 auto_merge/deploy). The north-star addition is the **autonomy budget**: a
per-project, per-failure-class level computed from the trailing outcome corpus, not configured
from optimism:

```text
budget(project, failure_class) =
  highest level L such that the trailing window shows
    - n(L-eligible runs) >= minimum sample,
    - accepted/merged rate >= threshold(L),
    - revert + recurrence rate <= threshold(L),
    - zero redaction/policy failures
```

A new project starts at L1 everywhere. A failure class graduates to L3 (draft PR) only by
accumulating accepted L2 proposals; to L4 only by accumulating merged-without-edit L3 PRs with
clean recurrence windows; L5 stays out of scope until a separate production-control safety
program exists (unchanged from the fixer boundary ADR). Every claim stays under the
[fixer outcome ledger](../decisions/fixer-boundary.md) wording rules: until dated result rows
exist, all of this is **design, not capability**.

This is the honest resolution of the tension between the moonshot and the repository's
discipline: the dream is the destination; the gates are the road; the autonomy budget is the
vehicle that converts measured outcomes into permitted autonomy.

## 4. The impossible triangle

Three demands, each easy pairwise, "impossible" together:

```text
        (1) Interactive analytics
             /            \
            /   pick all   \
           /     three      \
 (2) Lowest cost ——————— (3) Complete evidence
```

- (1)+(2) without (3): every metrics vendor — downsample, drop, sample. Evidence dies.
- (1)+(3) without (2): every enterprise APM — keep everything hot. Cost explodes.
- (2)+(3) without (1): every archive — S3 dumps nobody can query interactively.

The mechanisms that bend it (each already researched in this repository):

| Mechanism | What it buys | Evidence |
| --- | --- | --- |
| **Anchored retrieval as the hot path** | The product's critical query (all signals for one `trace_id`/`fingerprint`) is interactive on both engines at every tested scale (≪300 ms) — performance where it matters without paying for broad-scan supremacy | [storage-engine.md](../decisions/storage-engine.md) |
| **Object storage as the only copy** | ~$0.021/GB-month (S3 standard) vs ingest-priced SaaS; GreptimeDB's 1× shared-S3 design avoids N× replica copies | [size-and-object-cost.md](../storage/size-and-object-cost.md), [storage-cost-and-tiering.md](../storage/greptimedb-vs-clickhouse/storage-cost-and-tiering.md) |
| **Per-signal columnar compression** | ~10× blended assumption (metrics 10–50×, logs 5–12×, traces 5–10×); unmeasured on Parallax data, gated | [size-and-object-cost.md](../storage/size-and-object-cost.md) |
| **Pre-aggregation on ingest** | Error-rate rollups, fingerprint counters, metric downsampling computed once at write time, so the interactive layer reads tiny aggregates, not raw firehose | [autonomous-fix-loop.md](../architecture/autonomous-fix-loop.md) §Cost |
| **Tiered retention + evidence pinning** | Raw telemetry ages to cold/rollup on TTL, **but every raw slice cited by an evidence bundle is pinned** — the audit trail outlives retention, so completeness survives cheapness | [autonomous-fix-loop.md](../architecture/autonomous-fix-loop.md) §Cost |
| **Shared Rust roadmaps** | DataFusion scan and Parquet-Variant JSON work close GreptimeDB's remaining speed gaps as engineering, not physics | [greptimedb-parity-roadmap.md](../storage/greptimedb-vs-clickhouse/greptimedb-parity-roadmap.md) |

The no-trade-off rule, operationalized: when a design forces a pairwise choice, that is a design
smell, not a decision to make. The third vertex must come from **architecture** (what is hot, what
is pinned, what is pre-aggregated) — never from "pay more" or "keep less evidence."

## 5. What this changes in the repository (and what it does not)

**Does not change:** the GO-narrow verdict, the build phases and gates
([build-roadmap.md](../architecture/build-roadmap.md)), the fixer boundary, the A1–A7 ledgers,
or any claim-wording rule. The skeptical reassessments remain the operating posture.

**Changes — design requirements the ceiling imposes on the floor, effective now:**

1. Outcome records, recurrence watches, and validation linkage are **core schema**, not Phase-4
   nice-to-haves — they are the fuel of the autonomy budget. (Already largely true; reaffirmed.)
2. The loop's missing stages get concept-level designs so nothing in V1 paints them out:
   detection triggers, agent wake/dispatch, recurrence reconciliation, and the learn loop —
   now specified in [architecture/autonomous-fix-loop.md](../architecture/autonomous-fix-loop.md).
3. How applications, CI, deploy systems, browsers, and fixers attach to Parallax is a contract,
   not folklore — now specified in
   [architecture/integration-contract.md](../architecture/integration-contract.md).
4. Evidence pinning and pre-aggregation enter the storage design vocabulary (cost vertex).
5. A concept-proving Rust PoC is now permitted repository content (operator, 2026-06-11) — first
   artifact under [`poc/evidence-loop/`](../../../poc/evidence-loop/), proving the offline data
   plane: OTLP JSON → derived error events → fingerprint → bundle with redaction report and
   canonical hash.

## 6. Falsification

The north star is a bet, and it dies by the same gates as everything else here:

- **A1 fails** (bundles do not beat raw context for fix quality) → the evidence substrate has no
  customer; the loop premise collapses.
- **The outcome corpus does not move fix-accept rates** after real accumulation → "earned
  autonomy" was a story, and Parallax is a nicer Sentry, not a new layer.
- **The cost gates show the triangle cannot bend** (sized $/GB or cold-read latency falsify the
  object-storage economics) → the completeness vertex must shrink, and the honest product is
  shorter-retention evidence, not "everything forever."
- **An incumbent ships earned autonomy first** (measured per-class autonomy with open outcome
  records) → the window closed; revisit per
  [competitor-watch.md](../market/competitor-watch.md) falsification rules.
