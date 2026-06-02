# Rust Capture — Collection, Fidelity, and Stacktrace Grouping

<!-- markdownlint-disable MD013 -->

> Parallax V1 should capture Rust app-level errors in-process with Rust `tracing`, `tracing-error`, and OpenTelemetry over OTLP, then derive Parallax-owned `error_event` rows from span exception events, span error status, and ERROR/FATAL log records. Sentry-compatible panic/error ingest is future migration compatibility, not V1 scope. Treat eBPF as an optional complement for zero-instrumentation infrastructure signal, never as the primary error path — because panic messages, typed/`anyhow` source chains, span attributes, and release/environment metadata exist only as in-process language constructs that a kernel-level probe cannot read. Capture is not one feature but a measured contract across `tracing` `0.1.44`/`tracing-error` `0.2.1`/`tracing-opentelemetry` `0.33.0`, `opentelemetry`/`opentelemetry-otlp`/`opentelemetry-appender-tracing` `0.32.0`, future `sentry` compatibility fixtures, `anyhow` `1.0.102`/`eyre` `0.6.12`/`color-eyre` `0.6.5`, panic strategy, backtrace environment, debuginfo profile, and redaction, and that contract must be proven by fixtures before any "Rust errors are agent-ready" wording. Grouping is decided as a versioned, fixture-tested product primitive: a deterministic `rust-stack-v1` fingerprint that keeps the same logical Rust bug grouped across rebuilds and debuginfo layouts, computed after conservative Rust symbol normalization, with per-frame symbolication status and client-fingerprint precedence, and with release/environment/host/build-id/commit deliberately excluded from issue identity. A mandatory debuginfo policy (`debug = "line-tables-only"` with `strip = "none"`, or split debuginfo + server-side symbolication) is required or backtraces are worthless. The remaining open gate is measurement: the `rust-stack-v1` grouping claim is currently `not_measured`, and no "deterministic Rust grouping" claim is allowed until dated fixture runs covering capture paths, panic strategies, rebuilds, debuginfo variants, normalization, false splits/merges, client fingerprints, symbolication degradation, redaction, and source-field isolation pass and are published through the grouping ledger.

This note consolidates the following previously-separate research files, each preserved in full below:

- `rust-data-collection-and-instrumentation.md`
- `rust-capture-fidelity-recheck.md`
- `rust-stacktrace-grouping-and-symbolication.md`
- `rust-stacktrace-grouping-ledger.md`

## Rust Data Collection and Instrumentation

_Provenance: merged verbatim from `rust-data-collection-and-instrumentation.md` (2026-05-29 restructure)._

_(Shared note — see the markdownlint-disable directive at the top of this consolidated file.)_

Research date: 2026-05-25

### Executive Summary

The first product question for Parallax is not storage or UI. It is: **how do we
get enough data out of a Rust application to explain why it failed?** The answer
shapes everything downstream.

> Capture app-level errors in-process with Rust `tracing`, `tracing-error`, and
> OpenTelemetry over OTLP; derive Parallax error rows from exception span events
> and ERROR/FATAL logs. Treat eBPF as an optional complement for
> zero-instrumentation infrastructure signal, never as the primary error path.

The current capture-feature/version matrix and fixture additions are tracked in
[Rust capture fidelity recheck](rust.md). The important
update from that pass is that Parallax should not claim "Rust errors are
agent-ready" from a single SDK init: Sentry feature flags, `tracing-error`
`ErrorLayer`, OpenTelemetry log appender setup, backtrace environment, panic
strategy, and debuginfo profile all have to be measured.

The reason is architectural, not a maturity gap that will close later. A Rust
panic message, an `anyhow` context chain, a typed error's source chain, span
attributes, and release/environment metadata exist only as in-process language
constructs. They are never serialized to a syscall or a network packet, so a
kernel-level probe has nothing to read. eBPF can see *that* a request returned
HTTP 500 slowly. It cannot see *why* (`panicked at 'index out of bounds'`, with
a backtrace and the order ID still in scope). Parallax's entire value is the
*why*, so the collection layer must live where the *why* exists: inside the
process.

### Why Collection Method Decides the Product

Parallax exists to hand an AI agent enough evidence to propose a fix. The
evidence an agent needs is exactly the data that only the language runtime has:

| Agent needs | Where it lives | Reachable from kernel/eBPF? |
| --- | --- | --- |
| Panic / error message | In-process string | No |
| Typed error + source chain | In-process `Error` values | No |
| Symbolicated backtrace | Process memory + debuginfo | Partial, profiling-only |
| Span attributes / business context | In-process `tracing` spans | No |
| Release / environment | Process config | No |
| HTTP/gRPC RED metrics | Syscalls / sockets | Yes |
| Network / protocol topology | Packets / syscalls | Yes |
| CPU / off-CPU profiles | Scheduler / perf events | Yes |

