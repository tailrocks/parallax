# Browser RUM session model — remaining P0

**Status: not shipped.** Frozen P0 #8 remaining half (vitals page exists; sessions do not).

## What exists

- `/rum` vitals catalog via `histogramQuantile(q=0.75)` (`ui/src/features/rum/api/rum-vital-stats.graphql`).
- “Journeys” are `tracesPage` rows, not session rollups (`ui/src/features/rum/api/rum-journeys.graphql`).
- GraphQL `sessions(invocationId:)` pairs CLI `session.start`/`session.end` events for **invocations**, not browser RUM (`crates/parallax-api` journeys resolver).

## Blocker

There is no `rumSessions` type, no ingest derivation of a browser session from `session.id` / `session.start` on RUM spans, and no UI inbox of sessions with duration, errors, Web Vitals, and frontend→backend traces.

Reusing CLI `sessions(invocationId)` would mix product concepts (a coding-agent/CLI run vs a page visit). That is the architectural reason this is not a one-line alias.

## What would close the P0

1. Derive a session record from browser `session.id` (or `session.start`/`session.end`) at ingest, independent of `cli.invocation.id`.
2. GraphQL `rumSessions` (time range, service, errorful) with links to traces, issues, vitals exemplars.
3. `/rum` sessions table using that API, not `tracesPage`.
4. Playground `browser:rum_error` asserting a session id joins the page trace and the backend trace.

Until then `/rum` is a Web Vitals + journey-trace projection, not a session product.
