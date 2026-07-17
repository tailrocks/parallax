# Plan 124 — CI and flaky-test evidence residual (closed 2026-07-17)

## Scope closed

| Residual | Landed surface |
| --- | --- |
| Exact APIs/events/claim wording | `parallax_evidence::github_actions` (`API_VERSION_DEFAULT`, `CI_CLAIM_KEYS`, linkage-only wording) |
| Stable attempt identity + flaky multi-attempt | Prior slices + REST path reuses `attempt_identity` / `flaky_claim_from_attempts` |
| Signature-verified `workflow_job` webhook + Turso | Prior + claim-row seed on accept |
| Doctor inventory | deliveries/attempts/claim-row counts |
| REST backfill tick (rate-aware, cursor-monotonic) | `parallax_server::ci_backfill` + Turso `ci_backfill_state` |
| Bundle correlation without root-cause overclaim | `ci_adjacency` / `deploy_adjacency` hypotheses (`confidence: low`) |
| Dated claim coverage rows | Turso `evidence_claim_rows` domain `ci_evidence` |

## Config (opt-in)

```toml
[github_actions]
enabled = true
webhook_secret = "…" # or PARALLAX_GITHUB_ACTIONS_WEBHOOK_SECRET
backfill_enabled = true
backfill_repos = ["tailrocks/parallax"]
# token via PARALLAX_GITHUB_TOKEN preferred
backfill_interval_secs = 300
backfill_page_size = 30
backfill_max_runs_per_tick = 5
```

## Verification (2026-07-17)

```text
cargo test -p parallax-evidence --lib github_actions
cargo test -p parallax-metadata --lib claim
cargo test -p parallax-metadata --lib attempt_identity
cargo test -p parallax-metadata --lib backfill_cursor
cargo test -p parallax-server --lib tick_accepts
cargo test -p parallax-server --lib workflow_job
cargo xtask policy --only structural
```

All listed gates green on this head. Live GitHub REST backfill requires an operator token and remains opt-in; fixture-backed HTTP transport proves rate-limit / cursor / claim-row behavior without network.
