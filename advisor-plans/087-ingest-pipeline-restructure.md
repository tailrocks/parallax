# Plan 087: Restructure the ingest pipeline — gzip OTLP/HTTP, per-signal workers, stop building discarded batches, raw-bytes spool, bounded queue memory

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat df81d86..HEAD -- crates/parallax-server/src/otlp_http.rs crates/parallax-server/src/otlp_grpc.rs crates/parallax-server/src/worker.rs crates/parallax-server/src/serve.rs crates/parallax-storage/src/spool.rs crates/parallax-storage/src/adapter.rs crates/parallax-core/src/normalize.rs`
> Plans 070/073/076 legitimately edit worker.rs/spool.rs first — verify the
> excerpts below still match before proceeding.

## Status

- **Priority**: P1 (Step 1 alone is a data-loss interop bug) / P2 (the rest)
- **Effort**: L
- **Risk**: MED-HIGH (ack-vs-durability semantics; spool format change touches replay)
- **Depends on**: 073 (worker retry + spool replay semantics settle first), 076 (spool locks/handles + batched upserts land first — this plan builds on both)
- **Category**: perf (Step 1: bug)
- **Planned at**: commit `df81d86`, 2026-07-10

## Why this matters

The ingest design rule is "decode once, move ownership forward, never clone
telemetry on the hot path." The 2026-07-10 audit traced the live path and
found: (a) OTLP/HTTP exporters that enable gzip (a standard option;
`OTEL_EXPORTER_OTLP_COMPRESSION=gzip`) get HTTP 400 and silently lose data —
the OTLP spec requires servers to accept gzip, Parallax's own gRPC listener
and GreptimeDB's OTLP endpoint both do; (b) one worker task drains all three
signals FIFO, so a slow traces forward stalls logs+metrics acks behind it;
(c) for traces on the GreptimeDB path, the fully-normalized `Vec<SpanRow>` is
built per batch and then DISCARDED by the store (its parameter is `_spans`) —
2-3 full passes over every batch, per-span JSON materialization, all wasted
unless a live tail happens to be attached; (d) the spool serializes the whole
decoded batch to JSON pre-ack when the raw protobuf bytes are already in hand;
(e) each queued item holds decoded request + raw bytes (≈2× batch memory)
in a 1024-deep channel with no operator-tunable body/queue limits.

## Current state

- `crates/parallax-server/src/otlp_http.rs:27-71` — `ingest<R>()`: raw
  `Bytes` → `R::decode(body)` with NO Content-Encoding handling; no
  decompression layer in `otlp_http::router` (`serve.rs:340,363`). Error path
  returns `400 invalid OTLP protobuf body`.
- `crates/parallax-server/src/otlp_grpc.rs:32-50` — gRPC accepts gzip
  (`accept_compressed(CompressionEncoding::Gzip)`) — the asymmetry.
- `crates/parallax-server/src/worker.rs`:
  - `:20-24` `IngestItem::Traces(ExportTraceServiceRequest, bytes::Bytes)` —
    decoded + raw both queued.
  - `:58-64` single `run()` loop, strictly sequential `process(item).await`.
  - `:66-115` `process`: traces branch runs `normalize::normalize_traces(&request)`
    (full pass 1) + `derive::derive_from_traces(&request)` (full pass 2) +
    `register_runs(spans.iter()…)` (pass 3), THEN
    `self.store.ingest_traces(spans, raw)` — and the GreptimeDB impl signature
    is `ingest_traces(&self, _spans: Vec<SpanRow>, raw: Bytes)`
    (`greptime.rs:890`): the spans are dropped. `derive.rs:110` re-runs
    `attributes_to_json(&span.attributes)` for error spans (already
    materialized in pass 1).
  - `:77-80` live gating exists (`receiver_count() > 0`) but `spans.clone()`
    deep-copies the batch before `.into()` even though the store then ignores
    the original.
  - `:119-138` `register_runs` needs only `(run_id, ts)` pairs — run id comes
    from RESOURCE attributes (one per ResourceSpans group), not per-span data.
- `crates/parallax-server/src/serve.rs:279` — `worker::channel(1024)`,
  hardcoded; `:285-287` one worker task; `:365-368` tonic server built with
  defaults (no `max_decoding_message_size`); axum routes use the default 2 MB
  body limit (no `DefaultBodyLimit` anywhere).
- `crates/parallax-storage/src/spool.rs:126-146` — `append` does
  `serde_json::to_string(request)` of the decoded batch pre-ack (Plan 076
  fixes locks/handles/blocking-IO but explicitly leaves the NDJSON JSON format
  out of scope; the format is THIS plan's Step 4).
- `crates/parallax-storage/src/adapter.rs` — `TelemetryStore::ingest_traces/
  ingest_logs/ingest_metrics` take `(Vec<Row…>, Bytes)`.
- `worker.rs:37,52,135` — `seen_runs: HashSet<String>` grows unbounded.
- Engine fact (verified): GreptimeDB's OTLP endpoint accepts gzip request
  bodies (tower-http RequestDecompressionLayer at v1.1.2, `src/servers/src/http.rs:1139-1155`),
  and OTLP-over-gRPC was REMOVED upstream (PR #5605) — the HTTP forward is the
  only path, so the forward itself is already correct.

Conventions: strict clippy, cargo-nextest, `rtk` prefix, Conventional Commits
+ DCO + `Co-authored-by: Claude <noreply@anthropic.com>`, direct on `main`.
Progress-visibility rule: any new limits must produce a clear log line when
hit.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Build | `rtk cargo build --workspace` | exit 0 |
| Server tests | `rtk cargo nextest run -p parallax-server` | all pass |
| Storage tests | `rtk cargo nextest run -p parallax-storage` | all pass |
| Full suite | `rtk cargo nextest run --workspace` | all pass |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| gzip smoke (serve running) | `printf '' \| gzip \| curl -s -o /dev/null -w '%{http_code}' -XPOST http://127.0.0.1:4318/v1/traces -H 'content-type: application/x-protobuf' -H 'content-encoding: gzip' --data-binary @-` | `200` |

