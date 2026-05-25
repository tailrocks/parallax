# Rust Data Collection and Instrumentation

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-24

## Executive Summary

The first product question for Parallax is not storage or UI. It is: **how do we
get enough data out of a Rust application to explain why it failed?** The answer
shapes everything downstream.

> Capture app-level errors in-process with Rust SDKs over OTLP plus a
> Sentry-compatible panic/error layer. Treat eBPF as an optional complement for
> zero-instrumentation infrastructure signal, never as the primary error path.

The reason is architectural, not a maturity gap that will close later. A Rust
panic message, an `anyhow` context chain, a typed error's source chain, span
attributes, and release/environment metadata exist only as in-process language
constructs. They are never serialized to a syscall or a network packet, so a
kernel-level probe has nothing to read. eBPF can see *that* a request returned
HTTP 500 slowly. It cannot see *why* (`panicked at 'index out of bounds'`, with
a backtrace and the order ID still in scope). Parallax's entire value is the
*why*, so the collection layer must live where the *why* exists: inside the
process.

## Why Collection Method Decides the Product

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

## Option Space

| Path | What it collects | Fit for Parallax |
| --- | --- | --- |
| In-process SDK (Sentry SDK, OpenTelemetry SDK) → OTLP | Errors, panics, typed chains, spans, attributes, release/env | Primary. Only path with app-level error semantics. |
| Sentry ingestion API (envelopes) | Error events, grouping-ready payloads | Keep as a compatibility surface so existing Sentry SDKs work unchanged. |
| OpenTelemetry API / OTLP | Traces, metrics, logs (vendor-neutral) | Primary telemetry transport; standardize on it. |
| eBPF (zero instrumentation) | RED metrics, protocol spans, service maps, CPU profiles | Complement only. Cannot capture app-level error semantics. |

## eBPF: What It Is, and What It Can and Cannot Do

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

## Rust Error Capture Mechanisms

| Mechanism | Captures | Notes |
| --- | --- | --- |
| `std::panic::set_hook` + `std::backtrace::Backtrace` | Panic message, location, frames | Frames need debuginfo; capture gated by `RUST_BACKTRACE` / `RUST_LIB_BACKTRACE` (`force_capture` ignores them). |
| `sentry` crate | Panics, backtraces, `anyhow` chains, `tracing` events; `release`/`environment` | Closest off-the-shelf match; Sentry-compatible by definition. |
| `tracing` + `tracing-error` (`SpanTrace`) | Active span context + fields at the error site | Logical context, orthogonal to a call-stack backtrace; needs an `ErrorLayer`. |
| `tracing-opentelemetry` + `opentelemetry-otlp` | Traces, metrics over OTLP (gRPC/HTTP) | Does **not** export logs — use `opentelemetry-appender-tracing` for logs. |
| `anyhow` / `eyre` / `color-eyre` | Error source chain + backtrace | `anyhow::Error` auto-captures a backtrace (Rust ≥ 1.65) unless the inner error already has one; `.context()` builds the chain; `color-eyre` adds a `SpanTrace`. |

Sources:

- [std::backtrace](https://doc.rust-lang.org/std/backtrace/index.html)
- [docs.rs/sentry](https://docs.rs/sentry/latest/sentry/)
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

## Data Model: Fields to Store

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

Related: [self-hosted-observability-architecture.md](self-hosted-observability-architecture.md).

## Debuginfo Policy (Mandatory)

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
[Rust stacktrace grouping and symbolication](rust-stacktrace-grouping-and-symbolication.md).

## How This Fits the Parallax Pipeline

```text
Rust app
  -> tracing + OTLP exporter (traces/logs/metrics)
  -> Sentry-compatible panic/error layer (error events)
  -> Parallax ingest gateway (OTLP + Sentry envelope endpoints)
  -> durable stream (Apache Iggy)
  -> processors: normalize, group, symbolicate, correlate
  -> GreptimeDB (events/logs/spans/metrics) + metadata store
  -> context API + MCP server  (agent: primary)
  -> Sentry-like UI            (human: secondary)

[optional, later] eBPF agent -> infra RED metrics, service maps, CPU profiles
```

The error path and the telemetry path share the ingest gateway and stream. eBPF,
if added, is a separate optional producer feeding infrastructure-level signal
into the same storage — not a replacement for any of the in-process capture.

## Open Questions

1. Server-side symbolication: store split debuginfo keyed by build id, or require
   line-tables in the shipped binary? What is the retention cost of debuginfo?
   See the proof-gate policy in
   [Rust stacktrace grouping and symbolication](rust-stacktrace-grouping-and-symbolication.md).
2. How exactly do we stitch Sentry-style error events to OTLP `trace_id`/
   `span_id` so an error opens directly into its trace?
3. Rust frame normalization for stable grouping across releases (hash suffixes,
   generics, monomorphization, panic location) — now specified as
   `rust-stack-v1` in
   [Rust stacktrace grouping and symbolication](rust-stacktrace-grouping-and-symbolication.md).
4. PII and secrets in span fields / error context: redaction defaults for
   self-hosted teams.
5. Async backtraces: `tracing` span context (`SpanTrace`) is often more useful
   than a physical backtrace across `.await` points — how much do we lean on it?
6. SDK ergonomics: how thin can a `parallax`-flavored Rust setup be (one init
   call wiring panic hook + tracing + OTLP) to maximize adoption?

## Bottom Line

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
