# A1 Fair-Test Design — Runtime-Dependence Taxonomy and a Strong Raw Baseline

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-29

## Purpose

The [2026-05-29 skeptical re-assessment](../../decisions/skeptical-reassessment-2026-05.md)
elevated A1 — *does a bounded bundle beat raw context for agent fix-quality* — to the **#1
existential gate**, because frontier coding agents already resolve ~88–94% of SWE-bench Verified
from a **raw bash harness**, and GA SRE agents (Datadog Bits, AWS DevOps Agent) already do RCA from
raw telemetry + repo. This note sharpens the A1 experiment so it is a **fair and decisive** test of
that threat. It **extends, does not replace**, [bundle-value-evaluation.md](bundle-value-evaluation.md)
(arms, metrics, gate) and [bundle-value-seed-corpus.md](bundle-value-seed-corpus.md) (eligibility).

## The trap a naive A1 falls into

SWE-bench-style tasks are **repo-logic bugs**: the cause is fully determinable from the repository
plus the issue text. On those tasks a capable agent fixes from the repo alone, so a Parallax bundle
will tie raw context — **not because the bundle is worthless, but because the task never needed
runtime evidence.** A naive A1 therefore has two failure modes:

- **Rigged to fail (false NO-GO):** corpus is mostly repo-logic bugs → bundle ≈ raw → A1 "fails" and
  kills a product whose value is on a *different* bug class.
- **Rigged to win (false GO):** baseline is a weak static telemetry dump → bundle beats a strawman →
  A1 "passes" against a control no real 2026 agent would actually use.

Fixing both requires two things: classify tasks by **how load-bearing runtime evidence is**, and
steelman the **raw baseline** to match what a strong 2026 agent already does.

## Part 1 — Runtime-dependence taxonomy (classify every task)

Label each task, blind, before any runs, by how much runtime evidence the *correct* fix requires:

| Class | Definition | Examples | Expected bundle effect |
| --- | --- | --- | --- |
| **R0 — repo-logic** | Cause fully determinable from repo + issue text; no runtime state needed. | Off-by-one, null deref from obvious branch, wrong constant, type error, most SWE-bench Verified. | Bundle ≈ raw repo. **Sanity floor, NOT decisive.** |
| **R1 — runtime-disambiguated** | Several plausible repo causes; runtime evidence (which branch/exception/input/value) selects the real one. | "Crashes sometimes" where the trace shows which call path + args; one of 3 candidate handlers, the log says which. | Bundle should win moderately. |
| **R2 — runtime-only cause** | Cause is a runtime condition largely invisible in code: resource exhaustion (OOM/CPU/FD), dependency saturation/timeout, config/env drift, **deploy-correlated regression**, concurrency/race, data-shape/cardinality blowup, infra. | Latency spike after a deploy; OOM under a traffic shape; pool exhaustion; a downstream 5xx. | Bundle should win **large**; repo-alone should largely fail. |
| **R3 — cross-signal / cross-tier** | Cause spans services or frontend↔backend; needs correlation by `trace_id`/release across signals to locate. | Frontend error rooted in a backend span 3 hops away; error correlated to a specific release across services. | Correlation *is* the value; bundle should win large. |

**The decisive A1 claim is on the runtime-dependent classes (R1+R2+R3).** R0 is reported separately
as an honest floor — ties there are expected and fine; do not average R0 into the headline or the
result is diluted by tasks Parallax was never meant to help.

**Corpus-composition rule:** the eval cannot decide A1 unless the corpus contains a pre-registered
minimum share of R1–R3 (target **≥ 60%**, with R2 the largest single class). Record the class
distribution in the [result ledger](a1-eval-result-ledger-and-model-refresh.md). Because off-the-shelf
SWE-style datasets are R0-heavy, R2/R3 tasks mostly come from **fault injection on the reference app**
(dataset option 3 in [bundle-value-evaluation.md](bundle-value-evaluation.md)) and the **operator's
own repos** — the fault catalog must enumerate R2/R3 fault types explicitly.

## Part 2 — Steelman the raw baseline (the threat is *agentic* raw, not a dump)

