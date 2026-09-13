# Sentry SDK compatibility matrix

**Status: proven 2026-09-13 for 3 SDK legs; everything else unsupported or untested.**
SoT for "which Sentry SDK emits what Parallax understands". No aspirational checkmarks:
every ✅ below was observed live on the cited mains; ❌ was observed dropped/rejected
live or is absent from the cited code path; ➖ was not exercised.

## Proof run

- Parallax `824f0053` (origin/main, unmodified) + playground `26bb6641` + R4 bun-path fix.
- Rust + Java legs were green on unmodified mains; the JS leg needed the R4 fix
  (`run_bun_script_with_env` still pointed at the pre-`f05c883` root path
  `scenarios/c8-emit-js.ts`; the script lives at `web/scenarios/c8-emit-js.ts`).
- Operator-enabled serve profile (adapter stays fail-closed by default):

```toml
[sentry]
enabled = true
project_id = "1"
public_key = "c8public"   # or PARALLAX_SENTRY_PUBLIC_KEY
```

```bash
SENTRY_DSN="http://c8public@127.0.0.1:<api_port>/1" PARALLAX_URL="http://127.0.0.1:<api_port>" \
  cargo run --locked -p playground-cli -- scenario sentry:envelopes
# Sentry Rust/Java/JavaScript envelopes verified
```

## SDK legs (fp-v1 fingerprints, stable across hosts)

| SDK | Exact version (pin) | Emitter | Observed `(service, fingerprint, title)` |
| --- | --- | --- | --- |
| Rust `sentry` | 0.49.1 (`Cargo.lock`) | `libs/playground-telemetry/examples/c8_sentry_emit.rs` | `(…, e13e392d67589caf, "error: c8-rust-sdk PaymentError")` |
| Java `io.sentry:sentry-spring-boot-4-starter` | 8.53.0 (`services/catalog/build.gradle.kts`) | `C8SentryEmit.java` via `c8SentryEmit` | `(…, 05bbbb108b55e416, "IllegalStateException: c8-java-sdk PaymentError")` |
| JS `@sentry/node` | 10.70.0 (`web/bun.lock`) | `web/scenarios/c8-emit-js.ts` | `(…, 3fd4f904a2d731a5, "Error: c8-js-sdk PaymentError")` |

`service` is host-dependent (`server_name` fallback) and omitted. Re-running a leg
re-groups into the same fingerprint (observed `eventCount: 2`).

## Feature matrix

Code paths: envelope parse `crates/parallax-ingest/src/sentry_envelope.rs`,
derive `crates/parallax-analysis/src/sentry.rs`, HTTP `crates/parallax-server/src/sentry_http.rs`.

| Envelope feature | Rust 0.49.1 | Java 8.53.0 | JS 10.70.0 | Notes |
| --- | --- | --- | --- | --- |
| error event (`type=event`, message) | ✅ | ✅ | ✅ | ingest→issue live |
| exception + stacktrace | ➖ | ✅ | ✅ | Rust leg sends `capture_message` only (`attach_stacktrace` lands in `threads`, which derive ignores); Java 1 frame, JS 3 frames observed |
| `threads` stacktrace fallback | ❌ | ❌ | ❌ | derive reads `exception.values[0].stacktrace` only |
| exception chains (values[1..]) | ❌ | ❌ | ❌ | only `values[0]` read |
| release / environment | ✅ | ✅ | ✅ | → `serviceVersion` / `environment` |
| tags | ✅ | ➖ | ✅ | `c8.sdk` tag stored (`sentry.tags`); Java emitter sends no tags (same code path) |
| explicit `fingerprint` | ✅ | ✅ | ✅ | part of grouping; regroup observed |
| `contexts.trace` → trace/span link | ✅ | ✅ | ✅ | SDKs emit trace context even with sample rate 0 |
| breadcrumbs | ❌ | ❌ | ❌ | dropped at derive (not read, not stored) |
| `user` | ❌ | ❌ | ❌ | dropped by design (no user PII); Java leg sets `user.id=c8-java`, absent from stored row |
| `request` / `extra` / other `contexts` | ❌ | ❌ | ❌ | not read |
| attachments (side item) | ❌ | ❌ | ❌ | skipped, envelope still 200 when an event is present (live: `{"id":…}`) |
| sessions | ❌ | ❌ | ❌ | session-only envelope → `415 no_event_item` (live) |
| transactions | ❌ | ❌ | ❌ | transaction-only envelope → `415 no_event_item` (live) |
| profiles / replays / client reports / check-ins | ❌ | ❌ | ❌ | any non-`event` item skipped; envelope without `event` → 415 |

## Auth + framing contract (live)

- `X-Sentry-Auth` / `Authorization: Sentry …` header with `sentry_key` required;
  DSN query-string key is **rejected** (JS SDK 10 puts the key in the query string,
  so the leg injects the header explicitly — see `c8-emit-js.ts`).
- Trailing slash optional; gzip body accepted; other content-encodings → 415.
- Unknown project id or key → 401; adapter disabled → 404.

## Remainder (not this slice)

- Fixture-hash asserted test per SDK (POST a recorded envelope, assert exact
  `(service, fingerprint)`); fingerprints above are live-observed, not test-pinned.
- More SDK majors (python, go, php, ruby, cocoa, …) and `threads`/chain/breadcrumb
  support decisions.
- Dual-auth (query-string key) support if migration demand appears.
