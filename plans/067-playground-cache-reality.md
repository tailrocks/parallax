# Plan 067: Playground cache reality — real TTL cache in recommendation with hit/miss metrics + stampede knob (A26)

> **Executor instructions**: This plan targets the **playground repository**
> (`parallax-telemetry-playground`). Follow step by step; run every
> verification. On any STOP condition, stop and report. When done, update
> the status row in the Parallax repo's `plans/README.md`.
>
> **Drift check (run first)**: in the playground repo,
> `git diff --stat ed1f975..HEAD -- services/recommendation scenarios flags`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW (one leaf service; leak behavior preserved)
- **Depends on**: plan 036 (trace spine). Pairs with Parallax plans 044
  (metric label discovery/dashboards render the cache metrics) and 046
  (field explorer surfaces `cache.hit` span attrs).
- **Category**: direction
- **Planned at**: commit `408be17`/`ed5b10f` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

This is a stated-but-undelivered promise in the source itself: the
recommendation service's header says "related SKUs (**cache-backed in the
full design**)" but implements only a leak buffer — no lookups, no hit/miss,
no stampede. The research brief's database/cache domain (A26: "cache behavior
without new infrastructure") calls for hit/miss ratio, stampede, and leak
using in-process caches only. Cache-effectiveness dashboards and
thundering-herd traces are standard observability comparison material, and
Parallax's metric/dashboard surfaces (plan 044) need a metric family with
labels worth discovering. All of it fits in the existing service with zero
new infrastructure.

## Current state

Verified at playground commit `ed1f975`.

- `services/recommendation/src/main.rs:1-3` — the promise + today's whole
  chaos surface:

  ```rust
  //! Recommendation HTTP service — related SKUs (cache-backed in the full design).
  //! Chaos: ?leak=<n> grows a process-held buffer to emulate a cache/memory leak
  //! (B6) and adds latency, so the slow degradation is visible over repeated calls.
  ```

- `main.rs:9-12` — `leak_store()`: a `static Mutex<Vec<Vec<u8>>>` that only
  grows (`:29-33`). Keep it — B6 is a separate scenario.

- `main.rs:24-37` — the handler: `?sku`, `?leak`, `?slow` params; computes
  recs inline (`vec![format!("{}-ACCESSORY", p.sku), "WIDGET-2"]`); no
  cache.

- Metrics path: `tracing` fields become OTLP metrics via `MetricsLayer`
  (`libs/playground-telemetry/src/lib.rs:79-87,134-136`) — the repo
  convention for counters is `tracing` fields with the `counter.`/
  `histogram.` prefixes (check how existing services emit metric fields —
  grep `counter\.` / `monotonic_counter` across `services/` and match the
  working spelling; checkout/pricing use `tracing` fields that `MetricsLayer`
  picks up).

- flagd has a `cacheLeak` flag (`flags/flagd.json`) toggling only the leak;
  plan 042 wires flag reading — do not duplicate that machinery here.

- Scenario catalog: plan 037's `scenarios/run.sh` + README.

## Commands you will need

| Purpose | Command (playground root) | Expected |
|---------|---------------------------|----------|
| Build | `rtk cargo build` | exit 0 |
| Lint | `rtk cargo clippy --all-targets -- -D warnings` | exit 0 |
| Script lint | `bash -n scenarios/<new>.sh` | exit 0 |

## Scope

**In scope** (playground repo):
- `services/recommendation/src/main.rs` — TTL cache, hit/miss telemetry,
  stampede knob
- `services/recommendation/Cargo.toml` — ONLY if a tiny cache dep is chosen
  (see Step 1 decision; a hand-rolled `HashMap` + timestamps needs none)
- `scenarios/a26-cache.sh` (create) + catalog rows
- Nothing else

**Out of scope**:
- Redis or ANY new infrastructure — brief rule: in-process only.
- Postgres/db scenarios — plan 048.
- Java-side caches — deferred (one moving part).
- The leak path semantics — unchanged (only its comment may be clarified).
- Parallax-side dashboards — plan 044 renders what this emits.

## Git workflow

