# A1 Source Drift And Leakage Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The A1 seed-corpus plan depends on public, moving Hugging Face datasets. The
previous freeze check pinned SHAs, row counts, and field quarantine, but still
left three risks under-specified:

- the Hugging Face preview rows may not be complete enough for row hashes;
- adjacent datasets in the same orgs may look relevant while leaking results,
  trajectories, or model metadata;
- the Python-only SWE-bench-Live dataset was visible but not assigned a role.

This pass rechecked the current primary APIs and tightens the task-source gate.
The concrete full-row retrieval and hashing workflow is specified in
[A1 Hugging Face row hash procedure](a1-huggingface-row-hash-procedure.md).

## Verdict

No drift was found in the covered pinned source SHAs or row counts on
2026-05-25. A same-day follow-up API recheck again found the same SWE-bench-Live
and Nebius V2 source SHAs, the same split counts, and
`first-rows.truncated=true` for each checked preview. The weak claim was not
freshness; it was source governance.

The follow-up did find one adjacent source that this note had not named:
`nebius/SWE-rebench`, the older automated SWE-rebench dataset. It is not a
trajectory or leaderboard dataset, so it should not be excluded for the same
reason as solved-run/result sources. It should still be treated as
`expansion_only_legacy_high_risk`: useful as historical context or later
expansion material only after the smaller seed proves the source-field policy,
and superseded for first-pass source selection by SWE-rebench V2.

The A1 source rule is now:

> Treat each public dataset as a versioned task source with an explicit role,
> license, visibility/gating state, and exact revision. Hugging Face `first-rows`
> previews are useful for schema inspection, but they are not sufficient row-hash
> evidence when `truncated=true`. Selected task rows need full-row fetches from
> the pinned dataset revision before field policy, redaction, and agent-visible
> projection hashes are computed.

## Current Source Snapshot

