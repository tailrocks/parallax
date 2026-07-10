# Plan 073: Make the ingest pipeline's durability story true — retry transient failures, drain on shutdown, re-scope the spool doc

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- crates/parallax-server/src/worker.rs crates/parallax-server/src/serve.rs crates/parallax-storage/src/spool.rs`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED (changes ingest failure semantics; retry must not wedge the worker)
- **Depends on**: 070 (the `first_seen` MIN fix makes retried upserts safe;
  the char-boundary panic fix removes a known worker-killer). Execute 070 first.
- **Category**: bug / correctness
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

OTLP exporters treat a 200 response as durable delivery. Parallax acks before
processing: the endpoint appends the batch to a disk spool, queues it to a
single in-process worker, and returns 200. But:

1. The worker **drops the batch on any processing error** (one `tracing::error!`
   and move on) — a transient GreptimeDB or Turso hiccup permanently loses
   acked telemetry.
2. On shutdown every task is **aborted**, dropping whatever is buffered in the
   1024-slot channel.
3. The spool that exists to prevent exactly this is **write-only** — nothing
   ever reads it back. Its module doc claims "M1's workers consume from here
   into the storage engine", which is false and misleads every future reader.

A full write-ahead-log replay (offset tracking, idempotent redelivery) is a
bigger design; this plan makes the current behavior honest and removes the two
cheap loss windows (transient errors, shutdown), and re-scopes the spool's
documented role to what it actually is.

## Current state

- `crates/parallax-server/src/worker.rs:57-64` — drop-on-error loop:

  ```rust
  /// Drain the channel until all senders drop.
  pub async fn run(mut self, mut receiver: mpsc::Receiver<IngestItem>) {
      while let Some(item) = receiver.recv().await {
          if let Err(e) = self.process(item).await {
              tracing::error!("ingest worker item failed: {e:#}");
          }
      }
  }
  ```

  `process` (`worker.rs:66-115`) handles `Traces`/`Logs`/`Metrics`: normalize →
  `register_runs` (Turso) → live broadcast → `store.ingest_*` (GreptimeDB
  forward) → `record_errors` (Turso + GreptimeDB), each step `?`-propagating.
  A failure mid-sequence leaves earlier writes in place (non-atomic by
  design); a retry of the whole item re-runs earlier steps, so retry safety
  depends on their idempotence — see Step 1 notes.

- `crates/parallax-server/src/serve.rs:37-47` — shutdown aborts tasks:

  ```rust
  pub fn shutdown(&self) {
      if let Some(supervisor) = &self.supervisor {
          supervisor.stop();
      }
      for task in &self.tasks {
          task.abort();
      }
  }
  ```

  The worker task is `tasks[0]` (`serve.rs:287` `tokio::spawn(worker.run(receiver))`);
  the ingest channel is `worker::channel(1024)` (`serve.rs:279`).

- `crates/parallax-storage/src/spool.rs:1-6` — the stale module doc:

  ```rust
  //! The ingest spool: an NDJSON landing zone for raw OTLP export requests.
  //!
  //! M0's durability story: every accepted OTLP request is appended to a
  //! per-signal NDJSON file before the ingest endpoint acknowledges it. M1's
  //! workers consume from here into the storage engine; the spool then becomes
  //! the bounded WAL described in the implementation spec.
  ```

  Reality: appended by `otlp_http.rs:50` / `otlp_grpc.rs:63` and reaped by
  `spawn_spool_reaper` (`serve.rs:288-292`); no consumer exists anywhere.

- Ack path (`crates/parallax-server/src/otlp_http.rs:50-70`): spool append →
  channel send → 200. Same shape in `otlp_grpc.rs`.

- Idempotence facts relevant to retry (verified at planning):
  - `register_runs` upserts run rows — safe to repeat.
  - `ingest_traces`/`ingest_logs`/`ingest_metrics` forward raw OTLP to
    GreptimeDB's native endpoint — re-forwarding duplicates rows (native
    tables have no dedup) → retry must be bounded and only for failures
    *before* any partial success is ambiguous. This is why Step 1 retries the
    whole item a small number of times and then gives up loudly, rather than
    retrying forever.
  - `record_errors` re-upsert increments `event_count` again → a retry after
    `record_errors` partially succeeded can inflate counts. Acceptable at
    bounded retry counts; note it in the code comment.

- Conventions: `anyhow` errors; `tracing` for logs; tests are integration
  tests under `crates/parallax-server/tests/` using `storage.mode = "none"`
  (MemoryStore) — see `m1_pipeline.rs` for the boot-and-ingest pattern.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Build | `rtk cargo build --workspace` | exit 0 |
| Server tests | `rtk cargo nextest run -p parallax-server` | all pass |
| Full suite | `rtk cargo nextest run --workspace` | all pass |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |

## Scope

**In scope** (the only files you should modify):
- `crates/parallax-server/src/worker.rs`
- `crates/parallax-server/src/serve.rs`
- `crates/parallax-storage/src/spool.rs` (doc comment only)
- `crates/parallax-server/tests/` (one new test file or extension)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- Building spool replay/offset tracking — that is the deferred WAL design;
  this plan only documents its absence honestly.
- `crates/parallax-storage/src/metadata.rs` — occurrence idempotence keys
  (`(trace_id, span_id, fingerprint)` table) are a known deferred schema
  change.
- The OTLP endpoints' ack ordering (spool-then-ack stays as is).
- Worker parallelism/pooling (perf, Plan 076 territory).

## Git workflow

- Work directly on `main` (repo rule — `BRANCHING.md`).
- Conventional Commits, DCO signoff (`git commit -s`), trailer
  `Co-authored-by: Claude <noreply@anthropic.com>`. E.g.
  `fix(server): retry transient ingest failures and drain on shutdown`.

## Steps

### Step 1: Bounded retry in the worker loop

In `worker.rs`, wrap `process` in a small retry (3 attempts, 100ms/500ms/2s
backoff — constants at module top). Retry the WHOLE item; after the final
failure, log at `error!` with a distinct marker so operators can grep it:

```rust
const INGEST_RETRIES: usize = 3;
const INGEST_BACKOFF: [Duration; 3] = [/* 100ms, 500ms, 2s */];

