# Sentry envelope adapter contract

**Status:** implemented; compatibility wording remains fixture-version bounded
**Decision date:** 2026-07-17  
**Approver:** operator unblock directive (`plans/README.md`)  
**Owner:** closed — plan 118 DONE (2026-07-17)

## Decision

Parallax implements one migration endpoint:

```text
POST /api/<project_id>/envelope/
```

It is a bounded adapter into the existing durable ingest, error identity,
GreptimeDB-native telemetry, Turso issue state, redaction, retention, and
bundle-v2 boundaries. OTLP remains primary. No Relay-compatible forwarding,
parallel raw-signal table, or second issue model is introduced.

The shipped claim is **Sentry-compatible Rust error-event ingestion**, verified
against sanitized envelopes emitted by the latest stable `sentry` Rust SDK.
The 2026-07-17 documentation recheck found `sentry 0.48.5`; execution must
refresh that version before generating fixtures.

## Accepted subset

- Exactly one `event` item is normalized per envelope. The parser tolerates
  other well-framed current or unknown item types, records an explicit
  non-agent-visible `unsupported_item` outcome for each, and discards their
  payloads. Unsupported side items never poison an otherwise valid event.
- Event payload coverage starts with panic, exception/error chain, message,
  level, platform, release, environment, tags, breadcrumbs, explicit
  fingerprint, stacktrace, request/source context after redaction, and trace/
  span identifiers.
- `transaction`, attachment, replay, profile, session, client report,
  check-in, user report, and every other item type are unsupported. Expansion
  requires a separately measured, fixture-backed contract update.
- Item payloads use explicit byte `length`. The parser still recognizes the
  protocol's implicit-newline form so real SDK drift produces a typed outcome,
  not an unbounded read.
- Identity and gzip HTTP content encoding are accepted. Any other content
  encoding is terminally unsupported in the first contract.

## Bounds and parsing

Limits apply while streaming, before allocation or decompression completes:

| Surface | Limit |
| --- | ---: |
| Compressed request body | 1 MiB |
| Decompressed envelope | 1 MiB |
| Envelope header line | 8 KiB |
| Item header line | 8 KiB |
| Items | 16 |
| Accepted `event` items | 1 |
| Event payload | 768 KiB |
| Nesting depth | 64 |

Length overflow, premature EOF, a non-newline byte after a length-prefixed
payload, duplicate event items, invalid JSON, decompression overflow, and
trailing partial frames are malformed. Parsing is single-pass and does not
retain borrowed request buffers after normalization.

## Project mapping and acknowledgement

The path project ID, DSN project ID, and registered public key must resolve to
one server-owned project context. A Sentry public key is a routing credential,
not a secret or user authentication mechanism. Remote exposure therefore also
requires the shipped plan 109 bearer authentication contract; local-only exposure may use the
server-assigned local operator context.

Return success only after the accepted normalized record is durably appended
to the existing spool. The response event ID equals the validated 32-hex SDK
event ID. Replays of the same `(project, event_id, canonical payload hash)` are
idempotent; reuse of an event ID with different canonical content is a terminal
collision.

| Class | HTTP behavior | Retry |
| --- | --- | --- |
| Durable accept or exact duplicate | `200` + event ID | no |
| Malformed envelope/event | `400` | no |
| Unknown project/public-key mapping | `401` | no |
| Event-ID collision | `409` | no |
| Compressed/decompressed/field limit | `413` | no |
| Unsupported content encoding or no event item | `415` | no |
| Capacity/rate limit | `429` + `Retry-After` | yes |
| Spool unavailable/durability uncertain | `503` + `Retry-After` | yes |

Unsupported side items are reported through internal outcomes but do not
change a durable event acceptance response.

## Storage, identity, and access

- Raw envelopes and unsupported item payloads are not durable product data.
  Only the bounded normalized record enters the spool.
- Mutable issue membership stays in Turso. Raw observability correlation reads
  GreptimeDB native logs/traces/metrics; no Sentry raw table is allowed.
- Sentry duplicates use event ID. Cross-source OTLP echoes share the existing
  normalized issue fingerprint and correlation links; the adapter must not
  manufacture equality from message text alone.
- Every agent-visible projection passes canonical fail-closed redaction and
  bundle-v2 validation. No raw envelope reference is exposed in the first
  contract.

## Required fixture gate

Compatibility maintenance generates real sanitized Rust SDK fixtures
for every Plan 118 case and capture exact bytes plus expected parse,
normalization, outcome, identity, and redaction records. Negative fixtures must
cover both compressed and decompressed limits, all frame truncation points,
invalid lengths/newline termination, duplicate events, unknown items, PII/
secret canaries, duplicate IDs, and collision IDs.

This document fixes the shipped bounded contract. Compatibility remains only as
broad as current primary-source and live fixture evidence; failing new fixtures
must narrow the claim or fix the adapter, never weaken the gate.

## Primary sources

- [Sentry envelope format](https://develop.sentry.dev/sdk/foundations/envelopes/)
- [Envelope items](https://develop.sentry.dev/sdk/foundations/envelopes/envelope-items/)
- [Event payloads](https://develop.sentry.dev/sdk/foundations/envelopes/event-payloads/)
- [Sentry Rust `Envelope`](https://docs.rs/sentry/latest/sentry/protocol/struct.Envelope.html)
- [Sentry Rust `EnvelopeItem`](https://docs.rs/sentry/latest/sentry/protocol/enum.EnvelopeItem.html)
- [Sentry Rust transports](https://docs.rs/sentry/latest/sentry/transports/)
