# Plan 155: Test reporting surface residual

> **Executor instructions**: Tests as first-class surface fused with traces,
> logs, issues, runs. Native Greptime tables only; Turso for mutable registry.
> Identity never uses invocation/run as test identity (plan 156 contract:
> `(test_variant_key, cli.invocation.id, attempt)`). Do not half-land UI.

## Status

- **Priority**: P1
- **Effort**: XL residual
- **Risk**: MEDIUM
- **Depends on**: 149, 152, 153, 140 (DONE hard deps); soft 121/124 open
- **Category**: product capability / ingest derivation / UI surface
- **Status**: IN PROGRESS — domain model + identity derivation landed
- **Evidence base**:
  [`docs/research/market/test-reporting-ecosystem.md`](../docs/research/market/test-reporting-ecosystem.md)

## Landed (do not replay)

- `parallax-model` test reporting domain: versioned `TestCaseKey` /
  `TestVariantKey`, result identity, attempt chains, rollups (`FlakyPass`),
  flaky state machine boundary, identity fallback
  (explicit → code reference → name path).
- Suites in `crates/parallax-model/src/test_reporting/`.

Design decisions D1–D9 (identity, native tables, status taxonomy, attempt
chains, shared fingerprints, flaky SM, `/tests` surface, session semantics,
runner adapters) remain binding — see Git history of this plan for full text
if needed; do not reopen.

## Residual only

1. Turso migrations + metadata modules (`test_cases` / `test_variants` /
   `test_results`).
2. Ingest derivation: test root-span recognition, registry upsert, failed vs
   broken, fingerprint linkage (`parallax-analysis`).
3. GraphQL `tests` namespace + clamped queries.
4. UI `features/tests`: list + detail + live session tree (after architecture
   owners; React Flow not required here).
5. Flaky job over ingested results; mute/known flags (no runner quarantine
   enforcement in V1).
6. Runner adapters: nextest support crate, JUnit listener jar, JUnit XML
   reconciliation gap-fill.
7. Live e2e vs plan 154 W4 playground payload; validation evidence under
   `docs/research/validation/`.

## Done Criteria

- [ ] Identity/status/attempt/flaky contracts + tests; no run id in identity.
- [ ] Native Greptime only for raw test spans/logs; Turso for mutable state.
- [ ] GraphQL + `/tests` UI with Playwright gates; failed detail shows error,
      attempts, SUT trace, logs window, issue link, version, history.
- [ ] Flaky from ≥2 signals with expiry; live session SSE + unresolved headline.
- [ ] Both D9 adapters + killed-test reconciliation proven.

## STOP / Remove When

STOP on custom raw-signal table or identity that embeds invocation/run.
Delete when surface ships with live playground proof.