The top half of that table is the product. The bottom half is useful context
that eBPF can add cheaply later.

### Option Space

| Path | What it collects | Fit for Parallax |
| --- | --- | --- |
| In-process SDK (Sentry SDK, OpenTelemetry SDK) → OTLP | Errors, panics, typed chains, spans, attributes, release/env | Primary. Only path with app-level error semantics. |
| Sentry ingestion API (envelopes) | Error events, grouping-ready payloads | Keep as a compatibility surface so existing Sentry SDKs work unchanged. |
| OpenTelemetry API / OTLP | Traces, metrics, logs (vendor-neutral) | Primary telemetry transport; standardize on it. |
| eBPF (zero instrumentation) | RED metrics, protocol spans, service maps, CPU profiles | Complement only. Cannot capture app-level error semantics. |

### eBPF: What It Is, and What It Can and Cannot Do

**What it is.** eBPF runs sandboxed programs in a privileged context such as the
OS kernel, verified and JIT-compiled before execution. Programs attach to kernel
hook points: system calls, kernel tracepoints, network events, and dynamic
kprobes/uprobes "almost anywhere," plus XDP for packets. The architectural fact
that matters: eBPF observes at the **kernel / syscall / network boundary**, not
inside application logic.

Source:

- [eBPF.io — What is eBPF](https://ebpf.io/what-is-ebpf/)

**What eBPF can capture with zero code changes.** Auto-instrumentation tools
converge on a consistent envelope: RED metrics (rate/errors/duration),
transaction-level HTTP/gRPC spans, protocol tracing (Postgres/MySQL/Redis/Kafka/
DNS), service-map topology, and sampling CPU profiles.

- Grafana Beyla auto-instruments "RED metrics ... and transaction-level traces
  across HTTP/S and gRPC."
- Pixie sets kprobes on networking syscalls and uprobes on TLS libraries to read
  bodies before encryption.
- Coroot "automatically collects metrics, logs, traces, and profiling ... no
  code changes."
- Parca / Polar Signals do continuous CPU profiling by unwinding stacks in BPF.

Sources:

- [Grafana Beyla docs](https://grafana.com/docs/beyla/latest/)
- [Pixie — How Pixie uses eBPF](https://docs.px.dev/about-pixie/pixie-ebpf/)
- [Coroot — eBPF-based tracing](https://docs.coroot.com/tracing/ebpf-based-tracing/)
- [Polar Signals — Profiling without frame pointers](https://www.polarsignals.com/blog/posts/2022/11/23/introducing-profiling-without-frame-pointers)

**What eBPF cannot do — stated by the eBPF vendors themselves.** This is the
strongest evidence, because it is not a competitor's critique:

- OpenTelemetry's eBPF instrumentation (OBI, the donated Beyla) states it
  "cannot always recover application-specific details that are not visible from
  eBPF observation points" and tells you to "use language agents or manual
  instrumentation when you need custom spans, application-specific attributes,
  business events."
- Beyla's docs: "Beyla only provides generic metrics and transaction level trace
  span information. Language agents and manual instrumentation are still
  recommended" — explicitly no custom span attributes, no internal spans, no
  runtime metrics, no business telemetry.
- Coroot notes eBPF spans are individual and often lack a trace ID, so "we
  cannot connect them to show you the whole trace," and events can be dropped
  under load due to ring-buffer limits.
- Pixie's query language "has no support for classes and exceptions."

Sources:

- [OpenTelemetry eBPF Instrumentation (OBI)](https://opentelemetry.io/docs/zero-code/obi/)
- [Grafana Beyla docs](https://grafana.com/docs/beyla/latest/)
- [Coroot — Java TLS instrumentation with eBPF](https://coroot.com/blog/java-tls-instrumentation-with-ebpf/)
- [Pixie — How Pixie uses eBPF](https://docs.px.dev/about-pixie/pixie-ebpf/)

**Rust-specific eBPF pain.** eBPF stack unwinding needs frame pointers, which are
"disabled by default" on many distros and stripped by optimized native builds —
a problem that affects "C, C++, and Rust." The workaround is DWARF `.eh_frame`
unwinding inside BPF, which works for Rust as long as the ELF `.eh_frame` section
is produced, but is materially more complex and costly than frame-pointer
walking. Even when it succeeds, the result is a *CPU/profiling* stack, not a
symbolicated error event with a message and a typed chain.

Sources:

- [Polar Signals — Profiling without frame pointers](https://www.polarsignals.com/blog/posts/2022/11/23/introducing-profiling-without-frame-pointers)
- [Polar Signals — DWARF-based stack walking using eBPF](https://www.polarsignals.com/blog/posts/2022/11/29/dwarf-based-stack-walking-using-ebpf)

**Verdict.** eBPF is genuinely valuable for zero-touch infrastructure signal —
service maps, network/protocol RED metrics, continuous CPU/off-CPU profiling —
and Parallax can offer it later as a no-instrumentation onboarding path. But
routing the *error* pipeline through eBPF would be a strategic mistake: every
relevant vendor states it cannot produce the application-level error semantics
that are Parallax's whole product.

### Rust Error Capture Mechanisms

| Mechanism | Captures | Notes |
| --- | --- | --- |
| `std::panic::set_hook` + `std::backtrace::Backtrace` | Panic message, location, frames | Frames need debuginfo; capture gated by `RUST_BACKTRACE` / `RUST_LIB_BACKTRACE` (`force_capture` ignores them). |
| `sentry` crate | Panics, backtraces, `anyhow` chains, `tracing` events; `release`/`environment` | Closest off-the-shelf match; Sentry-compatible by definition. |
| `tracing` + `tracing-error` (`SpanTrace`) | Active span context + fields at the error site | Logical context, orthogonal to a call-stack backtrace; needs an `ErrorLayer`. |
| `tracing-opentelemetry` + `opentelemetry-otlp` | Traces, metrics over OTLP (gRPC/HTTP) | Does **not** export logs — use `opentelemetry-appender-tracing` for logs. |
| `anyhow` / `eyre` / `color-eyre` | Error source chain + backtrace | `anyhow::Error` auto-captures a backtrace (Rust ≥ 1.65) unless the inner error already has one; `.context()` builds the chain; `color-eyre` adds a `SpanTrace`. |

Current source check, 2026-05-25:

- Rust docs show `std` `1.95.0`; `Backtrace::capture` remains env-gated by
  `RUST_LIB_BACKTRACE` / `RUST_BACKTRACE`, while `force_capture` bypasses that
  gate.
- `sentry` remains `0.48.2`; default features include `backtrace`, `contexts`,
  `debug-images`, `panic`, `release-health`, and `transport`, while `tracing`,
  `anyhow`, and `opentelemetry` are opt-in.
- `tracing` is `0.1.44`; `tracing-error` is `0.2.1`; `tracing-opentelemetry`
  is `0.33.0` and depends on `opentelemetry` `0.32.0`.
- `opentelemetry`, `opentelemetry-otlp`, and
  `opentelemetry-appender-tracing` are `0.32.0`.
- `anyhow` is `1.0.102`, `eyre` is `0.6.12`, and `color-eyre` is `0.6.5`.

Sources:

- [std::backtrace](https://doc.rust-lang.org/std/backtrace/index.html)
- [docs.rs/sentry](https://docs.rs/sentry/latest/sentry/)
- [Rust capture fidelity recheck](rust.md)
- [Sentry Rust platform docs](https://docs.sentry.io/platforms/rust/)
- [docs.rs/tracing-error](https://docs.rs/tracing-error/latest/tracing_error/)
- [docs.rs/tracing-opentelemetry](https://docs.rs/tracing-opentelemetry)
- [opentelemetry-rust](https://github.com/open-telemetry/opentelemetry-rust)
- [docs.rs/anyhow](https://docs.rs/anyhow/latest/anyhow/)

Two complementary kinds of "stack" matter and should both be captured:

1. **Call-stack backtrace** — physical frames (crate/module/fn/file/line) from
   `Backtrace` / `anyhow`. Good for grouping and code navigation.
2. **`SpanTrace`** — the logical `tracing` span context active at the error site
   (request ID, user, operation, attributes). Good for agent context and
   correlation to OTLP traces.

### Data Model: Fields to Store

| Field | Purpose |
| --- | --- |
| Error/panic message (normalized) | Grouping fallback + human/agent summary. |
| Error type | Grouping + classification. |
| Source chain (`anyhow`/`eyre`) | Causal narrative for the agent. |
| Backtrace frames: crate/module/fn/file/line | Primary grouping signal + code navigation. |
| `SpanTrace` (span names + fields) | Logical context, correlation. |
| `trace_id` / `span_id` | Stitch the error to OTLP traces/logs/metrics. |
| Release / version | Regression detection ("started after release X"). |
| Environment | Scope and noise control. |

This fieldset is consistent with the Sentry-inspired event model already drafted
in the architecture research, so a Sentry-compatible ingest layer maps onto it
directly.

Related: [self-hosted-observability-architecture.md](../architecture/overview.md).

### Debuginfo Policy (Mandatory)

Without debug information, backtrace frames have no filename or line number —
they are unsymbolicated and nearly useless for grouping and for an agent. The
Rust-first recommendation:

- ship `debug = "line-tables-only"` for the minimal info needed to resolve
  filename/line in backtraces without variable/parameter info or binary bloat;
- or use `split-debuginfo` to ship symbols separately and symbolicate
  server-side from a build-id-keyed debuginfo store.

Sources:

- [Cargo — Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [std::backtrace](https://doc.rust-lang.org/std/backtrace/index.html)

The focused grouping and symbolication gate is defined in
[Rust stacktrace grouping and symbolication](rust.md),
and its claimable result rows are defined in the
[Rust stacktrace grouping ledger](rust.md).

### How This Fits the Parallax Pipeline

```text
Rust app
  -> tracing + tracing-error + OTLP exporter (traces/logs/metrics)
  -> Parallax ingest gateway (OTLP endpoints)
  -> durable stream (Apache Iggy)
  -> processors: derive error_event, normalize, group, symbolicate, correlate
  -> GreptimeDB (events/logs/spans/metrics) + metadata store
  -> context API + MCP server  (agent: primary)
  -> Sentry-like UI            (human: secondary)

[optional, later] eBPF agent -> infra RED metrics, service maps, CPU profiles
[future] Sentry-compatible panic/error layer -> same Parallax error_event model
```

The error path and the telemetry path share the ingest gateway and stream. eBPF,
if added, is a separate optional producer feeding infrastructure-level signal
into the same storage — not a replacement for any of the in-process capture.

### Open Questions

1. Server-side symbolication: store split debuginfo keyed by build id, or require
   line-tables in the shipped binary? What is the retention cost of debuginfo?
   See the proof-gate policy in
   [Rust stacktrace grouping and symbolication](rust.md)
   and the claim ledger in
   [Rust stacktrace grouping ledger](rust.md).
2. Which Rust OTLP fixture best preserves panic/error material: span exception
   events, ERROR/FATAL log records with `exception.*`, or both?
3. Rust frame normalization for stable grouping across releases (hash suffixes,
   generics, monomorphization, panic location) — now specified as
   `rust-stack-v1` in
   [Rust stacktrace grouping and symbolication](rust.md)
   and measured through
   [Rust stacktrace grouping ledger](rust.md).
4. PII and secrets in span fields / error context: redaction defaults for
   self-hosted teams.
5. Async backtraces: `tracing` span context (`SpanTrace`) is often more useful
   than a physical backtrace across `.await` points — how much do we lean on it?
6. SDK ergonomics: how thin can a `parallax`-flavored Rust setup be (one init
   call wiring panic hook + tracing + OTLP) to maximize adoption?

### Bottom Line

- In-process Rust SDKs are the primary error-capture path; only the runtime
  exposes panic messages, typed chains, span attributes, and release/env.
- Standardize on `tracing` + OTLP for telemetry and a Sentry-compatible
  panic/error layer for errors — matches the OTel + Sentry-compatible direction
  and reuses mature crates (`opentelemetry-otlp`, `sentry`, `tracing-error`).
- Capture both a symbolicated backtrace and a `SpanTrace`, plus trace/span IDs,
  to make errors first-class, correlatable, and agent-ready.
- Mandate a debuginfo policy (`line-tables-only` or `split-debuginfo` +
  server-side symbolication) or backtraces are worthless.
- Treat eBPF as an optional, later complement for infrastructure signal — not a
  dependency, and never the error pipeline.

## Rust Capture Fidelity Recheck

_Provenance: merged verbatim from `rust-capture-fidelity-recheck.md` (2026-05-29 restructure)._

_(Shared note — see the markdownlint-disable directive at the top of this consolidated file.)_

Research date: 2026-05-25

### Pass Target

Re-check the Rust-first capture claim from current primary sources. The weak
claim is not "Rust can produce panics and traces." The weak claim is whether a
single Parallax Rust setup can reliably capture the evidence agents need:

- panic payload and location;
- physical backtrace frames with useful file/line quality;
- typed error source chain and `anyhow`/`eyre` context;
- logical `tracing` span context through `SpanTrace`;
- OTLP trace, metric, and log export;
- release, environment, build/debug image, and redaction metadata.

### Short Verdict

The core decision still holds: Rust app failures need in-process capture.
eBPF/OBI can add network/protocol spans, RED metrics, and profiling, but it
cannot be the primary error path because app-specific error semantics live in
Rust values, panic hooks, and `tracing` spans.

The design needs a sharper capture-fidelity gate before any "Rust errors are
agent-ready" wording:

1. **Sentry default features are not the whole setup.** `sentry` `0.48.2`
   defaults include panic, backtrace, contexts, debug-images, release-health,
   and transport, but `tracing`, `anyhow`, and OpenTelemetry integrations are
   opt-in feature/setup paths.
2. **Backtrace presence is configuration-sensitive.** `Backtrace::capture` is
   disabled unless `RUST_LIB_BACKTRACE` or `RUST_BACKTRACE` enables it, and
   filenames/line numbers usually need debuginfo.
3. **Panic strategy matters.** `panic=abort` can still run the hook, but
   `panic=immediate-abort` explicitly does not call panic hooks.
4. **Logical span context requires an `ErrorLayer`.** `tracing-error` provides
   `SpanTrace`, but applications must install the layer and instrument errors.
5. **Logs need the log appender path.** `tracing-opentelemetry` is the tracing
   bridge, while `opentelemetry-appender-tracing` is the current log appender
   crate that should be pinned in fixtures.

### Current Primary-Source Snapshot

| Source | Current signal | Parallax implication |
| --- | --- | --- |
| [Rust `std::backtrace`](https://doc.rust-lang.org/std/backtrace/index.html) and [`Backtrace`](https://doc.rust-lang.org/std/backtrace/struct.Backtrace.html) | Rust docs show `std` `1.95.0`. Backtraces are best-effort; file/line reporting usually needs debuginfo; `Backtrace::capture` is environment-gated; `force_capture` bypasses the env gate; the env state is cached after first use. | Capture fixtures must record env vars, capture mode, status, and debuginfo quality. Do not treat absent frames as parser failure without this context. |
| [`std::panic::set_hook`](https://doc.rust-lang.org/std/panic/fn.set_hook.html) | Panic hooks are global, replace the prior hook unless chained, run before the panic runtime, and receive `PanicHookInfo` payload/location. | A Parallax init helper must chain previous hooks and record whether the capture hook actually ran. |
| [rustc codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html) | `panic` can be `abort`, `unwind`, or `immediate-abort`; `immediate-abort` terminates without calling panic hooks. `force-frame-pointers` and `force-unwind-tables` defaults depend on target. | `panic=immediate-abort` is a negative fixture. Target, panic strategy, frame-pointer, and unwind-table knobs must be in the run manifest. |
| [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) | Release defaults include `debug = false`; `line-tables-only` is the minimal debug info for filename/line backtraces; `split-debuginfo` and `strip` change symbol availability. | The capture contract must recommend line tables or split debuginfo and record strip/symbol state. |
| [`sentry` crate](https://docs.rs/crate/sentry/latest/features) | crates.io latest is `0.48.2`, updated 2026-05-11. Default features include `backtrace`, `contexts`, `debug-images`, `panic`, `release-health`, and `transport`; `tracing`, `anyhow`, and `opentelemetry` are explicit feature/setup paths. | The first fixture target remains Sentry Rust, but the manifest must record exact features. Default Sentry is not enough to prove span or `anyhow` chain capture. |
| [`tracing`](https://crates.io/crates/tracing) | crates.io latest is `0.1.44`, updated 2025-12-18. | Pin the instrumentation API version separately from OpenTelemetry bridge versions. |
| [`tracing-error`](https://docs.rs/tracing-error) | crates.io latest is `0.2.1`; docs describe `SpanTrace` and `ErrorLayer`, and mark the crate experimental. | `SpanTrace` is valuable logical context, but it is not automatic. Fixture rows must prove the layer was installed and captured. |
| [`tracing-opentelemetry`](https://crates.io/crates/tracing-opentelemetry) | crates.io latest is `0.33.0`, updated 2026-05-18, and depends on `opentelemetry` `^0.32.0`. Default features include `tracing-log` and `metrics`. | The current bridge moved after the base OpenTelemetry Rust release. Fixture manifests should pin the bridge, not only `opentelemetry`. |
| [`opentelemetry`](https://crates.io/crates/opentelemetry), [`opentelemetry-otlp`](https://crates.io/crates/opentelemetry-otlp), and [`opentelemetry-appender-tracing`](https://crates.io/crates/opentelemetry-appender-tracing) | `opentelemetry` and `opentelemetry-otlp` latest are `0.32.0`, updated 2026-05-08. `opentelemetry-appender-tracing` latest is `0.32.0`, updated 2026-05-09. | Trace/metric export and log export are separate setup surfaces. A "Rust OTLP three-signal" claim needs the appender path. |
| [`anyhow`](https://docs.rs/anyhow/latest/anyhow/) | crates.io latest is `1.0.102`, updated 2026-02-20. Docs state Rust 1.65+ captures/prints a backtrace if the underlying error does not provide one, but visibility depends on the standard backtrace env vars. | `anyhow` chains are a strong default, but capture status and env gates still need rows. |
| [`eyre`](https://crates.io/crates/eyre) and [`color-eyre`](https://docs.rs/color-eyre) | `eyre` latest is `0.6.12`; `color-eyre` latest is `0.6.5` with default `capture-spantrace` through `tracing-error`. | Support as common Rust error-reporting variants, but fixture them separately from `anyhow`. |
| [OpenTelemetry eBPF Instrumentation](https://opentelemetry.io/docs/zero-code/obi/) | Current OBI docs still say eBPF cannot always recover app-specific details outside its observation points and recommends language agents/manual instrumentation for custom spans, app attributes, and business events. | Confirms eBPF remains complementary for Parallax's Rust app error context. |

### Minimum Rust Capture Contract

The first Parallax Rust setup should produce a capture manifest like:

```json
{
  "capture_profile": "rust-app-v0",
  "sentry_version": "0.48.2",
  "sentry_features": [
    "backtrace",
    "contexts",
    "debug-images",
    "panic",
    "transport",
    "tracing",
    "anyhow"
  ],
  "tracing_version": "0.1.44",
  "tracing_error_version": "0.2.1",
  "tracing_opentelemetry_version": "0.33.0",
  "opentelemetry_version": "0.32.0",
  "opentelemetry_otlp_version": "0.32.0",
  "opentelemetry_appender_tracing_version": "0.32.0",
  "panic_strategy": "unwind|abort|immediate-abort",
  "backtrace_capture_mode": "sentry|std_capture|std_force_capture|disabled",
  "spantrace_layer": "installed|missing",
  "log_export_path": "opentelemetry-appender-tracing|missing",
  "debug_info_policy": "line-tables-only|split-debuginfo|full|none",
  "ebpf_role": "none|supplemental"
}
```

This is a fixture contract, not necessarily the literal product schema. It
forces the important distinction: capture success is a property of crate
versions, feature flags, runtime environment, build profile, and panic strategy.

### Required Fixture Additions

Add these to the Rust data-collection and stacktrace fixture suite:

| Fixture | Must prove |
| --- | --- |
| `sentry_default_panic` | Default `sentry` features capture panic payload/location and loaded-image metadata when the hook runs. |
| `sentry_anyhow_feature_on` | `sentry-anyhow` path captures error source/context chain and does not rely only on the rendered message. |
| `sentry_tracing_feature_on` | `sentry-tracing` path captures structured `tracing` fields without leaking denied fields. |
| `spantrace_error_layer_present` | `SpanTrace` appears only when `ErrorLayer` is installed and error instrumentation uses it. |
| `spantrace_error_layer_missing` | Missing `ErrorLayer` produces an explicit missing-evidence row, not a false success. |
| `otlp_three_signal_rust` | `tracing-opentelemetry` plus `opentelemetry-otlp` and `opentelemetry-appender-tracing` emit trace, metric, and log fixtures over the required OTLP transport profile. |
| `anyhow_backtrace_env_disabled` | `anyhow` chain is captured but backtrace status is disabled when env gates disable it. |
| `anyhow_backtrace_env_enabled` | Backtrace appears when `RUST_LIB_BACKTRACE` or `RUST_BACKTRACE` enables it and debuginfo supports file/line. |
| `panic_abort_hook_capture` | `panic=abort` still records hook behavior and termination path for the target. |
| `panic_immediate_abort_negative` | No panic-hook capture is claimed when `panic=immediate-abort` is used. |
| `line_tables_release_profile` | `debug = "line-tables-only"` yields file/line quality without full debuginfo. |
| `strip_without_symbols_negative` | Stripped/no-symbol build degrades safely unless matching debug companions exist. |
| `ebpf_supplemental_only` | OBI/eBPF-style network spans enrich context but cannot satisfy app-error capture rows. |

### Product Wording

Allowed now:

> Rust-first capture design: in-process Sentry and OpenTelemetry capture are
> required for app-level errors; eBPF is supplemental.

Allowed after fixture rows pass:

> Rust panic/error capture for the tested crate versions, feature flags, build
> profiles, panic strategies, and OTLP transports.

Avoid:

- "one-line Rust install captures everything" before feature/env/build fixture
  rows exist;
- "eBPF captures Rust errors" as a Parallax primary path;
- "OpenTelemetry logs are covered" unless the log appender path is in the
  manifest;
- "Sentry Rust captures `anyhow`/`tracing` context" unless those features and
  setup paths are enabled and measured.

### Falsification Triggers

Reopen the Rust capture plan if:

- `sentry`, `tracing-opentelemetry`, `opentelemetry`, `opentelemetry-otlp`,
  `opentelemetry-appender-tracing`, `tracing-error`, or standard backtrace/panic
  behavior changes;
- Rust 2026-era panic/backtrace behavior changes the hook or env-gate model;
- real fixture apps show the Sentry and OTLP paths duplicate or disagree on
  error identity in a way that cannot be normalized;
- `SpanTrace` is too unreliable across async/service boundaries to help bundles;
- OBI/eBPF or another zero-code tool proves reliable capture of typed Rust error
  messages, chains, and source-level backtraces without in-process hooks.

### Sources

- [Rust `std::backtrace`](https://doc.rust-lang.org/std/backtrace/index.html)
- [`Backtrace`](https://doc.rust-lang.org/std/backtrace/struct.Backtrace.html)
- [`std::panic::set_hook`](https://doc.rust-lang.org/std/panic/fn.set_hook.html)
- [rustc codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html)
- [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [`sentry` crate features](https://docs.rs/crate/sentry/latest/features)
- [`tracing` crate](https://crates.io/crates/tracing)
- [`tracing-error`](https://docs.rs/tracing-error)
- [`tracing-opentelemetry`](https://crates.io/crates/tracing-opentelemetry)
- [`opentelemetry`](https://crates.io/crates/opentelemetry)
- [`opentelemetry-otlp`](https://crates.io/crates/opentelemetry-otlp)
- [`opentelemetry-appender-tracing`](https://crates.io/crates/opentelemetry-appender-tracing)
- [`anyhow`](https://docs.rs/anyhow/latest/anyhow/)
- [`eyre`](https://crates.io/crates/eyre)
- [`color-eyre`](https://docs.rs/color-eyre)
- [OpenTelemetry eBPF Instrumentation](https://opentelemetry.io/docs/zero-code/obi/)

### Bottom Line

Rust capture is not one feature. It is a measured contract across Sentry
features, `tracing` layers, OpenTelemetry exporters, log appenders, panic
strategy, backtrace environment, debuginfo, and redaction. That contract should
be proven before Parallax tells users their Rust failures are agent-ready.

## Rust Stacktrace Grouping and Symbolication

_Provenance: merged verbatim from `rust-stacktrace-grouping-and-symbolication.md` (2026-05-29 restructure)._

_(Shared note — see the markdownlint-disable directive at the top of this consolidated file.)_

Research date: 2026-05-25

### Purpose

This note tightens proof gate #6 from
[Strategic verdict and research coverage](../decisions/strategic-coverage.md):

> Rust stacktrace grouping stability across release/debug-info variants.

The decision: **Parallax should treat Rust stack grouping as a versioned,
fixture-tested product primitive, not a side effect of whatever stack frames an
SDK happens to emit.**

The v0 target is not Sentry grouping parity. The target is a deterministic
`rust-stack-v1` fingerprint that keeps the same logical Rust bug grouped across
rebuilds and debuginfo layouts, while recording enough grouping material to
audit false splits and false merges later. The companion
[Rust stacktrace grouping ledger](rust.md) defines
the result rows and claim levels required before this becomes a product claim.

### Current Primary-Source Checks

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
| [Rust capture fidelity recheck](rust.md) | Current capture pass pins `tracing` `0.1.44`, `tracing-error` `0.2.1`, `tracing-opentelemetry` `0.33.0`, `opentelemetry`/`opentelemetry-otlp`/`opentelemetry-appender-tracing` `0.32.0`, `anyhow` `1.0.102`, `eyre` `0.6.12`, and `color-eyre` `0.6.5`; it also defines missing-layer and panic-strategy negative fixtures. |
| [sentry-panic 0.48.2](https://docs.rs/sentry-panic/0.48.2/sentry_panic/), [sentry-backtrace 0.48.2](https://docs.rs/sentry-backtrace/0.48.2/sentry_backtrace/), and [sentry-debug-images 0.48.2](https://docs.rs/sentry-debug-images/0.48.2/sentry_debug_images/) | The panic integration installs a panic handler and forwards to the prior hook; the backtrace crate converts and processes stacktraces; the debug-images integration attaches loaded-library metadata to events. Parallax fixtures must prove which integration produced each signal instead of treating "Sentry Rust event" as one opaque capture path. |
| [sentry-tracing 0.48.2](https://docs.rs/sentry-tracing/0.48.2/sentry_tracing/) | `tracing` error events can become Sentry events, breadcrumbs, logs, and spans; structured fields can become context fields or tags. Spans and breadcrumbs give causal context, but they should support grouping rather than replace physical stack identity. |
| [tracing-error](https://docs.rs/tracing-error/latest/tracing_error/) | `SpanTrace` captures logical span context at the error site. It is orthogonal to a call-stack backtrace and is especially useful across async boundaries. |

### Product Decision

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

### Debuginfo Policy

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

### Capture Path Policy

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

### `rust-stack-v1` Fingerprint

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

### Normalization Rules

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

### Physical Stack Plus Logical Context

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

### Fixture Matrix

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

### Success Gates

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
[Rust stacktrace grouping ledger](rust.md); otherwise
the claim remains `not_measured`.

### Schema Additions

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

### Relationship To Other Research

- [Rust data collection and instrumentation](rust.md)
  defines why in-process Rust error capture and debuginfo are mandatory.
- [Sentry-compatible ingestion](sentry-ingest.md) defines the
  envelope/event subset and first deterministic grouping path.
- [Sentry SDK fixture compatibility gate](sentry-ingest.md)
  supplies the SDK-generated fixture harness this gate should extend.
- [Rust stacktrace grouping ledger](rust.md) turns
  this gate's fixture matrix into claim levels, result rows, refresh triggers,
  and product wording.
- [Evidence bundle and open schema](../architecture/evidence-bundle-schema.md) should expose
  `grouping_confidence`, symbolication warnings, and redaction status to agents.
- [Redaction pipeline and secret safety](redaction.md)
  still has veto power over stack locals, breadcrumbs, tags, and span fields.
- [Phase 0 telemetry overlay contract](../validation/a1-bundle-value/phase0-telemetry-overlay-contract.md)
  and [A1 eval result ledger](../validation/a1-bundle-value/a1-eval-result-ledger-and-model-refresh.md)
  define the source-field isolation policy that decides whether fixture-derived
  frame/source/context fields can appear in agent-visible bundles at all.

### Bottom Line

Rust stacktrace grouping is buildable, but only if Parallax makes it explicit:
versioned grouping, conservative Rust symbol normalization, mandatory debuginfo
policy, per-frame symbolication status, and a fixture matrix that tries to break
the grouping algorithm before users do.

Do not ship "deterministic grouping" as a claim until this gate passes and the
ledger records a fresh `rust_grouping_stable` result.

## Rust Stacktrace Grouping Ledger

_Provenance: merged verbatim from `rust-stacktrace-grouping-ledger.md` (2026-05-29 restructure)._

_(Shared note — see the markdownlint-disable directive at the top of this consolidated file.)_

Research date: 2026-05-25

### Purpose

[Rust stacktrace grouping and symbolication](rust.md)
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
[Sentry SDK compatibility ledger](sentry-ingest.md): Sentry
Rust envelopes can parse and normalize before Rust grouping is proven.

### Current Source Snapshot

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
| [Rust capture fidelity recheck](rust.md) | Current capture pass pins the broader Rust capture layer: `tracing` `0.1.44`, `tracing-error` `0.2.1`, `tracing-opentelemetry` `0.33.0`, `opentelemetry`/`opentelemetry-otlp`/`opentelemetry-appender-tracing` `0.32.0`, `anyhow` `1.0.102`, `eyre` `0.6.12`, and `color-eyre` `0.6.5`. | Grouping claims need capture-layer version rows too; a Sentry envelope alone does not prove span traces, OTLP logs, or `anyhow`/`eyre` chains. |
| [sentry-panic 0.48.2](https://docs.rs/sentry-panic/0.48.2/sentry_panic/), [sentry-backtrace 0.48.2](https://docs.rs/sentry-backtrace/0.48.2/sentry_backtrace/), and [sentry-debug-images 0.48.2](https://docs.rs/sentry-debug-images/0.48.2/sentry_debug_images/) | The subcrates separately install a panic handler, convert/process stacktraces, and attach loaded-library metadata. | Result rows must identify which capture integration produced panic, stack, and loaded-image evidence; one SDK envelope is not enough proof. |

### Claim Levels

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

### Result Artifacts

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

### Run Manifest

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

### Row Schemas

#### Fixture Matrix Row

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

#### Capture Path Result Row

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

#### Build Variant Result Row

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

#### Frame Normalization Row

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

#### Fingerprint Result Row

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

#### Symbolication Result Row

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

#### Debug File Inventory Row

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

#### Source Field Policy Audit Row

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

#### False Split / False Merge Audit Row

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

#### Claim Ledger Row

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

### Counting Rules

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

### Initial Results Template

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

### Product Wording

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

### Refresh Triggers

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

### Relationship To Other Research

- [Rust stacktrace grouping and symbolication](rust.md)
  defines the `rust-stack-v1` design this ledger measures.
- [Sentry SDK fixture compatibility gate](sentry-ingest.md)
  supplies the Rust SDK envelope fixtures extended by this ledger.
- [Sentry SDK compatibility ledger](sentry-ingest.md) consumes
  this ledger for the L3 `rust_grouping_stable` compatibility subclaim.
- [Sentry-compatible ingestion](sentry-ingest.md) defines where
  grouping sits in the ingest pipeline.
- [Rust data collection and instrumentation](rust.md)
  defines why Rust in-process capture and debuginfo policy are mandatory.
- [A6 redaction red-team ledger](redaction.md) controls
  whether stack, breadcrumb, tag, and span context can enter agent-visible
  bundles.

### Bottom Line

Rust grouping should be a measured protocol-like contract. The design is
promising, but the claim starts only when fixture rows prove that `rust-stack-v1`
is stable across rebuilds and debuginfo variants, conservative against false
merges, honest about missing symbols, and safe to expose through redacted
agent-facing bundles.
