# Active Implementation Plans

`plans/` holds only future product or engineering work we still intend to
implement or improve. Completed work lives in Git history and, when useful,
under `docs/research/validation/`. Do not keep plan history here.

## Active

| Plan | Title | Status |
|------|-------|--------|
| [089](089-extension-table-grpc-writes.md) | Move derived extension-table writes to GreptimeDB's row API | BLOCKED — crates.io `greptimedb-ingester` still 0.18.0; upstream [PR #58](https://github.com/GreptimeTeam/greptimedb-ingester-rust/pull/58) OPEN (recheck 2026-07-17T17:18Z); HTTP SQL path remains |
| [114](114-retire-legacy-spool-reader.md) | Retire the legacy NDJSON spool reader | BLOCKED — only rolling `preview` tag; need one stable raw-frame release cycle + expired legacy segments (recheck 2026-07-17T17:18Z) |
| [162](162-fanout-lab-backend-pins.md) | Pin every fan-out-lab and playground infra image at current latest stable | TODO |
| [163](163-playground-example-upgrades.md) | Upgrade every playground example to current-latest instrumentation and re-verify emission | TODO — after 162 |
| [164](164-playground-feature-coverage.md) | Extend the playground until every Parallax feature has a scripted scenario | TODO — after 162, 163 |
| [165](165-user-lens-comparison.md) | Run the full playground sweep and record a user-lens comparison across all backends | TODO — after 162–164 |
| [166](166-production-readiness-fix-loop.md) | Drive every verified discrepancy to zero — the production-readiness fix loop | TODO — after 165; loops until W5 list empty |

Plans 162–166 implement the verification program defined in
[docs/research/reference/feature-inventory-and-playground-verification.md](../docs/research/reference/feature-inventory-and-playground-verification.md)
(workstreams W2, W1, W3, W4, W5 respectively). Execute in numeric order;
166 iterates with 165 as a loop. Playground-side changes land in
`tailrocks/parallax-telemetry-playground` via its own single PR per plan.

Deferred decisions from that program (do not re-audit): lab roster stays at
five backends (Parallax, OpenObserve, Maple, SigNoz, Sentry) — adding
Grafana LGTM / HyperDX / Uptrace is a separate operator decision; automated
cross-backend scoring stays out (comparison is manual by design); product
gaps (profiles signal, SLO/burn-rate, GraphQL subscriptions, alert email,
browser sessions) are roadmap items, not bugs, and stay in the inventory
doc's gap list.

## Constraints

These leftovers must preserve:

- GreptimeDB + Turso only; no product fallback engine.
- GreptimeDB native raw-signal tables.
- Native TLS only; never an active rustls backend.
- Bun only for JavaScript/TypeScript.
- Decode once and move ownership on the ingest hot path.
- Apache-2.0 throughout.

## Lifecycle

1. Numbered file `plans/NNN-kebab-case.md` plus one index row.
2. File states residual, blocker or next step, done criteria, and STOP.
3. When the work ships or is dropped, delete the file and this row in the same
   commit. Do not archive DONE plans here.
