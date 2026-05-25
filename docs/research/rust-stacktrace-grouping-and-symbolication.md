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
audit false splits and false merges later. The companion
[Rust stacktrace grouping ledger](rust-stacktrace-grouping-ledger.md) defines
the result rows and claim levels required before this becomes a product claim.

## Current Primary-Source Checks

| Source | What matters for Parallax |
| --- | --- |
| [Rust `std::backtrace`](https://doc.rust-lang.org/std/backtrace/index.html) | Backtrace capture is best-effort. Frame instruction pointers, symbols, filenames, and line numbers may be inaccurate; filename/line reporting usually requires debuginfo. `Backtrace::capture` is disabled unless `RUST_LIB_BACKTRACE` or `RUST_BACKTRACE` enables it, while `force_capture` ignores those environment gates. |
| [Rust `std::panic::set_hook`](https://doc.rust-lang.org/std/panic/fn.set_hook.html) and [`PanicHookInfo`](https://doc.rust-lang.org/std/panic/struct.PanicHookInfo.html) | Panic hooks run before the panic runtime for both aborting and unwinding runtimes, and receive payload plus source location when available. The hook is global and replaces the previous hook unless explicitly chained. |
| [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) | `debug = "line-tables-only"` is the minimal debuginfo level for backtraces with filename/line info. Release defaults still have `debug = false`. `split-debuginfo` can put debug info adjacent to the executable, and `strip` can remove symbols or debuginfo. |
| [rustc codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html) | `force-frame-pointers` and `force-unwind-tables` are target-dependent durability knobs for stack walking and unwinding; defaults vary by target. The `panic` codegen option also includes `immediate-abort`, which does not call panic hooks. |
| [rustc symbol mangling](https://doc.rust-lang.org/rustc/symbol-mangling/index.html) | Rust symbols are mangled for linker uniqueness and may need demangling for tooling. The mangling version can vary, so Parallax should store raw and demangled function material and normalize only conservatively. |
| [Sentry issue grouping](https://docs.sentry.io/concepts/data-management/event-grouping/) | Sentry considers custom `fingerprint` first, then stack trace, exception, then message. Stacktrace grouping primarily uses frames associated with the application and normalized frame material. Grouping algorithm versions matter because grouping changes should not silently rewrite old issues. |
| [Sentry Native debug information files](https://docs.sentry.io/platforms/native/data-management/debug-files/) | Debug files can provide original function names, source paths, line numbers, source context, and variable placement. Sentry requires access to application and system-library debug files for fully symbolicated crash reports, and its own retention is time-to-idle. Parallax must treat debug-file retention as part of the evidence-retention contract, not an incidental upload. |
| [Sentry debug file formats](https://docs.sentry.io/platforms/native/data-management/debug-files/file-formats/) | Sentry distinguishes debug information, symbol tables, source code, and unwind information. Symbol tables can recover function names, but not inline functions, filenames, or line numbers; source bundles are a separate `sources` artifact. |
| [Sentry debug identifiers](https://docs.sentry.io/platforms/native/data-management/debug-files/identifiers/) | Sentry distinguishes code identifiers for binaries/libraries from debug identifiers for debug companions; for ELF, the debug ID is derived from the GNU build ID/code ID. Parallax fixtures must record both IDs and the match status. |
| [Sentry debug-file uploads](https://docs.sentry.io/platforms/native/data-management/debug-files/upload/) and [source context](https://docs.sentry.io/platforms/native/data-management/debug-files/source-context/) | Sentry recommends uploading debug files before deploy; later uploads may take time to affect new reports and existing events are not reprocessed. Source context for compiled apps requires source bundles alongside debug info. Parallax must decide whether it reprocesses older events when symbols arrive and must run source context through source-field/redaction gates before agent exposure. |
| [Sentry Symbolicator](https://getsentry.github.io/symbolicator/) and [lookup strategy](https://getsentry.github.io/symbolicator/advanced/symbol-lookup/) | Symbolicator resolves function names, file locations, and source context in native and JavaScript stack traces. Its lookup flow computes code/debug IDs, searches sources, chooses the best file, and treats debug information, symbol tables, unwind data, and source bundles as different evidence qualities. |
| [Symbolicator system architecture](https://getsentry.github.io/symbolicator/advanced/system-architecture/) and [source bundles](https://getsentry.github.io/symbolicator/advanced/source-bundles/) | Symbolicator ranks debug/unwind files above symbol-table fallbacks, caches downloaded objects and derived symbol caches, and source bundles are ZIP archives with a manifest containing code ID, debug ID, object name, architecture, and original source paths. Parallax should record symbol source/cache/provenance and protect source paths/snippets as sensitive source-derived fields. |
| [sentry Rust crate 0.48.2](https://docs.rs/sentry/0.48.2/sentry/) and [feature flags](https://docs.rs/crate/sentry/0.48.2/features) | The current Rust SDK default features include `backtrace`, `contexts`, `panic`, `transport`, `debug-images`, and release health. Optional `anyhow`, `tracing`, and OpenTelemetry integrations require explicit features/setup. |
| [Rust capture fidelity recheck](rust-capture-fidelity-recheck.md) | Current capture pass pins `tracing` `0.1.44`, `tracing-error` `0.2.1`, `tracing-opentelemetry` `0.33.0`, `opentelemetry`/`opentelemetry-otlp`/`opentelemetry-appender-tracing` `0.32.0`, `anyhow` `1.0.102`, `eyre` `0.6.12`, and `color-eyre` `0.6.5`; it also defines missing-layer and panic-strategy negative fixtures. |
| [sentry-panic 0.48.2](https://docs.rs/sentry-panic/0.48.2/sentry_panic/), [sentry-backtrace 0.48.2](https://docs.rs/sentry-backtrace/0.48.2/sentry_backtrace/), and [sentry-debug-images 0.48.2](https://docs.rs/sentry-debug-images/0.48.2/sentry_debug_images/) | The panic integration installs a panic handler and forwards to the prior hook; the backtrace crate converts and processes stacktraces; the debug-images integration attaches loaded-library metadata to events. Parallax fixtures must prove which integration produced each signal instead of treating "Sentry Rust event" as one opaque capture path. |
| [sentry-tracing 0.48.2](https://docs.rs/sentry-tracing/0.48.2/sentry_tracing/) | `tracing` error events can become Sentry events, breadcrumbs, logs, and spans; structured fields can become context fields or tags. Spans and breadcrumbs give causal context, but they should support grouping rather than replace physical stack identity. |
| [tracing-error](https://docs.rs/tracing-error/latest/tracing_error/) | `SpanTrace` captures logical span context at the error site. It is orthogonal to a call-stack backtrace and is especially useful across async boundaries. |

## Product Decision

Parallax v0 should compute its own grouping fingerprint after normalization:

- accept client-provided Sentry `fingerprint` when policy allows it;
- otherwise use `rust-stack-v1` for Rust exception/panic stacks;
- store the exact normalized grouping material beside every event;
- store the algorithm version and confidence;
- never silently regroup historical issues when the algorithm changes.
- treat `debug_meta` / loaded-image metadata as lookup evidence, not proof that
  the event is already symbolicated;
- record the symbolication source, matched debug-file identity, source-context
  status, and whether late symbol uploads triggered reprocessing.

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
- keep debug files/source bundles for at least the telemetry and release
  rollback window Parallax claims to support; Sentry's time-to-idle retention is
  a useful reference, not automatically sufficient for Parallax's historical
  evidence goal;
- expose source context to agents only when source-field policy and A6 redaction
  pass for the exact bundle projection.

## Capture Path Policy

Do not treat "Rust stacktrace exists" as a single boolean. The fixture output
must preserve the chain that produced the grouping material:

1. **Panic hook signal**: whether `sentry-panic`, a custom hook, or no hook
   observed the panic; whether the previous hook was chained; and whether
   `PanicHookInfo` exposed payload and location.
2. **Backtrace signal**: whether frames came from `sentry-backtrace`, a Rust
   `Backtrace::capture`, `Backtrace::force_capture`, a parsed string, or an SDK
   event field; plus the `RUST_BACKTRACE` / `RUST_LIB_BACKTRACE` state.
3. **Unwind and target signal**: target triple, panic strategy, frame-pointer
   and unwind-table settings, and whether the build path can produce a stack at
   all. `panic=immediate-abort` should be a negative fixture because panic hooks
   are not called.
4. **Debug-image and symbolication signal**: whether `debug_meta` and loaded
   images were emitted; which code/debug IDs were present; whether a matching
   debug companion, symbol table, unwind file, or source bundle was found; which
   source provided it; and whether symbolication used uploaded files, a symbol
   server, or no server-side lookup.
5. **Agent-visible signal**: whether frame context, breadcrumbs, tags, span
   fields, and source snippets passed the A6 redaction pipeline and the A1
   source-field policy before entering a bundle.

This separation matters for Parallax's AI-native goal. An agent needs to know
whether it is looking at a reliable source frame, a function-only fallback, a
panic location without a stack, or a logical span trace that only explains the
operation. Those are different evidence classes with different fix confidence.

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
| Matching debug companion uploaded before event | Full symbolication only if code/debug ID match, file/line evidence appears, and result provenance is recorded. |
| Debug companion uploaded after event | Event starts degraded; any later improvement requires an explicit reprocess result and original grouping assignment audit. |
| Debug ID mismatch | No full symbolication claim; mismatch recorded without fabricating file/line precision. |
| Symbol-table-only fallback | Function names may appear, but fixture remains lower confidence because file/line and inline-frame data are missing. |
| Source bundle matched | Source context is linked, but not agent-visible until source-field policy and redaction pass for context lines and paths. |
| `panic=abort` with Sentry panic hook | Panic event captured if the hook fires; process termination path recorded; grouping confidence depends on captured frames and location. |
| `panic=immediate-abort` / no hook path | No Sentry panic event should be assumed; fixture should fail any capture-path claim if Parallax still advertises panic grouping for this build. |
| `Backtrace::capture` disabled by env | Stack absent or disabled status recorded; no high-confidence grouping unless another capture path supplies frames. |
| `Backtrace::force_capture` path | Stack captured despite env gates; overhead and policy caveat recorded separately from default production behavior. |
| Debug images disabled in SDK features | Event may still parse, but symbol-file matching and build/debug ID evidence are absent; cannot satisfy full symbolication claims. |
| Stack/breadcrumb/span source-field canary | Agent-visible bundle carries `redaction_report.source_field_policy.status = pass`; denied fields never appear in projected grouping material. |

## Success Gates

This proof gate is closed only when:

- unchanged fixture input produces stable fingerprints across parser releases;
- the same logical Rust bug groups across full debuginfo, line tables, split
  debuginfo, and rebuilt binaries;
- line-number-only churn does not split issues by default;
- unrelated app functions do not merge;
- missing debuginfo creates explicit low-confidence grouping, not fabricated
  source precision;
- loaded-image metadata is separated from actual symbolication results and debug
  companion/source bundle match status;
- capture path, panic strategy, SDK features, backtrace environment, and
  debug-image presence are recorded for every fixture;
- late debug-file uploads either have a measured reprocess path or remain
  degraded for existing events;
- `panic=immediate-abort`, missing hooks, disabled backtraces, and disabled
  debug images are negative fixtures with explicit degraded or failed claims;
- all grouping material can be reprocessed under `rust-stack-v2` without losing
  the original `rust-stack-v1` issue assignment.

Results must be published through the
[Rust stacktrace grouping ledger](rust-stacktrace-grouping-ledger.md); otherwise
the claim remains `not_measured`.

## Schema Additions

The normalized error-event model should add:

| Field | Purpose |
| --- | --- |
| `grouping_algorithm_version` | Example: `rust-stack-v1`. |
| `fingerprint_source` | `client`, `rust_stack`, `thread_stack`, `message`, or `manual`. |
| `grouping_material` | JSON snapshot of normalized frames and anchors used in the hash. |
| `grouping_confidence` | `high`, `medium`, `low`; drives UI and agent wording. |
| `grouping_warnings` | Missing debuginfo, no in-app frames, stack absent, unsymbolicated frames. |
| `capture_path` | `sentry_panic`, `sentry_backtrace`, `std_capture`, `std_force_capture`, `parsed_stacktrace`, or `sdk_event_only`. |
| `panic_hook_status` | `sentry_hook_invoked`, `custom_hook_invoked`, `not_invoked`, `not_applicable`, or `unknown`. |
| `backtrace_status` | Preserve Rust/Sentry capture status: `captured`, `disabled`, `unsupported`, `missing`, or `unknown`. |
| `sdk_feature_set` | Exact Sentry Rust features/integrations enabled for fixture generation. |
| `frame.raw_function` | Raw SDK/symbolicator function. |
| `frame.demangled_function` | Demangled Rust symbol, if available. |
| `frame.normalized_function` | Versioned normalized function used for grouping. |
| `frame.in_app_reason` | Why Parallax treated the frame as app/non-app. |
| `frame.symbolication_status` | Full/function-only/missing-line/unsymbolicated/etc. |
| `frame.build_id` / `frame.debug_id` | Link to loaded image and debug-file inventory. |
| `frame.code_id` | Native binary/library identifier used for lookup when present. |
| `symbolication_source` | `in_event`, `uploaded_debug_file`, `symbol_server`, `source_bundle`, or `none`. |
| `debug_file_match_status` | `matched`, `missing`, `mismatch`, `not_required`, or `unknown`. |
| `source_context_status` | `available`, `policy_denied`, `redacted`, `missing`, or `not_requested`. |
| `symbolication_reprocess_status` | Whether a late symbol/source upload reprocessed existing evidence. |

## Relationship To Other Research

- [Rust data collection and instrumentation](rust-data-collection-and-instrumentation.md)
  defines why in-process Rust error capture and debuginfo are mandatory.
- [Sentry-compatible ingestion](sentry-compatible-ingestion.md) defines the
  envelope/event subset and first deterministic grouping path.
- [Sentry SDK fixture compatibility gate](sentry-sdk-fixture-compatibility.md)
  supplies the SDK-generated fixture harness this gate should extend.
- [Rust stacktrace grouping ledger](rust-stacktrace-grouping-ledger.md) turns
  this gate's fixture matrix into claim levels, result rows, refresh triggers,
  and product wording.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) should expose
  `grouping_confidence`, symbolication warnings, and redaction status to agents.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  still has veto power over stack locals, breadcrumbs, tags, and span fields.
- [Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md)
  and [A1 eval result ledger](a1-eval-result-ledger-and-model-refresh.md)
  define the source-field isolation policy that decides whether fixture-derived
  frame/source/context fields can appear in agent-visible bundles at all.

## Bottom Line

Rust stacktrace grouping is buildable, but only if Parallax makes it explicit:
versioned grouping, conservative Rust symbol normalization, mandatory debuginfo
policy, per-frame symbolication status, and a fixture matrix that tries to break
the grouping algorithm before users do.

Do not ship "deterministic grouping" as a claim until this gate passes and the
ledger records a fresh `rust_grouping_stable` result.
