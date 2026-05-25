# A1 Hugging Face Row Hash Procedure

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The A1 source-drift recheck found that Hugging Face datasets-server `first-rows`
previews are truncated for the likely task sources. The A1 docs now require
full selected-row hashes from pinned dataset revisions, but that wording was
still too loose.

This note defines the concrete procedure for Hugging Face-backed task rows:

> A public Hugging Face task can count toward A1 only when Parallax records the
> pinned dataset commit, the exact split/config/row identity, the full decoded
> source-row hash, the source-field policy hash, and the agent-visible row hash.
> Preview endpoints can help inspect schemas, but they are not hash input.

## Current Primary-Source Checks

| Source | What it supports | A1 implication |
| --- | --- | --- |
| [Hugging Face Datasets loading docs](https://huggingface.co/docs/datasets/loading) | `load_dataset()` accepts a `revision` parameter; docs say the revision can be a tag, branch, or commit hash. | Preferred decoded-row path: load the dataset at the exact `hf_dataset_sha`, then select by split plus stable row identity. |
| [Hugging Face Hub download guide](https://huggingface.co/docs/huggingface_hub/en/guides/download) | `hf_hub_download()` and `snapshot_download()` accept `revision`; the guide says full commit hashes are required for commit-pinned downloads. | File-backed path: download only needed Parquet/JSON files from the pinned dataset repo, not from mutable `main`. |
| [HF URI syntax](https://huggingface.co/docs/huggingface_hub/main/package_reference/hf_uris) | `hf://[<TYPE>/]<ID>[@<REVISION>][/<PATH>]` supports dataset URIs pinned at branch, tag, commit SHA, or special ref. | For Parquet-backed sources, record canonical `hf://datasets/<repo>@<sha>/<path>` URIs for source files. |
| [HfApi `get_paths_info`](https://huggingface.co/docs/huggingface_hub/main/package_reference/hf_api#get_paths_info) | Path metadata can be requested with `revision` and `repo_type="dataset"`, returning file size, blob ID, and LFS/Xet metadata. | `benchmark-source-snapshot.json` should record file path metadata for any source file used to derive a selected row. |
| [Hugging Face Hub API endpoints](https://huggingface.co/docs/hub/en/api) | Hugging Face publishes open Hub API endpoints and an OpenAPI reference. | Use the dataset API to verify `sha`, `lastModified`, privacy/gating flags, siblings, and license metadata before task selection. |

A concrete current check against
`SWE-bench-Live/MultiLang@608f7ae9ab8ea1f9f0d030fe04562cf6bd1a0c8b` showed the
revision API returning the same SHA and the Rust Parquet file
`data/rust-00000-of-00001.parquet`. A `resolve/<sha>/...` header check for that
file returned `x-repo-commit: 608f7ae9ab8ea1f9f0d030fe04562cf6bd1a0c8b`,
`x-linked-etag`, and a final content length of `23514041` bytes. That is enough
to prove the pinned file path is addressable; it is not a substitute for the
decoded row hash.

## Required Row Identity

Every selected row must have a stable identity block:

```json
{
  "source": "SWE-bench-Live/MultiLang",
  "source_role": "seed_candidate",
  "hf_dataset_sha": "608f7ae9ab8ea1f9f0d030fe04562cf6bd1a0c8b",
  "dataset_config": "default",
  "split": "rust",
  "row_selector": {
    "kind": "instance_id",
    "value": "apache__datafusion-16726"
  },
  "row_offset_observed": 0,
  "source_file_uri": "hf://datasets/SWE-bench-Live/MultiLang@608f7ae9ab8ea1f9f0d030fe04562cf6bd1a0c8b/data/rust-00000-of-00001.parquet"
}
```

Use `instance_id` or an equivalent source-native task ID as the primary selector.
Record row offset only as an observation, not as the sole identity. Reject or
downgrade the task if the selected ID is missing, duplicated, or moves under the
same dataset SHA.

## Fetch Procedure

Use this order:

1. Verify dataset metadata through the Hub API or `HfApi.dataset_info()` at the
   exact `hf_dataset_sha`. Record `sha`, `lastModified`, license, privacy/gating
   flags, feature schema, split counts, and siblings.
2. Prefer `datasets.load_dataset(source, name=config, split=split,
   revision=hf_dataset_sha, trust_remote_code=False)` for decoded row material.
   Record the `datasets`, `huggingface_hub`, `pyarrow`, and Python versions.
3. If the dataset is file-backed and the source file path is known, use a pinned
   file URI such as
   `hf://datasets/SWE-bench-Live/MultiLang@<sha>/data/rust-00000-of-00001.parquet`
   or `hf_hub_download(repo_id=<repo>, filename=<path>, repo_type="dataset",
   revision=<sha>)`. Record path metadata and the final resolved commit/header
   evidence.
4. Select the row by stable ID. If using file-backed Parquet, decode the row
   with a pinned Parquet reader and record reader versions.
5. Independently verify the selected row's source-native ID, repo, base commit,
   and split against the task manifest before any field is removed.
6. Do not use datasets-server `first-rows` or `rows` preview responses as hash
   input. They can document schema shape and truncation status only.

If a dataset requires remote dataset code, custom transforms, or network calls to
materialize rows, do not count it toward A1 until the code is reviewed and pinned
or the row is re-exported into a simple pinned file source.

## Canonicalization

Before hashing, convert the decoded row to deterministic JSON:

- encode as UTF-8 JSON using RFC 8785/JCS canonicalization;
- preserve source strings exactly after JSON encoding;
- keep arrays in source order;
- sort object keys through JCS;
- render timestamps as RFC 3339 UTC strings;
- render bytes as base64 strings with field-level type notes;
- reject `NaN`, infinities, lossy floats, or unsupported scalar types unless a
  type-specific conversion is documented in `row-hash-procedure.json`.

Hash the canonical byte sequence with SHA-256.

## Hashes To Record

Each selected task should produce these hash records:

| Hash | Input | Public? |
| --- | --- | --- |
| `features_hash` | Canonical feature schema for the source config/split, not just field names. | Yes |
| `source_file_hash` | File metadata hash over path, size, blob/LFS/Xet IDs, ETag/header evidence, and pinned URI when file-backed. | Yes |
| `full_selected_row_hash` | Canonical full decoded row before field removal. | Yes, but not the private row content. |
| `source_field_policy_hash` | Canonical policy assigning every field to `agent_visible_seed`, `runner_private`, `grader_private`, `triage_private`, or `public_audit`. | Yes |
| `agent_visible_row_hash` | Canonical row after applying the source-field policy to agent-visible/audit-safe fields. | Yes |
| `grader_private_hash` | Canonical denied gold/verifier fields such as patch, test patch, fail/pass IDs, and resolving refs. | Yes, hash only |
| `triage_private_hash` | Canonical denied hints, generated metadata, LLM metadata, interfaces, PR descriptions, and quality labels. | Yes, hash only |

Do not commit `source-row.full.json` when it contains gold patch, test patch,
hidden verifier, resolving ref, generated hint, or LLM metadata. Commit
`source-row.public.json` only after source-field policy and redaction.

## Manifest Fields

Add these fields to `source.json` for Hugging Face task rows:

```json
{
  "selected_row_fetch_method": "hf_revision_load_dataset",
  "selected_row_fetch_ref": {
    "library": "datasets",
    "library_version": "x.y.z",
    "hub_library_version": "x.y.z",
    "pyarrow_version": "x.y.z",
    "python_version": "3.x.y",
    "revision": "608f7ae9ab8ea1f9f0d030fe04562cf6bd1a0c8b",
    "config": "default",
    "split": "rust",
    "selector": {"instance_id": "apache__datafusion-16726"}
  },
  "source_file_uri": "hf://datasets/SWE-bench-Live/MultiLang@608f7ae9ab8ea1f9f0d030fe04562cf6bd1a0c8b/data/rust-00000-of-00001.parquet",
  "source_file_hash": "sha256:<metadata-hash>",
  "features_hash": "sha256:<canonical-feature-schema>",
  "full_selected_row_hash": "sha256:<canonical-full-row>",
  "source_field_policy_hash": "sha256:<canonical-field-policy>",
  "agent_visible_row_hash": "sha256:<canonical-agent-visible-row>"
}
```

For file-backed rows, set `selected_row_fetch_method` to
`hf_revision_file_decode` and include the exact reader/version block. For
datasets-server previews, set `selected_row_fetch_method` to `schema_preview`
only; those rows cannot count toward A1.

## Pass And Fail Rules

An A1 task may count when:

- dataset SHA, feature schema, split count, source role, source file metadata,
  and selected-row hashes are recorded before the agent run;
- the selected row is fetched by stable ID from the pinned revision;
- full-row hash and agent-visible row hash are derived from deterministic JSON;
- denied field hashes exist for grader-private and triage-private content;
- source-field policy is committed before Arm A/B/C/D artifacts are generated.

Downgrade the task to `harness_debug` when:

- only `first-rows`, `rows`, dataset viewer UI, or mutable `main` was used;
- the row selector is only an offset;
- the source requires unreviewed remote code;
- row decoding is not reproducible across the recorded tool versions;
- the agent-visible row contains gold patch, hidden verifier, resolving ref,
  generated hint, LLM metadata, or trajectory/result evidence.

## Sources

- [Hugging Face Datasets loading docs](https://huggingface.co/docs/datasets/loading)
- [Hugging Face Hub download guide](https://huggingface.co/docs/huggingface_hub/en/guides/download)
- [Hugging Face HF URI syntax](https://huggingface.co/docs/huggingface_hub/main/package_reference/hf_uris)
- [Hugging Face Hub API docs](https://huggingface.co/docs/hub/en/api)
- [HfApi `get_paths_info` docs](https://huggingface.co/docs/huggingface_hub/main/package_reference/hf_api#get_paths_info)
- [Hugging Face revision API: SWE-bench-Live/MultiLang pinned SHA](https://huggingface.co/api/datasets/SWE-bench-Live/MultiLang/revision/608f7ae9ab8ea1f9f0d030fe04562cf6bd1a0c8b)
- [Pinned Rust Parquet file: SWE-bench-Live/MultiLang](https://huggingface.co/datasets/SWE-bench-Live/MultiLang/resolve/608f7ae9ab8ea1f9f0d030fe04562cf6bd1a0c8b/data/rust-00000-of-00001.parquet)
- [Datasets-server first rows: SWE-bench-Live/MultiLang rust](https://datasets-server.huggingface.co/first-rows?dataset=SWE-bench-Live%2FMultiLang&config=default&split=rust)

## Bottom Line

The A1 seed can rely on Hugging Face sources only if row identity and row hashes
come from pinned decoded data, not previews. This turns the previous "fetch full
row from pinned revision" rule into an auditable workflow that future eval runs
can implement without silently leaking gold fields into agent context.
