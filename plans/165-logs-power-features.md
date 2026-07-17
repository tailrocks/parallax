# Plan 165: Logs power features — brush-zoom histogram, pinned attribute columns, density controls, Drain pattern grouping

> **Executor instructions**: Follow this plan step by step. Read `ui/AGENTS.md`
> (browser-verification checklist applies after every step, against playground
> log scenarios). STOP conditions binding. Update this plan's status row in
> `plans/README.md` when done.
>
> **Drift check (run first)**:
> `git diff --stat <wave2-base>..HEAD -- ui/src/routes/logs.tsx ui/src/components/logs-table.tsx crates/parallax-analysis crates/parallax-api`
> `<wave2-base>` = the `main` commit closing Wave 1; plan 162/164 drift on these files
> is expected.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MED (logs table is shared by the invocation hub and trace views)
- **Depends on**: plans 162, 164 (facet sidebar lands the logs URL schema)
- **Category**: direction / UI + analysis / logs
- **Planned at**: `2288011`, 2026-07-17

### Landed by Grok (preliminary) — peer verify/extend + browser evidence

**Do not retire yet.** Pure layers only; peer must wire GraphQL/UI/playground
and complete Done criteria. Index status stays TODO (do not flip to DONE).

**Already landed (code present on `main`; may share the concurrent plan-162
closure commit — treat as preliminary handoff):**
- `crates/parallax-analysis/src/log_patterns.rs` — Drain-style clustering:
  mask UUID/IP/hex≥8/email/numbers → `<*>`, tokenize, fixed-depth prefix
  tree, similarity threshold (default 0.4), LRU cluster cap (default 512),
  severity mix + sample log id + first/last nanos, stable rank by count then
  template. Unit tests: masking, template stability under parameter churn,
  non-merge of distinct templates, spike ranks first, LRU bound, 10k-line
  completion under 2s with ~12 templates.
- `ui/src/lib/log-histogram-brush.ts` — pure brush helpers: snap to bucket
  edges, px↔time scale, uniform bucket builder (`DEFAULT_HISTOGRAM_BUCKETS=
  150`). Vitest coverage in `__tests__/log-histogram-brush.test.ts`.
- `ui/src/lib/log-table-prefs.ts` — `?columns=` pin encode/decode + pin/unpin
  toggle; density compact/comfortable parse + CSS class; wrap encode/parse
  (+ storage key constants). Vitest in `__tests__/log-table-prefs.test.ts`.

**Peer owns (verify/deepen/complete):**
- [ ] Clippy/format polish on Drain if needed; deepen masking if PII edge
  cases remain; live 10k timing on real log bodies.
- [ ] Step 2: GraphQL `logPatterns` + sampling via existing log query +
  live-engine cluster test (spiking template first).
- [ ] Step 3: histogram brush overlay on logs route → URL range; pinned
  columns (`?columns=`); density/wrap + localStorage.
- [ ] Step 4: Patterns toggle + expand-to-samples; browser evidence →
  `docs/research/validation/2026-07-wave2/165/`. The `l-patterns` playground
  scenario is already on the playground's main (`4741e64`): 20k lines, 11
  steady templates × 1,200 + spike × 6,800 in the last fifth of a 5-minute
  window, unit-tested; run `scenarios/run.sh l-patterns`.
- [ ] Full plan Done criteria; then retire file + index row.

## Why this matters

Parallax logs already beat the reference on live tail, but reading a burst
is still linear scanning: the histogram is display-only, interesting
attributes hide inside the document sheet, and 5k similar lines cannot be
collapsed. Three additions fix that: (1) **brush the histogram** to zoom the
time window; (2) **pin any attribute as a table column** (URL-encoded,
shareable); (3) **pattern grouping** — cluster log bodies into templates
(Drain algorithm) so a burst reads as "12 patterns, one spiking" instead of
5,000 rows. Drain is a well-known in-process algorithm (no engine support
needed) and the reference product runs it exactly that way.

## Reference (self-contained)

- Brush-zoom: drag across the severity-stacked histogram selects
  `[t1, t2)`; selection extends to bucket edges; on release the page range
  narrows (URL updates); Esc cancels. ~150 target buckets.
- Pinned columns: a "columns" control (and a pin action inside the document
  sheet) adds attribute keys as real table columns; keys live in the
  `?columns=` search param; unpinned attributes render as inline chips; row
  density toggle compact/comfortable + wrap toggle (localStorage).
- Drain pattern mining (Maple `packages/query-engine/src/drain/` — a
  TypeScript template miner; Parallax implements it in Rust): fixed-depth
  prefix tree over tokenized log bodies; masking pass first (UUIDs, IPs,
  hex ids, emails, numbers → `<*>` placeholders); similarity threshold
  (default 0.4) merges lines into cluster templates; LRU-bounded cluster
  set. Output per cluster: template string, count, severity mix, first/last
  seen, one sample log id. Runs over a bounded sample (≤10k bodies per
  request window), on demand — not at ingest.

## Current state

(verified at `2288011`)

- `ui/src/routes/logs.tsx` — histogram (`log_count_series`) above the
  virtualized table; live toggle (SSE); severity/service/text filters;
  saved views; "Load older" cursor pagination; document sheet on row click.
- `ui/src/components/logs-table.tsx` — fixed column set (time, severity,
  service, body + run/trace chips); no pinned columns, no density/wrap
  controls, no row expansion.
