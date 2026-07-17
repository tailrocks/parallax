# Plan 116 — Retention and prune lifecycle

**Status:** DONE (2026-07-17)

## Closed claims

| Claim | Evidence |
|---|---|
| Approved lifecycle contract | `docs/research/decisions/retention-and-prune-contract.md` + xtask `product.retention-decision` |
| Deterministic plan (dry-run default) | `parallax prune` / `parallax prune --json` |
| Turso discovery for all current classes | Unit tests + dry-run JSON |
| Issue cascade + invocation execute | Unit tests (eligible delete, unresolved/active preserved, idempotent) |
| Durable journal transitions | Journal unit tests (create/retry/complete) |
| CLI confirmation | `--execute` requires `--yes` for non-interactive |
| Native TTL reconcile incl. metric catalog | `GreptimeStore::reconcile_ttls` ALTERs fixed tables + discovered metric families |
| Physical reclaim honesty | CLI report notes async Greptime compaction |

## Dry-run capture

[2026-07-plan-116-prune-dry-run.json](../2026-07-plan-116-prune-dry-run.json) — plan over a QA metadata copy: 55 unresolved issues excluded, 132 active invocations excluded, dashboards/alerts retained_by_policy, spool disclosed.

## Commands

```sh
parallax prune              # dry-run human
parallax prune --json       # dry-run machine
parallax prune --execute --yes   # destructive (confirmed)
```

## Tests

```sh
cargo nextest run --locked -p parallax-metadata -E 'test(/prune/)'
```

## Known follow-ons (not blockers for this plan)

- Live evidence pin store (plan 106) is `protection_generation=pins:none` until that plan lands.
- Greptime raw-signal “delete rows” is TTL/compaction, not synchronous disk reclaim — contract and CLI state this explicitly.
