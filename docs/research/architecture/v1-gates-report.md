# V1 gate measurements (M5 slice)

Measured 2026-06-12 on the local profile (macOS arm64, M-series laptop,
managed GreptimeDB v1.0.2 standalone child, debug-profile server binary —
release builds can only improve these numbers).

| Gate (build plan §4.6) | Target | Measured | Verdict |
| --- | --- | --- | --- |
| Cold setup: install → serving | < 15 min | ~12 s first `parallax serve` (141 MB engine download 5.3 s on a home connection + checksum + bootstrap ~6 s), plus the install itself (`cargo install` minutes, brew seconds) | **pass** |
| Ingest-to-queryable, error span → GraphQL-visible trace | ≤ 5 s p95 | **p50 4 ms · p95 10 ms · max 60 ms** over 20 runs | **pass** |
| Real panic → grouped issue visible (scope §1 promise) | ~5 s | **32 ms** through the panic-hook → OTLP-log path to a GraphQL-visible issue | **pass** |
| Bundle assembly, warm | ≤ 300 ms | **p50 8 ms · p95 9 ms · max 9 ms** over 10 runs | **pass** |
| Canary leaks through redaction-lite on its fixtures | zero | zero (`m2_bundle` asserts the bearer + AWS-key canaries never reach the bundle JSON or Markdown) | **pass** |

Supporting number: warm server start with the engine binary already present
is ~6.2 s (supervisor spawn + health poll + DDL bootstrap + listeners).

## Reproduce

```sh
# latency gates (downloads/reuses a real GreptimeDB):
cargo test -p parallax-server --test m5_gates -- --ignored --nocapture

# redaction canaries:
cargo nextest run -p parallax-server --test m2_bundle

# cold start: delete the data dir, time the first serve
rm -rf /tmp/plx-cold && parallax serve --config <config with data_dir=/tmp/plx-cold>
```

## Honest caveats

- The latency gates were measured through the in-process test harness
  (loopback OTLP/gRPC, loopback GraphQL); there is no network between app
  and Parallax on the local profile, so this matches real usage.
- The p95s are far enough under the gates that debug-vs-release and
  laptop-load noise cannot flip the verdicts.
- The measurement run also caught a real engine-only bug (open-ended
  `0..=u128::MAX` ranges failed GreptimeDB's Timestamp cast in query
  planning — invisible on the in-memory store), fixed in the same change.
  The gate suite earns its keep as a regression net, not just a report.
