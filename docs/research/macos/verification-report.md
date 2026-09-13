# Verification report: macOS workstream (F), 2026-09-13

Environment: macOS 26.6.2 arm64 (Mac17,6), Swift 6.3.3, Python 3.14.
Baselines: parallax `a3a240f30c935021f243bb47f1c38d277c9d8a81`,
playground `f05c8838a567968fc2ab9e18d307bfebbd60c121`.
Live leg ran against the repos' current-debug `parallax` + GreptimeDB 1.1.2
in an isolated external-mode stack (API :14000, OTLP/HTTP :14328, engine
:24100–24103) so the sibling lab on :4000/:14318 was untouched.

## Result: 30/30 checks green

```bash
cd parallax-telemetry-playground/macos
python3 tools/verify.py --parallax   # rc=0, "30/30 checks passed"
```

Full log: `/tmp/verify_full3.log` (this host; rerun to reproduce).

## What is proven (all on real macOS, no containers)

- `swift build` + `swift test`: 9/9, incl. OTLP field-number regression test
  (events=11, Status.code=3/message=2, LogRecord.trace_id=9/span_id=10 —
  three real wire bugs the lab's Parallax 400s caught during development).
- Determinism: same `--seed` + `--frozen-time` → identical payloads/IDs.
- Stub wire proof: real OTLP protobuf POSTs; dependency-free parser asserts
  both scenario trace_ids in spans, ERROR status, `exception` event,
  native resource attrs (`os.type=darwin`, `device.model.identifier=Mac17,6`,
  thermal), log severities {9,13,17} sharing trace_ids, both metric names,
  and the 2400ms histogram sample in the (1000,5000] bucket.
- `os_log` roundtrip: marker emitted via `Logger`, read back with `log show`.
- Real crash: stripped copy `fatalError` (SIGTRAP, rc=-5) → ReportCrash
  `.ips` (header + body JSON, 5 stripped-image frames, 5/5 needing dSYM) →
  `dsymutil` dSYM whose UUID matches the binary → `atos` on the DWARF object
  resolves `static Harness.main() (Harness.swift:58)`.
- Live Parallax: ingest 200 on all signals; `trace()` returns
  `macos.checkout.submit`; `logsByTrace` correlated; `traceEvents` holds the
  `exception`; derived `CheckoutDeclined` issues point `lastTraceId` at the
  macOS trace.
- Backend join: `--parent-traceparent` makes the client span a child of a
  backend trace (unit-tested wire bytes incl. `parent_span_id`).

## Honestly blocked (proven, not assumed)

- **MetricKit payloads**: `--metrickit-probe-seconds 5` subscribes cleanly
  and receives 0 metric / 0 diagnostic payloads. Delivery needs an installed
  GUI app identity + Apple's aggregation window. No MetricKit ingestion was
  faked.
- **Parallax-side dSYM upload/symbolication**: does not exist in Parallax;
  outside this workstream's file ownership (no `crates/`/`ui/` writes).
  Client half (crash, dSYM, UUID, atos) is proven above.
- **SwiftUI lifecycle auto-instrumentation**: needs a real `.app` host; the
  harness emits equivalent lifecycle/hang telemetry manually.

## Observations for other workstreams (not mine to fix)

- Re-ingesting an identical trace+span yields duplicate span rows (observed
  8 logs / 4 exception events for one trace_id after repeated emits). No
  dedupe — backend workstream decision.
- Only one managed Parallax per host (engine ports 24000–24003 hardcoded);
  the isolated external-mode recipe in the harness README works around it.
- OTLP/HTTP rejects JSON (`invalid OTLP protobuf body`); protobuf-only is
  undocumented friction for minimal native clients.
