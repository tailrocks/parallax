# A1 Eval Result Ledger And Model Refresh

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The A1 bundle-value gate now has a task-source plan, a Phase 0 runbook, and a
telemetry-overlay contract. The missing piece is the public result artifact:

> A1 is not validated by "we ran the eval." It is validated by a committed,
> reproducible result ledger that records the task set, model snapshots, agent
> scaffold, evidence hashes, contamination checks, per-arm outcomes, and expiry
> conditions for the claim.

This note defines that ledger and the refresh policy. Without it, a Parallax
bundle win can be stale, cherry-picked, contaminated, scaffold-specific, or
unverifiable.

## Current Source Posture

Outside sources checked for this pass:

- OpenAI stopped reporting SWE-bench Verified for frontier launches after
  finding material test-design issues and contamination risk; it recommends
  SWE-bench Pro while investing in newer uncontaminated evaluations
  ([OpenAI](https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/)).
- SWE-bench-Live is explicitly designed around recent issue-resolution tasks and
  says it plans monthly dataset updates for fresher, more contamination-resistant
  evaluation. Its current Hugging Face org shows active updates; the current
  MultiLang viewer shows 743 rows across eight language splits, including 94
  Rust rows, plus build/test commands, log parsers, patches, and Docker images
  ([SWE-bench-Live](https://swe-bench-live.github.io/),
  [MultiLang](https://huggingface.co/datasets/SWE-bench-Live/MultiLang)).
- SWE-bench-Live's Hugging Face org now lists four datasets and shows recent
  activity: OS-bench (126 rows), MultiLang (743 rows), Windows (61 rows), and
  the Python-only SWE-bench-Live set. The official site says the benchmark plans
  monthly updates and notes a February 2026 Windows-specific task release. The
  Windows viewer exposes the same high-risk fields as other SWE-style sources
  (`patch`, `test_patch`, hints, commit URLs, commands, log parser, verifier
  lists, Docker image) and spans eight language values. OS/Windows slices are
  useful supplemental CLI/platform evidence, but because they are public,
  generated, and moving, they require dataset-revision snapshots and should be
  reported separately from wild production incident tasks
  ([SWE-bench-Live org](https://huggingface.co/SWE-bench-Live),
  [Windows](https://huggingface.co/datasets/SWE-bench-Live/Windows),
  [OS-bench](https://huggingface.co/datasets/SWE-bench-Live/OS-bench)).
- The official SWE-bench leaderboard reports resolved rate, cost, step limits,
  benchmark variants, model release date, and scaffold filtering, which is a
  useful result-shape reference even though SWE-bench itself lacks telemetry
  ([SWE-bench](https://www.swebench.com/)).
- SWE-bench Pro adds harder, long-horizon tasks and includes held-out and
  commercial splits; it is a better current coding benchmark reference but still
  does not isolate runtime telemetry or evidence-bundle value
  ([Scale Labs](https://labs.scale.com/papers/swe_bench_pro)).
- SWE-rebench V2 is current and large enough to matter for later expansion: the
  arXiv paper reports a language-agnostic pipeline with 32,000+ executable tasks
  across 20 languages and 3,600+ repositories, plus a larger PR-scale release.
  The current Hugging Face collection lists a 32.1k-row main dataset and a
  126k-row PR-scale dataset. That makes it useful for expansion or training-scale
  corpus work, but the first A1 seed should still prefer smaller, inspectable
  sources before relying on automatically collected/generated task rows
  ([arXiv](https://arxiv.org/abs/2602.23866),
  [collection](https://huggingface.co/collections/nebius/swe-rebench-v2)).
- Terminal-Bench publishes agent-terminal tasks and includes an explicit
  training-contamination canary on the public site, a good reminder that
  benchmark artifacts need leakage checks
  ([Terminal-Bench](https://www.tbench.ai/)).
- Datadog's Bits AI SRE eval-platform write-up is the most relevant public
  incumbent methodology source for incident-agent evaluation. It describes
  reconstructed investigation world snapshots, isolated scenario data layers,
  noisy environments with red herrings, segmentation by technology/problem/
  monitor/difficulty, stored per-scenario scores, `pass@k`, weekly full-set
  regression runs, feedback-derived labels, and full-label-set model-refresh
  checks. It is a methodology benchmark, not proof of Parallax value, because it
  does not publish portable bundles, a raw-dump-vs-bundle arm, public task rows,
  or an open result ledger
  ([Datadog](https://www.datadoghq.com/blog/engineering/bits-ai-eval-platform/),
  [Datadog note](datadog-bits-ai-eval-loop.md)).
- MCP `2025-11-25` separates human-readable tool text from JSON
  `structuredContent`, and RFC 8785/JCS gives a deterministic JSON
  canonicalization target. Arm C/D rows must therefore prove the agent saw a
  canonical bundle projection, not an untracked Markdown rendering
  ([MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools),
  [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785.html)).

Internal sources:

- [Bundle-value evaluation](bundle-value-evaluation.md) defines the A/B/C/D
  arms and says C must beat B, not only repo-only A.
- [Bundle-value seed corpus](bundle-value-seed-corpus.md) defines task
  eligibility and the initial seed mix.
- [A1 task source freeze check](a1-task-source-freeze-check.md) pins the current
  Hugging Face dataset SHAs, row/split counts, feature snapshots, and field
  quarantine posture for the likely Phase 0 task sources.
- [A1 source drift and leakage recheck](a1-source-drift-and-leakage-recheck.md)
  assigns source roles, excludes trajectory/result datasets from task-source
  use, and requires full selected-row hashes because `first-rows` previews can
  be truncated.
- [A1 Hugging Face row hash procedure](a1-huggingface-row-hash-procedure.md)
  defines how to fetch selected rows from pinned revisions and hash full,
  source-policy, and agent-visible row projections.
- [Bundle-value Phase 0 runbook](bundle-value-phase0-runbook.md) defines the
  first run matrix and scoring.
- [Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md)
  defines the frozen evidence bytes used by raw-dump and bundle arms.

## Why A1 Needs A Result Ledger

The A1 eval has more ways to produce false confidence than a normal benchmark:

| Failure mode | How it misleads Parallax | Ledger control |
| --- | --- | --- |
| Public task contamination | A model may already know an issue, fix, release note, or gold patch. | Record task freshness tier, canaries, contamination probes, and source dates. |
| Model drift | A bundle win can disappear when newer models handle raw context better. | Attach an expiry date and rerun triggers to every A1 claim. |
| Scaffold variance | The agent wrapper, tool permissions, step limit, or internet access may dominate the result. | Snapshot the exact scaffold, tool budget, model IDs, and run parameters. |
| Evidence asymmetry | Arm C can win because it gets curated evidence Arm B never had. | Link every result row to overlay hashes and evidence-parity gates. |
| Cherry-picked tasks or seeds | A handful of lucky runs can be reported as a product proof. | Commit pre-registration, all task outcomes, all seeds, and exclusions. |
| Result-only summary | Future readers cannot audit the claim from the repo. | Commit manifests, per-arm rows, scorecards, statistical summaries, and hashes. |

## Artifact Boundary

When A1 runs, create a public result area under the existing future layout:

```text
docs/research/bundle-value-eval/
  manifest.md
  preregistration.md
  result-ledger.md
  runs/<run_id>/
    run-manifest.json
    task-set.json
    benchmark-source-snapshot.json
    investigation-world-snapshots.jsonl
    noise-manifest.jsonl
    model-snapshots.json
    arm-results.jsonl
    projection-audit.jsonl
    scorecards.md
    statistics.md
    contamination-checks.md
    hashes.sha256
    private-artifacts.sha256
```

| Artifact | Commit? | Purpose |
| --- | --- | --- |
| `manifest.md` | Yes | Human entry point for the whole A1 result set. |
| `preregistration.md` | Yes | Task list, arms, model families, budgets, scoring, and decision rule before results are inspected. |
| `result-ledger.md` | Yes | Current gate status, claim level, expiry date, and links to all run IDs. |
| `run-manifest.json` | Yes | Exact run configuration, model snapshots, scaffold commit, task-set hash, bundle template version, redaction policy, and token ceilings. |
| `task-set.json` | Yes | Public task IDs, source, freshness tier, language, provenance, source-field policy hash, and inclusion/exclusion reason. |
| `benchmark-source-snapshot.json` | Yes | Dataset revisions, source roles, row counts, split counts, last-updated/checked timestamps, first-row truncation observations, full-row fetch/hash methods, and source-field quarantine summary for every public task source. |
| `investigation-world-snapshots.jsonl` | Yes | One frozen normalized world per task: evidence rows, topology/deploy/source links, raw-ref hashes, evidence completeness, and source-field policy status. |
| `noise-manifest.jsonl` | Yes | Distractor evidence included in the raw and bundle arms: unrelated but plausible services, logs, spans, alerts, deploys, and rationale for inclusion. |
| `model-snapshots.json` | Yes | Exact provider model IDs and API parameters used for that run. |
| `arm-results.jsonl` | Yes | One row per task/arm/model/seed outcome. |
| `projection-audit.jsonl` | Yes | Canonical bundle hash, projection manifest, CLI/HTTP/MCP equivalence, and MCP `structuredContent` validation for agent-visible bundle arms. |
| `scorecards.md` | Yes | Blind grading notes, root-cause accuracy, evidence-grounding counts, and patch-quality notes. |
| `statistics.md` | Yes | Paired C-vs-B/C-vs-A deltas, confidence intervals where possible, token/time deltas, and sensitivity checks. |
| `contamination-checks.md` | Yes | Canary status, publicness tier, freshness checks, and any contamination probes. |
| `private-artifacts.sha256` | Yes | Hashes of transcripts, raw telemetry, gold patches, and hidden verifiers kept outside the repo. |

Raw private telemetry, full agent transcripts containing secrets, and gold
patch/test-patch contents stay outside the repo unless explicitly redacted.

## Run Manifest Schema

Each A1 run needs a machine-readable manifest. Model names should not be treated
as durable prose; exact model IDs live here because they change over time.

```json
{
  "run_id": "a1-phase0-2026-05-25-r001",
  "research_date": "2026-05-25",
  "claim_expires_after": "2026-08-23",
  "repo_commit": "commit-sha-that-built-the-artifacts",
  "preregistered_at": "2026-05-25T00:00:00Z",
  "task_set_hash": "sha256:...",
  "benchmark_source_snapshot_hash": "sha256:...",
  "overlay_contract_version": "phase0-overlay-v1",
  "bundle_template_version": "bundle-v0",
  "bundle_schema_ref": {
    "uri": "https://parallax.dev/schemas/evidence-bundle/v0.json",
    "hash": "sha256:...",
    "canonicalization": "jcs-rfc8785"
  },
  "projection_surfaces_required": ["bundle_json", "bundle_markdown", "cli_output", "http_api", "mcp_structuredContent"],
  "mcp_output_schema_required": true,
  "redaction_policy_version": "phase0-redaction-v1",
  "a6_redaction_claim_level": "not_measured|synthetic_canary_pass|agent_visible_mixed_pass",
  "a4_correlation_claim_level": "not_measured|synthetic_only|backend_mvp_pass|frontend_cross_tier_pass",
  "agent_scaffold": {
    "name": "codex|claude-code|openhands|swe-agent|custom",
    "version_or_commit": "...",
    "scaffold_commit": "sha-or-none",
    "container_image_digest": "sha256:...|none",
    "system_prompt_hash": "sha256:<hex>|none",
    "developer_prompt_hash": "sha256:<hex>|none",
    "tool_version_manifest_hash": "sha256:<hex>",
    "internet_access": false,
    "tool_permissions": ["shell", "edit", "test"],
    "max_steps": 100,
    "max_wall_clock_minutes": 45
  },
  "task_sources": [
    {
      "source": "SWE-bench-Live/MultiLang",
      "source_role": "seed_candidate",
      "hf_dataset_sha": "608f7ae9ab8ea1f9f0d030fe04562cf6bd1a0c8b",
      "dataset_revision": "hf-revision-or-commit",
      "checked_at": "2026-05-25T00:00:00Z",
      "source_license": "mit",
      "source_visibility": {
        "private": false,
        "gated": false,
        "disabled": false
      },
      "hub_tags": ["license:mit", "size_categories:n<1K", "format:parquet"],
      "row_count": 743,
      "split_counts": {"rust": 94},
      "features_hash": "sha256:<hex>",
      "datasets_server_size_partial": false,
      "first_rows_truncated_observed": true,
      "selected_row_fetch_method": "hf_revision_load_dataset",
      "full_selected_row_hash": "sha256:<hex>",
      "agent_visible_row_hash": "sha256:<hex>",
      "source_field_policy_hash": "sha256:<hex>"
    }
  ],
  "world_snapshot": {
    "snapshot_manifest_hash": "sha256:<hex>",
    "noise_manifest_hash": "sha256:<hex>",
    "distractor_policy": "include_plausible_unrelated_context",
    "raw_dump_and_bundle_derive_from_same_snapshot": true
  },
  "models": [
    {
      "provider": "provider-name",
      "model_id": "exact-api-model-id",
      "model_id_alias_used": false,
      "model_family": "frontier-a",
      "api_version": "if applicable",
      "model_snapshot_source": "provider-doc-url-or-release-note",
      "model_release_or_snapshot_date": "YYYY-MM-DD|unknown",
      "availability_checked_at": "2026-05-25T00:00:00Z",
      "temperature": 0,
      "top_p": 1,
      "max_input_tokens": 0,
      "max_output_tokens": 0,
      "reasoning_or_effort_setting": "none|low|medium|high|provider_specific",
      "tool_call_mode": "disabled|auto|required|provider_specific",
      "context_window_observed": 0,
      "pricing_source_checked_at": "2026-05-25T00:00:00Z"
    }
  ],
  "arms": ["A_repo_only", "B_raw_dump", "C_parallax_bundle", "D_bundle_no_hypotheses"],
  "seed_count": 2,
  "token_ceiling_per_context": 0,
  "pre_registered_decision_rule": "C beats B by >=10pp, no higher median token cost, lower unsupported-claim rate",
  "rerun_triggers": [
    "new frontier model generation",
    "bundle template material change",
    "task-set contamination finding",
    "claim age > 90 days"
  ]
}
```

## Arm Result Row Schema

`arm-results.jsonl` is the audit spine. Each row represents exactly one
task/arm/model/seed attempt:

```json
{
  "run_id": "a1-phase0-2026-05-25-r001",
  "task_id": "swe-live-rust-example",
  "task_source": "SWE-bench-Live/MultiLang",
  "task_source_revision": "hf-revision-or-commit",
  "task_source_checked_at": "2026-05-25T00:00:00Z",
  "contamination_tier": "T1_fresh_public",
  "telemetry_provenance": ["observed_from_harness", "reconstructed_from_test_output"],
  "arm": "C_parallax_bundle",
  "model_id": "exact-api-model-id",
  "model_snapshot_hash": "sha256:<hex>",
  "agent_scaffold_hash": "sha256:<hex>",
  "model_family": "frontier-a",
  "seed": 1,
  "context_hash": "sha256:...",
  "bundle_schema_ref_hash": "sha256:...|not_applicable",
  "canonical_bundle_hash": "sha256:...|not_applicable",
  "projection_manifest_hash": "sha256:...|not_applicable",
  "projection_equivalence_passed": true,
  "mcp_structured_content_valid": true,
  "safety_fields_only_in_meta": false,
  "benchmark_source_snapshot_hash": "sha256:...",
  "normalized_overlay_hash": "sha256:...",
  "source_field_policy_hash": "sha256:...",
  "evidence_parity_passed": true,
  "gold_isolation_passed": true,
  "source_field_isolation_passed": true,
  "redaction_passed": true,
  "resolved": false,
  "root_cause_grade": "correct|partial|wrong|unsupported|not_applicable",
  "unsupported_claim_count": 0,
  "evidence_ref_count": 0,
  "input_tokens": 0,
  "output_tokens": 0,
  "tool_calls": 0,
  "wall_clock_seconds": 0,
  "patch_hash": "sha256:...",
  "grader_ref": "scorecards.md#task-id",
  "failure_class": "passed|wrong_fix|no_patch|timeout|tool_error|unsafe_patch|grader_error"
}
```

No result row counts toward A1 if `evidence_parity_passed`,
`gold_isolation_passed`, `source_field_isolation_passed`, or
`redaction_passed` is false. For Arm C and D, the row also does not count if
`bundle_schema_ref_hash`, `canonical_bundle_hash`, or
`projection_manifest_hash` is missing, if `projection_equivalence_passed` is
false, if MCP `structuredContent` is invalid for an MCP-delivered context, or if
any safety field appears only in `_meta`, a tool description, a prompt wrapper,
or Markdown.

Also exclude or downgrade rows when:

- the public dataset revision, row count, split count, or source-field
  quarantine summary is missing from `benchmark-source-snapshot.json`;
- the Hugging Face dataset SHA, source role, feature hash, selected-row full
  hash, agent-visible row hash, or datasets-server `partial=false` status is
  missing for a public Hugging Face task source;
- the task source changed after preregistration and before the run without a new
  task-set hash;
- the model was addressed through a mutable alias without a provider-visible
  snapshot/source record;
- the agent scaffold, system/developer prompts, container image, or tool-version
  manifest cannot be hashed.

## Contamination Tiers

Report task contamination risk explicitly:

| Tier | Description | Can support A1 gate? |
| --- | --- | --- |
| T0 old public | Historical public benchmark task or widely discussed issue. | Harness debugging only. |
| T1 fresh public | Recent public issue-resolution task, preferably from a live benchmark update. | Yes, but not alone. |
| T2 held-out or private audited | Operator/private or partner task with public redacted hashes and reviewer audit. | Yes; strong but watch n=1 bias. |
| T3 post-snapshot synthetic fault | Fault-injected reference task authored after model snapshot, with hidden verifier. | Yes for contamination resistance; weaker for wild-bug realism. |
| T4 production telemetry | Real incident with known fix and redacted, auditable evidence bundle. | Strongest for production claim if privacy permits audit. |

Public generated benchmark tasks such as OS-bench should normally be recorded as
T1 only when the task/update clearly post-dates the model snapshot or the
provider's likely training cutoff. Otherwise downgrade them to T0 for the A1
gate. Do not promote an external public generated task to T3 unless Parallax
controls the hidden fault/verifier and can prove it was inaccessible before the
run.

A public A1 pass should not rely on one tier. The first credible pass needs:

- at least one fresh public or live source;
- at least one synthetic or private/held-out source that post-dates the model
  snapshot or is inaccessible to model training;
- no single task source contributing more than half of the positive C-vs-B lift.

## Claim Levels

Use these labels in `result-ledger.md`:

| Claim level | Required evidence | Allowed wording |
| --- | --- | --- |
| `harness_debug` | Any incomplete or T0-heavy run. | "The harness works/does not work." |
| `provisional_signal` | 10-16 tasks, at least one model family, all no-cheat gates pass. | "Bundle value is promising enough for more research." |
| `a1_gate_pass` | 12+ tasks, at least two model families, mixed contamination tiers, C beats B per pre-registered rule, and no redaction/gold/parity/canonical-projection failures. | "A1 currently passes under the stated eval conditions." |
| `production_claim` | A1 gate pass plus real or fault-injected production-shaped telemetry reproduces the same direction. | "Bundles improved fixes on production-shaped evidence under these conditions." |

Never write "Parallax improves production fixes" from reconstructed public
benchmark overlays alone.

## Refresh Policy

Every A1 result has an expiration date. Default: **90 days from run date**.

Rerun earlier when any of these happen:

- a new frontier model generation or major coding-agent release is used in
  product positioning;
- any provider deprecates, aliases, or materially updates a model used in the
  last A1 pass;
- a public task-source dataset changes row counts, split counts, schema fields,
  license, Docker images, verifier commands, or source-field risk posture;
- OpenAI, SWE-bench, SWE-bench-Live, Terminal-Bench, or another primary source
  reports contamination or scoring issues affecting the task family used;
- the evidence-bundle schema, canonicalization method, projection renderer, MCP
  output schema, hypothesis block, truncation rule, source-field policy,
  redaction policy, or agent scaffold changes materially;
- more than 25 percent of the task set changes;
- A2 interviews produce real incidents that can replace synthetic/public tasks;
- a competitor publishes a credible telemetry-augmented agent eval.

Expired results still matter historically, but the GO verdict should treat A1 as
unproven again until a current run exists.

## Statistical And Reporting Rules

Minimum `statistics.md` contents:

- C-vs-B resolved-rate delta by model family;
- C-vs-A resolved-rate delta by model family;
- median token and wall-clock delta by arm;
- unsupported-claim-rate delta;
- per-task paired table;
- sensitivity excluding T2/T4 private tasks;
- sensitivity excluding the single largest C-vs-B contributor;
- bootstrap confidence interval over tasks where sample size permits;
- all discarded tasks and why they were discarded.

False-positive triggers:

| Trigger | Consequence |
| --- | --- |
| C wins only because it used more context than B | Do not count as bundle value; report as more-context value. |
| C evidence includes rows or facts not present in B's source overlay | Discard task for A1. |
| C context is only Markdown/text or has a projection hash mismatch | Discard the C/D row until the canonical bundle and projections are regenerated. |
| MCP-delivered context lacks valid `structuredContent` for the bundle schema | Discard the MCP-delivered row; it can debug the adapter but cannot prove A1. |
| One task accounts for most C-vs-B lift | Downgrade to provisional signal. |
| One model family wins while another loses | Do not claim model-general A1 pass. |
| Unsupported-claim rate rises in C | Treat as safety regression even if tests pass. |
| Any gold patch/test-patch leakage appears in agent context | Discard task and rotate canaries. |

Repo-intent sub-study results should be linked from the same A1 run only when
they use the same task/model/scaffold snapshots. The paired rows and claim
levels live in [Repo-intent value ledger](../repo-intent.md), because
repo-intent can improve constraint adherence while still failing broad-market
degraded-mode requirements.

## Bottom Line

A1 is a moving target. Frontier models, coding-agent scaffolds, public benchmark
sets, and contamination findings will keep changing. The only durable way to
use A1 as a GO gate is to treat every eval result as a versioned, expiring
research artifact with exact model snapshots, task provenance, evidence hashes,
canonical bundle hashes, projection audits, and public failure rows. If the
ledger is missing or expired, A1 is not proven.
