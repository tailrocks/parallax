# GitHub deploy/change context adapter (plan 121)

**Status:** first GitHub webhook ingest slice implemented; broader coverage remains fixture-gated
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

The HTTP webhook endpoint and durable ingest path are implemented. Plan 124
(CI evidence) closed 2026-07-17 with REST backfill + claim rows. Plan 121
residual gates:

1. Deploy API backfill/reconciliation under rate limits
2. Broader deploy/change entity coverage beyond deployment webhooks
3. Bundle projection for deploy adjacency (linkage helpers landed; API wiring residual)
4. Measured claim-level rows (Turso `evidence_claim_rows` domain `deploy_context` seeded on webhook accept)
