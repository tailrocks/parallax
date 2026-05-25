# A1 Task Source Freeze Check

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The A1 bundle-value eval depends on moving public datasets. The current A1 docs
say to freeze dataset revisions, row counts, split counts, and source-field
policies, but the seed-corpus note still used mostly human-readable "checked"
phrasing.

This note pins the current primary API snapshot for the most likely Phase 0 task
sources and makes one rule explicit:

> A future A1 run must record dataset repository SHA plus datasets-server row and
> feature snapshots. A row count without a dataset SHA is not enough.

The companion [A1 source drift and leakage recheck](a1-source-drift-and-leakage-recheck.md)
tightens the selected-row workflow: Hugging Face `first-rows` previews were
observed with `truncated=true`, so they cannot be the source of row hashes.

## Result

No row-count drift was found from the current A1 notes for the checked datasets.
The missing piece was not a changed count; it was insufficient pinning detail.

| Dataset | HF API SHA | Last modified | Size snapshot | Phase 0 use |
| --- | --- | --- | --- | --- |
| [SWE-bench-Live/MultiLang](https://huggingface.co/datasets/SWE-bench-Live/MultiLang) | `608f7ae9ab8ea1f9f0d030fe04562cf6bd1a0c8b` | `2026-05-16T02:18:12Z` | 743 rows, 20 columns. Splits: C 37, C++ 74, Go 138, JS 93, Rust 94, Java 109, TS 111, C# 87. `partial=false`. | Best first public seed source because it is fresh, executable, multilingual, and includes a real Rust slice. |
| [SWE-bench-Live/OS-bench](https://huggingface.co/datasets/SWE-bench-Live/OS-bench) | `53ccce58d8ca4d1273755658d68d4643afadb7de` | `2026-05-23T02:25:29Z` | 126 rows, 17 columns. Splits: `windows2linux` 126, `linux2windows` 0. `partial=false`. | Useful for at most one OS/CLI/platform task. The zero-row split is a manifest trap and must be recorded. |
| [SWE-bench-Live/Windows](https://huggingface.co/datasets/SWE-bench-Live/Windows) | `ac8b120eaf36957da1884dde9f71fd28ed632487` | `2026-05-14T14:42:33Z` | 61 rows, 21 columns. Split: `test` 61. `partial=false`. | Supplemental Windows/platform slice, not a production-telemetry substitute. |
| [SWE-bench-Live/SWE-bench-Live](https://huggingface.co/datasets/SWE-bench-Live/SWE-bench-Live) | `a637bd46829f3132e12938c8a0ca93173a977b8e` | `2025-09-18T07:36:47Z` | 3,688 total rows, 18 columns per split. Splits: `test` 1000, `lite` 300, `verified` 500, `full` 1888. `partial=false`. | Python-only harness shakeout or supplement, not the Rust-first default seed source. |
| [nebius/SWE-rebench-V2](https://huggingface.co/datasets/nebius/SWE-rebench-V2) | `475dd5e8703bb5fb22dd3c60b5d038b019eba1e0` | `2026-05-12T14:00:30Z` | 32,079 rows, 16 columns. Split: `train` 32,079. `partial=false`. | Expansion source after the seed run; too large/automatic for the first headline Phase 0 slice. |
| [nebius/SWE-rebench-V2-PRs](https://huggingface.co/datasets/nebius/SWE-rebench-V2-PRs) | `40faf2c1bb160de625f3c3270ac9d62ea45f3f9c` | `2026-03-03T09:41:05Z` | 126,300 rows, 16 columns. Split: `train` 126,300. `partial=false`. | PR-scale expansion/corpus-mining source, not a seed source. |

## Field Quarantine Snapshot

The checked datasets mix useful issue text with runner metadata, hidden verifier
inputs, gold patches, generated hints, and LLM/filter metadata. These fields
were confirmed through the Hugging Face datasets-server `first-rows` endpoint.

| Dataset family | Fields that need quarantine before agent context |
| --- | --- |
| SWE-bench-Live MultiLang and Windows | `patch`, `test_patch`, `hints_text`, `all_hints_text`, `commit_urls`, `commit_url`, `log_parser`, `FAIL_TO_PASS`, `PASS_TO_PASS`, plus runner-only command/image fields such as `docker_image`, `rebuild_cmds`, `test_cmds`, and `print_cmds`. |
| SWE-bench-Live Python-only | `patch`, `test_patch`, `hints_text`, `all_hints_text`, `commit_urls`, `commit_url`, `log_parser`, `difficulty`, `FAIL_TO_PASS`, `PASS_TO_PASS`, and `test_cmds`. |
| SWE-bench-Live OS-bench | `patch`, `test_patch`, `FAIL_TO_PASS`, `PASS_TO_PASS`, `log_parser`, `metadata`, `difficulty`, `migration_direction`, `docker_image`, `rebuild_cmds`, `test_cmds`, and `print_cmds`. |
| SWE-rebench V2 / PRs | `patch`, `test_patch`, `FAIL_TO_PASS`, `PASS_TO_PASS`, `interface`, `meta`, `install_config`, and `pr_description`. |

The practical rule is stricter than "hide the patch": runner/parser fields can
shape the harness, but they are not automatically agent-visible evidence. If any
of these fields is promoted into Arm A, B, C, or D, the run must pre-register the
reason and downgrade contamination risk if the field leaks implementation or
grader knowledge.

## Snapshot Method Correction

The Hugging Face datasets-server `first-rows` endpoint is useful for confirming
field names and rough shape, but this recheck observed `truncated=true` for the
selected previews. Do not use preview rows as full row hashes or full leakage
audits. A valid A1 source snapshot must combine:

- Hugging Face dataset repository SHA;
- datasets-server size snapshot with `partial=false`;
- feature-list hash;
- explicit `source_role`;
- `first_rows_truncated_observed`;
- full selected-row JSON fetched from the pinned dataset revision;
- full selected-row hash before field removal;
- agent-visible row hash after source-field policy.

## Required Manifest Fields

Every public task-source entry in `benchmark-source-snapshot.json` should include
at least:

```json
{
  "source": "SWE-bench-Live/MultiLang",
  "hf_dataset_sha": "608f7ae9ab8ea1f9f0d030fe04562cf6bd1a0c8b",
  "hf_last_modified": "2026-05-16T02:18:12Z",
  "datasets_server_size_checked_at": "2026-05-25T00:00:00Z",
  "datasets_server_size_partial": false,
  "row_count": 743,
  "column_count": 20,
  "split_counts": {
    "rust": 94
  },
  "features_hash": "sha256:<hash-of-feature-list>",
  "source_field_policy_hash": "sha256:<hash>",
  "source_role": "seed_candidate",
  "first_rows_truncated_observed": true,
  "selected_row_fetch_method": "pinned_revision_parquet_or_datasets_library",
  "full_selected_row_hash": "sha256:<hash-before-field-removal>",
  "agent_visible_row_hash": "sha256:<hash-after-field-policy>"
}
```

For selected task rows, `source.json` should copy the dataset SHA and a
row-level hash of the source record after removing private fields from public
artifacts. The run can store grader-private hashes for patch/test data, but not
the patch/test contents in agent-visible contexts.

## Impact On A1

This pass strengthens A1 without changing its decision gate:

- C still must beat B; row-count freshness does not prove bundle value.
- SWE-bench-Live MultiLang remains the best public seed source.
- OS-bench and Windows remain supplemental, because they are public/generated
  platform tasks and can carry high-risk source fields.
- SWE-bench-Live Python-only is useful for harness shakeout or a Python
  supplement, but not as the Rust-first default source.
- SWE-rebench V2 remains a scale-out source after a smaller inspected seed run.
- Adjacent trajectory or leaderboard/result datasets should be excluded as task
  sources unless the experiment is explicitly a contamination study.
- Any A1 result without dataset SHA, row-count snapshot, feature snapshot,
  source role, selected-row hash method, source-field policy hash, and
  `partial=false` status should be downgraded to `harness_debug`.

## Falsification Triggers

Mark this snapshot stale and recheck before citing it when:

- any checked dataset SHA changes;
- datasets-server reports a changed row count, split count, feature list, or
  `partial=true`;
- Hugging Face marks a dataset private/disabled or changes license tags;
- source fields add new generated hints, verifier fields, LLM metadata, command
  fields, or resolving references;
- selected rows cannot be fetched in full from a pinned revision;
- the A1 run uses a new task family not covered here.

## Sources

- [Hugging Face dataset API: SWE-bench-Live/MultiLang](https://huggingface.co/api/datasets/SWE-bench-Live/MultiLang)
- [Datasets-server size: SWE-bench-Live/MultiLang](https://datasets-server.huggingface.co/size?dataset=SWE-bench-Live/MultiLang)
- [Hugging Face dataset API: SWE-bench-Live/OS-bench](https://huggingface.co/api/datasets/SWE-bench-Live/OS-bench)
- [Datasets-server size: SWE-bench-Live/OS-bench](https://datasets-server.huggingface.co/size?dataset=SWE-bench-Live/OS-bench)
- [Hugging Face dataset API: SWE-bench-Live/Windows](https://huggingface.co/api/datasets/SWE-bench-Live/Windows)
- [Datasets-server size: SWE-bench-Live/Windows](https://datasets-server.huggingface.co/size?dataset=SWE-bench-Live/Windows)
- [Hugging Face dataset API: SWE-bench-Live/SWE-bench-Live](https://huggingface.co/api/datasets/SWE-bench-Live/SWE-bench-Live)
- [Datasets-server size: SWE-bench-Live/SWE-bench-Live](https://datasets-server.huggingface.co/size?dataset=SWE-bench-Live/SWE-bench-Live)
- [Hugging Face dataset API: SWE-rebench V2](https://huggingface.co/api/datasets/nebius/SWE-rebench-V2)
- [Datasets-server size: SWE-rebench V2](https://datasets-server.huggingface.co/size?dataset=nebius/SWE-rebench-V2)
- [Hugging Face dataset API: SWE-rebench V2 PRs](https://huggingface.co/api/datasets/nebius/SWE-rebench-V2-PRs)
- [Datasets-server size: SWE-rebench V2 PRs](https://datasets-server.huggingface.co/size?dataset=nebius/SWE-rebench-V2-PRs)
- [Hugging Face API search: SWE-bench-Live org datasets](https://huggingface.co/api/datasets?author=SWE-bench-Live&search=SWE-bench-Live)
- [Hugging Face API search: Nebius SWE-rebench datasets](https://huggingface.co/api/datasets?author=nebius&search=SWE-rebench)

## Bottom Line

The task-source facts still support the planned A1 seed run, but only if the
future run freezes dataset SHAs and feature snapshots. Moving public benchmarks
are useful input sources, not stable evidence unless Parallax commits the exact
snapshot it used.
