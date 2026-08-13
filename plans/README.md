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
| [166](166-production-readiness-fix-loop.md) | Drive every verified discrepancy to zero — the production-readiness fix loop | TODO — after 165, 167; loops until W5 list empty |
| [167](167-agent-browser-ui-verification.md) | Verify every Parallax UI surface with agent-browser — functional + responsive | TODO — after 163, 164; runs alongside 165 |
| [168](168-rust-correctness-test-wave.md) | Close the correctness-critical Rust test gaps (wave 1) | TODO — independent; do first in the QA program |
| [169](169-rust-parity-and-structural-tests.md) | Fake/engine parity, resolver depth, and metadata versioning (wave 2) | TODO — after 168 (real integration gate) |
| [170](170-playwright-critical-coverage.md) | Playwright coverage for every critical user flow | TODO — independent; dataset seam first |
| [171](171-competitor-informed-feature-uplift.md) | Preview-before-save, agent issue lease, MCP evals, instrumented onboarding | TODO — after 168; feature 2 gated on decision-record approval |
| [172](172-design-system-and-guide.md) | Design-system uplift and the documented design guide (foglamp-informed) | TODO — after 170 (regression net); coordinates with 171 feature 4 |

Plans 162–167 implement the verification program defined in
[docs/research/reference/feature-inventory-and-playground-verification.md](../docs/research/reference/feature-inventory-and-playground-verification.md)
(162→W2, 163→W1, 164→W3, 165→W4, 166→W5, 167→the agent-browser UI
verification pass). Execute in numeric order; 167 runs alongside 165 on the
same seeded stack; 166 consumes both and iterates as a loop.
Playground-side changes land in `tailrocks/parallax-telemetry-playground`
via its own single PR per plan.

Plans 168–172 are the QA + quality program (2026-08-13 deep audit:
Playwright/Rust coverage, Maple feature study, foglamp design study).
168→169 sequence; 170 independent; 171 after 168 with feature 2 gated on an
operator decision-record amendment; 172 after 170. Both programs share the
`DISCREPANCY:` pipeline consumed by plan 166.

Deferred decisions from these programs (do not re-audit): lab roster stays
at five backends (Parallax, OpenObserve, Maple, SigNoz, Sentry) — adding
Grafana LGTM / HyperDX / Uptrace is a separate operator decision; automated
cross-backend scoring stays out (comparison is manual by design); product
gaps (profiles signal, SLO/burn-rate, GraphQL subscriptions, alert email,
browser sessions) are roadmap items, not bugs, and stay in the inventory
doc's gap list. Maple-inspired ideas deliberately NOT planned: session
replay, anomaly-detection incidents, K8s/host infra pages, AI
investigations, web analytics, digest emails (roadmap candidates; Maple is
FSL-licensed — ideas only, never code). foglamp items rejected: squircle
corner plugin (Chromium-only), grayscale chart tokens (Parallax's dataviz
palette is stronger for many-series telemetry), repo-wide border ban
(tables keep hairline borders). Playwright visual-snapshot expansion to
dense data tables rejected (churn > value; a11y/overflow checks instead).
Rust line-coverage chasing in parallax-greptime SQL builders rejected
(only meaningful against the live engine — the real-engine gate covers
it).

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