pub async fn run(mut self, mut receiver: mpsc::Receiver<IngestItem>) {
    while let Some(item) = receiver.recv().await {
        let mut attempt = 0;
        loop {
            match self.process(&item).await {
                Ok(()) => break,
                Err(e) if attempt < INGEST_RETRIES => {
                    attempt += 1;
                    tracing::warn!("ingest attempt {attempt} failed, retrying: {e:#}");
                    tokio::time::sleep(INGEST_BACKOFF[attempt - 1]).await;
                }
                Err(e) => {
                    tracing::error!("ingest item DROPPED after {INGEST_RETRIES} retries: {e:#}");
                    break;
                }
            }
        }
    }
}
```

This requires `process(&item)` to take a reference (or the item to be
`Clone`). Check `IngestItem`'s definition in `worker.rs`: its variants hold
prost request types + `Bytes`, both cheaply clonable — either borrow or clone
per attempt; prefer changing `process(&mut self, item: &IngestItem)` and
cloning the request only where ownership is needed (`Bytes` clone is
zero-copy). Do NOT deep-clone telemetry more than the current code already
does per attempt beyond the retry path (zero-copy ingest is an operator design
rule; retries are the exception, first attempt must not add clones).

Add a code comment stating the known double-count caveat: a retry after a
partial `record_errors` success can increment `event_count` twice; bounded at
3 attempts; durable idempotence keys are the deferred fix.

**Verify**: `rtk cargo build -p parallax-server` → exit 0;
`rtk cargo nextest run -p parallax-server` → existing suites pass.

### Step 2: Drain the ingest channel on shutdown

In `serve.rs`, make shutdown graceful for the worker specifically:

1. Keep a dedicated handle for the worker task separate from the listener
   tasks (a `worker_task: JoinHandle<()>` field on `ServerHandle` instead of
   inside `tasks`), and hold the `sender` side's lifetime such that shutdown
   can drop it: store the `IngestState`'s sender is cloned into the axum/grpc
   state — the practical drain is: `shutdown()` aborts listener tasks first
   (no new sends), then drops/closes its own sender clone if `ServerHandle`
   holds one, then `await`s the worker task with a timeout.

2. Since `shutdown(&self)` is sync today, add an async variant and keep the
   sync one delegating with a bounded wait:

   ```rust
   pub async fn shutdown_graceful(self) {
       if let Some(supervisor) = &self.supervisor { supervisor.stop(); }
       for task in &self.listener_tasks { task.abort(); }
       // receiver ends when all senders drop; listener tasks owned the senders
       let _ = tokio::time::timeout(Duration::from_secs(5), self.worker_task).await;
   }
   ```

   Adjust to the real ownership you find: the key invariant is that after
   listener abort, no sender clones remain alive except ones this handle can
   drop, so `worker.run`'s `while let Some(...)` loop terminates naturally
   after draining the buffer.

3. Update the `parallax serve` shutdown path (find the ctrl-c/signal handler
   in `serve.rs` or `crates/parallax-cli`) to call the graceful variant. Keep
   the abrupt `shutdown()` for test teardown (tests rely on it being fast).

**Verify**: `rtk cargo nextest run -p parallax-server` → all pass (the
existing tests use the abrupt path and must be unaffected).

### Step 3: Tell the truth in the spool doc

Replace `spool.rs:1-6` module doc with reality:

```rust
//! The ingest spool: an NDJSON landing zone for raw OTLP export requests.
//!
//! Every accepted OTLP request is appended here before the ingest endpoint
//! acknowledges it. Nothing reads the spool back today: it is a diagnostic
//! record and crash-forensics trail, reaped by size/age (`reap`), NOT a
//! write-ahead log. If the worker drops an item after retries (see
//! `parallax-server::worker`), the data survives only here. Replay/WAL
//! semantics are a deferred design — do not claim durability beyond this.
```

**Verify**: `rtk cargo doc -p parallax-storage --no-deps` → exit 0 (doc builds).

### Step 4: Regression test for retry

Add an integration test (new file `crates/parallax-server/tests/m1_retry.rs`
or extend `m1_pipeline.rs`, following its boot pattern with
`storage.mode = "none"`): this requires a store that fails N times. The
MemoryStore has no fault injection — so test at the worker level instead:
construct a `Worker` with a small `TelemetryStore` test double (implement the
trait in the test file, delegating to `MemoryStore` but failing the first 2
`ingest_logs` calls), send one Logs item through `worker::channel`, run the
worker, assert the logs ARE stored (retry succeeded) and the double recorded 3
attempts.

If implementing the 38-method trait in a test is prohibitive, wrap
`MemoryStore` in a struct that delegates every method via a macro or by
`Deref`-style forwarding — if that still proves unreasonable (>~150 lines of
boilerplate), STOP and report; do not ship the retry untested.

**Verify**: `rtk cargo nextest run -p parallax-server m1_retry` (or the
extended file) → passes; total suite passes.

### Step 5: Full gates

**Verify**: `rtk cargo fmt --all`;
`rtk cargo clippy --workspace --all-targets` → zero warnings;
`rtk cargo nextest run --workspace` → all pass.

## Test plan

- Worker-level retry test (Step 4): transient failure ×2 then success →
  data stored, 3 attempts observed; permanent failure (always-fail double) →
  item dropped after 3 retries, loop continues with next item (send two items,
  assert the second lands).
- Existing `m1_pipeline.rs` / `m2_*` suites unchanged and green (they assert
  the happy path end-to-end).
- Shutdown drain has no deterministic test harness today (abrupt path is used
  in tests); verified by code review + the timeout bound. Note this in the
  commit message.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `grep -n "INGEST_RETRIES" crates/parallax-server/src/worker.rs` → ≥2 matches
- [ ] `grep -n "DROPPED after" crates/parallax-server/src/worker.rs` → 1 match
- [ ] `grep -n "workers consume from here" crates/parallax-storage/src/spool.rs` → 0 matches
- [ ] `grep -n "NOT a" crates/parallax-storage/src/spool.rs` → 1 match (re-scoped doc)
- [ ] `grep -n "shutdown_graceful" crates/parallax-server/src/serve.rs` → ≥1 match
- [ ] `rtk cargo nextest run --workspace` exits 0 incl. the new retry test
- [ ] `rtk cargo clippy --workspace --all-targets` → zero warnings
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- `IngestItem` variants are not cheaply re-processable (something in `process`
  consumes the item in a way that forces a deep telemetry clone per attempt) —
  that collides with the zero-copy ingest rule; report the conflict.
- The sender/receiver ownership in `serve.rs` doesn't allow the drain shape in
  Step 2 without restructuring `IngestState` across otlp_http/otlp_grpc —
  report the actual ownership graph and a proposal instead of refactoring
  broadly.
- The trait-double in Step 4 exceeds ~150 lines of boilerplate.
- Any existing integration test becomes flaky (they use real sleeps; retry
  backoff adds latency — if a bounded poll times out, report rather than
  raising its cap blindly).

## Maintenance notes

- The real WAL (spool replay with offsets + durable occurrence idempotence
  keys) remains open; this plan's honest doc + bounded retry is the interim.
  When replay is designed, the retry loop in Step 1 becomes its in-memory
  fast path, and Bug 3 of Plan 070 (`first_seen` MIN) plus a
  `(trace_id, span_id, fingerprint)` seen-table become prerequisites.
- If worker parallelism lands later (perf), the retry loop must move into the
  per-item pipeline, and ordering assumptions in `record_errors` should be
  re-checked.
- Reviewer: scrutinize that the first-attempt path adds zero new telemetry
  copies (operator zero-copy rule), and that shutdown cannot hang longer than
  the 5s timeout.
