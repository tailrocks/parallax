# Plan 116 — Retention and prune lifecycle

**Status:** DONE (2026-07-17)

## Closed claims

| Claim | Evidence |
|---|---|
| Approved lifecycle contract | `docs/research/decisions/retention-and-prune-contract.md` + xtask `product.retention-decision` |
| Deterministic plan (dry-run default) | `parallax prune` / `parallax prune --json` |
| Turso discovery for all current classes | Unit tests + dry-run JSON |
| Issue cascade + invocation capability | Unit tests prove eligible predicates, cascade, unresolved/active preservation, and idempotence; product execution remains fail-closed pending Plan 106 protection |
| Durable journal transitions | Journal unit tests (create/retry/complete) |
| CLI confirmation and protection gate | `--execute` requires `--yes`; metadata execution also requires a non-placeholder Plan 106 protection snapshot |
| Native TTL reconcile incl. metric catalog | `GreptimeStore::reconcile_ttls` ALTERs fixed tables + discovered metric families |
| Physical reclaim honesty | CLI report notes async Greptime compaction |

## Dry-run capture

[2026-07-plan-116-prune-dry-run.json](../2026-07-plan-116-prune-dry-run.json) — plan over a QA metadata copy: 55 unresolved issues excluded, 132 active invocations excluded, dashboards/alerts retained_by_policy, spool disclosed.

## Commands

```sh
parallax prune              # dry-run human
parallax prune --json       # dry-run machine
parallax prune --execute --yes   # confirmed; fails closed while pins:none
```

## Tests

```sh
cargo nextest run --locked -p parallax-metadata -E 'test(/prune/)'
```

## Execution limitation

- Live evidence pin storage remains owned by Plan 106. The current CLI records
  `protection_generation=pins:none`, so core authorization deliberately rejects
  metadata execution with `ProtectionUnavailable`. This is not evidence that
  issue/invocation pruning works end-to-end; only dry-run, guarded capabilities,
  and predicate-level unit tests are proven. When Plan 106 lands, it must supply
  bounded reachability exclusions and a stable generation before this execution
  path can authorize.
- Greptime raw-signal “delete rows” is TTL/compaction, not synchronous disk reclaim — contract and CLI state this explicitly.
