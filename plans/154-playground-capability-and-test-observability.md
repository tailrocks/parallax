# Plan 154: Live multi-backend fan-out acceptance residual

> **Executor instructions**: W1–W5 implementation complete. Do not redesign.
> Parallax-backend arm DONE via plan 159. External multi-backend **plumbing**
> five-way green 2026-07-17 (one external at a time). Remaining: playground
> acceptance wrappers + product disposition rows + workflow.

## Status

- **Priority**: P1
- **Effort**: S remaining
- **Risk**: MEDIUM
- **Depends on**: Docker host (available)
- **Category**: cross-repository playground / live validation
- **Status**: IN PROGRESS — fan-out matrix green; acceptance/disposition residual
- **Evidence**:
  [`docs/research/validation/2026-07-17-plan-154-multi-backend/`](../docs/research/validation/2026-07-17-plan-154-multi-backend/README.md)

## Landed (do not replay)

Playground W1–W5 source, tests, CI workflow, plan-159 Parallax-backend
acceptance. Durable commands in playground `README.md` / `VERIFICATION.md`.
Evidence bundle under
`docs/research/validation/2026-07-unified-cli-observability/`.

**2026-07-17 live multi-backend (one external at a time, host 64 GiB):**

| Backend | Result |
|---|---|
| OpenObserve | PASS — 102 traces via Rotel; Parallax copy 102 |
| Maple v0.0.12 | PASS — `maple traces` shows `maple-fanout`; Parallax 102 |
| SigNoz v0.129.0 | PASS — CH `signoz-smoke=102`, `signoz-smoke2=82`; OpAMP first-org gate |
| Sentry self-hosted 26.6.0 | PASS — `verify.sh` OTLP 200 + A15/A16 `times_seen=5` |

## Residual only

1. Collector-backed playground acceptance wrappers per stack
   (`parallax run start -- scripts/observable-test-session.sh <stack>
   --acceptance` + `test-verify`) against the fan-out path (Parallax arm was
   plan 159).
2. W5 / disposition rows in playground `VERIFICATION.md`:
   - ~~Parallax exponential-histogram drop~~ code-confirmed 2026-07-17
     (`parallax-ingest` normalize `_ => {}` arm).
   - Still open: Maple/SigNoz/OpenObserve/Sentry histogram live disposition;
     cross-language `PaymentError` product grouping rows; db-semconv rows.
3. Push playground workflow at same head; preserve SHA + artifacts for the
   acceptance residual.
4. Reconcile plan 122 disposition; retire when acceptance + disposition +
   workflow are green.

## Done Criteria

- [x] Collector fan-out plumbing: OpenObserve, Maple, SigNoz, Sentry (+ prior
      Parallax).
- [ ] Scenario/acceptance wrappers + exact-head playground workflow pass.
- [ ] Disposition rows recorded for histogram / cross-language error.

## STOP / Remove When

STOP if real fan-out replaced by mocks/screenshots or a product fallback
backend is introduced. Delete when acceptance wrappers + disposition rows +
workflow pass on top of the green five-backend plumbing matrix.
