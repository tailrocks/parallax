# Scheduled measurement sample — run 29589577179

Research date: 2026-07-17.
Source: GitHub Actions `scheduled-measurement.yml` dispatch
[run 29589577179](https://github.com/tailrocks/parallax/actions/runs/29589577179)
on `ubuntu-latest` (second independent job after 29582532812).

## Bench mids (three repeats)

| Bench | r1 | r2 | r3 | notes |
|---|---:|---:|---:|---|
| normalize (µs) | 290.64 | 300.91 | 292.63 | within ~3.5% |
| spool_append_4k (µs) | 39.432 | 40.062 | 39.086 | tight |
| spool_line_count (ms) | 21.354 | 12.995 | 12.661 | **~40% within-job swing** |
| arrow_decode (ms) | 1.8911 | 1.9011 | 1.8969 | tight |
| arrow_zstd (ms) | 2.1639 | 2.1736 | 2.1648 | tight |

## Allocation

```
normalize_metrics_1k_points allocation-profile: 7011 allocations/call, 1022357 bytes/call
```

**Identical** to run 29582532812 (7011 / 1022357). Two independent scheduled
jobs agree on the integer — strongest candidate for a fail-closed absolute
allocation ceiling, still pending a third confirmation and a CI gate design
that does not auto-refresh.

## Cross-run vs prior sample

- Timing means are close for normalize/spool_append/arrow.
- `spool_line_count` is **not** stable enough for a tight relative ratchet:
  this job alone spans ~12.7–21.4 ms. Any fail-closed timing gate must either
  exclude this bench, use a wide band, or investigate segment-size growth
  (bench note: grows with segment size).
- Fuzz artifacts still show redaction/canonical crashes — this job ran on a
  head **before** the control-strip / number-fixpoint fixes landed. Next
  campaign after those fixes is the re-verify.

## Ratchet stance (n=2)

Still **no timing ratchets**. Allocation absolute is promising (identical
twice) but plan 103 keeps the multi-run bar; wait for ≥1 more scheduled job
and an explicit CI check without baseline auto-refresh before retiring the
plan.
