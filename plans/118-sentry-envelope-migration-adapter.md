# Plan 118: Add a bounded Sentry envelope migration adapter

> **Executor instructions**: This is a compatibility adapter, not a second
> telemetry architecture. Do not begin until the operator opens the scope and
> evidence shows that Sentry migration is the next adoption constraint. Preserve
> OTLP as the primary ingest contract, GreptimeDB + Turso as the only product
> stores, native raw-signal tables, native TLS, and the canonical redacted bundle.

## Status

- **Priority**: P3
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 093, 099, 104, 111, 116
- **Category**: future compatibility / ingest / security
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: IN PROGRESS — ingest-path slice landed 2026-07-17
- **Blocker**: none for local adapter path. Residual: real sanitized SDK
  fixtures (compatibility claim), idempotent event-id collision handling,
  full redaction/bundle gates, retention/doctor coverage.

## Why

The research corpus contains a detailed Sentry envelope design and compatibility
matrix, but leaving its five implementation phases in a capture note made that
note a second executable backlog. The adapter remains a plausible migration
surface for Rust services already using Sentry SDKs. It must be isolated behind
an explicit product decision because envelope parsing, DSN mapping, attachment
handling, raw retention, redaction, and cross-signal identity materially expand
the attack surface.

## Current Evidence

- The operator unblock directive opened Sentry-compatible ingest. The
  preliminary event-only subset, bounds, mapping, durable acknowledgement,
  outcomes, raw-retention, and fixture gates are fixed in
  [`sentry-envelope-adapter.md`](../docs/research/decisions/sentry-envelope-adapter.md).
  The executor must verify and extend that shape with real current SDK fixtures;
  no compatibility claim is proven yet.
- **2026-07-17 parser slice**: pure framing parser in
  `crates/parallax-ingest/src/sentry_envelope.rs` with contract limits
  (1 MiB envelope, 8 KiB headers, 16 items, 768 KiB event, one event item).
  Unit tests cover accept+unsupported side item, duplicate event reject,
  no-event reject, premature EOF, trailing garbage, implicit length, size
  ceiling, dashed event-id normalize. Hand-crafted protocol fixtures only —
  **not** a compatibility claim.
- **2026-07-17 ingest-path slice**:
  - `ErrorSource::SentryEnvelope` on the shared error model.
  - `parallax_analysis::sentry::derive_from_sentry_event` maps exception/
    message/stack/trace/tags into `ErrorEventRow` with the same fingerprint
    function as OTLP derivation; sensitive tags dropped; explicit fingerprint
    changes grouping.
  - `Signal::Sentry` spool lane stores the **normalized** accept record (not
    the raw envelope).
  - `POST /api/<project_id>/envelope[/]` on the API router: gzip via
    tower-http, project_id + `X-Sentry-Auth`/`Authorization: Sentry`
    public-key mapping from `[sentry]` config / `PARALLAX_SENTRY_PUBLIC_KEY`,
    durable spool then worker enqueue, typed HTTP outcomes (400/401/404/413/
    415/503). Disabled by default (`[sentry] enabled = false`).
  - Worker `IngestItem::Sentry` records issues + `write_error_events` only —
    no custom Greptime raw-signal table.
  - Real sanitized `sentry` 0.48.x SDK fixtures remain required before any
    product compatibility claim.
- `docs/research/capture/sentry-ingest.md` records the envelope framing,
  Sentry-Rust event shapes, compatibility levels, and source-field constraints.
- OTLP HTTP/gRPC is the shipped primary ingest contract.
- The spool, normalized error model, Turso issue membership, GreptimeDB native
  tables, and canonical bundle projection already provide the reusable product
  boundaries; the adapter must not create parallel ones.
- Plans 099, 104, 111, and 116 own typed/idempotent ingest behavior, the bundle
  contract, fail-closed redaction, and retention/prune policy respectively.

## Scope

In scope after the trigger clears:

- A versioned decision that fixes the supported Sentry SDK/version/item subset,
  project/DSN mapping, outcome semantics, size limits, retention, and claim level.
- Real envelope fixtures from the latest stable Sentry Rust SDK for panic,
  `anyhow`, `eyre`, tracing/breadcrumb, explicit fingerprint, trace context,
  missing stack, PII-shaped header, duplicate ID, unsupported item, and oversized
  attachment cases.
- A bounded `POST /api/<project_id>/envelope/` gateway supporting `event` first,
  authenticating the public key/project mapping and recording explicit outcomes.
- Durable acceptance through the existing spool contract before acknowledgement.
- Normalization into the existing error/event identity model, issue membership in
  Turso, and correlation to GreptimeDB-native OTLP spans/logs by trace/span ID.
