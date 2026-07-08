# Plan 063: Playground trace-shape scenarios — A19 long/wide stress trace, structural compare pair, real backdated clock skew

> **Executor instructions**: This plan targets the **playground repository**
> (`parallax-telemetry-playground`). Follow step by step; run every
> verification. On any STOP condition, stop and report. When done, update the
> status row in the Parallax repo's `plans/README.md`.
>
> **Drift check (run first)**: in the playground repo,
> `git diff --stat ed1f975..HEAD -- services/checkout services/inventory libs/playground-telemetry scenarios`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW (opt-in query params; default behavior unchanged)
- **Depends on**: plan 036 (trace spine — stress traces must stitch);
  plan 037's catalog receives rows. Pairs with Parallax plans 040
  (virtualization), 051 (traceCompare), 061 (view modes + skew banner).
- **Category**: direction
- **Planned at**: commit `408be17`/`ed5b10f` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

Three Parallax surfaces have no telemetry to prove them. (1) Plan 040
virtualizes the waterfall and plan 061 adds a minimap — but the biggest
playground trace is ~48 spans (`VERIFICATION.md` counts: checkout=30,
pricing/inventory/recommendation=6 each); the research brief's A19
"long/wide trace" scenario exists nowhere, so rendering-at-scale claims are
untested. (2) Plan 051 ships `traceCompare` — but no two traces of the same
operation differ *structurally* on success; today's contrast is only
success-vs-error (`RELEASE=v2` fails before the fan-out). (3) The B18 "clock
skew" scenario doesn't skew any telemetry timestamp — it logs an old number
as a field value (`skewed_unix_s`), so plan 061's skew banner would never
fire on it. This plan adds honest knobs for all three.

## Current state

Verified at playground commit `ed1f975`.

- `services/checkout/src/main.rs:26-63` — `CheckoutParams` today: `sku`,
  `quantity`, `fail`, `slow`, `canary`, `n1` (extra sequential inventory
  calls), `retry`, `timeout_ms`, `cpu_ms`, `lock`, `tenant`, `tier`,
  `degrade`, `skew`. No fan/depth knob.

- The existing N+1 knob is the pattern to extend —
  `services/checkout/src/main.rs:157-161`:

  ```rust
  // B9: N+1 — fire N extra sequential inventory calls (a classic hotspot).
  for i in 0..p.n1 {
      let _ = reserve(&p.sku, 1).await;
      tracing::debug!(i, "n+1 inventory call");
  }
  ```

- The fake skew — `services/checkout/src/main.rs:126-134`:

  ```rust
  if p.skew {
      // B18: a span event timestamped far in the past (clock skew across hops).
      let skewed = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
      ...
      tracing::warn!(skewed_unix_s = skewed, "clock-skew event (1h in the past)");
  }
  ```

  The LogRecord/span timestamps stay real; only the field value is old.

- Contrast machinery — `main.rs:135-155`: `RELEASE=v2`/`?fail=1` fails
  before the fan-out (returns 502 or degraded 200); `?slow=<ms>` delays; no
  structural variant of a *successful* checkout exists.

- Telemetry lib: `tracing`-only span creation
  (`libs/playground-telemetry/src/lib.rs:130-139` — subscriber wiring);
  spans via `#[tracing::instrument]`. `tracing` cannot backdate a span
  timestamp — real skew needs the OTel API directly (Step 3 does this in a
  contained helper).

- Scenario driver: shell scripts under `scenarios/` + plan 037's `run.sh`
  catalog; params thread through checkout's query string.

## Commands you will need

| Purpose | Command (playground root) | Expected |
|---------|---------------------------|----------|
| Build | `rtk cargo build` | exit 0 |
| Lint | `rtk cargo clippy --all-targets -- -D warnings` | exit 0 |
| Script lint | `bash -n scenarios/<new>.sh` | exit 0 |

## Scope

**In scope** (playground repo):
- `services/checkout/src/main.rs` — `fan`, `depth`, `variant` params + the
  internal span generator; replace the fake skew body
- `libs/playground-telemetry/src/lib.rs` — one helper: emit a span with an
  explicit backdated start/end via the OTel API
- `scenarios/a19-long-trace.sh`, `scenarios/a20-compare-pair.sh` (create);
  update `scenarios/b-degradation.sh`'s skew line comment if it references
  B18 semantics
- Catalog rows (`scenarios/run.sh` + `scenarios/README.md`, plan 037 format)

**Out of scope**:
- GraphQL/gRPC/messaging scenario families — plans 047/049.
- Sampling/cron/field-spike — plan 054.
- Any Parallax-repo change.
- Cross-service *real* clock skew (containers sharing the host clock can't
  genuinely drift) — the single-hop backdated-span approach below is the
  honest lab version; state it in the scenario output.

## Git workflow