| Source | Current API state | A1 source role |
| --- | --- | --- |
| [SWE-bench-Live/MultiLang](https://huggingface.co/datasets/SWE-bench-Live/MultiLang) | SHA `608f7ae9ab8ea1f9f0d030fe04562cf6bd1a0c8b`; last modified `2026-05-16T02:18:12Z`; license tag `mit`; `private=false`, `disabled=false`, `gated=false`; 743 rows across C 37, C++ 74, Go 138, JS 93, Rust 94, Java 109, TS 111, C# 87; `partial=false`; `first-rows` returned `truncated=true`. | `seed_candidate`, especially Rust and fresh multilingual slices. |
| [SWE-bench-Live/OS-bench](https://huggingface.co/datasets/SWE-bench-Live/OS-bench) | SHA `53ccce58d8ca4d1273755658d68d4643afadb7de`; last modified `2026-05-23T02:25:29Z`; license tag `cc-by-4.0`; `private=false`, `disabled=false`, `gated=false`; 126 rows in `windows2linux`, 0 rows in `linux2windows`; `partial=false`; `first-rows` returned `truncated=true`. | `supplemental_cli_platform`; record the zero-row split and license separately from MIT seed sources. |
| [SWE-bench-Live/Windows](https://huggingface.co/datasets/SWE-bench-Live/Windows) | SHA `ac8b120eaf36957da1884dde9f71fd28ed632487`; last modified `2026-05-14T14:42:33Z`; license tag `mit`; `private=false`, `disabled=false`, `gated=false`; 61 `test` rows; `partial=false`; `first-rows` returned `truncated=true`. | `supplemental_cli_platform`; not production telemetry. |
| [SWE-bench-Live/SWE-bench-Live](https://huggingface.co/datasets/SWE-bench-Live/SWE-bench-Live) | SHA `a637bd46829f3132e12938c8a0ca93173a977b8e`; last modified `2025-09-18T07:36:47Z`; license tag `mit`; `private=false`, `disabled=false`, `gated=false`; 3,688 rows across `test` 1000, `lite` 300, `verified` 500, `full` 1888; `first-rows` returned `truncated=true`. | `harness_shakeout` or Python-only supplement; not the Rust-first default seed. |
| [nebius/SWE-rebench-V2](https://huggingface.co/datasets/nebius/SWE-rebench-V2) | SHA `475dd5e8703bb5fb22dd3c60b5d038b019eba1e0`; last modified `2026-05-12T14:00:30Z`; license tag `cc-by-4.0`; `private=false`, `disabled=false`, `gated=false`; 32,079 train rows; `partial=false`; `first-rows` returned `truncated=true`. | `expansion_only` after inspected seed tasks; keep license and generated-source status visible. |
| [nebius/SWE-rebench-V2-PRs](https://huggingface.co/datasets/nebius/SWE-rebench-V2-PRs) | SHA `40faf2c1bb160de625f3c3270ac9d62ea45f3f9c`; last modified `2026-03-03T09:41:05Z`; license tag `cc-by-4.0`; `private=false`, `disabled=false`, `gated=false`; 126,300 train rows; `partial=false`; `first-rows` returned `truncated=true`; preview schema includes `meta.llm_metadata`. | `expansion_only_high_risk`; do not use as a seed source. |
| [nebius/SWE-rebench](https://huggingface.co/datasets/nebius/SWE-rebench) | SHA `89cdfbab4ab1bd8f5a658bb212d1b63624f4f881`; last modified `2025-12-23T19:41:57Z`; license tag `cc-by-4.0`; `private=false`, `disabled=false`, `gated=false`; 27,878 rows across `test` 21,336 and `filtered` 6,542; `partial=false`; `first-rows` on `filtered` returned `truncated=true`; README describes a fully automated issue/PR harvesting pipeline with LLM-derived setup and task-quality fields. | `expansion_only_legacy_high_risk`; do not use as a Phase 0 seed when V2 and smaller fresh sources exist. |

The current [SWE-bench-Live org API](https://huggingface.co/api/datasets?author=SWE-bench-Live&search=SWE-bench-Live)
lists only the four SWE-bench-Live datasets above. The current Nebius search
also lists [SWE-rebench](https://huggingface.co/datasets/nebius/SWE-rebench),
[SWE-rebench OpenHands trajectories](https://huggingface.co/datasets/nebius/SWE-rebench-openhands-trajectories),
and [SWE-rebench leaderboard](https://huggingface.co/datasets/nebius/SWE-rebench-leaderboard).
The older SWE-rebench dataset is expansion-only legacy material. The trajectory
and leaderboard datasets should be `excluded_leakage_source` for A1 task
selection because they can reveal solved paths, tool actions, model behavior, or
benchmark outcomes. They may inform contamination tests, not task prompts.

## Leakage Findings

`first-rows` is a schema preview, not a source-of-truth row snapshot. For every
checked dataset preview in this pass, the response included `truncated=true`.
That means it can confirm field names and rough field shape, but it cannot prove
the complete selected row content, support a stable task-row hash, or audit
whether long fields hide solution text.

Field policy also needs to extend past obvious gold artifacts:

| Field class | A1 policy |
| --- | --- |
| Gold artifacts | `patch`, `test_patch`, fail-to-pass tests, pass-to-pass tests, fixed commits, and resolving commit URLs stay grader-private or hashed audit metadata. |
| Generated or model-derived metadata | `hints_text`, `all_hints_text`, `interface`, `meta`, `llm_metadata`, PR descriptions, difficulty labels, and filter annotations are triage-private by default. |
| Runner metadata | Docker images, install configs, rebuild/test/print commands, log parsers, cache settings, and resource settings may run the harness, but parser bodies and exact verifier internals are not agent context. |
| Issue text | `problem_statement` can be agent-visible only after a leakage review checks for embedded solution, post-fix, hidden-test, or generated-hint content. |
| Result/trajectory datasets | Leaderboards, agent trajectories, and solved-run traces are excluded as task sources unless the experiment is explicitly about contamination detection. |

The PR-scale SWE-rebench source is the riskiest expansion source because its
schema combines issue text with `hints_text`, `interface`, `install_config`,
`meta`, and `meta.llm_metadata`. The older SWE-rebench dataset has related
automated-pipeline risk: `hints_text`, `install_config`, `meta`, environment
fields, Docker/image fields, gold patches, and verifier lists all need the same
quarantine posture. A1 can mine these later, but only after a smaller seed run
proves the source-field policy and full-row hashing workflow.

## Required Snapshot Method

For each public Hugging Face task source, `benchmark-source-snapshot.json` should
record:

```json
{
  "source": "SWE-bench-Live/MultiLang",
  "source_role": "seed_candidate",
  "hf_dataset_sha": "608f7ae9ab8ea1f9f0d030fe04562cf6bd1a0c8b",
  "hf_last_modified": "2026-05-16T02:18:12Z",
  "source_license": "mit",
  "source_visibility": {
    "private": false,
    "gated": false,
    "disabled": false
  },
  "hub_tags": ["license:mit", "size_categories:n<1K", "format:parquet"],
  "datasets_server_size_checked_at": "2026-05-25T00:00:00Z",
  "datasets_server_size_partial": false,
  "row_count": 743,
  "split_counts": {"rust": 94},
  "features_hash": "sha256:<hash-of-feature-list>",
  "source_field_policy_hash": "sha256:<hash>",
  "first_rows_checked": true,
  "first_rows_truncated_observed": true,
  "selected_row_fetch_method": "hf_revision_load_dataset",
  "full_selected_row_hash": "sha256:<hash-before-field-removal>",
  "agent_visible_row_hash": "sha256:<hash-after-field-policy>"
}
```

The selected-row hash should come from a full row fetched from the pinned dataset
revision, not from `first-rows`. The hash workflow should canonicalize JSON after
normalizing field order and before separating `agent_visible_seed`,
`runner_private`, `grader_private`, `triage_private`, and `public_audit` fields.

## Impact On A1

- MultiLang remains the best public seed candidate, but its selected rows still
  require full-row hashing and leakage review.
- OS-bench and Windows remain useful for one CLI/platform slice at most.
- The Python-only SWE-bench-Live dataset is available for harness shakeout or a
  Python supplement, but it should not displace Rust-first coverage.
- SWE-rebench V2 remains scale-out material; SWE-rebench V2 PRs is high-risk
  expansion material, not a first seed.
- The older `nebius/SWE-rebench` dataset is now explicitly classified as
  `expansion_only_legacy_high_risk`, not a seed source. Prefer V2 or smaller
  fresh sources unless a later experiment needs historical comparison.
- OpenHands trajectory and leaderboard datasets are excluded from task-source
  use because they are too close to solved-run/result evidence.

## Falsification Triggers

Recheck this note before using it if:

- any dataset SHA, license tag, privacy flag, row count, split count, or feature
  list changes;
- any source becomes gated, private, disabled, deleted, or re-licensed;
- Hugging Face `first-rows` behavior changes or stops reporting truncation for
  a source used by A1;
- A1 selects a new task source or a new split not covered here;
- a selected row cannot be fetched in full from the pinned revision;
- leakage review finds gold-patch, hidden-test, post-fix, model-generated, or
  trajectory evidence inside an agent-visible field.

## Sources

- [Hugging Face dataset API: SWE-bench-Live/MultiLang](https://huggingface.co/api/datasets/SWE-bench-Live/MultiLang)
- [Datasets-server size: SWE-bench-Live/MultiLang](https://datasets-server.huggingface.co/size?dataset=SWE-bench-Live/MultiLang)
- [Datasets-server first rows: SWE-bench-Live/MultiLang rust](https://datasets-server.huggingface.co/first-rows?dataset=SWE-bench-Live%2FMultiLang&config=default&split=rust)
- [Hugging Face dataset API: SWE-bench-Live/OS-bench](https://huggingface.co/api/datasets/SWE-bench-Live/OS-bench)
- [Datasets-server size: SWE-bench-Live/OS-bench](https://datasets-server.huggingface.co/size?dataset=SWE-bench-Live/OS-bench)
- [Datasets-server first rows: SWE-bench-Live/OS-bench](https://datasets-server.huggingface.co/first-rows?dataset=SWE-bench-Live%2FOS-bench&config=default&split=windows2linux)
- [Hugging Face dataset API: SWE-bench-Live/Windows](https://huggingface.co/api/datasets/SWE-bench-Live/Windows)
- [Datasets-server size: SWE-bench-Live/Windows](https://datasets-server.huggingface.co/size?dataset=SWE-bench-Live/Windows)
- [Datasets-server first rows: SWE-bench-Live/Windows](https://datasets-server.huggingface.co/first-rows?dataset=SWE-bench-Live%2FWindows&config=default&split=test)
- [Hugging Face dataset API: SWE-bench-Live/SWE-bench-Live](https://huggingface.co/api/datasets/SWE-bench-Live/SWE-bench-Live)
- [Datasets-server size: SWE-bench-Live/SWE-bench-Live](https://datasets-server.huggingface.co/size?dataset=SWE-bench-Live/SWE-bench-Live)
- [Datasets-server first rows: SWE-bench-Live/SWE-bench-Live](https://datasets-server.huggingface.co/first-rows?dataset=SWE-bench-Live%2FSWE-bench-Live&config=default&split=test)
- [Hugging Face dataset API: SWE-rebench V2](https://huggingface.co/api/datasets/nebius/SWE-rebench-V2)
- [Datasets-server size: SWE-rebench V2](https://datasets-server.huggingface.co/size?dataset=nebius/SWE-rebench-V2)
- [Datasets-server first rows: SWE-rebench V2](https://datasets-server.huggingface.co/first-rows?dataset=nebius%2FSWE-rebench-V2&config=default&split=train)
- [Hugging Face dataset API: SWE-rebench V2 PRs](https://huggingface.co/api/datasets/nebius/SWE-rebench-V2-PRs)
- [Datasets-server size: SWE-rebench V2 PRs](https://datasets-server.huggingface.co/size?dataset=nebius/SWE-rebench-V2-PRs)
- [Datasets-server first rows: SWE-rebench V2 PRs](https://datasets-server.huggingface.co/first-rows?dataset=nebius%2FSWE-rebench-V2-PRs&config=default&split=train)
- [Hugging Face dataset API: SWE-rebench](https://huggingface.co/api/datasets/nebius/SWE-rebench)
- [Datasets-server size: SWE-rebench](https://datasets-server.huggingface.co/size?dataset=nebius/SWE-rebench)
- [Datasets-server first rows: SWE-rebench filtered](https://datasets-server.huggingface.co/first-rows?dataset=nebius%2FSWE-rebench&config=default&split=filtered)
- [SWE-rebench dataset card](https://huggingface.co/datasets/nebius/SWE-rebench)
- [Hugging Face API search: SWE-bench-Live org datasets](https://huggingface.co/api/datasets?author=SWE-bench-Live&search=SWE-bench-Live)
- [Hugging Face API search: Nebius SWE-rebench datasets](https://huggingface.co/api/datasets?author=nebius&search=SWE-rebench)

## Bottom Line

The A1 corpus plan still points in the right direction, but the freeze gate must
be stricter. Public benchmark rows are not safe agent context until Parallax has
assigned a source role, fetched the full selected row from a pinned revision,
hashed both full and agent-visible projections, and excluded trajectory/result
datasets from task-source use.
