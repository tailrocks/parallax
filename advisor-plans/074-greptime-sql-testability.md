# Plan 074: Put the real SQL layer under test — golden-SQL units, escape() coverage, and an adapter conformance suite

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- crates/parallax-storage/src`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition. (Plan 070 legitimately edits
> `greptime.rs`/`metadata.rs` before this plan — that drift is expected;
> re-verify only that the functions named here still exist.)

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: LOW for adding tests; MED where query methods are refactored to
  expose SQL builders
- **Depends on**: 070 (lands first to avoid rebase churn in `greptime.rs`);
  069 recommended (the nightly real-engine job gives the conformance suite a
  scheduled deep gate)
- **Category**: tests
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

The production storage adapter (`greptime.rs`, 2,871 lines, ~50 query methods
building raw SQL strings) has exactly **two** unit tests, and neither tests a
query builder. All functional coverage runs against `memory.rs` — a 2,535-line
hand-reimplementation of the same semantics (19+ test fns) that even uses a
*different quantile algorithm* (`memory.rs:88 quantile_from_histograms` vs
`greptime.rs:2792 quantile_from_cumulative`). CI boots MemoryStore
(`serve.rs`, `storage.mode = "none"`); the four real-engine tests are
`#[ignore]`d. Net: a change that breaks every production query passes CI
green, and memory-vs-greptime semantic drift is invisible. This plan makes SQL
generation directly assertable and forces the two adapters through one shared
conformance suite.

## Current state

- `crates/parallax-storage/src/greptime.rs`:
  - `escape()` at `:35-37` — `text.replace('\'', "''")` — the single quoting
    helper on ~46 interpolation sites (service names, trace/run ids, attribute
    values, fingerprints; also INSERT tuple builders). **Zero direct tests.**
  - `escape_ident()` at `:40-42` and `quoted_ident()` at `:44` — identifier
    quoting; has one test.
  - The LIKE-pattern builders additionally escape backslashes
    (`greptime.rs:795 .replace('\\', "\\\\")`), but the equality/`IN` sites
    using `escape()` alone do not — whether GreptimeDB's SQL dialect treats
    backslash as an escape character in string literals is UNVERIFIED; a
    trailing backslash in a value could neutralize the closing quote. Step 5
    settles this with the real engine.
  - Existing test module at the file bottom (`:2838-2871`):
    `escape_ident_doubles_double_quotes_only` and
    `raw_sql_read_only_guard_rejects_writes_and_explain_analyze` — use these
    as the structural pattern for new unit tests.
  - Query methods are `async fn` on `GreptimeStore` that `format!` SQL inline
    and immediately `self.sql(...)`/`self.sql_lenient(...)` — e.g.
    `traces_search` (~`:1930-2012`), `histogram_count_series` (`:2495`),
    `metric_table_for_name` (`:495`). SQL strings are NOT separable today.
- `crates/parallax-storage/src/adapter.rs` — `trait TelemetryStore`
  (`:468+`, 38 async methods) with doc comments explaining the
  native-forward/tee design. Pure helper fns (e.g. the share math at
  `:455-466`) already live here.
- `crates/parallax-storage/src/memory.rs` — `MemoryStore`, the test-only
  adapter; its inline test module (19+ tests, `:1539+`) asserts query
  *semantics* (`attribute_compare_ranks_overrepresented_value_first`,
  `service_map_derives_trace_path_edges`, etc.). These stay — they become the
  seed of the conformance suite.
- Real-engine integration tests (all `#[ignore]`): `m1_greptime.rs`,
  `m1_table_inventory_greptime.rs`, `m2_metrics_greptime.rs`, `m5_gates.rs`
  under `crates/parallax-server/tests/` — they boot a downloaded GreptimeDB.
- Conventions: cargo-nextest; inline `#[cfg(test)]` modules; clippy zero
  warnings; GreptimeDB native tables are mandatory for raw signals (do not
  invent test tables that mimic raw-signal schemas — assert against the
  native names `opentelemetry_traces`, `opentelemetry_logs`).

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Storage tests | `rtk cargo nextest run -p parallax-storage` | all pass |
| Full suite | `rtk cargo nextest run --workspace` | all pass |
| Real-engine suite (slow, downloads engine) | `rtk cargo nextest run --workspace --run-ignored only` | all pass |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |

## Scope

