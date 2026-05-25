# Rust Stacktrace Grouping Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

[Rust stacktrace grouping and symbolication](rust-stacktrace-grouping-and-symbolication.md)
defines the `rust-stack-v1` grouping design, debuginfo policy, symbolication
status fields, and fixture matrix. This ledger defines the result artifacts and
claim levels required before Parallax can say Rust grouping is deterministic,
stable, or safe for agent-facing evidence bundles.

Current status: **not measured**. The repository has a grouping design and
fixture matrix, but no fixture-generated results. Until those results exist,
Parallax should treat Rust grouping as a planned proof gate, not a product
property.

The central rule:

> No "deterministic Rust grouping" claim without dated fixture runs covering
> rebuilds, debuginfo variants, frame normalization, false splits, false merges,
> client fingerprints, symbolication degradation, and grouping-version stability.

This ledger is narrower than the
[Sentry SDK compatibility ledger](sentry-sdk-compatibility-ledger.md): Sentry
Rust envelopes can parse and normalize before Rust grouping is proven.

## Current Source Snapshot

| Source | Current check | Why it matters |
| --- | --- | --- |
| [Rust `std::backtrace`](https://doc.rust-lang.org/std/backtrace/index.html) | Rust docs currently show `std` 1.95.0 and state that backtraces are best-effort, file/line reporting usually needs debug information, `Backtrace::capture` is disabled unless environment variables enable it, and `force_capture` bypasses those gates. | The ledger must record capture mode, status, and debuginfo availability; absence of file/line is expected and must become low confidence, not fabricated precision. |
| [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) | Release defaults have `debug = false`; `line-tables-only` is the minimal debuginfo level for filename/line backtraces; `split-debuginfo` and `strip` alter where symbols live. | Fixture variants must include release default, line tables, full debuginfo, split debuginfo, and stripped binaries. |
| [rustc codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html) | Frame-pointer and unwind-table defaults depend on target, while flags can force them. | The run manifest must pin target triple and stack-walking knobs because grouping stability is target-sensitive. |
| [rustc symbol mangling](https://doc.rust-lang.org/stable/rustc/symbol-mangling/index.html) | Rust symbol names are mangled for linker uniqueness; tooling may need demangling; rustc supports multiple mangling versions. | Store raw, demangled, and normalized symbols. Do not assume one mangling version or one compiler release. |
| [Sentry issue grouping](https://docs.sentry.io/concepts/data-management/event-grouping/) | Sentry versions grouping algorithms, considers fingerprint first, then stack trace, exception, and message, and uses in-app frames when available. | Parallax should copy the versioning discipline and client-fingerprint precedence, not claim exact Sentry grouping parity. |
| [Sentry debug identifiers](https://docs.sentry.io/platforms/flutter/data-management/debug-files/identifiers/) | Sentry distinguishes code identifiers from debug identifiers and uses debug IDs to locate matching debug companions. | Parallax should record build/debug IDs and symbol-file matching results as evidence, but not include build IDs in the logical issue identity. |
| [sentry Rust crate 0.48.2](https://docs.rs/sentry/latest/sentry/) | Docs.rs currently resolves `sentry` to `0.48.2`; default features include backtrace, contexts, panic capture, transport, and debug-image metadata. | The first fixture target remains current Sentry Rust SDK panic/error envelopes. |

## Claim Levels

Use these levels in `claim-ledger.jsonl`:

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No current fixture run exists. | "Rust grouping design exists; results pending." |
| `fixture_harness_ready` | Synthetic Rust apps can generate raw Sentry envelopes and build variants. | "Rust grouping fixture harness prepared." |
| `debug_info_policy_checked` | Fixture runs show how release default, line tables, full debuginfo, split debuginfo, and stripped binaries affect frame quality. | "Debuginfo impact measured for Rust grouping fixtures." |
| `rust_stack_v1_snapshot_stable` | Unchanged fixture input produces identical normalized grouping material and fingerprints across repeated parser runs. | "Grouping snapshots are deterministic for the tested fixture corpus." |
| `rebuild_stable` | Same logical bug groups across clean rebuilds when source identity is unchanged. | "Stable across tested rebuild variants." |
| `debug_variant_stable` | Same logical bug groups across line-tables-only, full debuginfo, split debuginfo, and acceptable stripped/server-symbolicated variants. | "Stable across tested debuginfo variants." |
| `false_split_controlled` | Line-only shifts, path hash changes, build IDs, and deploy metadata do not split the same logical bug unless a fixture marks the change as material. | "False-split controls pass for tested Rust fixtures." |
| `false_merge_controlled` | Different application functions, generic instantiations, closures, async frames, and unrelated panic sites do not collapse into one issue. | "False-merge controls pass for tested Rust fixtures." |
| `symbolication_degraded_safe` | Missing or partial symbols produce low-confidence grouping warnings and no fake file/line precision. | "Unsymbolicated Rust events degrade safely." |
| `rust_grouping_stable` | Required stability, false-split, false-merge, client-fingerprint, and degraded-symbolication rows pass for the dated matrix. | "Deterministic Parallax grouping for the tested Rust stacktrace matrix." |
| `claim_expired` | rustc/Cargo/Sentry SDK/grouping/redaction/parser inputs changed, or 90 days passed. | "Rust grouping result expired; rerun required." |
| `claim_failed` | Any required fixture fails for the advertised level. | No Rust grouping claim for the affected matrix. |

Initial Parallax level: `not_measured`.

## Result Artifacts

Create these only for real fixture runs:

```text
docs/research/rust-stacktrace-grouping-results.md
docs/research/rust-stacktrace-grouping-runs/<run_id>/manifest.json
docs/research/rust-stacktrace-grouping-runs/<run_id>/fixture-matrix.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/raw-envelopes/<fixture_id>.envelope
docs/research/rust-stacktrace-grouping-runs/<run_id>/build-variant-results.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/frame-normalization-results.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/fingerprint-results.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/symbolication-results.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/false-split-merge-audit.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/claim-ledger.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/hashes.sha256
```

Do not commit private production stacktraces. Fixture stacks should come from
synthetic Rust apps, public examples, or explicitly approved redacted incidents.

## Run Manifest

Each `manifest.json` should pin the moving parts:

```json
{
  "run_id": "rust-stack-YYYYMMDD-N",
  "research_date": "YYYY-MM-DD",
  "fixture_generator_commit": "<git-sha>",
  "parallax_parser_commit": "<git-sha>",
  "grouping_algorithm_version": "rust-stack-v1",
  "sentry_rust_version": "0.48.2",
  "sentry_types_version": "0.48.2",
  "rustc_version": "rustc <version>",
  "cargo_version": "cargo <version>",
  "target_triples": ["x86_64-unknown-linux-gnu"],
  "profiles": ["release_default", "line_tables_only", "full_debuginfo", "split_debuginfo", "stripped_no_symbols"],
  "codegen_knobs": {
    "force_frame_pointers": "default|enabled|disabled",
    "force_unwind_tables": "default|enabled|disabled",
    "symbol_mangling_version": "default|v0|legacy"
  },
  "redaction_policy_version": "a6-default-deny-vN",
  "fixture_count": 0,
  "notes": []
}
```

The manifest must separate compiler version, target, profile, Sentry SDK
version, parser version, grouping version, and redaction policy version. A pass
under one set does not automatically cover another.

## Row Schemas

### Fixture Matrix Row

```json
{
  "fixture_id": "rust_same_panic_line_tables",
  "scenario": "same_panic",
  "source_variant": "baseline|line_shift|function_rename|file_move|generic_monomorphization|async_closure|no_in_app_frames|client_fingerprint",
  "build_variant": "release_default|line_tables_only|full_debuginfo|split_debuginfo|stripped_no_symbols",
  "expected_group": "grp_same_panic",
  "expected_confidence": "high|medium|low",
  "expected_claim_level": "debug_variant_stable",
  "raw_envelope_hash": "sha256:<hex>"
}
```

### Build Variant Result Row

```json
{
  "fixture_id": "rust_same_panic_line_tables",
  "target_triple": "x86_64-unknown-linux-gnu",
  "profile": "line_tables_only",
  "debug_setting": "line-tables-only",
  "split_debuginfo": "off|packed|unpacked|platform_default",
  "strip": "none|debuginfo|symbols",
  "panic_strategy": "unwind|abort",
  "backtrace_status": "captured|disabled|unsupported|unknown",
  "frame_count": 12,
  "in_app_frame_count": 3,
  "file_line_available": true,
  "debug_id_present": true,
  "symbolication_input_quality": "full|line_tables|function_only|unsymbolicated"
}
```

### Frame Normalization Row

```json
{
  "fixture_id": "rust_async_closure_line_tables",
  "frame_index": 0,
  "raw_function": "_RNv...",
  "demangled_function": "checkout::discount::apply::{{closure}}",
  "normalized_function": "checkout::discount::apply::closure",
  "module": "checkout::discount",
  "filename": "src/discount.rs",
  "lineno": 42,
  "in_app": true,
  "in_app_reason": "crate_root",
  "dropped_as_runtime_noise": false,
  "normalization_notes": []
}
```

### Fingerprint Result Row

```json
{
  "fixture_id": "rust_same_panic_line_tables",
  "expected_group": "grp_same_panic",
  "fingerprint_source": "client|rust_stack|thread_stack|message|manual",
  "client_fingerprint_preserved": false,
  "grouping_algorithm_version": "rust-stack-v1",
  "grouping_material_hash": "sha256:<hex>",
  "fingerprint": "sha256:<hex>",
  "grouping_confidence": "high|medium|low",
  "grouping_warnings": [],
  "matched_expected_group": true
}
```

### Symbolication Result Row

```json
{
  "fixture_id": "rust_stripped_no_symbols",
  "build_id": "sha256-or-platform-id",
  "debug_id": "uuid-or-derived-id",
  "debug_file_status": "not_required|matched|missing|mismatch|unsupported",
  "symbolication_status": "full|function_only|missing_line|unsymbolicated|failed|unknown",
  "fabricated_precision": false,
  "agent_visible_warning": "Missing line information; grouping confidence is low.",
  "safe_for_agent_bundle": true
}
```

### False Split / False Merge Audit Row

```json
{
  "audit_id": "rust_group_audit_001",
  "audit_type": "false_split|false_merge",
  "fixture_ids": ["rust_same_panic_line_tables", "rust_same_panic_full"],
  "expected_same_group": true,
  "actual_same_group": true,
  "result": "pass|fail",
  "reason": "line-only shift did not change issue identity",
  "required_repair": null
}
```

### Claim Ledger Row

```json
{
  "run_id": "rust-stack-YYYYMMDD-N",
  "claim_level": "not_measured",
  "claim_status": "pass|fail|expired",
  "covered_matrix": ["sentry-rust@0.48.2", "rustc <version>", "x86_64-unknown-linux-gnu"],
  "product_wording": "Rust grouping design exists; results pending.",
  "required_caveats": ["not Sentry grouping parity"],
  "expires_at": "YYYY-MM-DD"
}
```

## Counting Rules

- No `rust_grouping_stable` claim until rebuild stability, debuginfo-variant
  stability, false-split controls, false-merge controls, client-fingerprint
  behavior, and degraded-symbolication behavior all pass in the same dated run.
- Client-provided fingerprints win only when policy allows them; record
  `fingerprint_source = client` and do not mix that pass with the default
  `rust-stack-v1` result.
- Build IDs, debug IDs, release, environment, host, deploy ID, and commit are
  evidence dimensions, not default issue-identity dimensions.
- Ordinary line-number churn must not split an issue by itself. Panic anchors or
  source-location macros may use line data only if fixtures prove the behavior.
- A stripped binary without matching debug companions can pass only as
  `symbolication_degraded_safe`, not as full grouping quality.
- Function-only symbolication is not full symbolication. It must carry a warning
  and a lower confidence level when file/line evidence is missing.
- No exact "same grouping as Sentry" claim. The claim is versioned Parallax
  grouping for the tested Rust matrix.
- No broad cross-language grouping claim. This ledger covers Rust stacktrace
  grouping only.
- Redaction must pass for any frame context, breadcrumbs, tags, or span fields
  exposed in agent-visible bundles.

## Initial Results Template

When measurement begins, create `docs/research/rust-stacktrace-grouping-results.md`:

```markdown
# Rust Stacktrace Grouping Results

Research window:
Last updated:
Current claim level: not_measured

## Gate Snapshot

| Metric | Current | Required for rust_grouping_stable | Status |
| --- | ---: | ---: | --- |
| Fixture scenarios run | 0 | >=12 | Pending |
| Build/profile variants covered | 0 | >=5 | Pending |
| Rebuild-stability failures | 0 | 0 | Pending |
| Debug-variant stability failures | 0 | 0 | Pending |
| False-split audit failures | 0 | 0 | Pending |
| False-merge audit failures | 0 | 0 | Pending |
| Fabricated symbolication precision | 0 | 0 | Pending |
| Agent-visible redaction leaks | 0 | 0 | Pending |

## Matrix Coverage

## Failures And Repairs

## Current Allowed Wording

## Decision
```

## Product Wording

Allowed after `not_measured`:

> Rust stacktrace grouping is designed but not yet fixture-proven.

Allowed after `rust_grouping_stable`:

> Deterministic Parallax grouping for the tested Rust stacktrace matrix.

Always link the result matrix. Avoid:

- "same grouping as Sentry";
- "deterministic grouping" without a dated matrix;
- "stable across releases" unless rebuild and line-churn fixtures pass;
- "symbolicated" when the result is function-only, missing-line, or
  unsymbolicated;
- "Rust errors are agent-ready" before redaction and bundle usefulness gates
  also pass.

## Refresh Triggers

Mark affected claims `claim_expired` when:

- supported `sentry`/`sentry-types` versions change;
- rustc/Cargo versions or target triples change materially;
- Cargo debuginfo, split-debuginfo, strip, panic, frame-pointer, unwind-table,
  or symbol-mangling behavior changes;
- `rust-stack-v1` normalization changes;
- redaction policy changes for stack/breadcrumb/span fields;
- a new Rust async/runtime pattern causes false splits or false merges;
- 90 days pass since the last run during active development.

## Relationship To Other Research

- [Rust stacktrace grouping and symbolication](rust-stacktrace-grouping-and-symbolication.md)
  defines the `rust-stack-v1` design this ledger measures.
- [Sentry SDK fixture compatibility gate](sentry-sdk-fixture-compatibility.md)
  supplies the Rust SDK envelope fixtures extended by this ledger.
- [Sentry SDK compatibility ledger](sentry-sdk-compatibility-ledger.md) consumes
  this ledger for the L3 `rust_grouping_stable` compatibility subclaim.
- [Sentry-compatible ingestion](sentry-compatible-ingestion.md) defines where
  grouping sits in the ingest pipeline.
- [Rust data collection and instrumentation](rust-data-collection-and-instrumentation.md)
  defines why Rust in-process capture and debuginfo policy are mandatory.
- [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) controls
  whether stack, breadcrumb, tag, and span context can enter agent-visible
  bundles.

## Bottom Line

Rust grouping should be a measured protocol-like contract. The design is
promising, but the claim starts only when fixture rows prove that `rust-stack-v1`
is stable across rebuilds and debuginfo variants, conservative against false
merges, honest about missing symbols, and safe to expose through redacted
agent-facing bundles.
