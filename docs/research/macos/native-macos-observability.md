# Native macOS observability: requirements and Parallax gaps

Researched 2026-09-13 from primary sources (linked inline). Target: a macOS
developer correlates a native failure or slow operation with backend, logs,
traces, metrics, release, and surrounding context (GOAL.md §6, §19).

## 1. Signals and Apple APIs

**Errors/crashes.** Three complementary sources, none sufficient alone:

1. In-process crash reporters (Sentry Cocoa, PLCrashReporter/KSCrash lineage):
   Mach exception + signal + uncaught-NSException handlers. Real-time, but
   blind to OOM/watchdog kills and must stay async-signal-safe.
2. MetricKit diagnostics (`MXCrashDiagnostic`, incl. OOM; `MXHangDiagnostic`):
   OS truth with `callStackTree`, delivered hours later, no alerting.
3. `.ips` reports in `~/Library/Logs/DiagnosticReports`: full thread state,
   governed by ReportCrash, the offline debugging source of truth.
**Stack traces.** `Thread.callStackSymbols` (symbolicated while symbols ship),
MetricKit `callStackTree` (always unsymbolicated: `binaryUUID` +
`offsetIntoBinaryTextSegment` + `address`), `.ips` frames (`imageIndex` +
`imageOffset`, symbolicated by ReportCrash only when symbols are findable).
Server symbolication is the production path: see §4.

**Structured logs.** `os_log` / unified logging (subsystem/category, privacy
annotations, `log show` predicates) for on-device; OTLP log records with
`trace_id`/`span_id` for backend correlation. opentelemetry-swift exports OTLP
logs over gRPC (stable) and HTTP (experimental).