## Scope

**In scope**:
- `crates/parallax-server/src/{otlp_http.rs,otlp_grpc.rs,worker.rs,serve.rs,config.rs}`
- `crates/parallax-storage/src/{spool.rs,adapter.rs,greptime.rs,memory.rs}`
  (adapter/greptime/memory: ONLY the ingest-method signatures + spool format)
- `crates/parallax-core/src/normalize.rs` (ONLY adding the light run-id/resource extraction helper)
- `advisor-plans/README.md`

**Out of scope**:
- Read paths anywhere (Plans 075/085/086).
- Issue upsert internals (076 owns), spool lock structure (076 owns).
- Multi-process or persistent-queue designs — per-signal tasks only.
- The live SSE payload type — Plan 086 Step 6 owns it; if it landed, keep its
  `LiveLogBatch` shape working.

## Git workflow

Direct on `main`; Conventional Commits + `git commit -s` + Claude trailer.
Step 1 is its own commit (it is a user-visible bug fix):
`fix(otlp): accept gzip-compressed OTLP/HTTP bodies`.

## Steps

### Step 1: Accept gzip on OTLP/HTTP (+ explicit body limits on both transports)

- Add gzip request decompression to the OTLP/HTTP router. Options: tower-http's
  `RequestDecompressionLayer` (check whether workspace `tower-http` version
  ships the `decompression-gzip` feature; the workspace pins tower-http 0.7 —
  feature `decompression-gzip` exists there) or manual: read `content-encoding`
  in `ingest<R>()` and gunzip via `flate2` before `R::decode`. Prefer the
  layer. IMPORTANT: the spool and the forward must receive the DECOMPRESSED
  bytes (the layer guarantees this by construction).
- Add explicit, config-backed size limits, matching across transports:
  `[limits] otlp_max_body_bytes` (default 16 MiB — larger than both framework
  defaults, explicit and documented): axum `DefaultBodyLimit::max(n)` on the
  OTLP routes; tonic `.max_decoding_message_size(n)` on the three services.
  Log a WARN with the payload size when a request is rejected.

