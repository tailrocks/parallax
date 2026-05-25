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
> capture paths, panic strategies, rebuilds, debuginfo variants, frame
> normalization, false splits, false merges, client fingerprints,
> symbolication degradation, redaction, source-field isolation, and
> grouping-version stability.

This ledger is narrower than the
[Sentry SDK compatibility ledger](sentry-sdk-compatibility-ledger.md): Sentry
Rust envelopes can parse and normalize before Rust grouping is proven.

## Current Source Snapshot

| Source | Current check | Why it matters |
| --- | --- | --- |
| [Rust `std::backtrace`](https://doc.rust-lang.org/std/backtrace/index.html) | Rust docs currently show `std` 1.95.0 and state that backtraces are best-effort, file/line reporting usually needs debug information, `Backtrace::capture` is disabled unless environment variables enable it, and `force_capture` bypasses those gates. | The ledger must record capture mode, status, and debuginfo availability; absence of file/line is expected and must become low confidence, not fabricated precision. |
| [Rust panic hooks](https://doc.rust-lang.org/std/panic/fn.set_hook.html) and [`PanicHookInfo`](https://doc.rust-lang.org/std/panic/struct.PanicHookInfo.html) | Panic hooks run before the panic runtime for aborting and unwinding runtimes, and carry payload plus source location when available. | Panic grouping fixtures must prove the hook ran and captured the location; a missing hook is a capture-path failure, not a grouping success with fewer frames. |
| [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) | Release defaults have `debug = false`; `line-tables-only` is the minimal debuginfo level for filename/line backtraces; `split-debuginfo` and `strip` alter where symbols live. | Fixture variants must include release default, line tables, full debuginfo, split debuginfo, and stripped binaries. |
| [rustc codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html) | Frame-pointer and unwind-table defaults depend on target, while flags can force them; `panic=immediate-abort` does not call panic hooks. | The run manifest must pin target triple, panic strategy, and stack-walking knobs because grouping stability and capture viability are target-sensitive. |
| [rustc symbol mangling](https://doc.rust-lang.org/stable/rustc/symbol-mangling/index.html) | Rust symbol names are mangled for linker uniqueness; tooling may need demangling; rustc supports multiple mangling versions. | Store raw, demangled, and normalized symbols. Do not assume one mangling version or one compiler release. |
| [Sentry issue grouping](https://docs.sentry.io/concepts/data-management/event-grouping/) | Sentry versions grouping algorithms, considers fingerprint first, then stack trace, exception, and message, and uses in-app frames when available. | Parallax should copy the versioning discipline and client-fingerprint precedence, not claim exact Sentry grouping parity. |
| [Sentry Native debug information files](https://docs.sentry.io/platforms/native/data-management/debug-files/) | Debug information files provide function names, source paths, line numbers, source context, and sometimes variable-placement evidence; Sentry needs access to app and system-library files for full symbolication and its uploaded debug files use time-to-idle retention. | The ledger must separate Parallax evidence-retention policy from Sentry's retention model and should not treat debug metadata as durable source evidence until the matching file inventory is recorded. |
| [Sentry debug file formats](https://docs.sentry.io/platforms/native/data-management/debug-files/file-formats/) | Debug information, symbol tables, source code bundles, and unwind information are distinct. Symbol tables are function-name fallbacks and do not provide file/line or inline-frame quality. | Fixture rows must distinguish `full`, `function_only`, `missing_line`, and source-context availability instead of using one symbolicated boolean. |
| [Sentry debug identifiers](https://docs.sentry.io/platforms/native/data-management/debug-files/identifiers/) | Sentry distinguishes code IDs from debug IDs; native events list loaded images with debug identifiers, and ELF debug IDs are derived from the GNU build ID/code ID. | Parallax should record code ID, debug ID, object name, and match status as evidence dimensions, but not include build/debug IDs in the logical issue identity. |
| [Sentry debug-file uploads](https://docs.sentry.io/platforms/native/data-management/debug-files/upload/) and [source context](https://docs.sentry.io/platforms/native/data-management/debug-files/source-context/) | Sentry recommends uploading debug files before deploy; later uploads can take time to affect new reports and existing events are not reprocessed. Source context for compiled applications requires uploaded source bundles. | Parallax must either prove late-symbol reprocessing or mark old events degraded; source context must pass source-field and redaction policy before entering agent bundles. |
| [Sentry Symbolicator](https://getsentry.github.io/symbolicator/), [lookup strategy](https://getsentry.github.io/symbolicator/advanced/symbol-lookup/), and [system architecture](https://getsentry.github.io/symbolicator/advanced/system-architecture/) | Symbolicator resolves function names, file locations, and source context; it computes code/debug identifiers, looks up files across sources, ranks debug/unwind files over symbol-table fallbacks, and caches downloaded objects plus derived symbol caches. | Ledger rows need symbol source, matched-file provenance, quality ranking, and cache/source status so agents can tell full evidence from fallback evidence. |
| [Symbolicator source bundles](https://getsentry.github.io/symbolicator/advanced/source-bundles/) | Source bundles are ZIP archives with manifests containing code ID, debug ID, object name, architecture, and original source paths. | Source snippets and paths are source-derived evidence; they require source-field policy checks and redaction before agent-visible projection. |
| [sentry Rust crate 0.48.2](https://docs.rs/sentry/0.48.2/sentry/) and [feature flags](https://docs.rs/crate/sentry/0.48.2/features) | Docs.rs currently resolves the explicit crate page to `0.48.2`; default features include backtrace, contexts, panic capture, transport, debug-image metadata, and release health. | The first fixture target remains current Sentry Rust SDK panic/error envelopes, but runs must record exact feature flags because `tracing`, `anyhow`, and OpenTelemetry are opt-in. |
| [Rust capture fidelity recheck](rust-capture-fidelity-recheck.md) | Current capture pass pins the broader Rust capture layer: `tracing` `0.1.44`, `tracing-error` `0.2.1`, `tracing-opentelemetry` `0.33.0`, `opentelemetry`/`opentelemetry-otlp`/`opentelemetry-appender-tracing` `0.32.0`, `anyhow` `1.0.102`, `eyre` `0.6.12`, and `color-eyre` `0.6.5`. | Grouping claims need capture-layer version rows too; a Sentry envelope alone does not prove span traces, OTLP logs, or `anyhow`/`eyre` chains. |
| [sentry-panic 0.48.2](https://docs.rs/sentry-panic/0.48.2/sentry_panic/), [sentry-backtrace 0.48.2](https://docs.rs/sentry-backtrace/0.48.2/sentry_backtrace/), and [sentry-debug-images 0.48.2](https://docs.rs/sentry-debug-images/0.48.2/sentry_debug_images/) | The subcrates separately install a panic handler, convert/process stacktraces, and attach loaded-library metadata. | Result rows must identify which capture integration produced panic, stack, and loaded-image evidence; one SDK envelope is not enough proof. |

## Claim Levels

Use these levels in `claim-ledger.jsonl`:

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No current fixture run exists. | "Rust grouping design exists; results pending." |
| `fixture_harness_ready` | Synthetic Rust apps can generate raw Sentry envelopes and build variants. | "Rust grouping fixture harness prepared." |
| `capture_path_checked` | Fixture runs show hook invocation, backtrace capture path, backtrace env state, SDK feature set, panic strategy, and debug-image presence. | "Rust stack capture paths measured for fixture builds." |
| `debug_info_policy_checked` | Fixture runs show how release default, line tables, full debuginfo, split debuginfo, and stripped binaries affect frame quality. | "Debuginfo impact measured for Rust grouping fixtures." |
| `rust_stack_v1_snapshot_stable` | Unchanged fixture input produces identical normalized grouping material and fingerprints across repeated parser runs. | "Grouping snapshots are deterministic for the tested fixture corpus." |
| `rebuild_stable` | Same logical bug groups across clean rebuilds when source identity is unchanged. | "Stable across tested rebuild variants." |
| `debug_variant_stable` | Same logical bug groups across line-tables-only, full debuginfo, split debuginfo, and acceptable stripped/server-symbolicated variants. | "Stable across tested debuginfo variants." |
| `false_split_controlled` | Line-only shifts, path hash changes, build IDs, and deploy metadata do not split the same logical bug unless a fixture marks the change as material. | "False-split controls pass for tested Rust fixtures." |
| `false_merge_controlled` | Different application functions, generic instantiations, closures, async frames, and unrelated panic sites do not collapse into one issue. | "False-merge controls pass for tested Rust fixtures." |
| `symbolication_degraded_safe` | Missing or partial symbols produce low-confidence grouping warnings and no fake file/line precision. | "Unsymbolicated Rust events degrade safely." |
| `rust_grouping_stable` | Required capture-path, stability, false-split, false-merge, client-fingerprint, degraded-symbolication, redaction, and source-field rows pass for the dated matrix. | "Deterministic Parallax grouping for the tested Rust stacktrace matrix." |
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
docs/research/rust-stacktrace-grouping-runs/<run_id>/capture-path-results.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/build-variant-results.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/frame-normalization-results.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/fingerprint-results.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/debug-file-inventory.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/symbolication-results.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/false-split-merge-audit.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/source-context-policy-results.jsonl
docs/research/rust-stacktrace-grouping-runs/<run_id>/source-field-policy-audit.jsonl
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
  "sentry_features": ["backtrace", "contexts", "panic", "debug-images", "transport"],
  "capture_layer_versions": {
    "tracing": "0.1.44",
    "tracing-error": "0.2.1",
    "tracing-opentelemetry": "0.33.0",
    "opentelemetry": "0.32.0",
    "opentelemetry-otlp": "0.32.0",
    "opentelemetry-appender-tracing": "0.32.0",
    "anyhow": "1.0.102",
    "eyre": "0.6.12",
    "color-eyre": "0.6.5"
  },
  "rustc_version": "rustc <version>",
  "cargo_version": "cargo <version>",
  "target_triples": ["x86_64-unknown-linux-gnu"],
  "profiles": ["release_default", "line_tables_only", "full_debuginfo", "split_debuginfo", "stripped_no_symbols"],
  "symbol_sources": ["in_event", "uploaded_debug_file", "symbol_server", "source_bundle", "none"],
  "symbolicator_reference_snapshot": "getsentry/symbolicator docs checked YYYY-MM-DD",
  "debug_file_inventory_ref": "debug-file-inventory.jsonl",
  "codegen_knobs": {
    "force_frame_pointers": "default|enabled|disabled",
    "force_unwind_tables": "default|enabled|disabled",
    "panic_strategy": "unwind|abort|immediate-abort",
    "symbol_mangling_version": "default|v0|legacy"
  },
  "backtrace_environment": {
    "RUST_BACKTRACE": "unset|0|1|full|other",
    "RUST_LIB_BACKTRACE": "unset|0|1|full|other"
  },
  "redaction_policy_version": "a6-default-deny-vN",
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "source_context_policy_version": "phase0-source-context-policy-vN",
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

### Capture Path Result Row

```json
{
  "fixture_id": "rust_panic_line_tables",
  "sdk_name": "sentry-rust",
  "sdk_version": "0.48.2",
  "sdk_features": ["backtrace", "contexts", "panic", "debug-images", "transport"],
  "panic_strategy": "unwind|abort|immediate-abort",
  "panic_hook_status": "sentry_hook_invoked|custom_hook_invoked|not_invoked|not_applicable|unknown",
  "previous_hook_chained": true,
  "panic_payload_kind": "str|string|non_string|not_applicable|unknown",
  "panic_location_available": true,
  "backtrace_capture_path": "sentry_backtrace|std_capture|std_force_capture|parsed_string|sdk_event_only|none",
  "backtrace_environment": {
    "RUST_BACKTRACE": "unset|0|1|full|other",
    "RUST_LIB_BACKTRACE": "unset|0|1|full|other"
  },
  "backtrace_status": "captured|disabled|unsupported|missing|unknown",
  "debug_images_enabled": true,
  "debug_meta_image_count": 1,
  "loaded_image_count": 1,
  "negative_fixture": false,
  "expected_degradation": null,
  "result": "pass|fail"
}
```

Capture path rows answer whether grouping evidence exists before testing whether
the grouping algorithm is stable. A panic event from `panic=immediate-abort`,
disabled SDK features, a non-invoked hook, or disabled backtrace capture should
produce an explicit degraded/failed row instead of silently flowing into
fingerprint checks.

### Build Variant Result Row

```json
{
  "fixture_id": "rust_same_panic_line_tables",
  "target_triple": "x86_64-unknown-linux-gnu",
  "profile": "line_tables_only",
  "debug_setting": "line-tables-only",
  "split_debuginfo": "off|packed|unpacked|platform_default",
  "strip": "none|debuginfo|symbols",
  "panic_strategy": "unwind|abort|immediate-abort",
  "backtrace_status": "captured|disabled|unsupported|unknown",
  "frame_count": 12,
  "in_app_frame_count": 3,
  "file_line_available": true,
  "code_id_present": true,
  "debug_id_present": true,
  "debug_file_uploaded": true,
  "source_bundle_uploaded": false,
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
  "symbolication_source": "in_event|uploaded_debug_file|symbol_server|source_bundle|none",
  "symbolication_input_quality": "full|line_tables|function_only|unsymbolicated",
  "build_id": "sha256-or-platform-id",
  "code_id": "native-code-id-or-null",
  "debug_id": "uuid-or-derived-id",
  "object_name": "binary-or-library-name",
  "debug_file_status": "not_required|matched|missing|mismatch|unsupported",
  "matched_debug_file_ref": "debug-file-inventory.jsonl#row-id-or-null",
  "symbolication_status": "full|function_only|missing_line|unsymbolicated|failed|unknown",
  "source_context_status": "available|policy_denied|redacted|missing|not_requested",
  "source_bundle_ref": "debug-file-inventory.jsonl#row-id-or-null",
  "late_symbol_upload_reprocess_status": "not_needed|reprocessed|not_reprocessed|unsupported|unknown",
  "fabricated_precision": false,
  "agent_visible_warning": "Missing line information; grouping confidence is low.",
  "source_field_policy_status": "pass|fail|not_applicable",
  "safe_for_agent_bundle": true
}
```

### Debug File Inventory Row

```json
{
  "debug_file_row_id": "dfi_001",
  "fixture_id": "rust_same_panic_split_debuginfo",
  "file_kind": "executable|debug_companion|symbol_table|unwind|source_bundle",
  "source": "uploaded_fixture|symbol_server_fixture|in_event",
  "object_name": "checkout-service",
  "architecture": "x86_64",
  "code_id": "native-code-id-or-null",
  "debug_id": "uuid-or-derived-id",
  "quality_tags": ["debug", "symtab", "unwind", "sources"],
  "uploaded_before_event": true,
  "retention_policy": "fixture|parallax-symbol-retention-vN",
  "content_hash": "sha256:<hex>",
  "source_paths_policy_status": "pass|fail|not_applicable"
}
```

### Source Field Policy Audit Row

```json
{
  "fixture_id": "rust_tracing_breadcrumbs_line_tables",
  "bundle_projection": "cli_json|cli_markdown|http_json|mcp_structured_content",
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "source_field_policy_hash": "sha256:<hex>",
  "checked_fields": ["frame.vars", "frame.context_line", "source_context.lines", "source_context.paths", "breadcrumbs.data", "tags", "contexts.trace", "span.attributes"],
  "denied_field_count": 0,
  "leaked_denied_fields": [],
  "redaction_report_ref": "sha256:<hex>",
  "result": "pass|fail"
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
  stability, capture-path checks, false-split controls, false-merge controls,
  client-fingerprint behavior, degraded-symbolication behavior, redaction
  checks, and source-field policy checks all pass in the same dated run.
- No `capture_path_checked` claim unless the run includes positive and negative
  fixtures for Sentry panic hooks, disabled hooks, `Backtrace::capture` env
  gates, `Backtrace::force_capture`, SDK debug-image feature on/off, and the
  configured panic strategies.
- `panic=immediate-abort` is a negative capture fixture. Do not claim Sentry
  panic grouping for it unless a separate non-hook capture path is proven.
- Client-provided fingerprints win only when policy allows them; record
  `fingerprint_source = client` and do not mix that pass with the default
  `rust-stack-v1` result.
- Build IDs, debug IDs, release, environment, host, deploy ID, and commit are
  evidence dimensions, not default issue-identity dimensions.
- `debug_meta` or loaded-image presence is lookup evidence only. Full
  symbolication requires a matched debug companion, in-event file/line evidence,
  or a documented symbol-server result with provenance.
- Source bundles and source context are source-derived evidence. They must pass
  source-field and redaction policy before any agent-visible projection can show
  source paths, context lines, or snippets.
- A late debug-file or source-bundle upload must either produce a dated
  reprocess result or leave prior events in degraded status. Do not silently
  upgrade historical grouping confidence.
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
- Source-field policy must pass for every agent-visible projection produced from
  fixture, eval, or corpus rows. A denied source field in projected grouping
  material fails the Rust grouping bundle claim even if the fingerprint matched.

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
| Capture-path failures | 0 | 0 | Pending |
| Debug ID / code ID match failures | 0 | 0 | Pending |
| Late-symbol reprocess unknowns | 0 | 0 | Pending |
| False-split audit failures | 0 | 0 | Pending |
| False-merge audit failures | 0 | 0 | Pending |
| Fabricated symbolication precision | 0 | 0 | Pending |
| Agent-visible redaction leaks | 0 | 0 | Pending |
| Source-context policy leaks | 0 | 0 | Pending |
| Source-field policy leaks | 0 | 0 | Pending |

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
- Sentry Rust SDK feature defaults, `sentry-panic`, `sentry-backtrace`, or
  `sentry-debug-images` behavior changes;
- Symbolicator, Sentry CLI debug-file/source-bundle behavior, symbol-server
  lookup, or debug-file retention policy changes;
- `rust-stack-v1` normalization changes;
- redaction or source-field policy changes for stack/breadcrumb/span fields;
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
