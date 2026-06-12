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

## Live acceptance verification (2026-06-12, same machine)

Beyond the latency gates, the scope's operational promises were exercised
against a running `parallax serve` on the real ports with a fresh data dir:

- **All UI pages functional** (headless Chrome over the embedded SPA, zero
  console errors): issues list (grouping, culprit, counts) → issue detail
  (trend sparkline from `issueTrend`, stacktrace, occurrences, trace link,
  agent-handoff command) → trace waterfall with **correlated logs** (FATAL +
  INFO under the spans) → trace lookup by pasted ID → service overview
  (latency percentile chart from a real histogram) → dashboards (created
  "checkout ops" through the UI form; the chart renders the user-sent
  `checkout.queue.depth`) → runs (wrapper run listed with exit code).
- **CLI agent handoff** on the same data: `parallax issue context` printed
  the panic issue's bundle (identity, culprit, cross-service trace).
- **doctor / prune / uninstall to spec**: doctor reported data-dir size, API
  + engine health, server and engine versions, spool backlog, engine-data /
  metadata sizes; prune reclaimed and reported; uninstall refused without
  `--purge`, prompted without `--yes`, then deleted the data dir.
- **Supervision orphan safety**: SIGKILLing serve orphaned the engine child
  (ppid 1); the next serve logged `reaping stale greptime child`, killed it,
  and started a fresh supervised child. SIGTERM now shuts down as cleanly as
  Ctrl-C (no surviving child, pidfile removed). This scenario was found —
  and is now prevented — by this verification run.
- Demo data for all of the above comes from
  `cargo run -p parallax-server --example seed`.

### Addendum — contract-completion surfaces verified live (2026-06-12, later same day)

After the spec-§8 contract completion landed, the new surfaces were exercised
the same way (running serve, real engine + the jackin data history, headless
Chrome, zero console errors):

- **Span detail pane** on the trace waterfall: clicking the `query orders`
  span showed `db.query.text` (`SELECT id, total FROM orders WHERE cart_id =
  $1`), `db.operation.name`, `db.system.name`, kind, duration — acceptance
  scenario 2's "query text in the waterfall span detail" now demonstrably
  renders in the UI, not only in the bundle.
- **Run model end-to-end**: `parallax run start -- …seed` → `run inspect`
  (status, exit code, 3 traces / 4 errors, both grouped issues) →
  `issue list --run <id>` → `run bundle <id>` (run section, issues-in-run,
  primary issue, trace with the captured query, hypotheses) → the same run's
  `/runs/$runId` page (record badges, issues, trace summaries, logs, bundle
  preview). Externally-seen run ids auto-register with status `external`.
- **Logs page** over the unified `logs` filter API with the real jackin
  firehose (hundreds of rows) + the count histogram; a missing
  `scopeName`/`resource` on the API's LogRecord was found by this run and
  fixed (the page's object view selects them).
- **Issues list** filters/sort/per-row sparkline/total and **issue detail**
  resolve button, SDK context section, breadcrumb logs, clickable trend.

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
