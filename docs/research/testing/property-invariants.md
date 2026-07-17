# Property-test invariants (plan 103, Step 1 record)

Research date: 2026-07-17. Each target names its defect class, input domain,
oracle, owner (the module the suite lives in), runtime class (all suites run
in normal `cargo nextest` CI — bounded, seeded, shrinkable), and removal rule
(remove with the code it guards).

| Invariant | Defect class | Input domain | Oracle | Suite |
|---|---|---|---|---|
| Counter deltas clamp at reset | negative rates/increases after counter restarts poison charts and alerts | arbitrary finite bucketed series, arbitrary step | every output point ≥ 0 and finite; increase length = n−1 | `parallax-storage/src/adapter_math.rs` |
| Histogram Δsum/Δcount averages | division shapes (0/0, negative growth) fabricate samples | cumulative non-decreasing sum/count series | outputs finite and ≥ 0; zero-growth buckets skipped | same |
| Canonical metric identity | drifting name normalization breaks the ingest-persisted `canonical_name` contract | arbitrary ≤64-char names | idempotent; output charset `[A-Za-z0-9_]` | `parallax-semconv/src/lib.rs` |
| JSON attribute path shape | backslash-escaped member quotes silently match nothing on the live engine (defect fixed 2026-07-17) | arbitrary quote-free keys | path is exactly one plainly quoted member (`$."…"`) | same |
| Where-clause literal balance | a hostile filter value terminating a SQL literal (injection) | arbitrary keys/values/operators across the span, log, and metric compile arms | single-quote count in every compiled condition is even | `parallax-greptime/src/greptime/attribute_filters.rs` |
| Redaction text idempotence | re-sanitizing already-redacted text mutates storage titles/messages | arbitrary ≤4 KiB UTF-8 strings | `sanitize_text` is a fixpoint (`f(f(x)) = f(x)`) | `parallax-evidence/src/redaction_policy.rs` property_tests |
| Canonical JSON + version-scoped hash stability | non-deterministic key order or non-fixpoint canonicalization breaks bundle-v2 hashes | arbitrary JSON trees depth ≤3, ≤4 children | `canonical_json` re-parse is a fixpoint; version-scoped hash stable | `parallax-evidence/src/envelope.rs` property_tests |
| Fingerprint determinism | non-pure fingerprinting splits one defect class across issues | arbitrary type/message/stack strings | same inputs → identical 16-hex fingerprint; `normalize_message` fixpoint | `parallax-analysis/src/fingerprint.rs` property_tests |
| OTLP normalize determinism | non-pure OTLP projection would fork identity/query results | service/name/body strings + status/severity ints | two normalize calls over the same request yield equal JSON-encoded rows | `parallax-ingest/src/tests.rs` property_tests |
| Trace parent graph determinism | cyclic parents recurse forever or ingest order changes analysis | bounded unique spans with arbitrary missing, self, and cyclic parents | traversal terminates; hop IDs stay unique/input-backed; permutation preserves path and sorted unattached IDs | `parallax-analysis/src/trace_analysis/tests.rs` |

## Fuzz boundaries (plan 103, Step 3 — first slice)

`fuzz/` (cargo-fuzz, nightly, excluded from the workspace) with four
boundaries, each smoke-run 20k executions clean on 2026-07-17:

