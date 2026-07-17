# GitHub deploy/change context adapter (plan 121)

**Status:** Plan 121 DONE/closed (2026-07-17) — webhook + REST backfill + claim rows +
GraphQL linkage-only adjacency shipped; see
[validation/2026-07-plan-121-deploy-context/README.md](../validation/2026-07-plan-121-deploy-context/README.md).
Broader entity coverage is design-only (no active plan owner).
**Decision date:** 2026-07-17  
**Approver:** operator unblock directive (plan 121 provider = GitHub)  
**Owner:** closed — plan 121 deleted; future expansion needs a new plan

## Decision

Parallax's first deploy/change provider is **GitHub** (read-only).

| Field | Contract |
| --- | --- |
| Provider | GitHub |
| REST API version header | `2022-11-28` (fixtures also record `X-GitHub-Api-Version` when present) |
| Webhooks (shipped accept set) | `deployment`, `deployment_status` (deploy path); CI path also accepts `workflow_job` (plan 124 DONE) on the same `POST /webhooks/github` router |
| Later entities | `push`/`create` release markers, PR/file list, `check_run`, `workflow_run`, PR reviews (not handlers today) |
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

## Status

Plan 121 closed 2026-07-17: webhook + REST Deployments backfill + claim rows +
GraphQL linkage-only bundle adjacency. Broader entity coverage (push/release/
PR file lists) is out of this plan's residual and may open a future plan if
measured need appears.
