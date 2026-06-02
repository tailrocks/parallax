# Bundle-Value Evaluation: Does Parallax Context Actually Help Agents?

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This designs the single most important experiment for the whole project. The
[verdict](../../decisions/go-no-go.md) makes it kill criterion 3 and the [bear case](../../decisions/risks-and-bear-case.md)
makes it load-bearing assumption A1:

> A bounded Parallax evidence bundle makes a coding agent's diagnosis and fix
> materially better than the context it could get without Parallax.

If this is false, Parallax is elegant ceremony: the agent fixes just as well from
a raw stack trace plus the repo, and the evidence graph adds latency and cost,
not accuracy. Everything else — storage, schema, frontend, tiers — is plumbing
until A1 is measured. This document specifies how to measure it.

It is an experiment design, not results. No bundle-value claim should be made in
any other doc until this runs.

The cheapest runnable version of this experiment is specified in
[Bundle-value Phase 0 evaluation runbook](bundle-value-phase0-runbook.md). The
first corpus-selection pass is specified in
[Bundle-value seed corpus](bundle-value-seed-corpus.md).
The result-ledger and model-refresh policy for turning a run into a current A1
claim is specified in
[A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md).

## Why The Field Has Not Answered This

Current coding-agent benchmarks evaluate fixing from **repository context only**:

- SWE-bench (Princeton): real GitHub issue -> reproduce bug, find root cause,
  write fix, keep tests green.
- SWE-bench Verified (human-cleaned subset), SWE-bench Pro (Scale AI, 1,865
  multi-language tasks), SWE Atlas (RCA, codebase QnA, test writing), SWE-bench-CL
  (continual learning).

Every one of these gives the agent the **repo and an issue description**. None
provides production **telemetry** — logs, traces, metrics, error events, deploy
context — at the moment of failure. So the field mostly measures "can an agent
fix from code + issue text," not "does runtime evidence make the fix better."
That is precisely Parallax's bet, and it is currently unproven by any mainstream
benchmark. This is both the risk (no prior signal) and the opportunity (a
telemetry-augmented eval would be novel and is the natural moat artifact).

Current benchmark reality:

| Benchmark / source | What it measures | Parallax gap |
| --- | --- | --- |
| SWE-bench / ICLR 2024 | Issue/PR pairs from GitHub, evaluated by repository tests. | No production telemetry, deploy context, traces, logs, or agent evidence bundle. |
| SWE-bench Verified | 500 human-validated SWE-bench samples. | Still repo + issue context; OpenAI later warned Verified no longer measures frontier coding capability well because test flaws and benchmark saturation became material. |
| SWE-bench Pro | Longer-horizon software-engineering tasks, 1,865 tasks, multi-language. | Better task difficulty, but still not a telemetry-value benchmark. |
| SWE Atlas | Codebase Q&A, test writing, and refactoring workflows; includes root-cause-style questions. | Measures repository investigation, not runtime evidence added to fix generation. |
| SWE-bench-Live | Fresher live GitHub issues to reduce contamination. | Valuable for freshness, but still not evidence-bundle-vs-raw-telemetry. |

Primary sources:

