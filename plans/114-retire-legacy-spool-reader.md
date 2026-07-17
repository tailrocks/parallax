# Plan 114: Retire the legacy NDJSON spool reader

> **Executor instructions**: Do not remove compatibility until a released
> raw-frame writer completed one full release cycle and every supported legacy
> segment is older than its maximum retention window.

## Status

- **Priority**: P2 when triggered
- **Effort**: S
- **Risk**: HIGH
- **Depends on**: Qualifying stable release cycle + expired legacy segments
- **Category**: ingest / compatibility retirement
- **Status**: BLOCKED
- **Blocker**: Only published tag is rolling `preview`
  (`0.1.0-preview.1295+e37a65d`, prerelease, 2026-06-15). No qualifying
  stable release cycle for raw-frame writer + full compatibility window
  (recheck 2026-07-17T16:08Z UTC; sole tag still `preview`).

## Residual only (after trigger)

1. Prove first raw-frame release + one full compatibility cycle + max retention
   inventory (doctor telemetry across upgrade paths).
2. Pre-removal doctor/release warning; then remove NDJSON parser, dispatch,
   deps, mixed fixtures, compatibility promises.
3. Unknown legacy files → actionable upgrade error (never silent delete).
4. Re-run crash/replay/rotation/reaper/prune/doctor + upgrade rehearsal.

## Done Criteria

- [ ] Release-cycle + retention evidence complete.
- [ ] Legacy parser/dispatch/deps/promises removed.
- [ ] Unknown old files fail clearly; current spool gates green.

## STOP / Remove When

STOP if any supported install may still hold a readable legacy segment.
Delete after removal + verification, or operator permanently keeps the reader.