**In scope** (the only files you should modify):
- `crates/parallax-storage/src/greptime.rs` (extract SQL builders + tests)
- `crates/parallax-storage/src/adapter.rs` (conformance suite location, if
  chosen — see Step 3; or a new `crates/parallax-storage/src/conformance.rs`)
- `crates/parallax-storage/src/memory.rs` (wire into conformance suite only —
  do not change query semantics)
- `crates/parallax-storage/src/lib.rs` (module wiring if a new file is added)
- `crates/parallax-server/tests/` (one gated conformance-over-greptime test)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- Changing any query's SEMANTICS (WHERE clauses, ordering, scaling) — this
  plan pins current behavior; Plan 075 changes `traces_search` semantics
  AFTER this net exists.
- `metadata.rs` (Turso) — parameterized already; different risk profile.
- Fixing memory-vs-greptime divergences you discover — REPORT them (that's
  the point of the suite); each divergence is its own follow-up decision.

## Git workflow

- Work directly on `main` (repo rule — `BRANCHING.md`).
- Conventional Commits, DCO signoff (`git commit -s`), trailer
  `Co-authored-by: Claude <noreply@anthropic.com>`. E.g.
  `test(storage): golden-sql units for greptime query builders`.

## Steps

### Step 1: Unit-test the escaping helpers

In the existing `greptime.rs` test module add tests for `escape()`:
single quote doubled; already-doubled quotes; embedded newline preserved;
empty string; a value ending in a single backslash (assert current behavior —
backslash passes through unchanged — with a comment pointing at Step 5's
dialect verification). Also test `quoted_ident` composition.

**Verify**: `rtk cargo nextest run -p parallax-storage greptime` → pass.

### Step 2: Extract pure SQL builders for the highest-risk queries (golden-SQL tests)

Refactor mechanically, no behavior change: for each of these methods, move the
`format!` block into a pure associated fn returning `String`, called by the
async method:

1. `traces_search` → `fn traces_search_sql(query: &TraceSearchQuery, ...) -> (String, String)` (the `listed` composition + the count wrapper)
2. `histogram_count_series` → `fn histogram_count_series_sql(count_table: &str, ...) -> String`
3. `histogram_quantile`'s bucket query
4. `select_spans` / `select_logs` (the SELECT + WHERE composition)
5. `span_attribute_counts` (feeds `attribute_compare`)

Golden tests: call each builder with fixed inputs (incl. adversarial strings —
a service name containing a single quote, a trace id with a double quote) and
assert the EXACT SQL string. Keep expected strings as raw literals next to the
test (not snapshot files — repo has no snapshot tooling; follow the existing
inline-assert style).

Rules: do not alter the SQL text while extracting (byte-identical output);
`rustfmt` may rewrap the literal — acceptable. If a method's SQL depends on an
awaited lookup (e.g. `metric_table_for_name`), the builder takes the resolved
value as a parameter (as shown for `count_table`).

**Verify**: after EACH extraction, `rtk cargo nextest run -p parallax-storage`
→ all pass; golden tests assert the expected strings.

### Step 3: Shared conformance suite over both adapters

Create `crates/parallax-storage/src/conformance.rs` (compiled under
`#[cfg(any(test, feature = "conformance"))]` — add the feature to
`crates/parallax-storage/Cargo.toml` so the server integration test can use
it): a set of `pub async fn` scenarios, each taking `&dyn TelemetryStore`,
seeding data through the trait's ingest methods, querying, and asserting.

Port 4 high-value scenarios from `memory.rs`'s existing tests (keep the
originals in place for now):
- trace search: sorts, offsets, duration band
- attribute compare: over-represented value ranks first
- service map: trace-path edges derived
- log count series / overview totals over a seeded window

Wire them:
- In `memory.rs` tests: `conformance::trace_search_scenario(&store).await`
  (async tests already exist there — follow their tokio pattern).
- New gated integration test `crates/parallax-server/tests/m6_conformance_greptime.rs`
  with the same `#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]`
  attribute and boot pattern as `m1_greptime.rs`, running every conformance
  scenario against the real store.

CAVEAT: seeding through `ingest_*` requires building raw OTLP protobuf
`Bytes` for the greptime path (the native forward uses `raw`, ignoring the
decoded tee). Reuse whatever helper the existing `m1_greptime.rs` /
`m2_metrics_greptime.rs` tests use to construct OTLP requests — do not invent
a second fixture builder. If the conformance scenarios can't seed identical
data through both adapters via the public trait (memory uses the decoded tee,
greptime uses raw bytes), construct BOTH from one source-of-truth request
object per scenario.

