# Sentry-Compatible Ingestion

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Executive Summary

Parallax should support the Sentry SDK ingestion surface, but it should not copy
Sentry's internal architecture.

The right first target is:

> Accept Sentry envelopes for error events, normalize the event payload into a
> Parallax-owned error model, compute deterministic grouping, and correlate the
> event with OpenTelemetry traces/logs/metrics.

This gives users the main migration benefit: existing Sentry SDKs can keep
sending production error events with minimal application changes. It also avoids
the main trap: reimplementing Relay, Kafka, Snuba, sessions, replay, profiling,
billing quotas, and the full Sentry product.

## Recommendation

Build a small Rust ingest gateway with this boundary:

```text
Sentry SDK
  -> POST /api/<project_id>/envelope/
  -> parallax-ingest
       - DSN/public-key validation
       - envelope parser
       - item allowlist
       - size limits
       - redaction
       - raw envelope retention
       - event normalization
       - deterministic grouping
       - append to local WAL or Iggy
       - write normalized error event to GreptimeDB
       - write issue/project metadata to Turso
```

Do not expose "Sentry-compatible" as a promise that every SDK feature works on
day one. Expose it as:

> Sentry-compatible error ingestion for the envelope event path, starting with
> Rust services and expanding item-by-item.

That is narrow enough to build and test.

## Current Sentry Architecture To Learn From

Sentry's architecture is built for a large product, not only error ingestion:

- Relay receives and forwards or processes events.
- Kafka buffers ingestion topics.
- Snuba consumes Kafka topics and stores/query-events in ClickHouse.
- Postgres stores relational product data.
- Redis and workers support queues and product workflows.

Sources:

