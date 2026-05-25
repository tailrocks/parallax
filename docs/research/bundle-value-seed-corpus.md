# Bundle-Value Seed Corpus

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The [bundle-value Phase 0 runbook](bundle-value-phase0-runbook.md) says to test
whether a hand-built Parallax bundle beats a raw telemetry dump. The missing
piece is the first task corpus.

This note defines the seed-corpus selection rule:

> Do not hand-pick random public GitHub issues as the first A1 corpus. Start
> from current executable issue-resolution datasets for the issue/fix/test leg,
> then add a Parallax telemetry overlay. Use hand-picked public incidents only
> as supplemental reality checks.

The reason is blunt: Parallax needs tasks with a pre-fix repo, known fix, and
reproducible verifier. Public issues often have stack traces but no isolated
test, no clean fix, or several confounded changes. Current SWE-style datasets
solve much of the reproducibility problem, but they do **not** solve the
telemetry problem. Parallax must generate or attach the telemetry leg itself.

## Current Primary-Source Checks

| Source | What it provides | Parallax gap |
| --- | --- | --- |
| [SWE-bench dataset docs](https://www.swebench.com/SWE-bench/guides/datasets/) | Standard task fields include repository, issue URL, PR URL, base commit, gold patch, test patch, fail-to-pass tests, and pass-to-pass tests. This is the right manifest shape for issue/fix/test tasks. | No runtime telemetry, trace, log, deploy, redaction, or evidence-bundle artifacts. |
| [SWE-bench-Live site](https://swe-bench-live.github.io/), [SWE-bench-Live Hugging Face org](https://huggingface.co/SWE-bench-Live), and [SWE-bench-Live MultiLang](https://huggingface.co/datasets/SWE-bench-Live/MultiLang) | The site says SWE-bench-Live is designed for recent issue-resolution tasks and plans monthly updates. The current Hugging Face org shows active updates, and the current MultiLang viewer shows 743 rows across 8 language splits: C 37, C++ 74, Go 138, JS 93, Rust 94, Java 109, TS 111, and C# 87. Rows include base commit, patch, test patch, problem statement, commit URLs, rebuild/test/print commands, log parser, fail-to-pass/pass-to-pass tests, and Docker image. | Strong freshness and execution harness, with a meaningful Rust slice, but still issue-to-patch, not telemetry-to-patch. Counts and split mix are moving source facts and must be rechecked before selecting the seed manifest. |
| [SWE-bench-Live Hugging Face org](https://huggingface.co/SWE-bench-Live), [OS-bench](https://huggingface.co/datasets/SWE-bench-Live/OS-bench), and [Windows](https://huggingface.co/datasets/SWE-bench-Live/Windows) | The org currently lists OS-bench (126 rows), MultiLang (743 rows), Windows (61 rows), and the Python-only SWE-bench-Live set. Windows rows include language, base commit, `patch`, `test_patch`, problem statement, hints, commit URLs, rebuild/test/print commands, log parser, fail/pass test lists, and Docker image; created-at values span 2024-12-27 to 2026-04-17. | Useful for a CLI/OS/platform slice, but these are public/generated benchmark rows, not wild production telemetry or a replacement for issue-resolution tasks. Freeze dataset revision, row counts, split counts, and field policy before selecting any task. |
| [SWE-bench Multilingual](https://www.swebench.com/multilingual) | 300 curated tasks across 42 repositories and 9 languages, including Rust; tasks follow SWE-bench issue/PR/test format and are designed to run quickly. | Small and high quality, but no runtime evidence. Useful as the first Rust/system seed, not a complete Parallax corpus. |
| [Multi-SWE-bench](https://github.com/multi-swe-bench/multi-swe-bench) | 1,632 issue-resolution tasks across Java, TypeScript, JavaScript, Go, Rust, C, and C++, with open data, code, and environments. | Larger multilingual pool; quality and environment friction must be checked per task before inclusion. |
| [SWE-rebench V2 paper](https://arxiv.org/abs/2602.23866), [dataset collection](https://huggingface.co/collections/nebius/swe-rebench-v2), [main dataset](https://huggingface.co/datasets/nebius/SWE-rebench-V2), and [PR-scale dataset](https://huggingface.co/datasets/nebius/SWE-rebench-V2-PRs) | Language-agnostic pipeline for harvesting executable real-world SWE tasks. The current main dataset viewer shows 32.1k rows, with the card specifying 32,079 samples across 20 languages; fields include base commit, image name, language, license, patch, test patch, fail-to-pass/pass-to-pass tests, install config, and LLM metadata. The PR-scale dataset shows 126k rows and a quick-start length of 126,300. | Best for expansion or training-scale corpus work after the seed run, not as the first headline A1 source. The scale is attractive, but the seed should prefer smaller, inspectable tasks before relying on automatically collected/generated problem statements and metadata-filtered rows. |
| [BugsJS](https://bugsjs.github.io/) | 453 validated JavaScript bugs with bug reports, isolated bug/fix/test revisions, and a Docker-backed framework. Good for server-side JS and frontend-adjacent failure shapes. | Historical and not agent-benchmark-native; use as a controlled supplemental source, not the headline freshness source. |
| [CrashAnalysis dataset](https://crashanalysis.github.io/Dataset-CrashAnalysis) | Thousands of exception stack traces, including GitHub-linked reports. Useful for stacktrace-shape and crash-report realism. | Access is gated and many issues are historical Android cases; it lacks the clean issue/fix/test/verifier contract needed for Phase 0 headline tasks. |

## Seed Corpus Shape

The first corpus should be small enough to inspect manually and large enough to
catch obvious bundle-vs-raw differences:

```text
12 tasks x 3 arms x 2 seeds x 1 model = 72 runs
```

Recommended seed mix:

| Slice | Count | Source priority | Why |
| --- | --- | --- | --- |
| Rust/systems tasks | 4 | SWE-bench Multilingual Rust, SWE-bench-Live MultiLang Rust, Multi-SWE-bench Rust | Parallax is Rust-first; this tests stack/error shapes closest to the first product. |
| Fresh multilingual tasks | 4 | SWE-bench-Live MultiLang | Keeps contamination and stale-fixture risk lower than old benchmark pools. |
| JS/TS user-facing or server tasks | 2 | SWE-bench Multilingual JS/TS, Multi-SWE-bench JS/TS, BugsJS | Frontend and browser/server JS errors are part of the prompt, but should not dominate the Rust-first seed. |
| Synthetic cross-tier or CLI tasks | 2 | Parallax reference app / fault injection; optionally one SWE-bench-Live OS-bench or Windows task | Supplies the telemetry shapes public datasets lack: frontend-to-backend traces, CLI invocation traces, and known side effects. OS-bench/Windows can add a fresh CLI/OS/platform failure, but they remain public benchmark evidence. |

Operator-private incidents can replace at most two public tasks in the first
run, but label them separately and exclude them from public claims unless the
artifacts can be safely shared or independently audited.

Use OS-bench and Windows conservatively. Either source can replace at most one
of the first two synthetic/CLI slots unless the run is explicitly an OS/CLI or
platform slice. Treat `patch`, `test_patch`, `log_parser`, hints, commit URLs,
generated statements, platform labels, and verifier lists as
contamination-sensitive grader or source metadata; agent arms should see only
the allowed issue text plus the derived overlay artifacts. If a task statement
over-specifies the implementation, keep it for harness debugging or score its
diagnosis separately from wild-bug root-cause accuracy.

## Task Eligibility

A task can enter the seed corpus only if all of these are true:

| Gate | Requirement |
| --- | --- |
| Reproducible verifier | The task has fail-to-pass and pass-to-pass tests, or an equivalent deterministic verifier. |
| Isolated fix | The resolving PR or fix patch addresses one bug class and does not mix broad refactors, formatting churn, or unrelated feature work. |
| Runnable environment | The pre-fix repo can run in a documented container or local harness within a bounded setup time. |
| Observable failure | The failing command can emit at least one anchor: exception, panic, failed assertion, failed HTTP/API response, CLI error, or test failure event. |
| Telemetry overlay possible | The harness can collect a Sentry-style event or CI failure event, stdout/stderr logs, span/timing data, and release/commit context without changing the target fix. |
| Gold patch hidden | The context builder may use issue metadata and failing output, but the agent arm must not see the gold patch or test patch content except through the verifier. |
| License/publicness | The task artifacts can be committed as redacted manifests, hashes, and generated telemetry fixtures. |
| Source-field isolation | The task has a source-field policy that separates agent-visible issue/failure context from runner-private, grader-private, and triage-private fields. |

Reject tasks when the fix depends on a private service, network flakiness, huge
dependency downloads, non-deterministic timing, multiple unrelated PR changes,
manual UI steps that cannot be scripted, or a source row whose issue/problem
statement cannot be separated cleanly from gold patch, resolving commit, hidden
test, generated hint, or LLM-filter metadata.

## Telemetry Overlay

Every accepted public task needs a generated telemetry overlay. Without this,
the evaluation only tests issue-resolution from benchmarks, not Parallax.
The exact artifact contract, provenance labels, no-cheat rules, normalized row
shape, and raw-vs-bundle evidence-parity gate are specified in
[Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md).

Minimum overlay per task:

| Artifact | How to generate | Notes |
| --- | --- | --- |
| Error / failure event | Wrap the failing test or command and convert panic/exception/assertion output into a Sentry-style event. | Mark as `reconstructed` unless captured from a real SDK. |
| Trace / span tree | Instrument the task runner as root span; add child spans for setup, failing command, selected test, subprocesses, and relevant app calls where feasible. | For most public tasks, this is harness telemetry, not production telemetry. Say so in the bundle. |
| Logs | Capture bounded stdout/stderr, test logs, and app logs with stable line refs. | Raw dump arm and bundle arm must use the same underlying logs. |
| Metrics / timings | Capture duration, retry count, exit code, memory/time limits, and relevant test counts. | Metrics are optional if not causally useful, but timing helps compare agent flailing. |
| Release/change context | Record base commit, issue URL, task source, dataset version, and resolving PR/commit hashes. | Resolving PR/commit URLs and gold patch hashes can be in private or audit manifests, not in agent context. |
| Source-field policy | Classify every source field before building Arm A/B/C. | Resolving PR/commit URLs, hints, parser source, hidden verifier IDs, LLM metadata, and gold artifacts stay out of agent-visible artifacts by default. |
| Redaction report | Run the same seeded canary/redaction policy used by the bundle schema docs. | A task without a redaction report is invalid. |

All overlay artifacts must carry provenance:

```text
observed_from_sdk | observed_from_test_output | reconstructed_from_harness
```

The Phase 0 report must separate results on real telemetry, harness-generated
telemetry, and synthetic fault-injection telemetry. A win on reconstructed
telemetry is a reason to continue; it is not a public production-telemetry claim.

## Bundle Construction Discipline

The raw-dump arm and bundle arm must be built from exactly the same evidence.

Rules:

- Build the raw artifact first, then derive the Parallax bundle from it.
- Pre-register the truncation rule before any agent run.
- Keep token ceilings equal across arms B and C.
- Include `missing_evidence` when the task lacks production traces, deploy data,
  frontend breadcrumbs, or real SDK events.
- Include `query_manifest` entries even when the "queries" are file reads over
  generated fixtures.
- Keep hypotheses conservative; if the issue statement already names the fix,
  do not let the bundle repeat gold-patch knowledge as if telemetry discovered
  it.

## Seed Manifest

Use this future layout for the first corpus:

```text
docs/research/bundle-value-eval/
  manifest.md
  tasks/<task_id>/task.md
  tasks/<task_id>/source.json
  tasks/<task_id>/source-field-policy.json
  tasks/<task_id>/telemetry/raw.ndjson
  tasks/<task_id>/telemetry/redaction-report.json
  tasks/<task_id>/arm-a-context.md
  tasks/<task_id>/arm-b-raw-dump.md
  tasks/<task_id>/arm-c-bundle.json
  tasks/<task_id>/arm-c-bundle.md
  tasks/<task_id>/grader-private.sha256
```

`source.json` should include:

```json
{
  "task_id": "swe-live-rust-example",
  "source": "SWE-bench-Live/MultiLang",
  "source_version": "hf-viewer-checked-2026-05-25",
  "dataset_revision": "hf-revision-or-commit",
  "row_count_at_selection": 743,
  "split_counts_at_selection": {"rust": 94},
  "source_checked_at": "2026-05-25T00:00:00Z",
  "repo": "owner/repo",
  "base_commit": "...",
  "issue_url": "https://github.com/owner/repo/issues/123",
  "resolving_ref_hash": "sha256:...",
  "language": "Rust",
  "failure_anchor": "panic|exception|assertion|cli_exit|http_error",
  "telemetry_provenance": ["reconstructed_from_harness"],
  "license_review": "public source; generated redacted artifacts only",
  "excluded_gold_artifacts": ["patch", "test_patch", "FAIL_TO_PASS", "PASS_TO_PASS", "commit_url", "commit_urls", "hints_text", "all_hints_text", "log_parser", "interface", "meta", "llm_metadata"]
}
```

Do not commit private raw telemetry. Commit public/generated fixtures, redacted
projections, hashes, manifests, and summaries.

## Decision

The next A1 step is not "build storage." It is:

1. Select the 12-task seed manifest using the gates above.
2. Generate the telemetry overlay for each task.
3. Build raw dump and bundle artifacts from the same overlay.
4. Run the [Phase 0 eval](bundle-value-phase0-runbook.md).
5. Only then decide whether automated bundle generation deserves Phase 1 build
   work.

When the run happens, publish its results through the
[A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md)
so the task mix, contamination tier, model snapshot, and claim expiry are
auditable.

If no 12-task seed can be assembled without cheating, that is itself a negative
signal: Parallax's strongest claim may depend on a dataset that does not yet
exist. In that case, the correct research move is to build and publish the
telemetry-linked fixture corpus as the first moat artifact, before claiming
bundle lift.

## Relationship To Other Research

- [Bundle-value evaluation](bundle-value-evaluation.md) defines the experiment;
  this note defines the first corpus.
- [Bundle-value Phase 0 runbook](bundle-value-phase0-runbook.md) defines arms,
  run protocol, and scoring.
- [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md)
  defines how task-source freshness and contamination tiers are reported in the
  eventual result artifact.
- [Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md)
  defines the overlay artifact set, provenance labels, no-cheat rules, and
  parity gates required before a task can count toward A1.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines the
  bundle artifact generated for arm C.
- [Schema adoption and corpus moat gate](schema-adoption-and-corpus-moat-gate.md)
  becomes more credible if the seed corpus is public, reproducible, and
  conformance-tested.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  keeps A1 before storage and stream work.

## Bottom Line

Use current executable SWE-style datasets for clean issue/fix/test tasks, but do
not pretend they already contain Parallax evidence. The first valuable artifact
is a small telemetry-augmented seed corpus with honest provenance labels. That is
the cheapest way to test whether bundles beat raw dumps without building the
full backend first.
