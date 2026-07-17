# Plan 123 — Fixer offline outcome residual (closed 2026-07-17)

> **Pass 154:** offline unit tests re-ran green (`fixer_outcome` ×3). This does
> **not** close research-agenda #6 live replay / Detect trigger ledger — see
> [loop-stage-claim-status-recheck-2026-07-17.md](../loop-stage-claim-status-recheck-2026-07-17.md).

## Scope closed (offline only)

| Residual | Landed surface |
| --- | --- |
| Versioned request/outcome SM | `parallax_evidence::fixer_outcome` |
| Offline multi-arm harness | unit tests: PR≠success, review-required success, unmerged/recurrence arms |
| Append-only Turso outcomes | `fixer_outcomes` table + immutable hash PK |
| Draft-PR adapter | **deferred** — not in Parallax core (STOP: optional after offline gates) |
| Read-only feedback surface | Turso latest/count inventory; no automatic policy learning |

## Non-goals reaffirmed

- Parallax core has no checkout/patch/PR/merge/deploy ownership.
- Opened PR is never success.
- Success requires human review + no runtime recurrence.

## Verification

```text
cargo test -p parallax-evidence --lib fixer_outcome
cargo test -p parallax-metadata --lib append_only_preserves
cargo xtask policy --only structural
```
