# Scheduled measurement sample — run 29592948190

Research date: 2026-07-17.
Source: GitHub Actions `scheduled-measurement.yml` dispatch
[run 29592948190](https://github.com/tailrocks/parallax/actions/runs/29592948190)
on `ubuntu-latest`, head `154db5130ad119263993ab843ac49206a460e8c5`
(third independent job after 29582532812 and 29589577179).

## Bench mids (three repeats)

| Bench | r1 | r2 | r3 | notes |
|---|---:|---:|---:|---|
| normalize (µs) | 291.47 | 293.99 | 289.60 | tight |
| spool_append_4k (µs) | 39.496 | 39.043 | 41.068 | slight r3 rise |
| spool_line_count (ms) | 21.922 | 21.443 | 21.937 | high absolute vs job-2 low |
| arrow_decode (ms) | 1.7952 | 1.7972 | 1.8217 | tight |
| arrow_zstd (ms) | 2.0720 | 2.0729 | 2.0842 | tight |

## Allocation

```
normalize_metrics_1k_points allocation-profile: 7011 allocations/call, 1022357 bytes/call
```

**Identical** to runs 29582532812 and 29589577179 (7011 / 1022357). Three
independent scheduled jobs agree — adopt fail-closed absolute allocation
ceiling in `docs/research/testing/bench-baselines.toml`.

## Cross-run model (n=3)

| Bench | Job max mid | Cross-run character |
|---|---:|---|
| normalize | ~301 µs | stable |
| spool_append_4k | ~41 µs | stable |
| spool_line_count | ~22 ms with job-2 dips to ~13 ms | **unstable** — excluded from tight timing ratchet |
| arrow_decode | ~1.90 ms | stable |
| arrow_zstd | ~2.18 ms | stable |

Ratchets adopted from this third sample: absolute allocation + generous
ceilings on the four stable timing benches only.
