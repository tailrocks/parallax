# Plan 020: Bound the ingest spool — rotation, retention config, doctor visibility

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat f7f2c17..HEAD -- crates/parallax-storage/src/spool.rs crates/parallax-server/src/serve.rs crates/parallax-server/src/config.rs crates/parallax-cli/src/doctor.rs`
> On excerpt mismatch, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: none
- **Category**: bug / perf
- **Planned at**: commit `f7f2c17`, 2026-07-03

## Why this matters

Every accepted OTLP request is appended to a per-signal NDJSON spool **before ack** — and then never read again: the ingest worker consumes the in-memory mpsc channel, and raw bytes are forwarded to GreptimeDB directly. The spool is pure write-amplification with no rotation, size cap, or TTL; the only reclaim is a human running `parallax prune`. Observed cost on a live install: **1.65 GiB `logs.ndjson`** (vs 431 MiB of actual engine data) — a silent disk-fill on any long-lived server. Engine-side telemetry already has TTL retention (traces/logs 7 d, metrics 14 d via `x-greptime-hints`); the spool is the one unbounded store left.

## Current state

`crates/parallax-storage/src/spool.rs` (verified firsthand, full file ~73 lines):

```rust
// :30-34  Append-only NDJSON spool, one file per signal.  Spool { dir, write_lock: Mutex<()> }
// :51-62  append(): serde_json line; OpenOptions::new().create(true).append(true); write line + \n
// :65-71  line_count(): reads whole file, counts lines (tests + doctor)
```

- Writers: `crates/parallax-server/src/otlp_http.rs:50`, `otlp_grpc.rs:64` — append THEN queue to worker (write-before-ack is the M0 durability contract; the file header comment says the spool "then becomes the bounded WAL described in the implementation spec" — i.e. bounding is the spec'd intent, unimplemented).
- No reaper: `crates/parallax-server/src/serve.rs` spawns worker + receivers + API only (verified by prior recon grep).
- Manual reclaim: `crates/parallax-cli/src/doctor.rs:147` (`prune` truncates via `std::fs::write(path, b"")`).
- Retention config exemplar: `crates/parallax-server/src/config.rs:42-49` + `:95-104` — `[retention]` table already exists for engine TTLs; extend it, don't invent a second config section.
- Worker forward: `crates/parallax-server/src/worker.rs:56-101` — consumes channel, dual-writes raw bytes to GreptimeDB `/v1/otlp`. There is **no replay-from-spool path today** (nothing reads the spool), so rotation cannot break replay — but the spec calls it a future WAL; keep rotated segments readable NDJSON, don't compress.
- Conventions: main-branch commits, `-s`, one Claude co-author trailer; zero-copy ingest law (do not add per-request allocation/cloning on the hot path); nextest; clippy `-D warnings`; CLI long-running commands narrate + ready banner.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| fmt/clippy | `rtk cargo fmt --all -- --check` ; `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| Tests | `rtk cargo nextest run --workspace --all-targets` | pass |
| Integration slice | `rtk cargo nextest run -p parallax-server` | pass (some tests need GreptimeDB; respect existing skip/gating in the m-series tests) |

## Scope

**In scope**: `crates/parallax-storage/src/spool.rs`, `crates/parallax-server/src/serve.rs` (reaper task), `crates/parallax-server/src/config.rs` (`[retention]` keys), `crates/parallax-cli/src/doctor.rs` (report cap + segment count), tests, `docs/research/architecture/v1-implementation-spec.md` (WAL/bounding contract note), user doc touch if `docs/guide/` documents the spool (grep `rg -rn "spool" docs/guide/` — update if present).

**Out of scope**: replay-from-spool implementation (future WAL work); changing write-before-ack; compression; GreptimeDB TTLs.

## Git workflow

- `main`, per-step `git commit -s` (`feat(storage): size-rotate ingest spool`, `feat(server): spool retention reaper`, …), one Claude co-author trailer each.

## Steps

### Step 1: Segment rotation in `Spool::append`

Rotate by size at write time (cheap: `File::metadata()` on the already-open handle, or track running size in the struct to avoid the stat syscall — preferred given the zero-copy hot-path law):