**Verify**: `rtk cargo nextest run -p parallax-storage` → conformance
scenarios pass on MemoryStore. Then run the real thing once:
`rtk cargo nextest run -p parallax-server m6_conformance --run-ignored only`
→ scenarios pass on GreptimeDB, OR produce a divergence list (see STOP
conditions — divergences are reportable results, not failures of this plan).

### Step 4: Record divergences

If Step 3 surfaces memory-vs-greptime divergences (different ordering,
different quantile values, boundary off-by-ones): capture each as a short
entry in the "Findings considered and rejected / open" section of
`advisor-plans/README.md` (file, scenario, both observed values). Loosen the
specific assertion to the agreed common contract ONLY where the difference is
legitimate floating-point noise (document epsilon); otherwise leave the
scenario failing on the ignored/greptime side and report.

**Verify**: divergence entries exist in `advisor-plans/README.md`, or a note
"no divergences found".

### Step 5: Settle the backslash question against the real engine

Extend `m6_conformance_greptime.rs` with one focused test: seed a span whose
service name ends in a backslash and one containing `\' ` (backslash then
quote), then query it back via a typed method that filters on service equality
(e.g. `select_spans` with a service filter). Assert the round-trip returns
exactly the seeded rows.

- If the round-trip is exact → GreptimeDB does not treat backslash as an
  escape in single-quoted literals; add a comment on `escape()` recording the
  verification (engine version + date) and close the question.
- If rows are missing/mismatched → the equality path IS injectable/lossy:
  STOP and report with the observed behavior; the fix (escape `\` in
  `escape()` like the LIKE path at `greptime.rs:795`) changes ~46 call sites'
  output and needs the golden tests updated deliberately, not silently.

**Verify**: the test passes (question closed) or a STOP report exists.

### Step 6: Full gates

**Verify**: `rtk cargo fmt --all`;
`rtk cargo clippy --workspace --all-targets` → zero warnings;
`rtk cargo nextest run --workspace` → all pass (ignored suite unchanged in CI;
Plan 069's nightly runs it on schedule).

## Test plan

- Step 1: ≥5 `escape`/`quoted_ident` unit tests.
- Step 2: ≥5 golden-SQL tests with adversarial inputs.
- Step 3: 4 conformance scenarios × 2 adapters.
- Step 5: 1 real-engine escaping round-trip test.
- Structural patterns: `escape_ident_doubles_double_quotes_only`
  (`greptime.rs:2840`) for units; `memory.rs:1539+` for scenario style;
  `m1_greptime.rs` for the gated boot.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `grep -cn "fn.*_sql" crates/parallax-storage/src/greptime.rs` → ≥5
- [ ] `crates/parallax-storage/src/conformance.rs` exists with ≥4 `pub async fn` scenarios
- [ ] `crates/parallax-server/tests/m6_conformance_greptime.rs` exists and is `#[ignore]`-gated
- [ ] `grep -n "fn escape_handles" crates/parallax-storage/src/greptime.rs` → ≥1 (or equivalently-named escape tests)
- [ ] `rtk cargo nextest run --workspace` exits 0
- [ ] `rtk cargo clippy --workspace --all-targets` → zero warnings
- [ ] Divergence notes (or "none found") recorded in `advisor-plans/README.md`
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- Extracting a builder cannot produce byte-identical SQL (hidden state in
  `&self` beyond resolvable parameters).
- Step 5 shows the equality-filter round-trip is lossy (backslash IS an
  escape) — report; the `escape()` change is deliberate follow-up.
- Conformance seeding cannot reach parity between adapters through public
  APIs without >~200 lines of new fixture code.
- The real-engine run fails for environmental reasons (download blocked, port
  conflicts) twice — report the environment issue.

## Maintenance notes

- Every future query-method change should update its golden test in the same
  commit — reviewers should reject greptime.rs query diffs without a golden
  diff.
- Plan 075 (traces_search windowing) deliberately CHANGES SQL semantics; its
  first step is updating these golden tests to the new expected strings.
- The duplicated aggregation math (two quantile impls) remains; the
  conformance suite makes the drift visible but the shared `aggregate.rs`
  extraction (audit finding DEBT-02) is deferred — reconsider once
  divergences from Step 4 are known.
- When the A6 redaction program or MCP adapter needs storage fixtures, reuse
  the conformance seed builders.
