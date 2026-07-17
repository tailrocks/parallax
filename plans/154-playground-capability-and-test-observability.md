# Plan 154: Live multi-backend fan-out acceptance residual

> **Executor instructions**: W1–W5 implementation complete. Do not redesign.
> Parallax-backend arm DONE via plan 159. Remaining: self-hosted multi-backend
> matrix one backend at a time (no external SaaS credentials).

## Status

- **Priority**: P1
- **Effort**: M remaining
- **Risk**: MEDIUM
- **Depends on**: Docker host (available); self-hosted Maple/SigNoz/
  OpenObserve/Sentry configs
- **Category**: cross-repository playground / live validation
- **Status**: BLOCKED — multi-backend arm only
- **Blocker**: Operator must run four self-hosted externals one-at-a-time on
  the 16 GB host (unblock: no external credentials; local Docker only). Not
  inventable from source inspection.

## Landed (do not replay)

Playground W1–W5 source, tests, CI workflow, plan-159 Parallax-backend
acceptance. Durable commands in playground `README.md` / `VERIFICATION.md`.
Evidence bundle under
`docs/research/validation/2026-07-unified-cli-observability/`.

## Residual only

1. Start full topology + one self-hosted backend at a time.
2. `parallax run start -- scripts/observable-test-session.sh <stack>
   --acceptance` then `playground test-verify`.
3. Prove baggage/gateway chains, Kafka causal link, scenario signals.
4. Record failed Playwright/Rust attempts + histogram/db-semconv/
   cross-language error disposition in playground `VERIFICATION.md` for each
   backend.
5. Push playground workflow at same head; preserve SHA + artifacts.
6. Reconcile plan 122 disposition; preserve validation evidence; retire.

## Done Criteria

- [ ] Collector-backed acceptance for Maple, SigNoz, OpenObserve, Sentry
      (self-hosted) + prior Parallax arm.
- [ ] Scenario sweep + exact-head playground workflow pass.

## STOP / Remove When

STOP if real fan-out replaced by mocks/screenshots or a product fallback
backend is introduced. Delete when five-backend matrix + workflow pass.
