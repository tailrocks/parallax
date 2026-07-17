# GitHub deploy/change context adapter (plan 121)

**Status:** preliminary approved shape; fixture proof required before product claim  
**Decision date:** 2026-07-17  
**Approver:** operator unblock directive (plan 121 provider = GitHub)  
**Owner:** Plan 121

## Decision

Parallax's first deploy/change provider is **GitHub** (read-only).

| Field | Contract |
| --- | --- |
| Provider | GitHub |
| REST API version header | `2022-11-28` (fixtures also record `X-GitHub-Api-Version` when present) |
| Webhooks (first slice) | `deployment`, `deployment_status` |
| Later entities | `push`/`create` release markers, PR/file list, check runs, workflow runs (plan 124) |
| Auth | Webhook HMAC `X-Hub-Signature-256` (`sha256=<hex>`); API backfill uses least-privilege token later |
| Permissions (least) | Deployments: read; Contents: read (for SHA); Metadata: read |
| Claim level | **not_measured** until coverage ledger rows land |
| Causality | Linkage only — never root-cause from deploy adjacency alone |

## Edge strength (from research)

| Edge | Strength | Requires |
| --- | --- | --- |
| deploy → commit_sha | strong | GitHub deployment `sha` or status with ref SHA |
| deploy → environment | strong | non-empty environment string |
| deploy → release | medium | release tag/name present and matched |
| time-window only | weak | never causal proof |

## Privacy

- Deployment `description`, payload, and log bodies: ref/redacted by default
- Actor login retained as structural identity; email never stored
- Raw webhook payloads are short-lived accept records, not agent-visible

## Primary sources

- [GitHub Deployments API](https://docs.github.com/en/rest/deployments/deployments)
- [GitHub webhooks](https://docs.github.com/en/webhooks/webhook-events-and-payloads)
- [Validating webhook deliveries](https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries)

## Open gates

1. HTTP endpoint + durable Turso state + delivery-id idempotency
2. API backfill/reconciliation under rate limits
3. Bundle projection + doctor coverage diagnostics
4. Measured claim-level rows
