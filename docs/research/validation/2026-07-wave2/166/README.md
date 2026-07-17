# Plan 166 — Service map v2 (ELK + React Flow + focus + externals)

**Status:** DONE (2026-07-17)  
**Closing commits:** backend external derivation `2dea2d2`+; ELK/layout
`f4aeea0`/`0ad35fd`/`dcdc020`; focus URL `e83a414`; React Flow cutover
`526ed9d`; playground `eco-external` `15de5c3`.

## Closed claims

| Claim | Evidence |
|---|---|
| External nodes from generic attrs only | Live GraphQL: `database` (`playground`/postgresql), `queue` (`kafka/orders`), `external` (`api.stripe.test`, `flagd`); instrumented pairs stay service edges |
| ELK layout + worker + fallback | Layout unit tests; Vite emits `ecosystem-layout.worker-*.js`; fallback &lt;100ms on host |
| Focus / hops / dim|hide + traffic presets URL | Route search + pure topology model; browser captures |
| React Flow renderer (AGENTS rule 24) | `@xyflow/react` mount in ecosystem graph; graph tests assert `.react-flow` |
| `eco-external` playground scenario | playground `main` `15de5c3` + matrix row |
| No vendor inference | `rg hyperdrive\|planetscale ui/src crates/` → 0 |

## Live GraphQL kinds (2026-07-17 QA stack)

```text
kinds: browser, cli, database, external, queue, service
external samples: api.stripe.test, flagd
queue samples: kafka/orders, inprocess/orders
database sample: playground (system=postgresql) with catalog+inventory fan-in
```

## Browser evidence

| File | Claim |
|---|---|
| [ecosystem-reactflow-external.png](./ecosystem-reactflow-external.png) | React Flow map: checkout→api.stripe.test, fulfillment→kafka/orders |
| [focus-checkout-1hop-dim-rf.png](./focus-checkout-1hop-dim-rf.png) | Focus checkout 1-hop (URL) |
| [traffic-filter-1pct-rf.png](./traffic-filter-1pct-rf.png) | `minTraffic=1%` shows hidden chip |
| [focus-checkout-1hop-dim.png](./focus-checkout-1hop-dim.png) | Earlier focus capture (pre RF polish) |
| [focus-checkout-2hop-hide.png](./focus-checkout-2hop-hide.png) | Earlier 2-hop hide capture |
| [traffic-filter-hidden-chip.png](./traffic-filter-hidden-chip.png) | Earlier traffic chip capture |

## Tests

- Ecosystem graph + URL + focus route tests: 11 green
- Backend external/service-map nextest filter: green (scratch `plan-166-backend.txt`)
- UI typecheck green after React Flow cutover