| Target | Boundary | Oracle |
|---|---|---|
| `otlp_metrics_normalize` | OTLP metrics protobuf decode + normalization | no panic/unbounded loop for arbitrary bytes |
| `otlp_traces_normalize` | OTLP traces protobuf decode + normalization | same |
| `redaction_text` | redaction text projection | no panic; `sanitize_text` idempotent |
| `bundle_envelope_json` | evidence bundle JSON parse + canonicalization | no panic; `canonical_json` is a fixpoint |
| `spool_framing` | PSPL frame counting over arbitrary on-disk bytes | no panic; no unbounded allocation (hostile length prefixes now seek, never allocate — defect fixed 2026-07-17) |
| `arrow_decode` | GreptimeDB Arrow IPC response decode | no panic; the first-message hostile-prefix allocation class is rejected before the reader allocates (found by this target, fixed 2026-07-17; deeper flatbuffer-framed message lengths remain the arrow crate's domain) |

Run: `cargo +nightly fuzz run <target>` from the repo root. Minimized crash
corpora will be committed under `fuzz/corpus/<target>/` when found.
All five planned boundaries are landed. Target/workflow drift validation and the
scheduled CI lane are still open.

## UI property suites (plan 103, Step 2 — landed 2026-07-17 after 133/147/148 closed)

fast-check + vitest, seeded/shrinkable, in normal CI:

| Invariant | Defect class | Suite |
|---|---|---|
| Logs URL search round-trip + total/idempotent validation | saved views and permalinks silently drop or mutate state | `features/logs/tests/model/logs-search.property.test.ts` |
| Live merge dedup/contract-order/capacity | live prepend can duplicate rows, reorder segments, or grow unbounded | `features/logs/tests/model/merge-live-logs.property.test.ts` |
| Where-clause serialize→parse fidelity, total parse | filters drift between editor, URL, and SQL compiler | `shared/tests/where-clause.property.test.ts` |
| Query-key determinism + hierarchical prefixes | cache invalidation misses or cross-key collisions | `features/investigations/tests/queries/keys.property.test.ts` |

Previously deferred (blocked on their owners): UI search round-trips, runtime decoder
accept/reject domains, Query-key identity, SSE ordering (plans 133/147/148);
fuzz targets and performance baselines follow as separate plan-103 steps.

## Performance baselines (plan 103, Step 4 — first measurement)

Criterion benches (measurement only; thresholds wait for variance modeling
on a stable runner). First local run, 2026-07-17, Apple Silicon dev host,
`--quick`:

| Bench | Crate | First observation |
|---|---|---|
| `normalize_metrics_1k_points` | parallax-ingest | ~238 µs |
| `spool_append_4k` | parallax-spool | ~23 µs |
| `spool_line_count` | parallax-spool | ~330 µs (grows with segment size) |
| `arrow_decode_10k_rows` | parallax-greptime | ~840 µs |
| `arrow_decode_10k_rows_zstd` | parallax-greptime | ~980 µs |

First CI validation (2026-07-17): manual dispatch run 29582532812 completed
on ubuntu, uploading `fuzz-campaign-*` (five-minute campaign per boundary)
and `bench-samples-*` (three criterion repeats) artifacts. **Corrected:**
two fuzz boundaries asserted (`redaction_text`, `bundle_envelope_json`);
defects fixed same day (control-strip order; number fixpoint loop). Durable
sample + within-run variance table:
[`measurement/2026-07-17-run-29582532812/`](measurement/2026-07-17-run-29582532812/README.md).

Run: `cargo bench -p <crate> --bench <name>`.

**Ratchets adopted 2026-07-17 (plan 103 residual closed):** after three
independent scheduled jobs (`29582532812`, `29589577179`, `29592948190`),
fail-closed ceilings live in
[`bench-baselines.toml`](bench-baselines.toml) and are enforced by
[`scripts/ci/check-bench-baselines.sh`](../../scripts/ci/check-bench-baselines.sh)
on every scheduled bench job. The script never rewrites baselines; ceilings
change only by commit. `spool_line_count` is excluded (cross-run unstable).
Allocation absolute is 7011 / 1_022_357 bytes for
`normalize_metrics_1k_points`.

## Allocation instrumentation (plan 103, Step 4)

`bench-alloc/` (standalone, workspace-excluded so its counting allocator
never weakens the product crates' `forbid(unsafe_code)`) measures the
ingest hot path. First observation, 2026-07-17, Apple Silicon dev host:

- `normalize_metrics` over a 1k-point gauge batch: ~7,011 allocations/call,
  ~1.02 MB/call (≈7 allocations per point, dominated by per-point attribute
  JSON conversion) — the reference point for zero-copy follow-ups; optimize
  only when evidence warrants (plan rule).

Run: `cd bench-alloc && cargo run --release`. The scheduled-measurement
workflow records it nightly beside the criterion samples.

## Advanced-tool evaluation (plan 103, Step 6 — 2026-07-17)

Per the plan's adoption bar (named uncovered defect class, owner, cost
baseline, decision threshold, removal policy), none of the candidate tools
is adopted now:

- **Miri**: the workspace forbids unsafe code in product crates; the only
  unsafe lives in the excluded bench-alloc counting allocator, which
  delegates verbatim to the system allocator. No uncovered defect class.
- **Mutation testing (cargo-mutants)**: the ratchet/policy machinery already
  fails on assertion-count drops, and runtime cost on this workspace is
  hours per run; revisit if a real escaped-mutant incident names a class.
- **Dylint**: no repository-specific lint need that oxc/clippy strict plus
  the xtask policy families do not already express.
- **Hakari**: workspace-hack optimizes build times, not correctness; CI
  caching (sccache, per-job target caches) already owns that concern.
- **Chaos / self-hosted runners**: no measured flakiness or runner-capacity
  signal; the live-engine lanes are deterministic and green.

Removal rule for this record: revisit whenever one of the above gains a
named defect class with evidence; otherwise it stands.

## Fail-closed performance ratchets (plan 103, Step 5 — adopted 2026-07-17)

Three independent scheduled ubuntu jobs (29582532812, 29589577179,
29592948190; nine criterion repeats per bench) modeled variance: CV ≤2.6%
for the four stable benches; allocation profile identical (7,011
allocations / 1,022,357 bytes per call) across all three. Ceilings live in
`docs/research/testing/bench-baselines.toml` (~1.35× max observed mid);
`scripts/ci/check-bench-baselines.sh` fails the scheduled job on any
breach and never rewrites a baseline — validated green against run #3's
real artifacts. The original `spool_line_count` was excluded: its 20-24%
CV was a bench design flaw (counting a growing file), replaced by
`spool_line_count_1k`, which gains its ceiling after three post-fix jobs.
