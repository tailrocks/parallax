# Sentry Envelope Item Policy Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Re-check whether Parallax's Sentry compatibility fixture scope is precise enough
after the Urgentry pass. The weak claim is not the current SDK version matrix;
it is the item-policy boundary. A lightweight competitor now proves broad Sentry
item handling can fit a small product, while current Sentry SDKs expose
different item sets across languages.

## Short Verdict

Keep Parallax v0 event-first. Do not chase broad Sentry replacement behavior.

But the parser and fixture policy must become stricter and more explicit:
Parallax cannot advertise Sentry-compatible ingestion unless unsupported
envelope items are outcome-recorded, retry-safe, non-agent-visible by default,
and unable to poison valid `event` item processing.

The important design change is this:

```text
Sentry compatibility is not "accept every item."
Sentry compatibility is "handle supported event items and produce explicit,
auditable outcomes for everything else."
```

## Current Source Snapshot

| Source | Current item-model signal | Parallax implication |
| --- | --- | --- |
| Rust `sentry` / `sentry-types` | crates.io still reports `sentry` and `sentry-types` `0.48.2`, updated 2026-05-11. `sentry-types` models `event`, `session`, `sessions`, `transaction`, `attachment`, `check_in`, `log`, and `trace_metric`; item enums are non-exhaustive. Its parser deserializes the item header into a typed enum and then parses known payloads. | Useful oracle for the Rust-first path, but too narrow and strict to be Parallax's only production parser if the desired behavior is "valid event survives unknown side item." |
| JavaScript `@sentry/core` | npm latest is `10.53.1`. The package type definitions include `client_report`, `user_report`, `feedback`, `session`, `sessions`, `transaction`, `attachment`, `event`, `profile`, `profile_chunk`, `replay_event`, `replay_recording`, `check_in`, `span`, `log`, `metric`, `trace_metric`, and `raw_security`; it also defines streamed span, log, metric, profile-chunk, replay, and raw-security envelopes. | Browser/Node smoke will include more than the Rust crate item set. Unsupported-item fixtures need JS-generated envelopes, not only Rust fixtures. |
| Go `sentry-go` | Go module proxy latest is `v0.46.2` from 2026-05-04. The internal protocol docs expose envelope item constants for `event`, `transaction`, `check_in`, `attachment`, `log`, `trace_metric`, and `client_report`; logs and metrics convert into batched envelope items. | Go smoke can hit `client_report`, log, and trace-metric behavior even if Parallax v0 ignores them. |
| Python `sentry-sdk` | PyPI latest is `2.60.0`, uploaded 2026-05-13. The Python envelope source adds `event`, `transaction`, `profile`, `profile_chunk`, `check_in`, `session`, and `sessions`; its data-category mapping also recognizes `span`, `log`, `trace_metric`, `client_report`, `profile`, and `profile_chunk`. | Python confirms profile/profile-chunk and SDK-side envelope constraints are real drift risks. |
| Urgentry | Focused recheck found source-level handling for transactions, sessions, replay, profiles, client reports, check-ins, attachments, and metric buckets. | Competitor pressure means Parallax cannot rely on "lightweight tools only store events" as a durable framing. |

## Parser Design Consequence

Do not wire a strict typed Sentry parser directly into the production ingress
path without a tolerant envelope scan in front of it.

The production behavior Parallax needs:

1. Parse envelope and item headers enough to classify item type, length, and
   payload boundaries.
2. Process supported `event` items with the typed event parser.
3. Convert known unsupported items into outcome rows.
4. Convert unknown future items into outcome rows.
5. Preserve only bounded raw references needed for compatibility audit.
6. Keep unsupported payloads out of agent-visible bundles unless a later policy
   explicitly allows a safe projection.

Typed SDK/protocol crates are still useful as fixture generators, typed payload
parsers, and test oracles. They should not define the whole production
accept/reject contract.

## Required Item Policy