- Canonical bounded bundle output with source-field status, redaction report,
  stable hash, projection manifest, and approved raw references.
- Conditional transaction, attachment, and additional-language expansion only
  after measured correlation value and all preceding gates pass.

Out of scope:

- Full Sentry API, Relay, Discover, performance, replay, profile, or SDK parity.
- A second issue model, alternate metadata/telemetry engine, custom raw OTLP
  table, or engine-substitution abstraction.
- Agent access to raw envelopes or unredacted GraphQL/SSE surfaces.
- Attachments before explicit redaction, cost, size, retention, and access policy.
- Product MCP work, which remains independently gated by plan 112.

## Steps

1. Reproduce the trigger. Record the operator decision and source-linked demand
   evidence showing that this migration path outranks other adoption work. If it
   does not, refresh the blocker and stop.
2. Recheck the latest stable Sentry protocol and Rust SDK. Write the supported
   subset/claim-level decision before code, including envelope/item limits,
   authentication, outcomes, duplicate IDs, retryability, raw-reference policy,
   and unsupported-item behavior.
3. Generate and inspect real SDK envelopes. Commit sanitized fixtures plus
   expected parse/normalization/outcome records; never commit live credentials,
   customer data, or provider-shaped secrets.
4. Add the minimal endpoint through the existing receiver/spool boundaries.
   Authenticate project mapping, bound bytes/items, accept only the approved
   item subset, acknowledge only durable acceptance, and expose typed retryable
   versus terminal outcomes.
5. Normalize accepted events into the existing domain model and idempotency
   contract. Store mutable issue membership only in Turso; use approved derived
   extension rows and GreptimeDB native telemetry reads without inventing a raw
   Sentry signal table.
6. Correlate trace/span identifiers to native OTLP evidence and build the same
   bounded, fail-closed-redacted bundle used by CLI/HTTP. Verify stable identity
   when the same failure arrives through both OTLP and Sentry channels.
7. Measure whether transactions, bounded attachments, or another SDK language
   improve correlation enough to justify their cost. Add each only through a
   separately fixture-gated contract update; otherwise leave it unsupported.

## Test Plan

- Parser golden tests for valid, truncated, malformed, multi-item, compressed,
  oversized, and unknown-item envelopes from real sanitized SDK fixtures.
- Authentication/project mapping, request limit, explicit outcome, retry, and
  durable-acknowledgement integration tests.
- Normalization/grouping parity for panic, error chains, explicit fingerprint,
  missing stack, trace context, duplicate event ID, and cross-source OTLP echoes.
- Real GreptimeDB + Turso tests proving native-table correlation, issue updates,
  retention/prune behavior, and no custom raw-signal table.
- Seeded PII/secret tests proving fail-closed redaction before every bundle
  projection; raw envelopes remain inaccessible to agents.
- CLI/HTTP bundle hash/projection compatibility tests and bounded-query evidence.

## Done Criteria

- [x] (2026-07-17) Operator decision + demand packet in decision record; local
  adapter path opened.
- [ ] The supported SDK/protocol/item claim is explicit and fixture-backed
  (real sanitized `sentry` SDK envelopes still required).
- [x] (partial, 2026-07-17) Accepted requests are bounded, public-key mapped,
  durably spooled (normalized record), and return typed outcomes; malformed/
  unsupported inputs fail predictably. Residual: collision `409`, exact-
  duplicate idempotency ledger.
- [ ] Duplicate and cross-source events preserve one stable issue/occurrence
  contract without replaying completed effects.
- [x] (2026-07-17) Turso owns mutable issue state via existing
  `upsert_issue_occurrences`; no custom Greptime raw Sentry table; OTLP native
  tables remain the raw observability source.
- [ ] Every agent-visible projection is canonical, bounded, and fail-closed
  redacted, with stable hashes and approved raw-reference behavior.
- [ ] Real-engine, spool, retention, strict Rust, nextest, and API gates pass
  end-to-end with live Greptime+Turso for the Sentry path.
- [ ] Any expanded transaction/attachment/language support has its own measured
  value, limits, fixtures, and security evidence.

## STOP Conditions

- The operator has not explicitly opened Sentry-compatible ingest or the demand
  evidence does not justify it as the next adoption path.
- Implementation requires weakening OTLP-first, GreptimeDB + Turso, native-table,
  native-TLS, redaction, retention, or agent-access policy.
- The endpoint cannot acknowledge only after bounded durable acceptance.
- Current protocol/SDK behavior cannot be established from primary sources and
  sanitized real fixtures.
- Cross-source identity or duplicate-event semantics remain ambiguous.

## Remove When

Delete this plan and index row when the approved bounded adapter is shipped with
all compatibility/security evidence, or when an explicit durable decision rejects
the adapter and no actionable migration work remains.