- `Spool` gains `max_segment_bytes: u64` (constructor param; default 64 MiB) and per-signal running sizes (`Mutex` already serializes writers).
- When a write would exceed the cap: rename `logs.ndjson` → `logs.<unix-ts>.ndjson`, open fresh, reset counter. Rename+create under the existing `write_lock`.
- `line_count` counts the active segment only (its two consumers are tests + doctor; doctor gets richer info in Step 3).

**Verify**: unit tests in `spool.rs` (inline `#[cfg(test)]`, matching the repo's inline-test convention): writes crossing the cap produce a rotated segment + small active file; NDJSON lines never split across segments.

### Step 2: Retention reaper task

- `config.rs [retention]`: add `spool_max_total_bytes` (default 512 MiB) and `spool_max_age_hours` (default 72). Follow the existing retention keys' serde/default pattern (`:42-49`).
- New async task spawned in `serve.rs` beside the worker: every 10 min, list `*.ndjson` segments per signal dir, delete rotated (non-active) segments older than `spool_max_age_hours` OR oldest-first while total size > `spool_max_total_bytes`. **Never delete the active segment.** Log one summary line per sweep that reclaimed anything (match the server's existing tracing style — grep `tracing::info!` in `serve.rs` for tone).
- Startup narration: the serve ready banner (see `crates/parallax-cli/src/main.rs:247-262` convention) is CLI-side — no change needed unless serve prints retention config today; if it prints engine TTLs, add the spool line beside them.

**Verify**: reaper unit test with a temp dir (fabricate old segments via `filetime` or by naming timestamp — age check may key off the filename timestamp from Step 1's rename, which avoids a `filetime` dev-dep; choose filename-ts and document it): oversized/old segments removed, active file untouched.

### Step 3: Doctor + prune awareness

`doctor.rs`: report per-signal spool as `active <size> + <n> rotated segments (<total>)` and the configured caps; `prune` (`:147`) also removes rotated segments (glob `*.ndjson` in the spool dir, truncate active as today).

**Verify**: existing doctor tests (grep `rg -n "doctor" crates/parallax-cli/src` for the test module) still pass; extend with a rotated-segment fixture if the test harness supports a temp spool dir.

### Step 4: Spec + docs

`v1-implementation-spec.md`: the spool section gains the bounded-WAL contract: write-before-ack unchanged; segments rotate at `max_segment_bytes`; reaper enforces `[retention] spool_*`; rotated segments are reclaim-eligible immediately because forwarding is synchronous with ingest (channel), and replay — when built — must treat only segments newer than the last engine-ack watermark as replayable (note as open design point for the future WAL, not implemented here).

**Verify**: `rtk cargo nextest run --workspace --all-targets`, fmt, clippy → all green.

## Test plan

Step 1 rotation tests; Step 2 reaper tests (age + total-size policies, active-file immunity); Step 3 doctor/prune extension. Pattern: existing inline tests in `spool.rs`-adjacent storage modules and `commands.rs` unit tests.

## Done criteria

- [ ] Writes rotate at the cap; NDJSON integrity preserved (tests)
- [ ] Reaper enforces both `[retention] spool_*` keys; active segment never deleted (tests)
- [ ] `parallax doctor` shows spool caps + segment counts; `prune` clears rotated segments
- [ ] Spec updated; fmt/clippy/nextest green
- [ ] `plans/README.md` row added/updated for 020

## STOP conditions

- Anything actually READS the spool today (a replay path the recon missed — re-grep `rg -n "Spool" crates/ --type rust` and check every non-append call): rotation semantics then need the replay design first.
- The m-series integration tests assert spool file names/paths (grep `rg -rn "ndjson" crates/parallax-server/tests/`) — reconcile fixtures before renaming behavior.
- Write-path size tracking measurably regresses ingest throughput in existing benches (if any exist — grep `benches/`): report numbers.

## Maintenance notes

- When the replay/WAL work lands, the reaper must honor the ack watermark (spec note from Step 4) — that is the interaction to re-review.
- Defaults (64 MiB segment / 512 MiB total / 72 h) are guesses sized to the observed 1.65 GiB pathology; tune from doctor output after a week of real use.
- jackin-side volume reduction (its plans 002/005/008) will shrink spool pressure independently; don't let that mask a reaper bug — test with synthetic load.