- Playground repo, `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: TTL cache in the handler

Decision by inspection: a hand-rolled cache is ~30 lines and dependency-free
— prefer it (repo keeps leaf services lean; no new dep unless the hand-roll
grows past ~60 lines):

```rust
// A26: in-process TTL cache for recommendations (hit/miss/stampede demos).
struct CacheEntry { value: Vec<String>, inserted: std::time::Instant }
fn rec_cache() -> &'static tokio::sync::Mutex<std::collections::HashMap<String, CacheEntry>> { ... }
const REC_TTL: std::time::Duration = std::time::Duration::from_secs(30);
```

Handler flow for `?sku=X`: lock, lookup non-expired → **hit**; else **miss**
→ simulate the expensive compute (`tokio::time::sleep(80ms)` inside a child
span `compute_recommendations` via `#[tracing::instrument]` or an explicit
span) → insert. Emit per request:
- span attribute/log field `cache.hit = true|false` on the request span's
  event stream (`tracing::info!(cache.hit = hit, sku = %p.sku, ...)`);
- metrics via the `MetricsLayer` field convention the repo already uses
  (Step 0 of your work: grep how existing counters are spelled; emit
  `cache.hits` / `cache.misses` counters and a `cache.size` gauge-ish
  value — if `MetricsLayer` supports only counters/histograms from fields,
  log `cache.size` as a histogram sample and note it).
Add `?cache=0` to bypass (baseline for comparison) and `?ttl_ms=` override
(clamped 100..=300_000) for demo pacing.

**Verify**: `rtk cargo build && rtk cargo clippy --all-targets -- -D warnings`
→ clean. Unit test the cache helper (insert/hit/expiry) with a fake clock or
short TTL.

### Step 2: Stampede knob

`?stampede=<n>` (clamp ≤ 100): the handler first **invalidates** the sku's
entry, then spawns `n` concurrent internal requests for the same sku
(`join_all` on the lookup-or-compute path). Without protection every task
misses and computes — the classic herd: `n` parallel
`compute_recommendations` spans in one trace + a miss burst in the metrics.
Do NOT add single-flight protection — the pathology is the demo (comment
this explicitly so nobody "fixes" it).

**Verify**: build+clippy clean; live: `?stampede=10` trace shows ~10
parallel compute spans (record span count via Parallax trace view or SQL).

### Step 3: Scenario + catalog

`scenarios/a26-cache.sh`:
1. Cold+warm phase: 10 requests same sku → 1 miss, 9 hits;
2. Ratio phase: 20 requests across 5 skus;
3. Stampede: one `?stampede=10`;
prints "Check in Parallax: Dashboards → metric `cache.hits`/`cache.misses`
(rate agg); trace detail → parallel compute spans; Logs/Field explorer →
`cache.hit` field." Register in `scenarios/run.sh` + README (037 format;
README-only if 037 absent).

**Verify**: `bash -n` clean; live run recorded (which checks ran, hit/miss
counts observed).

## Test plan

- Rust unit tests: cache hit/expiry/bypass; stampede clamp.
- The a26 script + recorded live observations are the integration test
  (repo convention for services).

## Done criteria

- [ ] Build + clippy `-D warnings` clean; unit tests pass
- [ ] Same-sku repeat requests hit (recorded); `cache.hits`/`cache.misses`
      visible as metrics in Parallax `metricNames` (recorded)
- [ ] `?stampede=10` produces the parallel-compute trace (recorded)
- [ ] `?cache=0` bypass works; leak path (B6) unchanged
- [ ] a26 cataloged
- [ ] Status row updated in Parallax repo `plans/README.md`

## STOP conditions

- The `MetricsLayer` field convention doesn't actually produce OTLP
  counters from `tracing` fields in this codebase (verify against a live
  export BEFORE building the whole plan on it) — report the working
  mechanism you found (direct `opentelemetry` meter API is the fallback;
  use it only after confirming the layer path fails).
- The mutex-held-across-await pattern trips clippy
  (`await_holding_lock`) — restructure to drop the guard before compute
  (lookup, drop, compute, re-lock, insert — accept the double-compute race,
  it IS the stampede surface); do not silence the lint.

## Maintenance notes

- Parallax plan 044's dashboards + 046's field explorer are the consumers;
  after they land, add a TOUR beat (plan 054).
- Java-side cache (Caffeine on catalog) is the named deferred follow-up —
  one moving part at a time.
- Reviewer: the stampede path must stay unprotected (comment guards it);
  TTL/clamps are exported consts; sku cardinality stays bounded (5-sku demo
  list).