**Verify**: gzip smoke command → `200` (empty request decodes as empty
protobuf; acceptable smoke). New integration test in `parallax-server` tests:
gzip-compress a small `ExportTraceServiceRequest`, POST it, assert 200 and the
item reaches the worker (existing test harness patterns in
`crates/parallax-server/tests/`). `rtk cargo nextest run -p parallax-server` →
all pass.

### Step 2: Per-signal workers

Replace the single channel+worker with three (traces/logs/metrics):
`worker::channels(buffer_per_signal)` returning a small struct; spawn three
`Worker::run` tasks (Worker is already `store`+`metadata`+`live` — make it
`Clone`-able or construct three). `IngestState.sender` becomes per-signal
senders (the receivers already know their signal). Channel buffer: move
`1024` to `config.rs` (`[limits] ingest_queue_batches`, default 256 per
signal — smaller per-signal default keeps worst-case memory similar).

Semantics preserved: per-signal FIFO ordering (unchanged — ordering across
signals was never guaranteed); ack-after-spool unchanged (spool append happens
in the receivers, before send).

`seen_runs`: worker instances must share it now — wrap in
`Arc<tokio::sync::Mutex<…>>` AND bound it (simple cap: if len > 100_000, clear
— `ensure_run` is idempotent; a comment explains).

**Verify**: full suite passes; new test: enqueue a slow-store traces item
(memory-store test double with a delay) and a logs item; assert the logs item
completes without waiting for traces (tokio::time::pause-based test, model on
existing worker tests at `worker.rs:198+`).

### Step 3: Stop building batches the store discards

Change the ingest trait so adapters pull what they need instead of receiving
pre-chewed rows:

```rust
// adapter.rs
async fn ingest_traces(&self, request: &ExportTraceServiceRequest, raw: bytes::Bytes) -> anyhow::Result<()>;
// same shape for logs; metrics keeps its normalized extras (see below)
```

- `greptime.rs`: ignores `request`, forwards `raw` — unchanged behavior, no
  wasted normalization.
- `memory.rs`: calls `normalize::normalize_traces(request)` itself (it is the
  only consumer of the rows).
- `worker.rs` traces branch becomes:
  1. `let errors = derive::derive_from_traces(&request)` (still needs the raw
     request — it reads span events),
  2. run-id registration via a NEW light helper
     `normalize::resource_run_ids(&request) -> impl Iterator<Item = (String, u128)>`
     that walks ONLY `resource_spans[].resource.attributes` + first-span ts
     (add to normalize.rs next to `normalize_traces`; ~20 lines; unit-test it),
  3. live tee: `if receiver_count() > 0 { let spans = normalize::normalize_traces(&request); … }`
     — normalization now happens ONLY when a tail is attached, and the
     `.clone()` disappears (build once, send the Arc, done),
  4. `store.ingest_traces(&request, raw)`.
- Logs branch: same restructure (`normalize_logs` only when live subscribers
  exist or the memory adapter runs — the adapter change covers the latter).
  NOTE: `derive::derive_from_logs(&logs)` currently takes NORMALIZED logs —
  check its signature; if it needs `Vec<LogRow>`, keep normalizing on the logs
  path for now and note it (logs normalization is lighter than traces), OR
  refactor derive to take the raw request — choose the smaller diff, record
  the choice.
- Metrics branch unchanged (its normalized points/exemplars feed extension
  tables — real consumers).
- Adapter trait now needs `parallax-proto` types — parallax-storage already
  depends on parallax-proto? CHECK `crates/parallax-storage/Cargo.toml`; it
  imports `parallax_proto::semconv` (`greptime.rs:14`), so yes.

**Verify**: full suite passes (memory-adapter tests exercise the moved
normalization); the live-tail tests still pass; `rtk cargo clippy` zero
warnings. Grep gate: `grep -n "_spans" crates/parallax-storage/src/greptime.rs`
→ 0 matches (parameter gone).

### Step 4: Spool raw protobuf frames instead of JSON

Replace the NDJSON-of-decoded-request format with length-prefixed raw frames:
`[u8 signal-agnostic magic "PSPL1"][per record: u32-LE len + raw protobuf bytes]`
per segment file. The receivers already hold the exact bytes (`otlp_http.rs:39`
`raw = body.clone()`; `otlp_grpc.rs:67` re-encoded). `append` takes
`(signal, &Bytes)` — the `serde_json::to_string` disappears.

