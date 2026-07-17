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

Deferred (blocked on their owners): UI search round-trips, runtime decoder
accept/reject domains, Query-key identity, SSE ordering (plans 133/147/148);
fuzz targets and performance baselines follow as separate plan-103 steps.
