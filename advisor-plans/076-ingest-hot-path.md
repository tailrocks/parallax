# Plan 076: Cut ingest hot-path overhead — spool lock/IO discipline, batched issue upserts, normalize allocation churn

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- crates/parallax-storage/src/spool.rs crates/parallax-storage/src/metadata.rs crates/parallax-core/src/normalize.rs crates/parallax-server/src/worker.rs`
> Plans 070/073 legitimately edit `worker.rs`/`metadata.rs` first; verify the
> excerpts below still match the shape of the live code before proceeding.

## Status

- **Priority**: P2
- **Effort**: M-L
- **Risk**: MED (touches the ack path and the shared metadata connection)
- **Depends on**: 073 (worker retry shape settles first), 070
- **Category**: perf
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

The operator design rule for ingest is "decode once, move ownership forward,
never clone telemetry on the hot path." Three measured violations/costs:

1. **Spool append** (runs before every OTLP ack): serializes the entire
   decoded batch to JSON, then — while holding ONE global async mutex shared
   by traces+logs+metrics — opens the file and performs blocking `write_all`
   syscalls on a Tokio runtime thread. Concurrent HTTP and gRPC exports
   serialize against each other's disk writes.
2. **Issue upserts**: each derived error event costs 3-4 sequential Turso
   statements, each behind the single metadata connection lock that ALL
   GraphQL reads share — an error burst stalls both ingest and the console.
3. **normalize.rs**: per-byte `format!("{b:02x}")` for every id (~24+ tiny
   String allocations per span) and a deep `resource_json.clone()` per
   span/log row — thousands of redundant JSON tree clones for a batch sharing
   one resource.

The gRPC-side `encode_to_vec` re-encode is a DOCUMENTED exception
(`otlp_grpc.rs:52-55` — framing difference) and stays.

## Current state

- `crates/parallax-storage/src/spool.rs:126-146` — `append`:

  ```rust
  pub async fn append<T: Serialize>(&self, signal: Signal, request: &T) -> anyhow::Result<()> {
      let line = serde_json::to_string(request)?;
      let write_len = u64::try_from(line.len().saturating_add(1)).unwrap_or(u64::MAX);
      let path = self.dir.join(signal.file_name());
      let mut sizes = self.sizes.lock().await;
      if sizes.get(signal) > 0
          && sizes.get(signal).saturating_add(write_len) > self.max_segment_bytes
      {
          self.rotate_active(signal)?;
          sizes.set(signal, 0);
      }
      let mut file = std::fs::OpenOptions::new()
          .create(true)
          .append(true)
          .open(path)?;
      file.write_all(line.as_bytes())?;
      file.write_all(b"\n")?;
      let next_size = sizes.get(signal).saturating_add(write_len);
      sizes.set(signal, next_size);
      Ok(())
  }
  ```

  `sizes` is one `tokio::sync::Mutex<SegmentSizes>` covering all three
  signals. Callers: `otlp_http.rs:50`, `otlp_grpc.rs:62-66` — both on the
  pre-ack path. Note the JSON serialize happens BEFORE the lock already —
  the lock-held work is rotate + open + write.

- `crates/parallax-storage/src/metadata.rs:137-141`:

  ```rust
  pub struct MetadataStore {
      /// Turso forbids concurrent statement use on one connection; the worker
      /// upserts while the API reads, so every operation takes this lock.
      conn: tokio::sync::Mutex<turso::Connection>,
  }
  ```

  `upsert_issue_occurrence` (`:158-216`): per call takes the lock, runs
  INSERT issues (ON CONFLICT), INSERT issue_buckets (ON CONFLICT), SELECT
  tags, UPDATE tags — with a documented turso constraint that the SELECT's
  statement must drop before the UPDATE (`:194-197`).

- `crates/parallax-server/src/worker.rs:140-161` — `record_errors` calls
  `upsert_issue_occurrence` in a `for event in &errors` loop (errors already
  deduped by `dedup_error_events`), then `write_error_events` once.

- `crates/parallax-core/src/normalize.rs:14-16`:

  ```rust
  pub fn hex(bytes: &[u8]) -> String {
      bytes.iter().map(|b| format!("{b:02x}")).collect()
  }
  ```

  Called per span for trace/span/parent ids, per link, per log, per exemplar.
  And per-row resource clones — spans at `:213` `resource: resource_json.clone()`,
  logs at `:265` (same pattern).

- `SpanRow`/`LogRow` are defined in `crates/parallax-storage/src/model.rs`
  with `resource: serde_json::Value` fields; consumers include both storage
  adapters and `parallax-api` serialization (`.to_string()` on attributes /
  resource in resolvers).

- normalize.rs has a strong inline test module (per the repo audit) — it will
  catch regressions.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Build | `rtk cargo build --workspace` | exit 0 |
| Core tests | `rtk cargo nextest run -p parallax-core` | all pass |
| Storage tests | `rtk cargo nextest run -p parallax-storage` | all pass |
| Full suite | `rtk cargo nextest run --workspace` | all pass |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |

## Scope

**In scope** (the only files you should modify):
- `crates/parallax-storage/src/spool.rs`
- `crates/parallax-storage/src/metadata.rs`
- `crates/parallax-server/src/worker.rs` (call the batch API)
- `crates/parallax-core/src/normalize.rs`
- `crates/parallax-storage/src/model.rs` (ONLY if Step 4's Arc variant is
  chosen — see its STOP condition)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- Spool FORMAT changes (NDJSON→binary raw bytes) — entangled with the
  deferred WAL/replay design (Plan 073's maintenance note); the reaper,
  `doctor`, and `line_count` all assume NDJSON lines.
- `otlp_grpc.rs` re-encode — documented exception.
- Worker parallelism/pooling.
- `memory.rs`/`greptime.rs` query methods (Plans 074/075 own those).

## Git workflow

- Work directly on `main` (repo rule — `BRANCHING.md`).
- Conventional Commits, DCO signoff (`git commit -s`), trailer
  `Co-authored-by: Claude <noreply@anthropic.com>`.

## Steps

### Step 1: Per-signal spool locks + off-runtime writes

In `spool.rs`:

1. Replace the single `sizes: Mutex<SegmentSizes>` with a per-signal
   structure, e.g. `[Mutex<SignalState>; 3]` indexed by signal, where
   `SignalState { size: u64 }` (check how `SegmentSizes`, `rotate_active`,
   `reap`, and `line_count` use the current struct and adapt — `reap` scans
   the directory independently, so it mostly needs a way to reset a signal's
   size; keep its behavior identical).
2. Move the open+write into `tokio::task::spawn_blocking` (or keep sync but
   document why — measure nothing here; spawn_blocking is the requested
   shape). The lock must still serialize appends *per signal* to keep line
   atomicity and size accounting exact: acquire the signal's async lock,
   then run the rotate-check + write inside `spawn_blocking` while the lock
   is held across the await (a `tokio::sync::Mutex` guard can live across
   `.await` — that is why it's the async mutex).
3. Keep the file open handle cached per signal inside `SignalState`
   (open once, reuse; reopen after rotation) — removes the per-append
   `OpenOptions::open` syscall.

Behavioral invariants to preserve (the existing spool tests assert them):
rotation at `max_segment_bytes`, NDJSON one-line-per-request, `line_count`,
reap by age/total size.

**Verify**: `rtk cargo nextest run -p parallax-storage spool` → all existing
spool tests pass unchanged.

### Step 2: Batched issue upserts

In `metadata.rs`, add:

```rust
pub async fn upsert_issue_occurrences(&self, occurrences: &[IssueOccurrence<'_>]) -> anyhow::Result<()>
```

acquiring `self.conn.lock().await` ONCE, then for each occurrence running the
same statement sequence as today. Group the tag-cache read-merge-write by
fingerprint: for occurrences sharing a fingerprint, SELECT tags once, merge
all attribute sets, UPDATE once. Preserve the documented turso constraint
(drop the SELECT statement before the UPDATE — same block structure as
`:198-206`).

Keep `upsert_issue_occurrence` delegating to the batch fn with a one-element
slice (API compatibility; its existing tests keep passing).

In `worker.rs` `record_errors`, replace the loop with one call:
`self.metadata.upsert_issue_occurrences(&occurrences).await?` (build the
occurrence vec first; lifetimes: `IssueOccurrence` borrows from `errors` —
build `Vec<IssueOccurrence>` in the same scope, it already works per-item so
the borrow structure is unchanged).

**Verify**: `rtk cargo nextest run -p parallax-storage metadata` → existing
upsert tests pass; add one new test: batch of 3 occurrences (two sharing a
fingerprint) → `event_count` 2 and 1 respectively, buckets correct, tags
merged once.

### Step 3: Allocation-light hex

Replace `hex()` with a single-allocation nibble encoder:

```rust
pub fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}
```

**Verify**: `rtk cargo nextest run -p parallax-core` → existing normalize
tests pass (they assert hex output for known ids). Add a test comparing
`hex(&[0x00, 0xff, 0x1a])` → `"00ff1a"` if not already covered.

### Step 4: Share the resource JSON across a batch's rows

Preferred shape: change `SpanRow.resource` / `LogRow.resource` in
`crates/parallax-storage/src/model.rs` from `serde_json::Value` to
`std::sync::Arc<serde_json::Value>` and replace the per-row
`resource_json.clone()` with `Arc::clone(&resource_json)` in `normalize.rs`
(`:213`, `:265`, and the metrics path if it has the same pattern).

BEFORE committing to this: run `rtk cargo check --workspace` after the type
change and count the fallout. Expected adjustments: `memory.rs` filters/reads
(deref), `greptime.rs` insert builders (`.to_string()` works on Arc via
Deref), API resolvers (Deref). If the fallout exceeds ~25 edit sites or
requires changing any public GraphQL shape, STOP and report — the alternative
(leave `Value` but intern nothing) is a valid "not worth it" outcome; record
it.

`serde::Serialize` for `Arc<Value>` works out of the box (serde's `rc` is not
needed for Arc<T: Serialize>? — it IS gated: serde requires the `rc` feature
for Arc. Check `Cargo.toml`: if `serde` lacks `features = ["rc"]`, either add
`rc` to the workspace serde features or serialize via `&*value`. Prefer
adding the feature; it changes no semantics for these read-only trees).

**Verify**: `rtk cargo nextest run --workspace` → all pass;
`rtk cargo clippy --workspace --all-targets` → zero warnings.

### Step 5: Full gates

**Verify**: `rtk cargo fmt --all`; clippy zero warnings; full nextest pass.
If a local `parallax serve` + telemetry source is available, sanity-run
ingest; otherwise state that only the automated gates ran.

## Test plan

- Spool: existing rotation/reap/line_count tests must pass unchanged; add one
  test asserting two different signals can append without corrupting each
  other's sizes (sequential calls, assert per-signal sizes).
- Metadata: new batch-upsert test (Step 2).
- Normalize: hex round-trip test; existing suite.
- No throughput benchmark is claimed — the repo's bench discipline (four-build
  matrix) doesn't cover ingest; if you measure locally, put numbers in the
  commit message, not in docs.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `grep -n "spawn_blocking" crates/parallax-storage/src/spool.rs` → ≥1 match
- [ ] `grep -n "upsert_issue_occurrences" crates/parallax-storage/src/metadata.rs crates/parallax-server/src/worker.rs` → ≥2 matches
- [ ] `grep -n "format!(\"{b:02x}\")" crates/parallax-core/src/normalize.rs` → 0 matches
- [ ] `rtk cargo nextest run --workspace` exits 0 (incl. new tests)
- [ ] `rtk cargo clippy --workspace --all-targets` → zero warnings
- [ ] `git status` shows no modified files outside the in-scope list (model.rs
      only if Step 4 chosen)
- [ ] `advisor-plans/README.md` status row updated (incl. Step 4 outcome —
      done or recorded as not-worth-it)

## STOP conditions

Stop and report back (do not improvise) if:

- Holding the per-signal lock across `spawn_blocking` proves impossible
  without an ownership restructure of `Spool` (e.g. `rotate_active` needs
  `&self` + the guard) — report the actual structure.
- Step 4's Arc fallout exceeds ~25 edit sites or touches GraphQL shapes.
- The batch upsert hits turso's statement constraints in a way the
  existing comment doesn't cover (new "reports success but does not persist"
  behavior) — this store has a documented lost-update hazard
  (`metadata.rs:194-197`); any new instance is a finding to report.
- Any existing spool/metadata test needs its ASSERTIONS weakened — that means
  behavior changed, which this plan forbids.

## Maintenance notes

- The spool stays NDJSON; when the WAL/replay design lands (deferred from
  Plan 073), revisit format (raw protobuf frames) and fold the per-signal
  handles into it.
- A dedicated writer connection for metadata (separate from API reads) is the
  next step if lock contention still shows; deferred because turso
  multi-connection semantics on one local file need verification first.
- Reviewer: confirm ack-path ordering unchanged (spool append still completes
  before the 200), and no new telemetry clones on the first-attempt path.
