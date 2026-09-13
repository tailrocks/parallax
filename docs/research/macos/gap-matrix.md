# macOS gap matrix (Workstream F, 2026-09-13)

Evidence: harness `parallax-telemetry-playground/macos/` + live Parallax
(GraphQL) unless noted. Parallax baseline `a3a240f3`.

| Workflow / capability | Parallax today | Best implementation | Product | Why better | Pri | Backend gap | UI gap | Playground coverage | Status |
|---|---|---|---|---|---|---|---|---|---|
| Native error → trace → logs | issues derived from OTLP error spans/logs, `lastTraceId` points at trace; `logsByTrace`/`traceEvents` work | error → trace → span → logs → metric chain in one view | Sentry | auto-correlation, no manual trace_id copy | P0 | none for OTLP path | cross-signal navigation pass (workstream C/D) | `failure` scenario, verified live | proven |
| Crash capture (in-process) | none (no native SDK) | Mach/signal handlers, real-time envelopes | Sentry Cocoa | exists; Parallax has no client | P1 | envelope ingest exists (Sentry compat); needs native payload mapping | issue view exists | harness proves OTLP equivalent, not SDK | specified |
| dSYM upload + server symbolication | none | dSYM upload keyed by UUID; issues show file:line; "symbols missing" banner | Sentry | exists; Parallax gap is total | P0 | upload API + DWARF resolve + re-symbolication | missing-symbols state | client half proven (dSYM build, UUID match, atos) | specified, client proven |
| OOM / watchdog truth | none | `MXCrashDiagnostic` incl. OOM | MetricKit | OS-level visibility Parallax cannot get otherwise | P1 | ingest `callStackTree` JSON from OTel `exception.stacktrace`, structured frames | crash issue view | blocked (needs app identity; probe proves 0 payloads in CLI) | specified, blocked |
| Hang diagnosis | long spans queryable; no hang workflow | hang inbox grouped by blocking frame + main-thread attribution | Sentry/Embrace | purpose-built grouping + context | P1 | hang detector/grouping on span duration + attrs | inbox | `slow-op` triple (span+log+histogram) verified | specified, signal proven |
| Release health (crash-free %, adoption) | `service.version` stored; no health surface | releases × crash-free sessions/users × adoption | Sentry | answers "which release broke it" | P0 | session math from session spans; build-UUID link | releases view for native | version flows; health missing | specified |
| Native sessions | none | sessions-as-spans + lifecycle events | Embrace | extends execution-context model to clients | P1 | session span convention + derivation | session view | lifecycle log only | specified |
| W3C backend join | `parent_span_id` honored (unit-tested); trace join proven | same | OTel standard | parity | P2 | none | none | `--parent-traceparent` test | proven |
| OTLP/HTTP+JSON ingest | protobuf only (400s JSON) | both encodings | OTel collectors | lower native client friction | P1 | JSON proto3 decode in `otlp_http` | none | N/A (harness speaks protobuf) | specified |

Dropped/observed (not Parallax gaps): local in-app symbolication — rejected
(sentry-cocoa v9 removed it as deadlock-prone); MetricKit-as-sole-source —
rejected (hours of delay, no alerting).
