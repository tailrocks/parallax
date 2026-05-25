# Bundle-Value Phase 0 Evaluation Runbook

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note turns [Bundle-value evaluation](bundle-value-evaluation.md) into a
cheap first runbook for proof gate #7 from
[Strategic verdict and research coverage](strategic-verdict-and-research-coverage.md):

> Agent fix quality with bounded Parallax bundles versus raw Sentry/CI context.

The decision: **before building the full storage/stream system, run a small
paired agent eval that tests whether a hand-built Parallax-style bundle beats
raw telemetry at similar token/time cost.**

The seed task source is now specified separately in
[Bundle-value seed corpus](bundle-value-seed-corpus.md): start from current
executable SWE-style datasets for issue/fix/test reproducibility, then add a
Parallax telemetry overlay with provenance labels. Public GitHub issues enter
only when they pass those eligibility gates.
The exact telemetry-overlay artifact contract is specified in
[Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md); a
task that fails its no-cheat or evidence-parity gates can debug the harness but
cannot count toward the A1 decision.
The public result ledger, contamination tiers, model snapshot, and claim expiry
rules are specified in
[A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md).

This is not the final statistically powered benchmark. It is the cheapest way to
find out whether the central product claim is promising enough to keep building.

## Current Primary-Source Checks

| Source | What matters for Parallax |
| --- | --- |
| [SWE-bench official leaderboards](https://www.swebench.com/) | The standard metric is percent resolved over task instances. The public leaderboard tracks resolved rate, cost, step limits, and several benchmark variants. It remains a useful harness pattern, but tasks are issue/repo/test based, not telemetry based. |
| [OpenAI on SWE-bench Verified](https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/) | OpenAI stopped reporting SWE-bench Verified for frontier launches, citing test flaws and contamination, and recommends SWE-bench Pro. It also restates the SWE-bench setup: the model gets the issue text and repo before the fix, and passes only if hidden tests pass. |
| [SWE-bench Pro](https://labs.scale.com/papers/swe_bench_pro) | SWE-bench Pro has 1,865 harder long-horizon tasks from 41 repositories and held-out/commercial splits. It is a stronger modern coding benchmark, but still does not isolate runtime telemetry or evidence-bundle value. |
| [SWE-bench-Live](https://swe-bench-live.github.io/) | SWE-bench-Live plans monthly updates, has expanded across repositories and languages, and now includes Windows-specific task coverage. It helps reduce contamination, but still tests issue-to-patch ability rather than production-failure evidence. |
| [SWE-bench-Live OS-bench](https://huggingface.co/datasets/SWE-bench-Live/OS-bench), [SWE-bench-Live Windows](https://huggingface.co/datasets/SWE-bench-Live/Windows), and [RepoLaunch](https://www.microsoft.com/en-us/research/publication/repolaunch-automating-buildtest-pipeline-of-code-repositories-on-any-language-and-any-platform/) | OS-bench exposes generated OS migration rows; Windows currently exposes 61 rows with commands, log parsers, Docker images, patches, test patches, hints, commit URLs, and pass/fail test lists. RepoLaunch is the underlying automation direction for arbitrary-language, arbitrary-OS repository build/test setup. | Useful for one CLI/OS/platform task in Phase 0, but still public/generated benchmark evidence rather than production incident telemetry. Freeze dataset revision, row counts, and source-field policy before inclusion. |
| [Terminal-Bench](https://www.tbench.ai/) | Terminal-Bench evaluates agents in terminal environments with task verifiers and concrete artifacts. It is a useful execution-harness model for Parallax agent runs, but it does not answer whether runtime error evidence improves bug fixes. |

## What Phase 0 Must Prove

Phase 0 answers a narrower question than the whole product:

> Given the same repo, same model, same agent scaffold, and same tool budget, does
> a bounded Parallax-style bundle beat raw telemetry dumps for fixing or
> diagnosing failures?

The decisive comparison is still **bundle versus raw dump**. Repo-only is a
baseline, but beating repo-only merely proves telemetry helps. Parallax needs the
bundle to beat or materially improve on raw telemetry.

## Minimum Task Set

Use 10 to 16 tasks for the first run.

| Source | Count | Why |
| --- | --- | --- |
| Telemetry-augmented executable SWE-style tasks | 8-10 | Gives real issue/fix/test structure with low contamination risk. Use the [seed-corpus note](bundle-value-seed-corpus.md) to prioritize SWE-bench-Live MultiLang, SWE-bench Multilingual, Multi-SWE-bench, or SWE-rebench V2 candidates, then reconstruct telemetry by running failing tests under instrumentation. |
| Fault-injected reference app tasks | 2-4 | Gives real Parallax-style traces/logs/error events with known fixes by construction. Include at least one frontend-to-backend or CLI-triggered failure if possible. |
| Externally generated OS/CLI/platform benchmark task | 0-1 replacement | A SWE-bench-Live OS-bench or Windows task may replace one reference-app/CLI task when it passes the same no-cheat and overlay gates. Keep it separate in analysis because it is public generated benchmark evidence. |
| Operator real incidents | 0-2 if available | Reality check using true telemetry and real fix history. Treat these as replacements for public tasks, not additive count inflation, and label separately because n=1 bias is high. |

Do not use SWE-bench Verified for headline Phase 0 decisions. It can shake out
the harness, but current public evidence says it is too contaminated and test
flawed for frontier claims.

Each task must have:

- repo snapshot before the fix;
- known correct patch or fix commit;
- hidden test or verifier that fails before and passes after the fix;
- failure title and short issue statement;
- Sentry-style error event or CI failure event;
- logs, traces/spans, metrics or timings where relevant;
- release/commit context;
- source-field policy separating agent-visible context from runner-private,
  grader-private, and triage-private fields;
- redaction report;
- token-counted raw telemetry dump;
- token-counted Parallax bundle;
- task manifest with all artifact hashes.

## Arms

Run at least arms A, B, and C. Arm D can run on a smaller subset.

| Arm | Context | Rule |
| --- | --- | --- |
| A repo-only | Repo, issue title/body, top stack or failing test output. | Same as standard coding benchmark style. |
| B raw telemetry | Repo plus raw logs/spans/events/metrics in the failure window. | Same underlying evidence as C, but unranked and weakly structured. Enforce the same token ceiling as C. |
| C Parallax bundle | Repo plus bounded bundle with evidence refs, edge strengths, missing-data warnings, and redaction report. | Product arm. Must be generated by a deterministic template, even if hand-assembled. |
| D bundle minus hypotheses | Arm C without ranked hypothesis section. | Optional Phase 0 ablation for whether ranking matters. |

For fairness, if the raw dump exceeds the token ceiling, truncate by a fixed
pre-registered rule: error event first, then same-trace spans, then time-ordered
logs, then metrics/deploy context. Do not hand-pick raw snippets after seeing
agent output.

## Agent Run Protocol

Pre-register the run before looking at results:

- exact task list;
- exact public dataset revisions, row counts, split counts, source-field policy
  hashes, and benchmark-source snapshot hash;
- exact arms per task;
- model(s), temperature, and seed count;
- exact provider model IDs, model snapshot/source refs, alias status, API
  parameters, and context/output token limits;
- agent scaffold, prompt hashes, container image digest, tool versions, and tool
  permissions;
- time, token, and tool-call limits;
- whether internet access is disabled;
- scoring rubric;
- grader identities or judging model;
- planned statistical comparison.
- result-ledger claim level and expiration policy from the
  [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md).

Minimum Phase 0 matrix:

```text
10 tasks x 3 arms x 2 seeds x 1 model = 60 runs
```

Better Phase 0 matrix:

```text
12 tasks x 4 arms x 2 seeds x 2 model families = 192 runs
```

The better matrix is expensive but catches model-specific bundle sensitivity.
If only one model family is used, Phase 0 can justify more research but cannot
justify public product claims.

Every run should execute in a fresh workspace with:

- no access to the gold patch;
- no access to hidden tests except through the evaluator;
- identical repo checkout per task;
- identical tool list;
- logged commands, file reads, file writes, and test runs;
- final patch diff;
- final diagnosis text with evidence refs required.

## Scoring

Primary score:

- **resolved**: hidden/verifier tests pass and regression tests remain green.

Secondary scores:

| Metric | Rule |
| --- | --- |
| Root-cause accuracy | Blind grade against labeled cause: correct, partially correct, wrong, or unsupported. |
| Evidence grounding | Count final claims with valid evidence refs versus unsupported claims. |
| Token cost | Input plus output tokens, reported raw and per resolved task. |
| Time/tool efficiency | Wall time, commands, file reads, test runs, failed attempts. |
| Patch quality | Human spot-check for overfitting, broad rewrites, brittle conditionals, or unsafe changes. |
| Calibration | Whether the agent says "inconclusive" when the supplied evidence is intentionally insufficient. |

Treat "tests pass but diagnosis is wrong" as resolved but risky. Parallax is an
evidence product, so unsupported successful patches are weaker evidence than
successful patches with a correct, cited diagnosis.

## Analysis

Use paired comparisons by task because each arm sees the same underlying bug.

Minimum reporting:

- C vs B resolved-rate delta;
- C vs A resolved-rate delta;
- C vs B token delta;
- unsupported-claim-rate delta;
- per-task run table;
- qualitative failure taxonomy.

Full gate reporting:

- bootstrap confidence interval over tasks for C-B resolved-rate delta;
- McNemar-style paired pass/fail comparison where sample size permits;
- per-model-family results;
- cost-adjusted resolved rate per 1k tokens;
- sensitivity analysis excluding operator-real tasks.

## Phase 0 Decision Rules

These rules decide what to do next, not the final market verdict.

| Result | Action |
| --- | --- |
| C beats B by >= 10 percentage points, has no higher median token cost, and reduces unsupported claims | Continue. Build automated bundle generation earlier in Phase 1. |
| C beats A but not B | Narrow the claim. Telemetry access matters, but Parallax bundling is not yet proven. Improve bundle schema before building storage depth. |
| C ties A and B | Treat kill criterion 3 as live. Reopen the GO verdict before more infrastructure work. |
| B beats C | The bundle is actively filtering out useful evidence. Stop and redesign evidence selection/ranking. |
| D ties C | Deprioritize hypothesis ranking. Keep deterministic evidence selection. |
| C beats D | Ranking adds value. Invest in hypothesis generation only after redaction gates pass. |

For a public claim, Phase 0 is not enough. Require the larger matrix across at
least two model families and a fresh/live task source, with C beating B at
statistically meaningful confidence.

## Artifact Format

Store each task/run under a future results area such as:

```text
docs/research/bundle-value-eval/
  manifest.md
  preregistration.md
  result-ledger.md
  tasks/<task_id>/task.md
  tasks/<task_id>/source-field-policy.json
  tasks/<task_id>/arm-a-context.md
  tasks/<task_id>/arm-b-raw-dump.md
  tasks/<task_id>/arm-c-bundle.json
  tasks/<task_id>/arm-c-bundle.md
  tasks/<task_id>/grader.md
  runs/<run_id>/run-manifest.json
  runs/<run_id>/arm-results.jsonl
  runs/<run_id>/statistics.md
  runs/<run_id>/contamination-checks.md
  runs/<run_id>/patch.diff
  runs/<run_id>/transcript.ref
  results.md
```

Do not commit private raw telemetry or secrets. Commit synthetic fixtures,
redacted Markdown/JSON, hashes, manifests, and result summaries. Put private raw
artifacts behind the same raw-access policy described in
[Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md).

## Relationship To Other Research

- [Bundle-value evaluation](bundle-value-evaluation.md) defines the full
  experimental design; this is the first runnable pass.
- [Bundle-value seed corpus](bundle-value-seed-corpus.md) defines the first
  task-source mix, task eligibility gates, and telemetry overlay requirements.
- [Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md)
  defines the normalized overlay rows and raw-vs-bundle parity checks this
  runbook depends on.
- [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md)
  defines the committed result ledger, model snapshot, contamination tiers, and
  refresh triggers that keep A1 claims current.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines the
  artifact Arm C must use.
- [Risks and bear case](risks-and-bear-case.md) names A1 as existential; this is
  the cheapest falsification path.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  puts this in Phase 0 before deep storage work.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  governs all raw telemetry and agent-visible artifacts.

## Bottom Line

Do not wait for the full Parallax backend to test the thesis. Hand-build enough
bundles to run a paired eval now. If bounded bundles cannot beat raw telemetry
dumps under equal budgets, the product must narrow or pivot before the team
spends months perfecting storage, streams, and symbolication.
