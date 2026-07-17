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
- **Status**: IN PROGRESS — ADR landed; config/rehearsal residual
- **Decision**:
  [`docs/research/decisions/v2-server-profile.md`](../docs/research/decisions/v2-server-profile.md)
- **Blocker**: none for validated config composition + rehearsals

## Residual only

1. ~~ADR~~ landed (`docs/research/decisions/v2-server-profile.md`).
2. ~~Validated example config composition~~ landed
   (`docs/research/validation/2026-07-plan-115-v2-server-profile/example-config.toml`
   + `Config::load` unit gate).
3. ~~Rehearsal checklist~~ landed
   (`docs/research/validation/2026-07-plan-115-v2-server-profile/rehearsal-checklist.md`).
   Still open: **live** non-loopback/TLS-edge run logs, measured backup/restore/
   upgrade/rollback/disk-pressure packets.
4. Package via verified release pipeline; remote CLI acceptance for
   issue/bundle/trace.
5. Publish load packet that can unblock plan 110 only after measured worker
   saturation.
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
