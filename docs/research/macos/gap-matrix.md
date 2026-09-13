# macOS gap matrix (Workstream F, 2026-09-13)

Evidence: harness `parallax-telemetry-playground/macos/` (`tools/verify.py`
38/38 on this host). Live Parallax GraphQL is a **separate step** — this run
had no listener on :4000/:14000; stub parse is the harness unit proof.
Parallax HEAD at verify time `5c64e711`; playground `36d38266`.

| Workflow / capability | Parallax today | Best implementation | Product | Why better | Pri | Backend gap | UI gap | Playground coverage | Status |
|---|---|---|---|---|---|---|---|---|---|
| Native error → trace → logs → metrics | issues derived from OTLP error spans/logs, `lastTraceId` points at trace; `logsByTrace`/`traceEvents`/`metricExemplars` exist | error → trace → span → logs → metric chain in one view | Sentry Cocoa 9.28.0 | auto-correlation, no manual trace_id copy | P0 | none for OTLP path | cross-signal navigation (workstream C/D) | `failure`: INTERNAL submit + CLIENT `HTTP POST` child, exception+real stack on span **and** ERROR log, failure counter + duration histogram **exemplars carry the same trace_id**, `session.id` | proven (stub); live GraphQL when server up |
| Crash capture (in-process) | none (no native SDK) | Mach/signal handlers, real-time envelopes | Sentry Cocoa 9.28.0 | exists; Parallax has no client | P1 | envelope ingest exists (Sentry compat); needs native payload mapping | issue view exists | harness proves OTLP equivalent + out-of-process `.ips`; not an SDK | specified |
| dSYM upload + server symbolication | none | dSYM upload keyed by UUID; issues show file:line; "symbols missing" banner | Sentry Cocoa 9.28.0 | exists; Parallax gap is total | P0 | upload API + DWARF resolve + re-symbolication | missing-symbols state | client half proven: `dsymutil`, UUID match vs `macos.build_uuid`/`LC_UUID`, `atos` on DWARF → `Harness.main()` | specified, client proven |
| OOM / watchdog truth | none | `MXCrashDiagnostic` incl. OOM | MetricKit | OS-level visibility Parallax cannot get otherwise | P1 | ingest `callStackTree` JSON from OTel `exception.stacktrace`, structured frames | crash issue view | blocked: MetricKit CLI probe **and** unsigned `.app` probe both 0/0 payloads | specified, blocked |
| Hang diagnosis | long spans queryable; no hang workflow | hang inbox grouped by blocking frame + main-thread attribution | Sentry/Embrace | purpose-built grouping + context | P1 | hang detector/grouping on span duration + attrs | inbox | `slow-op`: 2.4s span + WARN log + `macos.hang.stack` event (real `Thread.callStackSymbols`) + histogram exemplar + thermal/low-power attrs | specified, signal proven |
| Release health (crash-free %, adoption) | `service.version` stored; no health surface | releases × crash-free sessions/users × adoption | Sentry | answers "which release broke it" | P0 | session math from session spans; build-UUID link | releases view for native | CLI fallback `0.0.0-dev+cli`; unsigned `.app` Info.plist → `1.2.3 (45)` from bundle; `macos.build_uuid` = Mach-O `LC_UUID` on every signal | specified (version+UUID flow proven; health math missing) |
| Native sessions | none | sessions-as-spans + lifecycle events | Embrace | extends execution-context model to clients | P1 | session span convention + derivation | session view | `lifecycle`: `macos.app.session` + cold-start child + `lifecycle.cold_start`/`foreground` events; **same `session.id`** as failure/slow-op; span **links** from ops → session | specified, signal proven |
| W3C backend join | `parent_span_id` honored | same | OTel standard | parity | P2 | none | none | `--parent-traceparent` on all three scenarios; real `URLSession` injects `traceparent` sharing failure `trace_id` (local 502 echo) | proven |
| Metric exemplar → trace | `metricExemplars` GraphQL exists | exemplar dots on every chart | Grafana / Parallax | tie once producers emit | P1 | none | keep | histogram field 8 + sum field 5 exemplars on failure, slow-op, lifecycle | proven (stub wire); live query when server up |
| Native attributes | resource attrs stored | device/os/thermal/memory on every signal | Sentry Cocoa / OTel semconv | hang/error context without extra hops | P2 | none | surface on issue/trace | `os.type=darwin`, `os.name=macOS`, `device.model.identifier`, thermal, low-power, physical+resident memory, CPU count, `telemetry.sdk.language=swift` | proven |
| OTLP/HTTP+JSON ingest | protobuf only (400s JSON) | both encodings | OTel collectors | lower native client friction | P1 | JSON proto3 decode in `otlp_http` | none | N/A (harness speaks protobuf) | specified |

Dropped/observed (not Parallax gaps): local in-app symbolication — rejected
(sentry-cocoa v9 removed it as deadlock-prone, still true as of 9.28.0);
MetricKit-as-sole-source — rejected (hours of delay, no alerting);
faking MetricKit / signed GUI `.app` from this CLI harness — rejected
(probe proves 0 payloads; see verification-report).
