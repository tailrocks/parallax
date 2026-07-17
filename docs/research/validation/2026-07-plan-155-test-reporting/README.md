# Plan 155 residual closure (2026-07-17)

Research date: 2026-07-17.

## Landed residual slices

### Batched `testCase` metadata

GraphQL `testCase` no longer issues per-variant history + flaky round-trips.
`MetadataStore::test_case_detail` loads case + variants + joined results +
joined flaky states in a fixed query budget (case, variants, results JOIN,
flaky JOIN). Per-variant history clamps apply in process.

Evidence: `crates/parallax-metadata` test
`test_case_variants_and_variant_history_are_bounded_and_isolated` asserts
`result_limit=2` clamps; API resolver tests remain green.

### D9 adapter export lifecycle

`parallax-analysis::test_adapter_export` composes:

- nextest per-attempt env identity (`CLI_INVOCATION_ID` mandatory; never
  `NEXTEST_RUN_ID` as product invocation),
- JUnit parse + reconcile,
- `MissingAttemptGap` rows for killed/missing ordinals **without fabricating
  raw test result rows** (matches authority-layer design).

Java JUnit listener emission remains playground-owned (`playground test-report`
/ W4 acceptance); Parallax consumes OTLP + JUnit authority for gap evidence.

### UI + Playwright gate

`/tests` list + case detail already expose attempts, status, service version,
invocation / trace / issue links. Full-stack Playwright:

- `ui/tests/e2e/full-stack/tests.spec.ts` — `@pw-full-stack-tests` product chrome.

### Flaky + unresolved surface

Scheduled flaky evaluation (`tick_once`, ≥2 signals, recovery threshold) shipped
earlier. V1 unresolved headline = explorer flaky-state filter + detail badges.
Dedicated tests-page SSE is not a separate product surface: live progress during
a run belongs to the invocations/session hub (`@live`, plan 147/140). Test
registry rows appear after OTLP indexing (query/reload).

### Explicit non-goals retained

- mute / known / owner fields: no V1 schema owner yet (residual said "when
  schema exists"); not invented.
- Fabricated persistence for missing JUnit ordinals: forbidden by authority
  design; gaps only.

## Done criteria

| Criterion | State |
| --- | --- |
| Identity/status/attempt/flaky; no run id in identity | landed |
| Native Greptime raw signals; Turso registry | landed |
| GraphQL + `/tests` UI + Playwright gate; detail links | landed |
| Flaky ≥2 signals + recovery; unresolved via explorer flaky | landed |
| D9 adapters + killed-test gap evidence | landed (export lifecycle + pure reconcile) |

## Commands

```bash
cargo test --locked -p parallax-analysis test_adapter_export
cargo test --locked -p parallax-metadata test_case_variants
cargo test --locked -p parallax-api resolvers::tests
# Playwright (with full-stack serve):
# bunx playwright test ui/tests/e2e/full-stack/tests.spec.ts
```
