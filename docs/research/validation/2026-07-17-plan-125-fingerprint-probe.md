# Plan 125 Step 1 — live fingerprint-column probes (preliminary, helper agent)

Research date: 2026-07-17. Host: operator arm64 macOS, Docker Desktop.
Engines probed (fresh standalone data dirs, native tables auto-created by one
OTLP/HTTP trace write through the `greptime_trace_v1` pipeline):

- **Stable**: `greptime/greptimedb:v1.1.3` (latest stable at probe time)
- **Nightly**: `greptime/greptimedb:v1.2.0-nightly-20260713` (latest nightly)

Raw transcripts (every SQL statement + full JSON response, in order):

- [2026-07-17-plan-125-fingerprint-probe-stable.log](2026-07-17-plan-125-fingerprint-probe-stable.log)
- [2026-07-17-plan-125-fingerprint-probe-nightly.log](2026-07-17-plan-125-fingerprint-probe-nightly.log)

Probe sequence per engine: fresh `SHOW CREATE TABLE` baseline → exact
pre-`f21bc65` legacy statement `ALTER TABLE opentelemetry_traces ADD COLUMN
"fingerprint" STRING` → duplicate-ADD error shape → value visibility →
`DROP COLUMN` → duplicate-DROP error shape → re-add → container restart →
drop → second restart.

## Findings (identical on stable and nightly)

1. **Fresh installs are clean.** The auto-created native table has no
   `fingerprint` column (confirms `f21bc65` fresh-install behavior).
2. **The legacy ADD reproduces exactly.** `ADD COLUMN "fingerprint" STRING`
   appends a nullable STRING column; a second ADD fails closed with code 4003
   `Column fingerprint already exists…` (why the old startup deviation was
   *not* idempotent without `IF NOT EXISTS`).
3. **`DROP COLUMN "fingerprint"` succeeds** on the live native table
   (`affectedrows: 0`, ~30–45 ms), row count unchanged (2 before/after),
   schema immediately reflects the removal.
4. **Drop of a missing column fails closed** with code 4002
   `Column fingerprint not exists…` — a convergence migration must therefore
   probe `information_schema.columns` first or tolerate 4002, never assume
   idempotent DROP.
5. **Both directions persist across restart.** A re-added column survives a
   container restart; a post-restart DROP survives a second restart. No
   startup errors, no data loss, helper tables
   (`opentelemetry_traces_services`, `_operations`) unaffected.
6. **Anomaly worth carrying into Step 2:** immediately after ADD,
   `count("fingerprint")` returned `non_null = total = 2` on BOTH engines —
   the never-written legacy column does **not** read as SQL NULL for existing
   rows (reads as a non-NULL default under `greptime_trace_v1`
   `append_mode`). Any consumer that ever used `fingerprint IS NULL` /
   `count()` semantics on the legacy column would have been wrong; this
   strengthens the removal case.

## Consumer/query inventory (repo state `0b470a4`)

`rg fingerprint` over `crates/` and `ui/src`: every reader resolves to the
derived `error_events` relation (Greptime `error_events` DDL + queries,
`parallax-metadata` Turso occurrences, worker occurrence writer, GraphQL
issues/invocations resolvers, MCP spike, UI issue routes). **No product
query, resolver, CLI, bundle, or UI reads `opentelemetry_traces.fingerprint`**;
the only remaining mentions are the retirement comment in
`crates/parallax-greptime/src/greptime/lifecycle.rs` and plan/docs text.

## What this does NOT yet prove (peer/executor owns)

- Upgraded-from-legacy data directories (real pre-`f21bc65` installs), only a
  faithfully simulated legacy ALTER on fresh dirs.
- Step 2 decision record + implementation spec update, and any
  existing-install convergence code (including the 4002-tolerant guard), per
  plan 125 Steps 2–4.
- Index/backfill probes for the retention alternative (removal path made it
  unnecessary here, but the plan's Step 2 comparison should cite finding 6).