The 2026 "raw context is enough" threat is specifically that an agent **agentically retrieves** from
raw telemetry — it is not a static dump. So beating a static dump proves little. Add a baseline arm
that mirrors the real incumbent pattern:

| Arm | Context | Isolates |
| --- | --- | --- |
| A. Repo-only | Repo + issue/error title + stack trace. | SWE-bench floor. |
| B. Raw dump (static) | Repo + an unbounded dump of logs/spans/metrics in the window. | "More data" without retrieval skill. |
| **B′. Agentic-raw (the real control)** | Repo + **read tools over the raw, uncorrelated telemetry store** (query logs/traces/metrics yourself), same tool/time budget as C. | What a strong 2026 agent + a *dumb* telemetry backend already delivers. |
| C. Parallax bundle | Repo + the bounded, correlated, redacted bundle with ranked hypotheses. | The product. |
| D. Bundle − hypotheses | C without the ranked hypothesis block. | Correlation vs ranking. |

**The decisive comparison becomes C vs B′** (not C vs B, not C vs A). If C only beats B (static dump)
but ties B′ (agentic raw), then Parallax's value is "let the agent query telemetry" — deliverable by
any queryable store + an agent — and the **correlation/bundle moat is weak**. C must beat B′ to
justify building the evidence graph.

**Fairness controls for B′ vs C:** identical model, scaffold, tool budget, wall-clock cap, and the
same frozen noisy world snapshot (`noise_manifest`); B′'s telemetry store is realistically **messy and
uncorrelated** (no pre-joined `trace_id` graph, no hypotheses) — it is the raw store Parallax would sit
on top of. Blind, randomized arm order; held-out hidden tests for grading.

## Part 3 — Per-class decision gate (replaces the single-line gate for the threat)

Headline A1 passes only if, **on the runtime-dependent classes R1–R3**:

- **C > B′** on resolved rate at **equal-or-lower token + tool-call cost**, and
- the lift is statistically meaningful across **≥ 2 model families**, and
- **C > A** (sanity: runtime evidence helps at all on these tasks).

| Result (on R1–R3) | Interpretation | Action |
| --- | --- | --- |
| C > B′ > B ≈ A | Correlation/bundle beats even agentic-raw. | **Strong GO**; the bundle is the moat. Invest in schema + corpus. |
| C ≈ B′ > A | Telemetry helps, but agent-over-raw matches the bundle. | Moat collapses to "cheap retention + a queryable store + a good agent." Pivot per the bear case; do **not** build the schema as the moat. |
| C ≈ A | Even runtime evidence does not help these tasks. | Either the taxonomy/labeling is wrong, or kill criterion 3 triggers — reopen the verdict. |
| R0: C ≈ B′ ≈ A | Expected. | Report as floor; no action. |

Report R0 and R1–R3 **separately, never pooled.** A bundle that wins R2/R3 but ties R0 is a success;
pooling would hide it.

## Why this is the cheapest decisive test

It needs no Parallax engine build — only: a task corpus labeled by class, a frozen noisy telemetry
overlay per task ([overlay contract](phase0-telemetry-overlay-contract.md)), a raw queryable store for
B′, a hand-or-script-assembled bundle for C, and agent runs scored on hidden tests. If C cannot beat
B′ on runtime-dependent bugs, **stop before** the storage, stream, and schema work. If it can, that
result is itself the moat-seed and the strongest possible GO evidence.

## Relationship to other research

- [bundle-value-evaluation.md](bundle-value-evaluation.md) — the parent design; this adds the class
  taxonomy, the B′ agentic-raw arm, and the per-class gate.
- [bundle-value-seed-corpus.md](bundle-value-seed-corpus.md) — eligibility gates; add the class label +
  the ≥60% R1–R3 composition rule here.
- [phase0-telemetry-overlay-contract.md](phase0-telemetry-overlay-contract.md) — the frozen evidence B,
  B′, and C all derive from; B′ reads the uncorrelated form, C reads the bundle.
- [a1-eval-result-ledger-and-model-refresh.md](a1-eval-result-ledger-and-model-refresh.md) — must now
  record class distribution and per-class results.
- [../../decisions/skeptical-reassessment-2026-05.md](../../decisions/skeptical-reassessment-2026-05.md)
  — the threat this design answers.
