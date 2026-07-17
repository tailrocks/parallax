# Plan 164 — Faceted filters, duration presets, where-clause editor

**Status:** DONE (2026-07-17)  
**Closing commits (parallax `main`):** backend slices through `f3b5170` /
`6a63b3b`; UI logs wiring `6392459`; typecheck helper `e547903`.

## Scope closed

| Surface | Evidence |
|---|---|
| Backend `attributeFilters` (traces + logs) | Compiler unit tests + live GraphQL (injection → 0 rows) |
| `traceFacets` / `logFacets` / `invocationFacets` | Live GraphQL + UI sidebar counts |
| `traceDurationStats` p50/p95 | Live GraphQL + Duration chip UI (`> p50` / `> p95` / `> 1s`) |
| Where-clause AND-only grammar | 26 parser tests; editor autocomplete/apply tests |
| Traces route wiring | FacetSidebar + editor + duration + URL `?where=` |
| Logs route wiring | FacetSidebar + editor + `logFacets` loader (`6392459`) |
| Invocations route wiring | FacetSidebar from `invocationFacets` |
| `F` focuses where editor | Traces + logs key handlers |
| Facet value cap | 24 values per dimension (storage constants) |

## Live GraphQL (managed QA stack, 2026-07-17)

Prior packet: [live-graphql-assertions.md](./live-graphql-assertions.md)
(filter narrowing, injection proofs, facet/series three-way consistency).

### `f-attrs` 70/20/10 (this closure)

Playground `scenarios/run.sh f-attrs` → OTLP into `parallax serve` at
`127.0.0.1:4000` (managed GreptimeDB). Asserted:

```text
traceFacets(…, attributeFilters: [shape.case = f-attrs])
  http.request.method: GET 70, POST 20, DELETE 10
tracesPage(…, shape.case = f-attrs AND http.request.method = POST).total = "20"
```

Raw capture: implementer scratch `f-attrs-facets.json` / `f-attrs-run.log`.

## Browser evidence

Captured with `agent-browser` 0.32.1 against live GraphQL
(`parallax serve` :4000) and the plan-164 UI on Vite
(`localhost:3000` → proxies `/graphql` → :4000).

| File | Claim |
|---|---|
| [browser/traces-facets.png](./browser/traces-facets.png) | Service / status / method / error.type facets with counts |
| [browser/traces-post-facet.png](./browser/traces-post-facet.png) | POST facet toggles where chip + URL |
| [browser/traces-permalink-reload.png](./browser/traces-permalink-reload.png) | Reload of `?where=http.request.method+%3D+POST` restores selection |
| [browser/traces-duration-open.png](./browser/traces-duration-open.png) | `> p50` / `> p95` / `> 1s` chips from live stats |
| [browser/traces-p95.png](./browser/traces-p95.png) | p95 → `minMs` URL + summary chip |
| [browser/traces-compound-where.png](./browser/traces-compound-where.png) | `service = checkout AND http.request.method = POST` parses + applies |
| [browser/logs-facets-dev.png](./browser/logs-facets-dev.png) | Logs FacetSidebar (service + severity counts) + where editor |
| [browser/invocations-facets.png](./browser/invocations-facets.png) | Invocation facets (service / app.mode / …) |

Console: no product errors on logs walk (Vite/React DevTools only).

## Unit / UI gates (closure slice)

- `where-clause` + facet-sidebar + duration-filter + where-clause-editor: 48 tests green
- logs route where URL round-trip: green in `-logs.test.tsx`
- `bun run typecheck` + `bun run lint` green at closure
- Backend facet/filter nextest filter green (see scratch `plan-164-backend-tests.txt`)

## m9 ignore-gated suite

`m9_attribute_filters_greptime` remains available for isolated managed-engine
CI; ports 24000–24003 were held by the live QA stack during this closure.
Live GraphQL + `f-attrs` on that stack cover the same acceptance claims.

## Multi-select semantics

v1 keeps **AND across filters** (backend). Multiple `=` values for one
dimension are multiple AND predicates (typically empty). OR-within-dimension
is deferred; facet UI toggles one equality filter per click.