- Playground repo, `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: A19 — `fan` + `depth` internal-span generator

In `CheckoutParams` add `fan: u32` (default 0) and `depth: u32` (default 0,
both capped at sane maxima — `fan ≤ 50`, `depth ≤ 10`, `fan*depth`
product ≤ 2000; clamp, don't error). After the existing fan-out section,
when `fan > 0`:

```rust
// A19: synthetic wide/deep span tree for rendering-at-scale demos.
#[tracing::instrument]
async fn burst(level: u32, width: u32) { ... }
```

Recursive async fn: each level spawns `width` child spans named
`burst.l<level>` with 1-5ms sleeps (deterministic small jitter from the loop
index, no rand dep — match the `cli/src/main.rs:40-44` nanos-bucket
pattern); recurse until `level == depth`. `?fan=20&depth=3` ≈ 20+400+8000 →
clamped by the product cap to ~2000 spans. Keep it sequential-batched
(`join_all` per level) so the trace is wide AND deep.

`scenarios/a19-long-trace.sh`: drive `?fan=15&depth=2` (315 spans) and
print: "Check in Parallax: trace detail stays responsive; waterfall windows
(plan 040); minimap + lanes (plan 061)."

**Verify**: `rtk cargo build` + clippy clean; live: one request produces a
trace whose span count matches the formula (record via Parallax SQL:
`SELECT count(*) FROM opentelemetry_traces WHERE trace_id = '<id>'`).

### Step 2: Structural compare pair — `variant`

Add `variant: Option<String>` to `CheckoutParams`:
- `variant=v1` (and default/absent): current behavior — the `n1` loop only
  if `n1 > 0`, then the parallel pricing/inventory/recommendation fan-out.
- `variant=v2`: same successful checkout but structurally different — force
  the N+1 path (`8` sequential inventory calls) AND skip recommendation
  (drop one branch), so `traceCompare(a, b)` shows added spans (8×
  `reserve`), a removed span (recommend), and duration deltas — on two
  **green** traces.
- Stamp `compare.variant` as a span attribute on the root
  (`tracing::Span::current().record`/field on the instrument macro — match
  how existing fields are set, e.g. `fields(otel.kind = "server")` at
  `main.rs:90`; a `tracing::info!(compare.variant = %v, ...)` log line is
  the fallback if recording a dynamic field on the instrument macro is
  awkward — then put the attribute on a log, not the span, and say so).

`scenarios/a20-compare-pair.sh`: fire one `variant=v1` and one `variant=v2`
request, echo both trace ids (parse from the response if checkout returns
one; otherwise print the SQL to find them:
`SELECT trace_id FROM opentelemetry_traces WHERE span_name = 'checkout' ORDER BY "timestamp" DESC LIMIT 2`),
and print: "Check in Parallax: trace detail → Compare with… (plan 051):
8 added reserve spans, 1 removed recommend span."

**Verify**: build+clippy clean; `bash -n` clean; live pair recorded.

### Step 3: Real backdated skew

1. `libs/playground-telemetry/src/lib.rs`: helper
   `pub fn emit_backdated_span(name: &'static str, offset: std::time::Duration, duration: std::time::Duration)`
   — uses the OTel tracer API directly (`global::tracer(...)`,
   `SpanBuilder` `with_start_time(now - offset)` / explicit
   `end_with_timestamp`), parented to the **current** `tracing` span's OTel
   context (`tracing_opentelemetry::OpenTelemetrySpanExt::context()` on
   `tracing::Span::current()` — the crate is already a dependency,
   `lib.rs:133`). Result: a child span whose start precedes its parent by
   `offset` — exactly what Parallax plan 061's `detectSkew` flags.
2. `services/checkout/src/main.rs`: replace the `p.skew` body (`:126-134`)
   with `playground_telemetry::emit_backdated_span("skewed-op",
   Duration::from_secs(3600), Duration::from_millis(20))` plus a short
   comment. Keep the existing warn log line (it's the log-side witness) but
   fix its comment to say what now actually happens.

**Verify**: build+clippy clean; live: `?skew=1` trace contains a child span
starting ~1h before its parent (SQL:
`SELECT span_name, "timestamp" FROM opentelemetry_traces WHERE trace_id='<id>' ORDER BY "timestamp"` —
the skewed span sorts first by an hour). Record it. (Parallax's waterfall
will render it clamped until plan 061 lands — that mismatch is the demo.)

### Step 4: Catalog rows

Register a19 + a20 (+ the changed b-degradation skew semantics note) in
`scenarios/run.sh` and `scenarios/README.md` per plan 037's format; if 037
hasn't landed, README-only and note it.

**Verify**: `bash -n scenarios/run.sh` (if touched) → exit 0; rows present.

## Test plan

- Rust: unit test the clamp math (`fan`/`depth`/product) as a pure function
  (extract `fn clamp_shape(fan, depth) -> (u32, u32)`); unit test
  `emit_backdated_span` compiles against the OTel API (behavioral check is
  the live SQL verification — say which ran).
- Scripts: `bash -n` + live runs recorded.

## Done criteria

- [ ] `rtk cargo build` + clippy `-D warnings` clean
- [ ] `?fan=&depth=` produces the formula's span count, clamped at 2000
      (recorded live check)
- [ ] `variant=v1|v2` produce two green structurally-different traces with a
      recorded id pair
- [ ] `?skew=1` produces a genuinely backdated child span (recorded)
- [ ] a19/a20 cataloged; skew comment honest
- [ ] Status row updated in Parallax repo `plans/README.md`

## STOP conditions

- The installed `opentelemetry` crate's `SpanBuilder` lacks explicit
  start/end timestamp setters — report the crate version + API found (the
  0.27+ API has `with_start_time`; version policy says latest stable — an
  upgrade is its own change).
- The recursive burst generator trips tokio stack/depth issues at the caps —
  lower the caps and record the real ceiling; don't ship an unstable knob.
- Plan 036 not landed and stress traces fragment (context not propagated) —
  the A19 trace is then single-service; note it and proceed (it still
  stresses rendering), but say so in the catalog row.

## Maintenance notes

- Parallax plans 040/051/061 are the consumers; after they land, add a TOUR
  beat (plan 054) chaining a19 → modes → compare.
- The product cap (2000) mirrors Parallax's uncapped `spans_by_trace` — if
  Parallax later caps trace reads, keep the playground max below it.
- Reviewer: burst spans must carry no high-cardinality attributes (span name
  is `burst.l<level>`, level ≤ 10 — bounded).