**Traces.** Client spans (`SPAN_KIND_CLIENT`) for `URLSession` calls with W3C
`traceparent` injection; Parallax joins them with backend spans on `trace_id`
(proven by this workstream's harness). `service.name` + `service.version`
(CFBundle) on the resource; `parent_span_id` for explicit child-join.

**Metrics.** App-level counters/histograms via OTLP (MetricKit payloads are
daily aggregates, not real-time telemetry). Hang/slow-op durations belong in a
histogram with scenario/thread attributes, exemplars pointing at traces.

**Lifecycle.** `NSApplication`/`UIScene`-phase notifications mapped to spans
(cold/warm start, foreground/background) and session IDs. Embrace maps
sessions to OTel spans; crashes to OTel logs.

**Hangs/perf.** MetricKit `MXHangDiagnostic` (main-thread stalls, delayed);
real-time watchdogs (main-thread ping) for in-app detection; `os_signpost` /
Instruments points for deep dives. Slow operations surface as long spans +
WARN logs + histogram samples — the harness proves exactly this triple.

**Release/build context.** `CFBundleShortVersionString` + `CFBundleVersion` →
`service.version`; build UUID (`LC_UUID`) → dSYM matching key; release health
(crash-free sessions/users) derived per version.

**Native attributes.** `os.type=darwin`, `os.description`, `host.arch`,
`device.model.identifier` (sysctl `hw.model`), thermal state, memory class.
Thermal/memory explain hangs that code cannot.

## 2. SDK comparison (current, Sept 2026)

| SDK | Transport | Crashes | Hangs | Symbolication | macOS | Notes |
|---|---|---|---|---|---|---|
| `opentelemetry-swift` | OTLP gRPC stable, HTTP experimental | via MetricKit mapping (`exception.stacktrace` = callStackTree JSON) | same | none (needs backend) | yes | README still marks HTTP experimental; has Persistence + MetricKit instrumentation |
| `sentry-cocoa` (v9) | Sentry envelope | in-process Mach/signal handlers | MetricKit + watchdog | server-side via dSYM upload; v9 **removed** unsafe local symbolication (deadlock #6560) | yes | reference error workflow; proven crash pipeline |
| Embrace Apple SDK | OTel-native | crash → OTel log | yes | via OTel backend | yes | sessions-as-spans; portable vendor-agnostic data |
| MetricKit (Apple) | OS-delivered payloads, ~daily | `MXCrashDiagnostic` (incl. OOM) | `MXHangDiagnostic` | none (offsets only) | 12+ | delayed hours; complements, never replaces, a reporter |

Sources: [opentelemetry-swift README](https://github.com/open-telemetry/opentelemetry-swift/blob/HEAD/README.md),
[MetricKit instrumentation](https://github.com/open-telemetry/opentelemetry-swift/blob/HEAD/Sources/Instrumentation/MetricKit/README.md),
[sentry-cocoa DECISIONS](https://github.com/getsentry/sentry-cocoa/blob/HEAD/develop-docs/DECISIONS.md)
(v9 local-symbolication removal, issue #6560),
[Embrace built-on-OTel](https://embrace.io/docs/ios/open-source/built-on-otel/),
[WWDC26 Meet the new MetricKit](https://www.youtube.com/watch?v=WygVIj420KE),
[MetricKit in production, Dec 2025](https://medium.com/@mrhotfix/metrickit-in-production-what-apple-doesnt-document-and-why-you-still-need-crashlytics-sentry-2a3c9591ed05).

Best implementation per subproblem: **Sentry** for crash capture + issue
workflow + dSYM pipeline; **OTel SDKs** for portable traces/logs/metrics;
**MetricKit** for OS-truth OOM/hang coverage; **Embrace** for the
sessions-as-spans model Parallax's execution-context story should mirror.

## 3. What Parallax must build (P0/P1)

**P0-1 — dSYM upload + server symbolication.** Missing entirely. Required:
stable upload API keyed by `(bundle_id, version, build, LC_UUID, arch)`,
storage for dSYM/DWARF, and a symbolication step that resolves
`callStackTree`/`.ips`/Sentry-frame offsets at ingest or query time.
Contract: upload returns UUID; issues show symbolicated frames with
`file:line`; re-symbolication on late dSYM arrival. This is the single
largest macOS gap vs Sentry.

**P0-2 — Crash issue pipeline.** Parallax already derives issues from OTLP
error logs/span events (proven). Still needed: crash-specific grouping
(top-frame + exception type fingerprints), crash-free session math from
session spans, and `.ips`/MetricKit diagnostic ingestion (opentelemetry-swift
emits callStackTree JSON in `exception.stacktrace` — Parallax should parse,
store structured frames, and symbolicate via P0-1).

**P0-3 — Native release context.** `service.version` flows today, but there is
no release-health surface (versions × crash-free rate × adoption) and no
link from issue → build UUID → dSYM status ("symbols missing" banner is part
of Sentry's workflow and prevents silent unsymbolicated groups).

**P1-1 — Hang workflow.** Long-span detection + `macos.hang.suspected`
attributes exist in the harness; productize: hang inbox grouped by blocking
frame, main-thread attribution, thermal/memory context auto-attached.

**P1-2 — Session model.** Adopt sessions-as-spans (Embrace pattern): session
span + lifecycle events + crash/hang links. Extends Parallax's
execution-context differentiator to native clients.

**P1-3 — OTLP/HTTP JSON.** Parallax accepts protobuf only. Apple's background
upload constraints and minimal native clients favor small JSON payloads;
supporting OTLP/HTTP+JSON lowers native SDK friction (opentelemetry-swift's
HTTP exporter remains experimental partly for this reason).

## 4. dSYM workflow (proven client half)

Proven by `parallax-telemetry-playground/macos/tools/verify.py`:

1. `dsymutil` the unstripped binary → `.dSYM`.
2. `dwarfdump --uuid` matches binary `LC_UUID` (the join key).
3. Real `fatalError` in a stripped copy → ReportCrash writes `.ips`
   (one-line JSON header + JSON body; frames: `imageIndex`+`imageOffset`).
4. `atos -o <DWARF object> -arch arm64 -l <base> <base+offset>` resolves
   `static Harness.main() (Harness.swift:58)` — the DWARF object itself, so
   the proof cannot lean on binary symbols.

Parallax's missing half: steps for upload, keyed storage, and resolving
frames inside issue grouping (§3 P0-1). No Parallax code was touched by this
workstream (file ownership); the backend workstream owns implementation.

## 5. Anti-patterns observed

- Faking native verification on Linux: rejected — every check here runs on
  macOS (see verification report).
- Local (in-app) symbolication: rejected by sentry-cocoa v9 as deadlock-prone
  and useless for stripped production binaries; Parallax must not build it.
- MetricKit as sole crash source: delays of hours+ and missing real-time
  alerting; always pair with an in-process reporter.
