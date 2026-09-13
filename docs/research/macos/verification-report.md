# Verification report: macOS workstream (F), 2026-09-13

Environment: macOS 26.6.2 arm64 (Mac17,6), Swift 6.3.3, Python 3.14.
Parallax HEAD `5c64e711811e52be77b37a496bff012b9b2a1e19`,
playground HEAD `36d38266eb61f653693d663de4a1a71f896f2560`.
P0-freeze origin/main: parallax `cb9a3077`, playground `a23b0d84`.

This run: **no Parallax listener** on :4000/:14000/:4318/:14318/:14328.
Harness unit proof is the local stub. Server join is a separate step
(`python3 tools/verify.py --parallax` after `parallax serve`).

## Result: 38/38 checks green

```bash
cd parallax-telemetry-playground/macos
python3 tools/verify.py            # rc=0, "38/38 checks passed"
# log: scratch/macos-verify.log
```

`swift test`: 26/26 (OtlpProtoTests 17 + ScenarioCorrelationTests 9).

## What is proven (all on real macOS, no containers)

- Determinism: same `--seed` + `--frozen-time` → identical IDs/`session.id`.
- Stub wire: real OTLP protobuf POSTs for `failure` + `slow-op` + `lifecycle`.
  Parser asserts all three trace_ids in spans **and** logs **and** metric
  exemplars; ERROR status; `exception` event; `macos.hang.stack` event;
  span names `{macos.checkout.submit, HTTP POST, macos.report.render,
  macos.app.session, macos.app.lifecycle.cold_start}`; one shared
  `session.id`; native resource attrs (`os.type=darwin`, `os.name=macOS`,
  `device.model.identifier=Mac17,6`, `macos.build_uuid` =
  `15AF3BFD-81AB-3FFF-8A43-20B797C73244` matching `dwarfdump --uuid`);
  log severities {9,13,17}; metrics
  `{macos.playground.failures, macos.playground.operation.duration,
  macos.playground.sessions}`; 2400ms histogram sample in (1000,5000].
- Real `URLSession` POST to a local 502 echo injects
  `traceparent=00-<failure-trace-id>-<http-span-id>-01`; backend received it.
- Unsigned `.app` + Info.plist: `service.version=1.2.3 (45)`, source=`bundle`.
  CLI without plist: `0.0.0-dev+cli`.
- `os_log` roundtrip via `log show`.
- Real crash: stripped copy `fatalError` (SIGTRAP, rc=-5) → ReportCrash
  `.ips` (5 stripped-image frames, 5/5 needing dSYM) → `dsymutil` dSYM
  UUID matches binary → `atos` on the DWARF object resolves
  `static Harness.main() (Harness.swift:60)`.
- Backend join: `--parent-traceparent` unit-tested on failure, slow-op, **and**
  lifecycle (wire `parent_span_id`).

## Honestly blocked (proven, not assumed)

See also scratch `macos-blocked.md`.

- **MetricKit payloads**: `--metrickit-probe-seconds 2` from the CLI **and**
  from an ad-hoc-signed unsigned `.app` both return
  `metric_payloads=0, diagnostic_payloads=0`. Delivery needs an installed
  signed GUI app identity + Apple's aggregation window. No MetricKit
  ingestion was faked.
- **Signed GUI `.app`**: this environment has no Developer ID / notarization
  pipeline. Unsigned `.app` wrapping is as far as the CLI harness can go
  (enough for Info.plist `service.version`, not for MetricKit).
- **Parallax-side dSYM upload/symbolication**: does not exist in Parallax;
  outside this workstream's file ownership (no `crates/`/`ui/` writes).
  Client half (crash, dSYM, UUID, atos) is proven above.
- **Live Parallax ingest this run**: no server up. Stub proof stands.
  Re-run `python3 tools/verify.py --parallax` after `parallax serve` to
  exercise `trace` / `logsByTrace` / `traceEvents` / `issues` /
  `metricExemplars`.
- **SwiftUI lifecycle auto-instrumentation**: needs a real `.app` host; the
  harness emits equivalent lifecycle/hang telemetry manually.

## Observations for other workstreams (not mine to fix)

- Re-ingesting an identical trace+span can yield duplicate span rows (prior
  live run). No dedupe — backend workstream decision.
- Only one managed Parallax per host (engine ports 24000–24003 hardcoded);
  the isolated external-mode recipe in the harness README works around it.
- OTLP/HTTP rejects JSON (`invalid OTLP protobuf body`); protobuf-only is
  undocumented friction for minimal native clients.
