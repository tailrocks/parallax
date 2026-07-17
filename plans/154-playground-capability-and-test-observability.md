# Plan 154: Live multi-backend fan-out acceptance residual

> **Executor instructions**: W1–W5 implementation complete. Do not redesign.
> Parallax-backend arm DONE via plan 159. External multi-backend plumbing
> re-verified 2026-07-17 one-at-a-time (OO/Maple/SigNoz PASS; Sentry setup in
> flight). Remaining: Sentry finish + playground acceptance wrappers +
> disposition rows + workflow.

## Status

- **Priority**: P1
- **Effort**: S–M remaining
- **Risk**: MEDIUM
- **Depends on**: Docker host (available); Sentry self-hosted finish
- **Category**: cross-repository playground / live validation
- **Status**: IN PROGRESS — external fan-out 3/4 green; Sentry + acceptance residual
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
| SigNoz v0.129.0 | PASS — CH `signoz-smoke=102`, `signoz-smoke2=82`; first-org OpAMP gate re-confirmed |
| Sentry self-hosted | setup.sh running this session — not yet verify.sh |

## Residual only

1. Finish Sentry: `setup.sh` → `onboard.sh` → paste Rotel exporters →
   `verify.sh` (A1/A15/A16).
2. Collector-backed playground acceptance wrappers per stack still owed
   (`parallax run start -- scripts/observable-test-session.sh <stack>
   --acceptance` + `test-verify`) against the fan-out path (Parallax arm was
   plan 159).
3. Record W5 histogram / db-semconv / cross-language error **product
   disposition** rows in playground `VERIFICATION.md` (plumbing green; UI
   disposition pending).
4. Push playground workflow at same head; preserve SHA + artifacts.
5. Reconcile plan 122 disposition; preserve validation evidence; retire when
   five-backend matrix + acceptance + workflow are green.

## Done Criteria

- [x] Collector fan-out plumbing: OpenObserve, Maple, SigNoz (+ prior Parallax).
- [ ] Sentry self-hosted re-verify on current pin.
- [ ] Scenario/acceptance wrappers + exact-head playground workflow pass.
- [ ] Disposition rows recorded for histogram / cross-language error.

## STOP / Remove When

STOP if real fan-out replaced by mocks/screenshots or a product fallback
backend is introduced. Delete when five-backend matrix + acceptance wrappers
+ workflow pass.
