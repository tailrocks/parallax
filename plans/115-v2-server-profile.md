# Plan 115: Define and ship the supported V2 server profile

> **Executor instructions**: GreptimeDB + Turso mandatory; no Postgres/ClickHouse
> fallbacks. Use retired plan 109 bearer + context contract for remote surfaces;
> plan 102 verified release artifacts for packaging. Do not open plan 110
> concurrency without measured profile saturation.

## Status

- **Priority**: P1
- **Effort**: XL
- **Risk**: CRITICAL
- **Depends on**: Auth contract (plan 109 retired minimal slice);
  release pipeline (plan 102 retired four-target proof)
- **Category**: V2 / deployment / operability
- **Status**: IN PROGRESS — live non-loopback lab rehearsal landed; TLS-edge /
  upgrade / install / plan-110 isolation residual
- **Decision**:
  [`docs/research/decisions/v2-server-profile.md`](../docs/research/decisions/v2-server-profile.md)
- **Blocker**: none for remaining hardening packets

## Residual only

1. ~~ADR~~ landed (`docs/research/decisions/v2-server-profile.md`).
2. ~~Validated example config composition~~ landed
   (`docs/research/validation/2026-07-plan-115-v2-server-profile/example-config.toml`
   + `Config::load` unit gate).
3. ~~Rehearsal checklist + live non-loopback packet~~ landed
   ([`rehearsal-checklist.md`](../docs/research/validation/2026-07-plan-115-v2-server-profile/rehearsal-checklist.md),
   [`live-rehearsal-2026-07-17.md`](../docs/research/validation/2026-07-plan-115-v2-server-profile/live-rehearsal-2026-07-17.md)):
   bind `0.0.0.0` + bearer, GraphQL 401/200, remote CLI context, backup snapshot,
   prune dry-run, GraphQL micro-RPS + invocation timings. Still open: operator
   TLS edge, upgrade/rollback cutover log, disk-pressure reclaim, four-target
   install dogfood.
4. Package via verified release pipeline; broader remote CLI dogfood for
   issue/bundle/trace against TLS-edge profile.
5. Publish load packet that can unblock plan 110 only after measured worker
   saturation (current micro-packets **do not** open plan 110).
6. OTLP ingest tokens (deferred from auth minimal slice) belong here if the
   profile exposes remote ingest.

## Done Criteria

- [ ] Explicit support contract/ADR approved and implemented.
- [ ] Every profile uses GreptimeDB + Turso with native TLS; no fallback.
- [ ] Auth protects remote ingest/query/management surfaces.
- [ ] Backup/restore/upgrade/rollback rehearsals pass.
- [ ] Verified artifacts install on every supported server target.
- [ ] Remote CLI dogfood + workload/SLO evidence; plan 110 packet produced.

## STOP / Remove When

STOP on alternate DB, rustls, hidden fallback, unauthenticated remote, or
concurrency tuning before measurement. Delete when one profile is released and
support-evidenced, or V2 server scope is rejected.
