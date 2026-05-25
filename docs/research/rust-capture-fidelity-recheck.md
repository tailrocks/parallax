# Rust Capture Fidelity Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Re-check the Rust-first capture claim from current primary sources. The weak
claim is not "Rust can produce panics and traces." The weak claim is whether a
single Parallax Rust setup can reliably capture the evidence agents need:

- panic payload and location;
- physical backtrace frames with useful file/line quality;
- typed error source chain and `anyhow`/`eyre` context;
- logical `tracing` span context through `SpanTrace`;
- OTLP trace, metric, and log export;
- release, environment, build/debug image, and redaction metadata.

## Short Verdict

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

## Current Primary-Source Snapshot

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

## Minimum Rust Capture Contract

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

## Required Fixture Additions

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

## Product Wording

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

## Falsification Triggers

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

## Sources

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

## Bottom Line

Rust capture is not one feature. It is a measured contract across Sentry
features, `tracing` layers, OpenTelemetry exporters, log appenders, panic
strategy, backtrace environment, debuginfo, and redaction. That contract should
be proven before Parallax tells users their Rust failures are agent-ready.
