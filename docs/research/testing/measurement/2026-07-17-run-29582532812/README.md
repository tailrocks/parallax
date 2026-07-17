# Scheduled measurement sample — run 29582532812

Research date: 2026-07-17.
Source: GitHub Actions `scheduled-measurement.yml` manual dispatch
[run 29582532812](https://github.com/tailrocks/parallax/actions/runs/29582532812)
on `ubuntu-latest`, head `750d0395f6c2718d79a163594ea034ab28ab5ef9`.
Plan 103 residual: model variance before any fail-closed performance ratchet.

## Bench samples (three criterion repeats, same job)

| Bench | r1 mid | r2 mid | r3 mid | mean | within-run span % |
|---|---:|---:|---:|---:|---:|
| `normalize_metrics_1k_points` | 291.23 µs | 293.17 µs | 290.90 µs | 291.77 µs | 0.78% |
| `spool_append_4k` | 38.818 µs | 37.851 µs | 39.108 µs | 38.592 µs | 3.26% |
| `spool_line_count` | 21.282 ms | 21.225 ms | 21.125 ms | 21.211 ms | 0.74% |
| `arrow_decode_10k_rows` | 1.8093 ms | 1.8804 ms | 1.7986 ms | 1.8294 ms | 4.47% |
| `arrow_decode_10k_rows_zstd` | 2.0960 ms | 2.1758 ms | 2.0826 ms | 2.1181 ms | 4.40% |

Raw: `repeat-1.txt` … `repeat-3.txt`.

## Allocation profile (single sample, same job)

```
normalize_metrics_1k_points allocation-profile: 7011 allocations/call, 1022357 bytes/call
```

File: `allocation-profile.txt`.

## Variance model honesty (plan 103)

- **Within-job criterion variance** is low (sub-5% span on all named benches).
- **Cross-run / multi-night variance is not yet modeled**: this is the first
  durable scheduled sample committed to the tree. A second dispatch
  (`29589577179`) was started 2026-07-17 to grow n; thresholds wait until
  ≥3 independent scheduled jobs on `ubuntu-latest` establish a stable
  mean/dispersion. Inventing relative fail-closed ceilings from n=1 is
  forbidden by plan 103.
- **Allocation count** looks deterministic on this path (7011) but still has
  only one scheduled observation; do not fail-close until a second job
  confirms the same integer on the same runner class.

## Fuzz campaign (same run)

Five-minute campaigns per boundary. Two assert failures (libFuzzer exit 77)
were **real product defects**, not flaky harness noise:

| Target | Minimized input | Defect | Fix |
|---|---|---|---|
| `redaction_text` | `C@\u{6}2.srea@T` | Control-char strip **after** detectors broke `sanitize_text` fixpoint (email rule fired only on pass 2) | Strip controls **before** rules in `parallax-redaction` |
| `bundle_envelope_json` | `9E294` | serde_json ryu form of extreme floats not a single-pass fixpoint (`…1e+294` → `…2e+294` → `…3e+294`) | `stable_json_number` loop in `canonical_json` |

Crash logs: `redaction_text.log`, `bundle_envelope_json.log`. Regression tests
landed beside the property suites. Clean campaigns: `otlp_*`, `arrow_decode`,
`spool_framing`.

## Ratchet decision (this sample)

**No performance ratchet adopted from this packet alone.** Keep collecting
scheduled artifacts; next adoption gate is a multi-run table with proposed
relative ceilings (e.g. mean + k·σ or max within-run span) and an explicit
no-auto-refresh rule in CI.
