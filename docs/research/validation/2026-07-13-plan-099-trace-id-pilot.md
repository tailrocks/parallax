# Plan 099 TraceId pilot evidence

Research date: 2026-07-13

## Selection

A source scan across model, API, CLI, ingest, and storage found 186 `run_id`
sites and 165 `trace_id` sites. `TraceId` was selected despite the slightly
smaller frontier because it has a protocol-defined validation contract:
OpenTelemetry carries exactly 16 non-zero bytes, while GraphQL and CLI had
accepted arbitrary text. Run IDs remain producer-defined strings and need a
separate product naming/length decision before typing.

## Pilot boundary

`parallax_model::TraceId` validates 16 non-zero OTLP bytes or 32 non-zero hex
characters, normalizes text to lowercase, and serializes transparently as the
same JSON/string representation already persisted and exposed. GraphQL trace
anchors, CLI trace inspection, and OTLP trace/log/exemplar ingress now validate
through this type. Storage rows remain strings in this pilot, so no GreptimeDB,
Turso, GraphQL SDL, or OTLP wire representation changes.

## Verification

- Parse, OTLP-byte, lowercase normalization, display, and serde compatibility
  are covered in `parallax-model`.
- All 34 API tests and the unchanged SDL snapshot pass with valid 32-hex
  fixtures.
- OTLP receiver validation rejects zero and short trace IDs before spooling.
- Strict all-target/all-feature Clippy, product policy, and facade gates pass.

## Remaining frontier

The measured primitive frontier remains 186 `run_id` and 165 `trace_id` source
sites because this is deliberately a boundary pilot, not a model-wide field
sweep. A follow-up should only be proposed after measuring defects prevented by
this boundary validation; Plan 099 does not authorize another ID migration.