| Item family | v0 parser outcome | Agent visibility | Why |
| --- | --- | --- | --- |
| `event` | Parse, normalize, group, redact, dedupe, bundle. | Redacted projection only. | Core migration value. |
| `transaction` | Record unsupported/deferred outcome; optionally retain bounded ref for trace-id correlation research. | Not raw-visible. | OTLP should be the primary trace path. |
| `attachment` | Metadata-only or unsupported-ref outcome; no raw payload in bundle. | Metadata only, if safe. | High secret and storage risk. |
| `session` / `sessions` | Unsupported/deferred release-health outcome. | No. | Release health is not v0. |
| `client_report` | Internal telemetry outcome; no retry storm. | No. | Useful for drop accounting later, not agent context now. |
| `check_in` | Unsupported/deferred monitor outcome. | No. | Cron monitor surface is not v0. |
| `user_report` / `feedback` | Deferred feedback outcome; metadata only if linked to event. | Not by default. | Human feedback may contain PII. |
| `profile` / `profile_chunk` | Unsupported/deferred profiling outcome. | No. | Expensive and privacy-sensitive. |
| `replay_event` / `replay_recording` | Unsupported/deferred replay outcome. | No. | Session replay is a separate privacy/storage program. |
| `span` / streamed span container | Unsupported/deferred Sentry-span outcome. | No. | Prefer OTLP traces and explicit OTLP conformance. |
| `log` | Unsupported/deferred Sentry-log outcome. | No. | Prefer OTLP logs for normalized evidence. |
| `metric` / `trace_metric` | Unsupported/deferred Sentry-metric outcome. | No. | Prefer OTLP metrics for normalized evidence. |
| `raw_security` | Reject or unsupported security outcome. | No. | Security reports need their own redaction and trust model. |
| `statsd` | Legacy/competitor-observed unsupported metric outcome. | No. | Not in the current JS 10.53.1 type set, but seen in adjacent Sentry-compatible systems. |
| Unknown future item | Unsupported-unknown outcome; bounded raw ref only if policy allows. | No. | Forward-compatible audit without false support. |

## Response Semantics

The fixture suite should force three distinct response cases:

| Case | Expected behavior |
| --- | --- |
| Supported `event` plus unsupported side items | Process the event; emit unsupported-item outcomes; do not retry the whole envelope solely because side items are unsupported. |
| Unsupported-only envelope from a known SDK | Return a retry-safe accept/drop status and write outcome rows if authenticated; do not create an issue event. |
| Malformed, unauthenticated, oversized, or boundary-invalid envelope | Return a hard error before parsing expensive payloads; no agent-visible raw refs. |

Exact HTTP status can be decided during implementation, but the ledger must
record whether the response is retry-safe and whether SDK retry behavior was
observed for the fixture.

## Fixture Additions

Add a cross-SDK unsupported-item fixture subset before claiming even
`sentry_sdk_compatible_error_ingest`:

| Fixture | Source SDK | Must prove |
| --- | --- | --- |
| `rust_event_plus_log_trace_metric` | Rust `sentry-types`/Rust SDK fixture generator | Current Rust modeled containers do not poison event processing. |
| `go_event_plus_client_report` | `sentry-go` | `client_report` is outcome-recorded and not agent-visible. |
| `js_event_plus_profile_replay_span_log_metric` | `@sentry/core` or JS SDK fixture app | JS-only broader item families are classified and safely dropped/deferred. |
| `python_profile_chunk_checkin_session` | `sentry-sdk` | Python profile/profile-chunk/check-in/session item behavior is outcome-recorded. |
| `unknown_future_item_with_event` | synthetic envelope | Unknown type does not poison valid event processing. |
| `unsupported_only_envelope` | synthetic plus one real SDK where possible | Retry-safe drop/accept behavior does not create an issue. |

These are not broad product-support promises. They are safety fixtures for the
event-first migration promise.

## What Would Falsify The Current Scope

Reopen the v0 compatibility target if:

- real Rust SDK error fixtures routinely include unsupported side items that
  cannot be ignored without losing important error context;
- browser or Go migration demand appears before Rust proves value;
- A1 bundle evaluation shows Sentry logs/spans/profiles in envelopes materially
  improve fixes compared with OTLP evidence;
- unsupported-only envelopes trigger SDK retry loops under the planned response
  semantics;
- a competitor publishes open, fixture-proven broad Sentry item handling plus
  evidence bundles and action/outcome audit.

## Sources

- [Sentry envelopes docs](https://develop.sentry.dev/sdk/foundations/envelopes/)
- [Sentry envelope items docs](https://develop.sentry.dev/sdk/foundations/envelopes/envelope-items/)
- [sentry-types source on docs.rs](https://docs.rs/sentry-types/latest/src/sentry_types/protocol/envelope.rs.html)
- [sentry-types crate](https://crates.io/crates/sentry-types)
- [sentry crate](https://crates.io/crates/sentry)
- [`@sentry/core` npm package](https://www.npmjs.com/package/@sentry/core)
- [`sentry-go` module](https://pkg.go.dev/github.com/getsentry/sentry-go@v0.46.2/internal/protocol)
- [sentry-python envelope source](https://getsentry.github.io/sentry-python/_modules/sentry_sdk/envelope.html)
- [Urgentry Sentry Tiny Benchmark Recheck](urgentry-sentry-tiny-benchmark-recheck.md)

## Bottom Line

The honest v0 claim is still narrow: Sentry SDK error-event ingestion, starting
with Rust. The fixture suite must get broader than the product scope so that
unsupported Sentry item families are safe, audited, and non-agent-visible by
default.
