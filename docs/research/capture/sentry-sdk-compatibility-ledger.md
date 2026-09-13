# Sentry multi-SDK compatibility ledger

**Status: unproven on default Parallax serve.** Frozen P0 #10.

## Product blocker (concrete)

Default config is fail-closed:

```
crates/parallax-server/src/config.rs SentryConfig.enabled = false
```

`POST /api/<project_id>/envelope/` returns 404 until `[sentry] enabled = true` **and** a public key is set (`PARALLAX_SENTRY_PUBLIC_KEY` or `sentry.public_key`). Test: `crates/parallax-server/src/config/tests.rs` asserts `!config.sentry.enabled`.

A public “SDK X version Y fingerprints as Z” ledger cannot be proven on the default binary. Enabling the adapter is an operator action, not a silent default.

## Playground proof path (when adapter is on)

`mise run sentry:envelopes` (`cli/src/scenario_runner.rs` `sentry_envelopes`):

| SDK | How emitted | Title needle the scenario waits for |
| --- | --- | --- |
| Rust `sentry` 0.49 (`playground/Cargo.toml`) | `playground-telemetry` example `c8_sentry_emit` | `c8-rust-sdk` |
| Java `io.sentry:sentry-spring-boot-4-starter:8.53.0` | Gradle `c8SentryEmit` in `services/catalog` | `c8-java-sdk` |
| JavaScript `@sentry/tanstackstart-react` ^10.70.0 | `scenarios/c8-emit-js.ts` | `c8-js-sdk` |

The scenario polls `{ issues(limit: 50) { items { title errorType } } }` until all three needles appear (30s). That is ingest→issue derivation, not a versioned fingerprint table.

## What would close the P0

1. Operator-enabled Sentry adapter in a documented serve profile used by the playground.
2. A committed table: SDK package + exact version + envelope fixture hash + resulting `(service, fingerprint)` + title, asserted by a test that POSTs the fixture to the shipped envelope route.
3. Repeat for each supported SDK major, not only the three playground emitters.

Until then this ledger records the **blocker**, not a compatibility claim.
