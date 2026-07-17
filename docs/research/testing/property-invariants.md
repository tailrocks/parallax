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

Deferred (blocked on their owners): UI search round-trips, runtime decoder
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

Run: `cargo bench -p <crate> --bench <name>`. Ratchets are NOT set — the
plan forbids thresholds before variance is measured on a stable scheduled
runner; these numbers are the reference points for that work.
