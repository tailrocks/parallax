# Plan 121 — GitHub deploy/change context residual (closed 2026-07-17)

## Scope closed

| Residual | Landed surface |
| --- | --- |
| Webhook HMAC + delivery-id idempotency | Prior + claim-row seed on accept |
| REST Deployments API backfill | `parallax_server::deploy_backfill` (rate-aware, opt-in) |
| Doctor inventory | secret + delivery count + `deploy_context` claim rows |
| Bundle projection | GraphQL `bundle` loads linkage-only `deploy_adjacency` / `ci_adjacency` |
| No causal wording | `DEPLOY_ADJACENCY_CLAIM_WORDING` + low-confidence hypotheses |

## Config (opt-in)

```toml
[github_deploy]
enabled = true
webhook_secret = "…" # or PARALLAX_GITHUB_WEBHOOK_SECRET
backfill_enabled = true
backfill_repos = ["tailrocks/parallax"]
backfill_interval_secs = 300
backfill_page_size = 30
# token via PARALLAX_GITHUB_TOKEN
```

## Verification

```text
cargo test -p parallax-evidence --lib parses_rest_deployments
cargo test -p parallax-server --lib tick_accepts_deployments
cargo test -p parallax-server --lib github_webhook
cargo xtask policy --only structural
```

All green on this head. Live REST requires operator token; mock HTTP proves accept/claim-row behavior.