- [SWE-bench Princeton announcement](https://pli.princeton.edu/blog/2023/swe-bench-can-language-models-resolve-real-world-github-issues)
- [SWE-bench ICLR 2024 paper](https://proceedings.iclr.cc/paper_files/paper/2024/file/edac78c3e300629acfe6cbe9ca88fb84-Paper-Conference.pdf)
- [SWE-bench official leaderboards](https://www.swebench.com/)
- [SWE-bench Verified announcement](https://openai.com/index/introducing-swe-bench-verified/)
- [OpenAI on why SWE-bench Verified no longer measures frontier coding capabilities](https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/)
- [SWE-bench Pro paper page](https://labs.scale.com/papers/swe_bench_pro)
- [SWE Atlas paper page](https://labs.scale.com/papers/sweatlas)
- [SWE-bench-Live](https://swe-bench-live.github.io/)

Secondary methodology reference:

- [UTBoost: rigorous SWE-bench evaluation](https://arxiv.org/pdf/2506.09289)
- [Datadog Bits AI SRE eval platform](https://www.datadoghq.com/blog/engineering/bits-ai-eval-platform/) -
  useful industry methodology for incident-agent evals: world snapshots, noisy
  reconstructed environments, segmentation, score history, `pass@k`, weekly
  full-set regression runs, feedback-derived labels, and model-refresh checks.
  This source validates the need for A1 rigor, but it does not validate
  Parallax's bundle-value claim because Datadog does not publish a raw-dump-vs-
  bundle arm, open schema, public task rows, or self-hosted result ledger.

## Hypothesis And Arms

**H1:** an agent given a Parallax bundle resolves more issues, with more accurate
root causes, than the same agent given weaker context — at acceptable token/time
cost.

Four arms, same agent, same model, same repo access, only the context differs:

| Arm | Context provided | What it isolates |
| --- | --- | --- |
| **A. Repo-only (control)** | Repo + issue/error title + stack trace. | The SWE-bench-style baseline. Beating this proves runtime evidence helps at all. |
| **B. Raw telemetry dump** | Repo + stack + an unbounded-ish dump of logs/spans/metrics in the time window. | "More data" control. Beating this proves the **bundling/correlation**, not mere data access, is the value. |
| **C. Parallax bundle** | Repo + the bounded, correlated, redacted [evidence bundle](../../architecture/evidence-bundle-schema.md) with hypotheses. | The product. |
| **D. Bundle minus hypotheses** | Arm C without the ranked hypothesis block. | Isolates whether value is the **correlated evidence** or the **ranking**. |

The decisive comparison is **C vs B**, not C vs A. If C only beats A but ties B,
the value is raw data access — which is cheaper to deliver by dumping telemetry,
and the correlation/bundle moat is weak. C must beat B to justify Parallax.

Repo-held intent is a paired sub-study, not a replacement for the main A1 arms:
split Arm C into runtime-only and runtime-plus-intent variants as defined in the
[Repo-intent value ledger](../repo-intent.md). That measures whether
docs, decisions, tasks, roadmap, or agent instruction files add value without
making degraded runtime-only mode too weak.

## Dataset (The Hard Part)

A1 needs triples: **(failure, known-correct fix, real telemetry at failure
time)**. No off-the-shelf dataset has the telemetry leg. Three ways to build it,
worst-to-best on realism:

1. **Telemetry-augmented SWE-bench (semi-synthetic).** Take SWE-bench Verified /
   Pro tasks, run the failing test under `tracing`/OTLP + panic capture to
   synthesize the spans/logs/error event that a real run would emit, attach
   deploy/release metadata from git. Pro: reuses labeled fixes + passing tests as
   ground truth. Con: telemetry is reconstructed, not from production traffic.
2. **Operator's own repos.** Real Sentry/OTLP history joined to git "bug commit →
   fix commit" pairs. Pro: real telemetry. Con: small N, n=1 bias (bear case A2),
   labeling effort.
3. **Reference app + fault injection.** A seeded multi-service app (the same
   generator family as the [storage benchmark](../../storage/benchmark-plan.md)),
   inject known faults, capture real telemetry, the fix is known by construction.
   Pro: real telemetry + known fix + cross-tier frontend↔backend cases. Con:
   faults may be less representative than wild bugs.

Recommended: start with (1) for scale and labeled grading, validate the headline
result on (3) for telemetry realism, and use (2) as a reality check. Building this
telemetry-linked fix corpus is itself a research contribution and a moat seed.

The seed-corpus rule is stricter than this high-level menu: use current
executable SWE-style datasets for the issue/fix/test leg, then add a Parallax
telemetry overlay with honest provenance labels. Do not start by hand-picking
random public GitHub issues unless they pass the
[seed-corpus eligibility gates](bundle-value-seed-corpus.md).
Failed-agent-trajectory datasets such as AgentRx are a separate source class:
they can validate trajectory IR, invariant checks, failure-step localization,
and agent/CLI audit taxonomy, but they do not become headline A1 fix-quality
tasks unless they also satisfy the repo/fix/verifier/telemetry-overlay contract.
See [AgentRx trajectory IR source check](agentrx-trajectory-ir-source-check.md).
The overlay itself must satisfy the
[Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md) so
the raw dump and Parallax bundle are generated from the same frozen evidence and
cannot leak gold patches or silently invent production-grade links.

## Metrics

| Metric | How measured | Why |
| --- | --- | --- |
| Resolved rate | Known/hidden test passes after the agent's patch (SWE-bench-style). | The headline outcome. |
| Root-cause accuracy | Agent's stated cause vs labeled cause (LLM-judge + human spot-check, blind). | Diagnosis quality, not just lucky patch. |
| Token cost | Input+output tokens per task. | Bundles must not win only by dumping more tokens; cost-adjust the comparison. |
| Wall-clock / tool calls | Time and number of fetches to first patch. | Bounded bundle should reduce flailing vs raw dump. |
| Unsupported-claim rate | Fraction of agent claims with no evidence ref (hallucination proxy). | Bundles should reduce ungrounded reasoning. |
| Calibration | Says "inconclusive" when evidence is genuinely insufficient. | Safety-relevant; over-confidence is dangerous (agent-safety doc). |

Cost-adjust: report resolved-rate **per 1k tokens** as well as raw, so a bundle
that wins only by being bigger is exposed.

## Protocol And Controls

- **Multiple model families** (≥2, e.g. one frontier + one mid) so the result is
  not an artifact of one model's context handling.
- **Blind, randomized arm order**; graders do not know the arm.
- **Multiple seeds / repeated trials** per task; report variance, not just means.
- **Same agent scaffold** across arms; only the injected context differs.
- **Held-out hidden tests** for grading so the agent cannot game the visible test.
- Pre-register the decision gate below before looking at results.

Because SWE-bench Verified is now publicly questioned even by OpenAI, do not use
one benchmark family as the only evidence. The Parallax experiment should treat
telemetry-augmented SWE-bench as a scaling scaffold, then check the result on
fresh/live tasks and a fault-injected reference app.

The Datadog Bits AI SRE eval-platform source adds one more required control:
the raw dump and bundle arms need realistic distractors. A clean bundle that only
contains root-cause evidence is an open-book exam. Each task snapshot should
therefore include a `noise_manifest` with unrelated but plausible spans, logs,
alerts, deploys, services, and errors, plus the reason each distractor was
included. Arm B and Arm C must derive from the same frozen noisy world snapshot.

## Decision Gate

Parallax's bundle thesis passes only if:

- **C beats A** on resolved rate (runtime evidence helps), and
- **C beats B** on resolved rate at equal-or-lower token cost (the *bundle*, not
  raw data, is the value), and
- the lift is statistically meaningful across ≥2 model families.

Outcomes and what they mean:

| Result | Interpretation | Action |
| --- | --- | --- |
| C > B > A | Bundle and telemetry both add value; correlation is real. | Strong GO confirmation; bundle is the moat. |
| C ≈ B > A | Telemetry helps, but bundling adds little over dumping. | Narrow the claim: value is cheap retention + access, not correlation. Pivot moat toward retention/audit (bear case). |
| C ≈ A | Bundle does not beat repo-only. | Kill criterion 3 triggers; reopen verdict. |
| D ≈ C | Hypotheses add nothing; correlated evidence is the value. | Drop/deprioritize the hypothesis engine; keep deterministic evidence. |
| C > D | Ranking adds value. | Invest in the hypothesis layer. |

## Honest Limitations

- Semi-synthetic telemetry may flatter Parallax (clean trace IDs, perfect
  linkage) versus messy production — so the realism check on dataset (3) and (2)
  is mandatory before any public claim.
- Frontier models improve fast; an A1 win today can erode as models get better at
  fixing from less context. Re-run across model generations (this is why the gate
  requires ≥2 families and periodic re-runs).
- Resolved-rate on tests is not the same as a *good* fix; pair it with root-cause
  accuracy and human review.

## Relationship To Other Research

- [Verdict](../../decisions/go-no-go.md) — kill criterion 3, which this operationalizes.
- [Risks and the bear case](../../decisions/risks-and-bear-case.md) — assumption A1 (and A3,
  since a positive result is what makes the schema worth adopting).
- [Evidence bundle and open schema](../../architecture/evidence-bundle-schema.md) — the artifact
  under test (arm C/D).
- [Bundle-value Phase 0 evaluation runbook](bundle-value-phase0-runbook.md) —
  the concrete first pass to run before a full benchmark corpus exists.
- [Bundle-value seed corpus](bundle-value-seed-corpus.md) — selects the first
  public task sources and defines the telemetry overlay required before Phase 0.
- [A1 task source freeze check](a1-task-source-freeze-check.md) — pins current
  public dataset SHAs, row/split counts, and field quarantine requirements for
  the likely seed sources.
- [AgentRx trajectory IR source check](agentrx-trajectory-ir-source-check.md) —
  keeps failed agent trajectory sources in an auxiliary audit role, not the
  headline software-fix task pool.
- [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md)
  — defines the public result artifact, model snapshot, contamination tiers, and
  expiry rules for A1 claims.
- [Repo-intent value ledger](../repo-intent.md) — defines the paired
  runtime-only versus runtime-plus-intent sub-study under the Parallax-bundle
  arm.
- [Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md) —
  freezes the normalized overlay artifact, provenance labels, and no-cheat rules
  used to derive raw-dump and bundle arms from the same evidence.
- [Storage benchmark prototype](../../storage/benchmark-plan.md) — shares the
  seeded dataset/reference-app generator for dataset option (3).
- [Causal reconstruction and agent safety](../../architecture/causal-reconstruction.md)
  — calibration/unsupported-claim metrics feed the safety model.

## Bottom Line

The whole project rests on a claim no existing benchmark tests: that runtime
evidence, bundled and correlated, makes agents fix better. Design the eval with a
raw-telemetry-dump control (arm B), because beating repo-only is easy and
unconvincing — beating a raw dump is the real test of whether Parallax's
correlation is worth building. Run this before, not after, investing further in
the storage and stream layers. It is the experiment that converts the GO from
argued to proven.
