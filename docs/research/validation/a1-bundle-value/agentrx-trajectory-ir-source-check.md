# AgentRx Trajectory IR Source Check

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-02

## Verdict

AgentRx is useful for A1, but not as a direct Phase 0 issue/fix/test task
source.

Use it as:

- a design reference for trajectory IR, invariant checks, failure-step
  localization, and agent-action taxonomy;
- a supplemental audit source for Parallax's own coding-agent/CLI trace schema;
- a possible later benchmark for "does Parallax capture enough agent trajectory
  evidence to localize a failed step?"

Do not use it as:

- the headline A1 corpus for "does a Parallax bundle improve software fix
  quality versus raw telemetry?";
- a replacement for executable SWE-style tasks with hidden tests;
- production telemetry evidence.

Reason: AgentRx diagnoses failed agent trajectories. A1 needs bugs with a
pre-fix repo, known fix, verifier, and telemetry overlay. Those are different
contracts.

## Current Source Check

| Source | Current evidence | A1 implication |
| --- | --- | --- |
| AgentRx repository | Public MIT-licensed repo under `microsoft/AgentRx`; package name `agentrx`, version `0.1.0`, Python `>=3.10`. README pipeline is raw logs -> trajectory IR -> invariants -> checker -> LLM judge -> reports. Supported domains are Tau-bench retail, Magentic-One, Flash incident traces, and auto-detected unknown formats. | Useful architecture reference for Parallax's agent/CLI trace audit path. Not a storage/backend benchmark and not a Parallax bundle result. |
| AgentRx README | Claims the framework pinpoints the critical failure step, produces auditable validation logs, and classifies failure into a 10-category taxonomy: instruction/plan adherence, invention, invalid invocation, tool-output misinterpretation, intent-plan mismatch, underspecified intent, unsupported intent, guardrails, system failure, and inconclusive. | Add this taxonomy as candidate labels for Parallax agent-action outcome analysis. Also useful for scoring unsupported claims and calibration in A1. |
| AgentRx dataset API | Hugging Face API reports dataset `microsoft/AgentRx`, sha `88e871fecb58b2d090449f37ec80b8865594e0b5`, last modified `2026-02-26T05:32:10Z`, `private=false`, license `cc-by-4.0`, configs `default` and `trajectories`, with file refs for `tau_retail.jsonl`, `magentic_one.jsonl`, `magentic_dataset.jsonl`, and `tau_retail_dataset.jsonl`. | Metadata can be recorded, but selected-row hashing is currently blocked because raw file fetch returned Hugging Face restricted-access/authentication text in this environment. Do not include rows in A1 until access is granted and full-row hashes are produced. |
| AgentRx paper / Microsoft Research announcement | Paper abstract says the benchmark has 115 manually annotated failed trajectories across structured API workflows, incident management, and open-ended web/file tasks. Microsoft announcement says AgentRx improves failure localization by 23.6% and root-cause attribution by 22.9% over prompting baselines. | Strong evidence that trajectory structure and invariant checking can improve failure localization. It does not prove Parallax bundle value for code fixes, because no raw-dump-vs-bundle arm or Parallax-style runtime evidence bundle is published. |

## How To Use AgentRx In A1

AgentRx should add a new auxiliary source role:

```text
trajectory_audit_source
```

Meaning:

- may be used to validate schema coverage for agent/coding-tool/CLI traces;
- may be used to test whether Parallax-normalized agent traces preserve enough
  evidence to localize critical failure steps;
- may seed a failure taxonomy for final diagnosis grading;
- must not be counted as a Phase 0 software-fix task unless it also has a
  runnable repo snapshot, hidden verifier, known patch, and valid telemetry
  overlay.

## Schema Lessons For Parallax

Borrow these ideas:

- **Trajectory IR.** Parallax agent/CLI capture should normalize tool calls,
  model calls, file reads/writes, command executions, observations, approvals,
  and final patch/outcome into a canonical trajectory form. Raw transcripts are
  too lossy and too hard to score.
- **Invariant checks.** Before an LLM judges root cause, deterministic checks
  should flag tool schema errors, missing evidence, impossible state
  transitions, policy violations, and output/observation mismatches.
- **Auditable validation log.** Bundle construction should emit machine-readable
  checks that say which evidence supports each failure-step or hypothesis
  claim. This directly supports A1's unsupported-claim metric.
- **Inconclusive as first-class.** AgentRx's taxonomy includes inconclusive.
  Parallax bundles should preserve that category instead of pressuring every
  incident into a root-cause claim.

Do not borrow these as product proof:

- AgentRx's published improvement is over prompting baselines for trajectory
  diagnosis, not over agentic raw telemetry for code repair.
- AgentRx operates after a failed trajectory exists. Parallax's core A1 claim is
  upstream: capture, correlate, redact, and project runtime evidence so a fixer
  agent can patch production/runtime issues better.

## A1 Protocol Changes

Update Phase 0 expectations as follows:

1. Keep AgentRx-style datasets out of the headline task pool unless row access,
   row hashes, gold labels, repo snapshots, and verifier semantics are frozen.
2. Add an optional **Agent Trajectory Audit** appendix to A1 results:
   - input: Parallax-normalized agent/coding-tool/CLI traces;
   - baseline: raw transcript or raw OTel/MCP spans;
   - product arm: Parallax trajectory projection with invariant check log;
   - score: critical failure-step localization, taxonomy correctness,
     unsupported-claim rate, and inconclusive calibration.
3. Keep this appendix separate from C vs B-prime resolved-rate results. It can
   strengthen the agent-audit roadmap, but it cannot make the bundle-value gate
   pass.

## Open Questions

- Can the AgentRx Hugging Face dataset be accessed and row-hashed with accepted
  credentials, despite raw unauthenticated fetch being blocked here?
- Do the Flash incident traces include enough production-like telemetry shape
  to inform Parallax's R2/R3 runtime-dependent task design?
- Can AgentRx's trajectory IR map cleanly to OTel GenAI/MCP semantic
  conventions plus Parallax-specific action/outcome fields?
- Would invariant checks improve C vs B-prime in A1, or only postmortem
  explanation quality?

## Sources

- [AgentRx GitHub repository](https://github.com/microsoft/AgentRx)
- [AgentRx Hugging Face dataset](https://huggingface.co/datasets/microsoft/AgentRx)
- [AgentRx paper](https://arxiv.org/abs/2602.02475)
- [Microsoft Research AgentRx announcement](https://www.microsoft.com/en-us/research/blog/systematic-debugging-for-ai-agents-introducing-the-agentrx-framework/)
