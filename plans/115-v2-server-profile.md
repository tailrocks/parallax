# Plan 115: Define and ship the supported V2 server profile

> **Executor instructions**: This plan is blocked until the operator opens V2.
> GreptimeDB + Turso remain mandatory in every profile; do not revive Postgres,
> ClickHouse, `none`, or another engine from superseded projections. Complete
> authentication/context plan 109 before exposing a remote surface.

## Status

- **Priority**: P1 when V2 opens
- **Effort**: XL
- **Risk**: CRITICAL
- **Depends on**: 102, 109; operator opens V2 server scope
- **Category**: V2 / deployment / operability
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED
- **Blocker**: V2 server profiles are not open and no supported server hardware/
  workload/availability contract exists.

## Current Evidence

Historical architecture commits to a server goal, but current product support
is local-first. Old projections mention Postgres/ClickHouse fallbacks that are
now forbidden. Plan 110 intentionally presupposes, rather than creates, a
supported profile.

## Scope

In scope after the trigger:

- One supported self-hosted server profile with explicit hardware/workload,
  networking, native TLS, auth, storage, retention, backup/restore, upgrade,
  observability, and support boundaries.
- Managed/external GreptimeDB plus Turso composition only.
- Remote CLI context acceptance using plan 109 and deterministic artifacts from
  plan 102.

Out of scope:

- Multi-tenant SaaS, Kubernetes fleet management, alternate databases, hidden
  local fallbacks, or concurrency tuning before plan 110's measurement trigger.

## Steps After Trigger

1. Approve an ADR naming the supported topology, SLOs, hardware floor, workload
   envelope, availability/durability promise, ports, trust boundary, data dirs,
   retention, backup/restore, upgrade/rollback, and unsupported configurations.
2. Reconcile every historical server/cloud/Postgres/ClickHouse claim with the
   mandatory GreptimeDB+Turso stack and plan 109's auth/context contract.
3. Implement validated server configuration and startup composition with
   native TLS, least privilege, explicit external endpoints, progress output,
   health/readiness, graceful drain, and no in-memory product path.
4. Implement backup/restore, upgrade/rollback, disk pressure, retention, and
   disaster-recovery runbooks with live rehearsals.
5. Package and release the profile through plan 102's verified artifacts. Add
   remote CLI acceptance for issue/bundle/trace flows and failure modes.
6. Publish a reproducible supported-profile load packet; only measured worker
   saturation may unblock plan 110.

## Test Plan

- Config/topology/auth/TLS positive and negative matrix.
- Greptime/Turso unavailable, slow, corrupt, full-disk, restart, backup,
  restore, upgrade, rollback, and shutdown scenarios.
- Remote context and access-scope acceptance.
- Release install/verify/uninstall on supported server targets.
- Supported workload soak and resource/queue/storage evidence.

## Done Criteria

- [ ] Operator opens V2 and approves one explicit support contract/ADR.
- [ ] Every profile uses GreptimeDB + Turso with native TLS and no fallback.
- [ ] Plan 109 protects all remote ingest/query/management surfaces.
- [ ] Backup/restore/upgrade/rollback and failure runbooks pass live rehearsals.
- [ ] Verified release artifacts install on every supported server target.
- [ ] Remote CLI dogfood and supported workload/SLO evidence pass.
- [ ] Plan 110 receives a reproducible profile packet, not an assumed target.

## STOP Conditions

- V2 or its support/SLO contract is not approved.
- The design requires an alternate database, rustls, hidden fallback, or
  unauthenticated remote endpoint.
- Backup/restore or upgrade behavior cannot meet the approved durability claim.
- A concurrency change is proposed before supported-profile measurement.

## Remove When

Delete this plan and index row when V2 is rejected explicitly or one approved
server profile is released, operationally rehearsed, and support-evidenced.
