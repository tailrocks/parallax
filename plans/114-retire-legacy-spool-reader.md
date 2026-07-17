# Plan 114: Retire the legacy NDJSON spool reader

> **Executor instructions**: Do not remove compatibility until a released
> raw-frame writer has been available for a full release cycle and every
> supported legacy segment is older than its maximum retention window. Never
> delete or reinterpret an unread segment to satisfy the trigger.

## Status

- **Priority**: P2 when triggered
- **Effort**: S
- **Risk**: HIGH
- **Depends on**: A qualifying release cycle and expired legacy segments
- **Category**: ingest / compatibility retirement
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED
- **Blocker**: No release containing the raw-frame spool writer has completed
  the required compatibility cycle; the current `preview` tag predates it.

## Current Evidence

The raw-frame spool change deliberately retained reads for legacy `.ndjson`
segments. The default reaper age is 72 hours, but elapsed wall time alone is
not release evidence and operators may configure longer retention.

- 2026-07-17 recheck: only published tag is rolling `preview`. No qualifying
  stable release cycle completed for raw-frame writer + full compatibility
  window. Trigger **false** — do not remove legacy NDJSON reader.

## Scope

- Exact release/age/inventory trigger proof, legacy-reader and fixture removal,
  mixed/legacy-only error clarity, and format documentation cleanup.

Out of scope:

- Rewriting legacy segments in place, deleting unknown files, changing the
  current frame format, or weakening recovery/durability.

## Steps After Trigger

1. Identify the first published supported version containing raw-frame writes
   and prove at least one full compatibility cycle has elapsed.
2. Inventory configured maximum segment ages and doctor telemetry from all
   supported upgrade paths; stop if any readable legacy segment can remain.
3. Add a pre-removal release/doctor warning and explicit upgrade instruction
   where needed.
4. Remove the NDJSON parser, dispatch branch, dependencies, mixed-format
   fixtures, and docs that promise compatibility. Unknown legacy files must
   fail with an actionable version/upgrade message, never disappear silently.
5. Re-run crash/replay/rotation/reaper/prune/doctor tests and a fresh upgrade
   rehearsal from the oldest still-supported release.

## Test Plan

- Trigger calculation across release and configured retention boundaries.
- Current frame corruption/restart/replay suite.
- Unsupported legacy file produces a clear non-destructive error.
- Upgrade rehearsal and doctor/prune behavior.
- Dependency/dead-code check after parser removal.

## Done Criteria

- [ ] A released raw-frame version completed the documented compatibility cycle.
- [ ] No supported retention/upgrade path can retain a legacy segment.
- [ ] Legacy parser/dispatch/dependencies and promises are removed.
- [ ] Unknown old files fail clearly and are never deleted implicitly.
- [ ] Current spool durability, replay, doctor, and prune gates pass.

## STOP Conditions

- The release-cycle or maximum-age evidence is incomplete.
- Any supported install may still contain a legacy segment.
- Removal would delete, skip, or misparse data instead of failing safely.

## Remove When

Delete this plan and index row after the trigger is proved and the legacy
reader is removed with upgrade/durability verification.