Compatibility: check how 073's replay/doctor reads segments (`spool.rs` +
`parallax-cli` doctor). Rules:
- New segments get a new file extension (`.pspl`) so old NDJSON segments stay
  readable by the existing reader until reaped; reader dispatches on
  extension.
- `line_count`/rotation/reaper accounting move to frame counts/bytes — 076's
  structure (per-signal state) is where this lives; keep its tests green by
  updating them to the new format (their ASSERTED BEHAVIORS — rotation at
  max bytes, reap by age/size — stay identical).

**Verify**: `rtk cargo nextest run -p parallax-storage spool` → all pass
(updated for format); a mixed-directory test: one legacy `.ndjson` segment +
new frames both counted by `doctor`/`line_count` equivalents.

### Step 5: Shrink the queue payload

After Step 3 the worker needs `(request-for-derive/live, raw)` for
traces/logs. Keep the decoded request ONLY if derive/live need it per Step 3's
outcome; where it is needed, the memory cost stands but is now justified —
document the per-item memory shape in a comment on `IngestItem`. Metrics items
keep decoded+raw (both used). No further change; this step is a
review-and-document step unless Step 3 removed all decoded-request consumers
for some signal (then drop that field).

**Verify**: `rtk cargo build --workspace` → exit 0; comment present.

### Step 6: Full gates

**Verify**: `rtk cargo fmt --all`; clippy zero warnings; full nextest; if a
local playground is available, run an end-to-end smoke: serve + gzip HTTP
exporter + gRPC exporter, confirm both land rows (UI or raw SQL count).

## Test plan

- New: gzip integration test (Step 1); per-signal isolation test (Step 2);
  `resource_run_ids` unit test (Step 3); mixed-format spool test (Step 4).
- Existing worker/spool/live tests must pass with assertions unchanged except
  where the spool FORMAT tests encode the format itself (update those to the
  frame format deliberately).
- No throughput claims in docs; if you measure locally, numbers go in commit
  messages only (repo bench discipline).

## Done criteria

- [ ] gzip smoke returns 200; `grep -rn "content-encoding\|Decompression" crates/parallax-server/src` → ≥1 handling site
- [ ] `grep -n "max_decoding_message_size\|DefaultBodyLimit" crates/parallax-server/src/serve.rs` → both present
- [ ] `grep -n "worker::channel(1024)" crates/parallax-server/src/serve.rs` → 0 matches (config-driven, per-signal)
- [ ] `grep -n "_spans" crates/parallax-storage/src/greptime.rs` → 0 matches
- [ ] `grep -n "serde_json::to_string" crates/parallax-storage/src/spool.rs` → 0 matches
- [ ] `rtk cargo nextest run --workspace` exits 0; clippy zero warnings
- [ ] `git status` clean outside in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- Plan 073 or 076 has NOT landed (this plan assumes their worker/spool shapes;
  running before them = merge chaos).
- The tower-http version in the workspace lacks request decompression and the
  manual gunzip path would exceed ~40 lines — report the version constraint.
- Step 3: `derive_from_logs`' refactor forces changes in
  `parallax-core/src/derive.rs` beyond a signature swap (its derivation logic
  must not change here — that is Plan 070/026 territory).
- Step 4: 073's replay contract stores anything OTHER than the full request
  per line (e.g. it started storing metadata) — the frame format must carry
  the same information; report the actual contract.
- Any ack-ordering test (ack only after spool append) needs weakening.

## Maintenance notes

- Ack semantics unchanged: spool-append still precedes the 200/OK; reviewers
  should verify no path acks before append.
- Per-signal workers change failure isolation: a poisoned traces batch no
  longer blocks logs — but also means cross-signal arrival order is
  explicitly unordered (it already was, now more visibly). Note in docs if any
  doc claims ordering.
- The legacy-NDJSON reader can be deleted after one release cycle once spools
  have rotated out (reaper max age default 72h).
- Deferred: worker pool per signal (parallelism within a signal) — revisit
  with server profiles/V2; exporter-visible backpressure metrics.
