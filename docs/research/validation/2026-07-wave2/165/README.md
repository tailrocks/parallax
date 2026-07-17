# Plan 165 — Logs power features (brush, columns, Drain patterns)

**Status:** DONE (2026-07-17)

## Closed claims

| Claim | Evidence |
|---|---|
| Drain clustering (Rust) | `parallax-analysis` unit tests + GraphQL `logPatterns` |
| GraphQL `logPatterns` sample ≤10k | API unit test + live 24 clusters on QA corpus |
| Patterns UI toggle (`?patterns=1`) | Browser capture |
| Histogram brush-zoom | Existing `useChartBrush` on logs histogram (URL range) |
| Optional columns (`?cols=`) | Pre-existing logs column menu (URL-encoded) |
| Pure prefs helpers | `log-table-prefs` / `log-patterns-url` / brush helpers tested |

## Live GraphQL

QA stack sample (top templates):

```text
facet corpus request <*>                         count 200
[Consumer … Seeking to offset … partition …]     count 79
Record in retry and not yet recovered            count 72
…
```

Full capture: [live-log-patterns.json](./live-log-patterns.json)

## Browser

agent-browser walk (2026-07-17): Patterns toggle sets `?patterns=true` and
renders a Template/Count table over live `logPatterns` data. Capture note:
[browser/patterns-snapshot.txt](./browser/patterns-snapshot.txt). Histogram
brush remains the existing logs `useChartBrush` path (URL range on drag).

## Tests

- `cargo nextest -p parallax-api -E 'test(log_patterns)'` green
- Analysis Drain tests (prior commits)
- UI pure helpers for brush/prefs/URL (prior commits)
