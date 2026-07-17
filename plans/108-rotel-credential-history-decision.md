# Plan 108: Decide whether lab credentials require history remediation

> **Executor instructions**: This is blocked on an operator fact and explicit
> authorization. Do not inspect, rewrite, force-push, rotate, or publicize
> suspected secret values beyond the minimum safe metadata without approval.

## Status

- **Priority**: P1 if exposure is confirmed
- **Effort**: S-L depending on decision
- **Risk**: CRITICAL
- **Depends on**: Operator decision
- **Category**: security / Git history
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED
- **Blocker**: The operator must confirm whether non-default OpenObserve/Sentry
  credentials ever entered Git history and authorize any rotation/history rewrite.

## Preliminary execution landed (helper agent, 2026-07-17) — peer verify + retire

The operator's unblock directive (2026-07-17, `plans/README.md`) decided: NO
history rewrite; run the scan, rotate anything found, record evidence. The
scan and the secret-free record are done:
[docs/research/security/credential-history-scan-2026-07-17.md](../docs/research/security/credential-history-scan-2026-07-17.md)
— gitleaks 8.30.1 over 1,329 commits (5 findings: 4 intentional test
canaries/doc prose, 1 lab-internal ingest-only Sentry DSN on a non-routable
lab TLD; no real exposure) plus a clean current-tree scan at `b680fd8`.
Peer: verify the classification, confirm no rotation is actionable (the
self-hosted lab instance is down; DSN regeneration applies only if the
compose stack is revived), then retire this plan per the lifecycle.

## Current Evidence

The working-tree lab configuration was replaced by a safe
`bench/otlp-fanout/rotel.env.example` in completed work. Whether prior values
were real/non-default and require rotation or destructive history remediation
is intentionally unresolved. A rewrite affects every clone and remote ref and
cannot be inferred or authorized by an agent.

## Scope

In scope after the operator classifies the values:

- Secret-safe history/ref inventory, provider rotation/revocation, and current
  tree/history verification.
- A coordinated history rewrite only when explicitly authorized.
- A durable incident/no-incident decision that never records secret values.

Out of scope:

- Any destructive Git operation, force-push, provider call, or secret-value
  disclosure before explicit authorization.

## Steps After Trigger

1. Obtain an operator answer on whether the historical values were real and
   whether they remain valid. Record no secret values in the plan or logs.
2. If they were defaults/non-secrets, record the decision and evidence, verify
   the current tree/scanners are clean, and retire this plan.
3. If real, rotate/revoke at each provider first and capture non-secret
   revocation identifiers/timestamps.
4. Inventory affected commits/refs/clones with secret-safe hashes or rule IDs.
   Decide whether revocation alone is sufficient or an operator-authorized
   history rewrite is required.
5. For an authorized rewrite, prepare a coordination/runbook, backups, remote
   protection changes, collaborator notification, force-push sequence, clone
   invalidation, and post-rewrite scan. Execute only at the approved window.
6. Restore protections, verify all remote refs and current examples, and store
   a secret-free incident/decision record.

## Test Plan

- Current-tree and full-history scans using secret-safe output.
- Provider confirmation that exposed credentials are revoked/rotated.
- If rewritten: all remote refs scanned, old object reachability checked, and
  fresh clone verified at the authorized replacement history.
- Repository protections and CI restored and green.

## Done Criteria

- [ ] Operator classifies the historical values and required response.
- [ ] Any real credential is revoked/rotated before further remediation.
- [ ] Any history rewrite has explicit operator authorization and coordination.
- [ ] Current tree and required history/ref scope scan clean.
- [ ] A secret-free durable decision/incident record exists.

## STOP Conditions

- Operator classification or destructive-history authorization is absent.
- A command would reveal secret material in logs/chat/artifacts.
- Rotation/revocation has not completed before an authorized rewrite.
- Collaborator/remote/protection coordination is incomplete.

## Remove When

Delete this plan and index row after the operator's classification is recorded
and every required rotation, scan, or explicitly authorized rewrite is verified.