- [Sentry self-hosted data flow](https://develop.sentry.dev/self-hosted/data-flow/)
- [Snuba architecture overview](https://getsentry.github.io/snuba/architecture/overview.html)
- [Sentry Relay repository](https://github.com/getsentry/relay)

Relay itself is especially important because it is written in Rust and sits
exactly where Parallax's ingest gateway would sit. But Relay is not just a small
HTTP parser. The repository describes Relay as moving functionality from SDKs
and the Sentry server into a proxy process. Its optional processing feature
normalizes, filters, rate-limits, and produces events into Kafka. Relay
integration tests require Kafka and Redis, which is a good signal that copying
Relay's processing mode would pull Parallax back toward the operational shape it
is trying to avoid.

Source:

- [Sentry Relay repository](https://github.com/getsentry/relay)

Practical conclusion:

- Use Relay as a reference design.
- Do not embed Relay as the Parallax gateway.
- Do not copy the Relay/Kafka/Snuba split unless Parallax reaches that scale.

## Compatibility Target

The first endpoint should be:

```text
POST /api/<project_id>/envelope/
```

Sentry's developer docs describe envelopes as the ingestion, forwarding, and
offline-storage format. The older store endpoint is not the right modern target
for new error ingestion.

Sources:

- [Sentry envelopes](https://develop.sentry.dev/sdk/foundations/envelopes/)
- [Sentry event payloads](https://develop.sentry.dev/sdk/foundations/envelopes/event-payloads/)
- [sentry-rust DSN type](https://docs.rs/sentry/latest/sentry/types/struct.Dsn.html)

The gateway must validate:

- project ID in the path;
- DSN/public key;
- envelope header;
- envelope item headers;
- payload size;
- item count;
- accepted item types;
- content encoding;
- timestamp sanity;
- organization/project/environment policy.

For SDK compatibility, Parallax should use real SDK-generated envelopes in
fixtures, not hand-written JSON. The focused fixture strategy is specified in
[Sentry SDK fixture compatibility gate](sentry-sdk-fixture-compatibility.md).

## Envelope Item Support

Start with a strict item policy.

| Item | v0 decision | Reason |
| --- | --- | --- |
| `event` | Support first. | Core error-event object and main compatibility value. |
| `transaction` | Parse later for correlation only. | OTLP should be the primary trace path. |
| `attachment` | Metadata-only first, bounded storage later. | Attachments are useful but dangerous for cost and secrets. |
| `session` | Drop with explicit outcome. | Release health is not in the MVP. |
| `replay_event` / `replay_recording` | Reject. | Session replay is a massive storage and privacy surface. |
| `profile` | Reject. | Profiling is later and should likely come through OTEL/profiling-specific design. |
| `logs` / `metrics` / `spans` | Prefer OTLP. | Avoid chasing Sentry-specific telemetry extensions when OTEL is the core protocol. |
| Unknown item | Store bounded raw reference and drop normalized processing. | Preserves forward-compat debugging without pretending support. |

The important compatibility behavior is not accepting every item. It is failing
predictably, returning useful status/rate-limit behavior, and preserving enough
raw data to debug SDK drift.

## Event Payload Subset

The first normalized event should preserve these Sentry concepts:

| Sentry field/interface | Parallax use |
| --- | --- |
| `event_id` | Idempotency and raw-event lookup. |
| `timestamp` | Time-window correlation. |
| `platform` | Grouping and stack frame interpretation. |
| `level` / `logger` | Event severity and source. |
| `release` / `dist` / `environment` | Regression windows and deploy correlation. |
| `transaction` | Route/operation context. |
| `exception` | Primary error type, value, mechanism, and stacktrace. |
| `threads` | Fallback stacktrace when exception stack is absent. |
| `stacktrace.frames` | Primary grouping and code navigation signal. |
| `request` | HTTP route/method and allowed request context. |
| `user` | Policy-controlled user context; redacted by default. |
| `tags` | Service, runtime, tenant, region, feature flag, and owner hints. |
| `extra` | Redacted evidence only; never blindly forwarded to agents. |
| `contexts.trace` | `trace_id` / `span_id` stitching to OTLP. |
| `breadcrumbs` | Secondary timeline evidence. |
| `debug_meta` | Build IDs / debug files for symbolication. |
| `fingerprint` | Explicit grouping override. |

Sources:

- [Sentry event payloads](https://develop.sentry.dev/sdk/foundations/envelopes/event-payloads/)
- [Sentry stack trace interface](https://develop.sentry.dev/sdk/foundations/envelopes/event-payloads/stacktrace/)

Parallax should store the raw event separately from the normalized event. The
normalized event is for stable query and grouping. The raw event is for
compatibility bugs, audits, and future parser improvements.

## Grouping And Fingerprinting

Grouping is the product primitive that makes Sentry valuable. Without grouping,
Parallax is only an event sink.

Sentry's grouping docs are useful because they show the shape of the problem:

- grouping starts during ingestion;
- events are associated with issue groups;
- stack traces are the strongest default signal;
- explicit client or server fingerprints can override default grouping;
- fallback message grouping is weaker;
- Sentry now has AI-enhanced grouping after traditional hash lookup.

Source:

- [Sentry grouping internals](https://develop.sentry.dev/backend/application-domains/grouping/)

Parallax v0 grouping should be deterministic:

```text
if event.fingerprint exists and policy allows it:
  fingerprint = hash("client", project_id, event.fingerprint)
else if exception stack has in_app frames:
  fingerprint = hash(platform, error_type, normalized top in_app frames)
else if exception stack exists:
  fingerprint = hash(platform, error_type, normalized top frames)
else if threads stack exists:
  fingerprint = hash(platform, normalized top thread frames)
else:
  fingerprint = hash(platform, error_type, normalized first message line)
```

Store:

- `fingerprint`;
- `fingerprint_source`;
- `grouping_algorithm_version`;
- normalized frame material used in the hash;
- raw evidence refs used for grouping.

AI grouping can be a second-pass suggestion, but it should not be authoritative
until the deterministic grouping path is trusted.

## Stacktrace Normalization

The hard part is not parsing JSON. The hard part is turning stacktraces into
stable issue identity and useful code context.

Rust-first normalization rules:

- prefer frames marked in-app;
- preserve crate, module, function, file, and line;
- normalize Rust symbol hash suffixes conservatively;
- preserve panic file/line as a strong grouping signal;
- store `debug_meta` and build ID if present;
- record whether line numbers were symbolicated or missing;
- capture both physical backtrace and `SpanTrace` when available;
- keep grouping versioned so future changes do not silently regroup history.

The focused Rust grouping proof gate, including the proposed `rust-stack-v1`
algorithm, debuginfo policy, symbolication status fields, and fixture matrix, is
defined in
[Rust stacktrace grouping and symbolication](rust-stacktrace-grouping-and-symbolication.md).

Harder future targets:

- JavaScript source maps;
- minified browser stacktraces;
- native crash symbolication;
- mobile debug files;
- obfuscated function names;
- platform-specific in-app frame detection.

These are real Sentry strengths. Parallax should defer them until the Rust
server-side path works.

## Relay Features To Copy

Copy the principles, not the full system.

| Relay/Sentry feature | Parallax decision |
| --- | --- |
| Forward-compatible envelope parsing | Copy. Retain bounded raw payloads and do not crash on unknown fields/items. |
| Project config cache | Copy minimally. Cache DSNs, redaction policy, limits, and environment config. |
| Data scrubbing | Copy as core behavior. Redact before storage where possible and before agent output always. |
| Rate limits | Copy simple project/category limits. |
| Outcomes | Copy lightweight dropped/accepted counters. |
| Spooling | Replace with local WAL in tiny mode or Iggy in durable mode. |
| Full quota/billing categories | Do not copy early. |
| Dynamic sampling | Defer. Useful later for high volume, not first error-context MVP. |
| Session/replay/profile processing | Reject for MVP. |
| Kafka production from Relay | Replace with local WAL/Iggy abstraction. |

Relay operating guidance also shows that even "just Relay" is not free
operationally: Sentry recommends multiple Relay instances behind a reverse proxy
for availability, monitoring, metrics, and enough memory/CPU for buffering and
forwarding. That reinforces the Parallax decision to keep the tiny deployment
inside one Parallax binary.

Source:

- [Relay operating guidelines](https://docs.sentry.dev/product/relay/operating-guidelines/)

## What Is Easy, Hard, And Overbuilt

### Easy Enough

- Accepting `POST /api/<project_id>/envelope/`.
- Parsing envelope headers and item headers.
- Supporting `event` item payloads.
- Extracting basic fields, tags, release, environment, and exception values.
- Writing normalized rows to storage.
- Returning basic success/reject responses.

### Genuinely Hard

- SDK compatibility across languages and versions.
- Stacktrace normalization across platforms.
- Rust async context quality across `await` boundaries.
- Stable grouping across releases without overgrouping unrelated bugs.
- Symbolication and debug file management.
- PII/secret scrubbing before storage and before agent exposure.
- Idempotency under retries.
- Backpressure when storage is slow.
- Rate-limit behavior that SDKs understand.
- Attachments without cost or privacy blowups.

### Overbuilt For Parallax v0

- Full Relay processing mode.
- Kafka/Snuba-compatible topic graph.
- Billing-grade quotas and outcomes.
- Session replay.
- Release health.
- Profiling.
- Frontend source map pipeline.
- Cross-language grouping parity with Sentry.
- Full Sentry UI parity.

## Idempotency And Durability

Sentry SDKs may retry. Parallax should assume duplicate envelopes.

Use:

- `event_id` as the first idempotency key for events;
- project ID + event ID unique constraint in Turso metadata;
- raw envelope hash for debugging duplicate submissions;
- append accepted raw payloads to local WAL or Iggy before normalization;
- idempotent writes to GreptimeDB keyed by event ID where possible;
- grouping updates that tolerate repeated event processing.

Accepted-but-not-yet-normalized payloads must survive process restart. That is
why the local WAL exists even before Apache Iggy.

## Backpressure And Abuse Limits

The gateway should reject early and cheaply:

- request body too large;
- too many envelope items;
- unsupported item type;
- malformed JSON;
- project not found;
- public key mismatch;
- project rate limit exceeded;
- attachment disabled;
- raw retention full;
- downstream storage unavailable and WAL full.

Do not parse expensive payloads to compute perfect accounting under abuse. Relay
docs make the same practical point in the span/transaction rate-limit design:
some payload parsing may be too expensive when rejecting bad or limited data.

Source:

- [Relay transaction and span rate limiting](https://develop.sentry.dev/ingestion/relay/transaction-span-ratelimits)

## Data Safety

Sentry events can contain highly sensitive material:

- request headers;
- cookies;
- IP addresses;
- user IDs/emails;
- query strings;
- stack locals;
- breadcrumbs;
- tags and `extra` context;
- attachments.

Parallax should default to:

- redact known secret/token/header fields before storage;
- store raw envelopes with short TTL and access controls;
- expose raw events only through a privileged read-sensitive API/MCP tool;
- keep agent bundles bounded and redacted;
- record redaction policy version on every normalized event;
- make attachments opt-in by project.

This matters more for Parallax than for a dashboard-only product because agent
context is easy to over-share.

## Implementation Plan

### Phase 1: Parser Fixtures

- Capture real envelopes from the latest Sentry Rust SDK.
- Include panic, `anyhow`, `eyre`, and `tracing`/breadcrumb examples.
- Parse envelope header, item headers, and `event` payloads.
- Store raw fixture files under a future test fixture path.

### Phase 2: Minimal Gateway

- Implement `POST /api/<project_id>/envelope/`.
- Validate DSN/public key/project mapping.
- Support only `event` item.
- Reject unsupported items with explicit outcome records.
- Append accepted payload to local WAL.

### Phase 3: Normalization And Grouping

- Normalize Rust error events into the Parallax event model.
- Compute deterministic grouping fingerprints.
- Store issue membership in Turso.
- Store normalized event rows in GreptimeDB.

### Phase 4: Correlation

- Extract `trace_id` and `span_id`.
- Join to OTLP spans/logs already in storage.
- Build first evidence bundle:
  - representative stacktrace;
  - recent same-fingerprint events;
  - same-trace logs/spans;
  - release/deploy window;
  - redaction summary.

### Phase 5: Compatibility Expansion

- Add transaction parsing if it materially improves correlation.
- Add bounded attachments only after redaction and cost policy exist.
- Add Sentry SDK fixture suites for more languages only when the Rust path is
  stable.

## Compatibility Test Matrix

This table is the short version. The full SDK-generated fixture gate is
[Sentry SDK fixture compatibility](sentry-sdk-fixture-compatibility.md).

| Test | Expected result |
| --- | --- |
| Rust panic envelope | Accepted, normalized, grouped by panic frame. |
| Rust `anyhow` error event | Accepted, source chain preserved where present. |
| Event with explicit fingerprint | Accepted, grouped by client fingerprint. |
| Event with trace context | Accepted, context bundle finds matching OTLP trace. |
| Event with missing stacktrace | Accepted, grouped by type/message fallback. |
| Event with PII-like request headers | Accepted with redaction record. |
| Unknown envelope item | Raw reference retained, item ignored/rejected by policy. |
| Oversized attachment | Rejected before storage. |
| Duplicate event ID | Idempotent no-op or duplicate marker, no new issue event. |
| Storage unavailable | Accepted only if WAL/Iggy has capacity; otherwise explicit rejection. |

## Bottom Line

Sentry compatibility is strategically valuable because it removes adoption
friction. Existing SDKs already capture the error semantics Parallax needs:
message, stacktrace, release, environment, tags, breadcrumbs, and sometimes
trace context.

But Sentry compatibility must stay scoped. Parallax should be compatible at the
protocol edge and opinionated internally:

- Sentry envelopes for error events;
- OpenTelemetry for logs/traces/metrics;
- Turso for metadata;
- GreptimeDB for observability storage;
- local WAL first, Iggy when replay and processor separation matter;
- deterministic grouping before AI grouping;
- bounded evidence bundles before autonomous PRs.

That path gives Parallax the useful part of Sentry without recreating the
self-hosted Sentry service graph.

Related: [Sentry SDK fixture compatibility gate](sentry-sdk-fixture-compatibility.md).
