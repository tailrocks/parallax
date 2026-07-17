# Plan 167 — Alerting v1 (rules, evaluator, incidents, destinations)

**Status:** DONE (2026-07-17)  
**Closing commits:** Turso CRUD + pure machine/delivery/measurement helpers
through `bc146f0`/`4fe549b`; GraphQL surface; alerts UI; evaluator + delivery
loops + ready-banner `6f3ec1e`.

## Closed claims

| Claim | Evidence |
|---|---|
| Turso schema + CRUD | `parallax-metadata` alerts module + unit tests |
| Pure state machine (hysteresis, renotify, flap) | 42 server alerting unit tests green |
| Evaluator CAS tick + measurement adapters | `tick_once` + `AdapterMeasurementSource`; live breaches |
| Delivery worker (webhook/slack, backoff) | Unit tests + live webhook to `127.0.0.1:9876` |
| GraphQL CRUD | Live `alertRuleSave` / `alertDestinationSave` / queries |
| UI `/alerts` rules + incidents + destinations | Browser captures |
| Ready banner | `alerting   on (eval 5s / deliver 2s)` |
| Playground breach scenarios | playground `eee099a` / matrix `67be73a` |

## Live proof (2026-07-17 QA stack)

Serve: managed GreptimeDB + Turso, config intervals eval 5s / deliver 2s.

1. Created webhook destination `local-webhook` → `http://127.0.0.1:9876/hook`
2. Created rule `plan-167 high error rate` (`error_rate` `gt` `0`, 7d window)
3. Evaluator opened incident with `lastValue ≈ 0.103` (breach audit rows)
4. Delivery worker POSTed triggered payload (see `webhook-payloads.jsonl`)

Raw captures:

- [live-graphql-state.json](./live-graphql-state.json) — open incident + checks
- [webhook-payloads.jsonl](./webhook-payloads.jsonl) — delivered JSON body
- [ready-banner-alerting.txt](./ready-banner-alerting.txt)

## Browser evidence

| File | Claim |
|---|---|
| [browser/alerts-index.png](./browser/alerts-index.png) | Rules table, severity critical, enable switch |
| [browser/alerts-incidents.png](./browser/alerts-incidents.png) | Incidents tab (1 open) |

## Tests

- `cargo nextest -p parallax-server -E 'test(/alert|tick|deliver|measurement|state_machine/)'` → 42 passed
- UI: `alert-rule-form`, `alert-incident-timeline`, `-alerts.test` → 19 passed

## Deferred (honest, out of V1 done bar)

- Rule detail chart page (`/alerts/$ruleId`) and incident timeline page —
  index tabs cover list + create; detail is polish.
- Email destinations (explicit V1 deferral).
- Optional canvas particles N/A for this plan.
