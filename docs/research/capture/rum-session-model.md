# Browser RUM session model — shipped (R3, 2026-09-14)

**Status: shipped.** Frozen P0 #8 closed: sessions are first-class entities.

## What exists

- GraphQL `rumSessions(service, fromNanos, toNanos, errorOnly, limit)` and
  `rumSession(sessionId, limit)` (`crates/parallax-api/src/resolvers/rum.rs`).
- Sessions derive from spans grouped by `session.id` — independent of
  `cli.invocation.id` — via shared pure projections
  (`crates/parallax-storage/src/projections.rs`) implemented by both the
  in-memory and GreptimeDB adapters (`RumSessionStore` trait).
- Ingest path: existing browser OTLP payloads already land `session.id` on
  `SpanRow.session_id` (resource attribute, `crates/parallax-ingest`); no
  client change was needed. Contract locked by
  `normalize_traces_preserves_rum_session_payloads`.
- `/rum` sessions inbox + session timeline of page views (`app.screen.name`
  spans), vitals (`browser.web_vital` spans), and errors (ERROR-status spans),
  each linked to its trace (`ui/src/features/rum/`).
- Live proof: `crates/parallax-server/tests/rum_sessions_greptime.rs`
  (managed GreptimeDB: ingest session payloads, assert entity + timeline).

## Deliberate non-goals (still open)

- Browser `session.start`/`session.end` events: not emitted; session end is
  last activity, not an explicit close.
- Sentry session aggregates → RUM sessions join (release-health territory).
- Per-span `session.id` on non-root spans is not resolved (group-level
  root-or-resource resolution, same as `cli.invocation.id`).
- Vitals-as-metrics (`metricCatalog` path) carry no session link; the session
  timeline uses vitals-as-spans only.
- Playground `browser:rum_error` session-join assertion (playground-owned).
