# Upgrade and durability

Parallax treats a data directory as a contract. A directory written by
release N must open under release N+1 without silent loss. Telemetry that
is received and then dropped must increment a visible counter.

## What is versioned

| Store | Marker | Current |
| --- | --- | --- |
| Turso metadata | `PRAGMA user_version` | `3` (incident bundle columns) |
| Spool | PSPL1 frames (+ legacy `.ndjson` until plan 114) | frame-preserving append |
| GreptimeDB | checksum-pinned managed child + bootstrap `ALTER`s | engine data dir reused |

There is no downgrade path. A directory stamped newer than the binary
supports must fail closed (`newer than supported`) rather than rewrite
rows.

## What the upgrade harness proves

The ignored test `preview_data_dir_opens_losslessly_under_workspace`
downloads the rolling `preview` archive (or
`PARALLAX_UPGRADE_PREVIOUS`), seeds OTLP into that binary, then reopens
the same data dir with the workspace build.

Always-run tests cover the pieces that do not need a previous binary.

## Guarantee → test

| Guarantee | Test |
| --- | --- |
| New binary opens an old Turso file and keeps seeded issue rows | `upgrade_v0_issue_row_survives_current_schema` |
| Future `user_version` fails closed | `migration_rejects_future_user_version` |
| Fresh open stamps the current version | `migration_stamps_user_version_on_fresh_open` |
| Spool frame count is unchanged across reopen | `spool_frame_count_survives_reopen` |
| Preview binary data dir opens under this workspace | `preview_data_dir_opens_losslessly_under_workspace` (ignored; CI upgrade-harness) |
| Queue-full degrades `/health` | `queue_state_and_self_metric_filter_are_exact`, `health_reports_real_queue_overload_and_recovery` |
| Terminal ingest drop increments the counter and degrades `/health` | `retry_exhaustion_metrics_match_worker_attempts`, `queue_state_and_self_metric_filter_are_exact` |
| OTLP/Sentry reject after receipt is counted | `ingress_reject_increments_loss_json`, `otlp_http_and_health_endpoints_work` |
| Spool write failure degrades `/health` | `spool_write_fail_degrades_health` |
| Exponential-histogram / summary drop is counted, not silent | `exponential_histogram_is_dropped_today`, `summary_is_dropped_today`, `unsupported_metric_is_visible_and_does_not_degrade` |
| Live tail lag is counted and does not degrade `/health` | `live_tail_lag_is_counted_and_does_not_degrade` |
| `parallax doctor` prints the loss JSON when the API is up | doctor probes `GET /ingest/loss` |

## What "degraded" means

`GET /health` returns `503` with `degraded: <reason>` when:

- an ingest queue is at capacity
- a worker exhausted retries and dropped a batch
- a spool append failed after the request was received

It does **not** degrade for:

- live tail lag (lossy by design; counted)
- unsupported metric types (V1 model gap; counted so plan 166 can decide)

`GET /ingest/loss` is always `200` JSON:

```json
{"queue_unavailable":0,"terminal_drop":0,"ingress_reject":0,"spool_write":0,"unsupported_metric":0,"live_tail_lag":0}
```

## Spool non-promises

The spool is a forensic trail, not a WAL. It records accepted raw
frames. Retention may delete rotated segments. That delete is not an
ingest drop.

## Running the harness

```bash
cargo nextest run -p parallax-server -E 'test(/upgrade/)' --run-ignored only
```

Optional: `PARALLAX_UPGRADE_PREVIOUS=/path/to/parallax` skips the
download.