- `crates/parallax-analysis` — fingerprint/derive logic lives here (issue
  grouping); no log-pattern module.
- `crates/parallax-api` — `logs`, `log_count_series`, `logs_around` fields;
  no pattern field.
- Histogram drag: none (display only).

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Analysis tests | `cargo nextest run --locked -p parallax-analysis` | pass |
| API tests | `cargo nextest run --locked -p parallax-api` | pass |
| Live engine | `cargo nextest run --locked -p parallax-server -E 'binary(/greptime/)'` | pass |
| UI gates | `cd ui && bun run typecheck && bun run lint && bun run check && bun run --bun test:ci && bun run build` | exit 0 |
| Corpus | playground `scenarios/run.sh l-burst l-bodies l-patterns` | loaded (l-patterns added by this plan) |

## Scope

**In scope:**
- `crates/parallax-analysis/src/log_patterns.rs` (new): masking + Drain
  tree + cluster extraction, pure and heavily unit-tested; deterministic
  given input order-insensitivity requirements documented in tests.
- `crates/parallax-api` — `logPatterns(from,to, service?, severityMin?,
  attributeFilters?, limit?)` → clusters (template, count, severityCounts,
  firstNanos, lastNanos, sampleLogId…); samples ≤10k bodies via the
  existing log query path.
- `ui/src/routes/logs.tsx` — brush selection on the histogram; a
  **Patterns toggle** switching the table to cluster view (template rows,
  count, severity mix bar, expand → sample rows via the normal query with a
  template-derived filter); columns control + density/wrap toggles.
- `ui/src/components/logs-table.tsx` — pinned attribute columns
  (`?columns=`), inline attribute chips for unpinned keys, pin action in
  the document sheet, density/wrap.
- Playground (direct on its main): new scenario `l-patterns` — ≥20k lines from ~12
  templates with parameter churn (ids, ips, durations) + one "spiking"
  template, so clustering quality is assertable.

**Out of scope:** ingest-time pattern extraction or new storage tables
(query-time only, per the native-tables rule), the live-tail transport
(plan 147 owns stream mechanics), saved-view schema beyond adding the new
params, logs-table usage inside the invocation hub beyond it transparently
gaining the same features.

## Git workflow

- Work directly on `main` in BOTH repositories — no branches, no pull requests (operator
  delivery model, 2026-07-17; see plans/README.md Execution Preflight).
- Commit OFTEN: one small green slice per commit (a step, a component, a
  fixed defect), Conventional Commits, DCO `-s`, exactly one agent trailer.
- **Push to `main` immediately after every commit** — never batch pushes,
  never hold local-only work; never push a slice whose targeted checks are
  red. The parallax ruleset's "Bypassed rule violations" push notice is
  expected.

## Steps

### Step 1: Drain in Rust

Implement masking (UUID/IP/hex≥8/email/number token classes) +
fixed-depth-tree clustering with similarity threshold + LRU cap (default
512 clusters). Unit tests: template stability across parameter churn; two
distinct templates never merge below threshold; masking cases; 10k-line
performance sanity (< 500ms in a debug-build test is fine — assert it
completes, record timing).

**Verify**: `cargo nextest run -p parallax-analysis` → pass.

### Step 2: GraphQL field + live test

Wire sampling query + clustering into `logPatterns`. Live-engine test
against seeded `l-patterns`-shaped fixtures: expect ~12 clusters, the
spiking template ranked first by count.

**Verify**: API + live-engine lanes pass.

### Step 3: Histogram brush + columns + density

UI work per the reference contract. Brush = pointer drag on the Recharts
bar chart via a selection overlay (bucket-snapped); columns control +
document-sheet pin; density/wrap with localStorage persistence.

**Verify**: component tests (brush → URL range update; pin/unpin round-trip
via URL; density class switch); UI gates green.

### Step 4: Patterns view + browser closure

Patterns toggle + cluster rows + expand-to-samples. Browser walk per the
`ui/AGENTS.md` checklist against `l-burst`/`l-bodies`/`l-patterns`:
brush-zoom into the spike, switch to Patterns, spiking template on top,
expand it, pin an attribute column, share the URL, reload reproduces.
Screenshots to `docs/research/validation/2026-07-wave2/165/`.

**Verify**: all evidence captured; clean console.

## Done criteria

- [ ] Drain unit + live-engine cluster tests pass (spiking template first).
- [ ] UI gates green; brush/pin/density/pattern tests pass.
- [ ] Browser evidence incl. URL-reload reproduction and live-tail still
  working alongside the new controls.
- [ ] Playground `l-patterns` scenario + matrix row landed on the playground's main.
- [ ] `plans/README.md` status row updated.

## STOP conditions

- Sampling 10k bodies through the existing log query is too slow on the
  live engine (>2s) — report timings; do not silently lower the cap.
- Clustering quality on `l-patterns` is unusable (templates over-merge) at
  the default threshold — report with examples rather than shipping a
  misleading view.
- Pinned columns break the logs-table contract used by the invocation hub
  (its tests must pass unchanged).

## Maintenance notes

- Cluster templates are ephemeral query results — never persisted; if a
  future plan wants pattern-based alerting, that is a new contract
  decision, not an extension of this cache-free path.
- Reviewer focus: masking correctness (no PII-bearing raw tokens surviving
  into templates), URL schema stability, virtualizer behavior with dynamic
  column sets.
