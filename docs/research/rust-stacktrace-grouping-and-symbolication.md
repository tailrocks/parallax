# Rust Stacktrace Grouping and Symbolication

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note tightens proof gate #6 from
[Strategic verdict and research coverage](strategic-verdict-and-research-coverage.md):

> Rust stacktrace grouping stability across release/debug-info variants.

The decision: **Parallax should treat Rust stack grouping as a versioned,
fixture-tested product primitive, not a side effect of whatever stack frames an
SDK happens to emit.**

The v0 target is not Sentry grouping parity. The target is a deterministic
`rust-stack-v1` fingerprint that keeps the same logical Rust bug grouped across
rebuilds and debuginfo layouts, while recording enough grouping material to
audit false splits and false merges later.

## Current Primary-Source Checks

| Source | What matters for Parallax |
| --- | --- |
| [Rust `std::backtrace`](https://doc.rust-lang.org/std/backtrace/index.html) | Backtrace capture is best-effort. Frame instruction pointers, symbols, filenames, and line numbers may be inaccurate; filename/line reporting usually requires debuginfo. `Backtrace::capture` is disabled unless `RUST_LIB_BACKTRACE` or `RUST_BACKTRACE` enables it, while `force_capture` ignores those environment gates. |
| [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) | `debug = "line-tables-only"` is the minimal debuginfo level for backtraces with filename/line info. Release defaults still have `debug = false`. `split-debuginfo` can put debug info adjacent to the executable, and `strip` can remove symbols or debuginfo. |
| [rustc codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html) | `force-frame-pointers` and `force-unwind-tables` are target-dependent durability knobs for stack walking and unwinding; defaults vary by target. |
| [rustc symbol mangling](https://doc.rust-lang.org/rustc/symbol-mangling/index.html) | Rust symbols are mangled for linker uniqueness and may need demangling for tooling. The mangling version can vary, so Parallax should store raw and demangled function material and normalize only conservatively. |
| [Sentry issue grouping](https://docs.sentry.io/concepts/data-management/event-grouping/) | Sentry considers custom `fingerprint` first, then stack trace, exception, then message. Stacktrace grouping primarily uses frames associated with the application and normalized frame material. Grouping algorithm versions matter because grouping changes should not silently rewrite old issues. |
| [Sentry debug files](https://docs.sentry.io/platforms/flutter/data-management/debug-files/) | Debug files carry function names, source paths, line numbers, source context, and variable placement; Sentry needs access to application and system-library debug files for fully symbolicated crash reports. |
| [Sentry debug file formats](https://docs.sentry.io/platforms/flutter/data-management/debug-files/file-formats/) | Debug information, symbol tables, source bundles, and unwind information are distinct. Symbol tables can recover function names as a fallback but do not provide file/line or inline-frame quality. ELF release builds commonly strip debug info into companion files. |
| [Sentry debug identifiers](https://docs.sentry.io/platforms/flutter/data-management/debug-files/identifiers/) | Native crash reports and debug files are matched by code/debug identifiers. For ELF, Sentry uses GNU build identifiers to compute debug identifiers; missing or mismatched IDs weaken symbolication. |
| [sentry Rust crate 0.48.2](https://docs.rs/sentry/latest/sentry/) | The current Rust SDK captures panics, backtraces, Rust contexts, and debug-image metadata by default features, with optional `anyhow`, `tracing`, and OpenTelemetry integrations. |
| [sentry tracing integration 0.48.2](https://docs.rs/sentry/latest/sentry/integrations/tracing/index.html) | `tracing` events can become Sentry events, breadcrumbs, logs, and spans. Spans and breadcrumbs give causal context, but they should support grouping rather than replace physical stack identity. |
| [tracing-error](https://docs.rs/tracing-error/latest/tracing_error/) | `SpanTrace` captures logical span context at the error site. It is orthogonal to a call-stack backtrace and is especially useful across async boundaries. |

## Product Decision

Parallax v0 should compute its own grouping fingerprint after normalization:

- accept client-provided Sentry `fingerprint` when policy allows it;
- otherwise use `rust-stack-v1` for Rust exception/panic stacks;
- store the exact normalized grouping material beside every event;
- store the algorithm version and confidence;
- never silently regroup historical issues when the algorithm changes.

Release, environment, host, build ID, and commit should **not** be part of the
grouping identity. They are regression-window and symbolication dimensions. If
release is included in the identity, every deploy becomes a false split.

## Debuginfo Policy

Backtrace quality is a build contract.

The recommended first policy for Rust services and CLIs:

```toml
[profile.release]
debug = "line-tables-only"
strip = "none"
```

This keeps enough source location data for useful backtraces without full
variable or parameter debuginfo. Teams that cannot ship line tables in the
binary should instead use split debuginfo or external debug companions and
upload/register them before events arrive.

Policy requirements:

- record `debug_meta`, loaded images, build ID/debug ID, and symbolication status
  when present;
- treat `debug = false` without uploaded symbols as degraded evidence;
- treat stripped binaries as acceptable only when matching debug companions are
  available server-side;
- distinguish function-only symbol table fallback from full file/line
  symbolication;
- retain enough raw frame material to re-run grouping after a normalizer fix.

## `rust-stack-v1` Fingerprint

Use the client fingerprint first:

```text
if event.fingerprint exists and policy allows it:
  source = "client"
  fingerprint = hash(project_id, "client", event.fingerprint)
```

Otherwise compute Rust stack grouping:

```text
candidate_frames =
  exception.stacktrace.frames
  |> newest_call_last_to_top_frame_order
  |> mark_in_app(project_rules, crate_roots, sentry_in_app)
  |> drop_runtime_noise(std, core, tokio runtime, tracing, sentry, panic plumbing)
  |> normalize_rust_symbol
  |> attach_file_module_line_and_symbolication_status

primary_frames =
  top N in_app candidate_frames if any exist
  else top N candidate_frames

anchor =
  panic_location if present and symbolicated
  else first primary_frame

fingerprint = hash(
  project_id,
  "rust-stack-v1",
  exception.type,
  normalized anchor module/function/file,
  normalized primary_frames module/function/file sequence,
  panic message class when stack confidence is low
)
```

`N = 3` is a reasonable fixture starting point, not a final constant.

Line numbers should be stored as evidence, but the default fingerprint should
not depend on ordinary line numbers unless the frame is a panic macro location
or another explicit source location that materially identifies the failure. A
line-only shift above the function should not split an issue.

## Normalization Rules

Store both raw and normalized values.

| Field | Rule |
| --- | --- |
| `raw_function` | Preserve exactly as received from the SDK/symbolicator. |
| `demangled_function` | Store Rust-demangled function when available. |
| `normalized_function` | Remove only clearly compiler-generated hash/address suffixes; preserve enough module/function/generic structure to avoid overgrouping. |
| `module` / crate path | Prefer crate/module path over filename when both exist. |
| `filename` / `abs_path` | Normalize workspace root and revision/build-hash path segments; preserve relative source file. |
| `lineno` / `colno` | Store and show, but do not make ordinary line number the sole grouping discriminator. |
| `in_app` | Preserve SDK value and add Parallax `in_app_reason` from project crate roots and configured frame rules. |
| `panic_location` | Store separately. Use as a strong anchor, but fixture-test line churn to avoid oversplitting. |
| `build_id` / `debug_id` | Store per image/event and link to debug file inventory. |
| `symbolication_status` | `full`, `function_only`, `missing_line`, `unsymbolicated`, `failed`, or `unknown`. |

Avoid aggressive Rust symbol rewriting in v0. Generics, closures, async state
machines, monomorphization, and compiler-generated shims are exactly where false
merges can appear. Normalize slowly, with fixture snapshots.

## Physical Stack Plus Logical Context

Parallax should capture and store two related structures:

1. **Physical backtrace**: frames from `std::backtrace`, `sentry`/`backtrace`,
   or symbolicated SDK payloads. This is the primary grouping and code-navigation
   signal.
2. **Logical span context**: `SpanTrace`, breadcrumbs, OTLP spans, and
   `contexts.trace`. This is causal evidence for humans and agents.

Async Rust makes this split important. A physical backtrace across `.await`
boundaries can be shallow or dominated by executor frames. `SpanTrace` and OTLP
span context often explain the operation better, but using span names as the
primary issue identity can overgroup different bugs in the same handler. Use
logical context as supporting material and as fallback only when stack evidence
is weak.

## Fixture Matrix

Add these cases to the Sentry fixture suite after the L1 Rust envelope parser
passes:

| Fixture | Must prove |
| --- | --- |
| Same panic, `debug = "line-tables-only"` | Same `rust-stack-v1` fingerprint; file/line available. |
| Same panic, full debuginfo | Same fingerprint as line-tables-only; richer evidence only. |
| Same panic, split debuginfo uploaded | Same fingerprint after server-side symbolication. |
| Same panic, stripped binary without symbols | Degraded confidence and missing line evidence; no false precision. |
| Same code rebuilt | Same fingerprint despite changed addresses/build IDs. |
| Line-only shift above function | Same fingerprint unless panic anchor materially changes. |
| Function rename | New fingerprint or explicit migration record, because code identity changed. |
| File move without function/module change | Prefer same fingerprint if module path is stable; fixture decides final rule. |
| Generic monomorphization | Different concrete type params should not overgroup if they point to different app functions; normalize cautiously. |
| Closure or async panic | Executor frames ignored; app closure/async frame and span context retained. |
| No in-app frames | Use all frames with low confidence and warnings. |
| Explicit Sentry fingerprint | Client fingerprint wins and is recorded as `fingerprint_source = client`. |
| Missing build/debug ID | Event accepted, symbolication warning stored, no server-side symbol lookup assumed. |

## Success Gates

This proof gate is closed only when:

- unchanged fixture input produces stable fingerprints across parser releases;
- the same logical Rust bug groups across full debuginfo, line tables, split
  debuginfo, and rebuilt binaries;
- line-number-only churn does not split issues by default;
- unrelated app functions do not merge;
- missing debuginfo creates explicit low-confidence grouping, not fabricated
  source precision;
- all grouping material can be reprocessed under `rust-stack-v2` without losing
  the original `rust-stack-v1` issue assignment.

## Schema Additions

The normalized error-event model should add:

| Field | Purpose |
| --- | --- |
| `grouping_algorithm_version` | Example: `rust-stack-v1`. |
| `fingerprint_source` | `client`, `rust_stack`, `thread_stack`, `message`, or `manual`. |
| `grouping_material` | JSON snapshot of normalized frames and anchors used in the hash. |
| `grouping_confidence` | `high`, `medium`, `low`; drives UI and agent wording. |
| `grouping_warnings` | Missing debuginfo, no in-app frames, stack absent, unsymbolicated frames. |
| `frame.raw_function` | Raw SDK/symbolicator function. |
| `frame.demangled_function` | Demangled Rust symbol, if available. |
| `frame.normalized_function` | Versioned normalized function used for grouping. |
| `frame.in_app_reason` | Why Parallax treated the frame as app/non-app. |
| `frame.symbolication_status` | Full/function-only/missing-line/unsymbolicated/etc. |
| `frame.build_id` / `frame.debug_id` | Link to loaded image and debug-file inventory. |

## Relationship To Other Research

- [Rust data collection and instrumentation](rust-data-collection-and-instrumentation.md)
  defines why in-process Rust error capture and debuginfo are mandatory.
- [Sentry-compatible ingestion](sentry-compatible-ingestion.md) defines the
  envelope/event subset and first deterministic grouping path.
- [Sentry SDK fixture compatibility gate](sentry-sdk-fixture-compatibility.md)
  supplies the SDK-generated fixture harness this gate should extend.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) should expose
  `grouping_confidence`, symbolication warnings, and redaction status to agents.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  still has veto power over stack locals, breadcrumbs, tags, and span fields.

## Bottom Line

Rust stacktrace grouping is buildable, but only if Parallax makes it explicit:
versioned grouping, conservative Rust symbol normalization, mandatory debuginfo
policy, per-frame symbolication status, and a fixture matrix that tries to break
the grouping algorithm before users do.

Do not ship "deterministic grouping" as a claim until this gate passes.
