# Metadata Store — Turso-first, Postgres Fallback

<!-- markdownlint-disable MD013 -->

Decision date baseline: 2026-05-29 (decision extracted from the metadata research; evidence in [../storage/metadata/](../storage/metadata/)).

> **Decision — Turso-first for prototype/tiny-tier metadata, with Postgres as the active
> production and scale-out fallback.** The relational metadata store (product metadata, issue
> state, agent-session and CLI-invocation state, outcome and audit records) is **separate from
> the columnar telemetry engine** ([storage-engine.md](storage-engine.md)) — do not put metadata
> in GreptimeDB/ClickHouse. Turso is chosen for the tiny tier because it is Rust-written and
> SQLite-compatible (one-file local persistence, trivial to self-host). It is **not yet a
> production default**: Turso Database is still beta, so crash, backup/restore, concurrency, and
> migration gates must pass before any production-default claim; until then Postgres is the real
> fallback.

## Why Turso for the tiny tier

- **Rust-first + SQLite-compatible**, matching Parallax's low-overhead, self-hosted, one-binary tiny-tier goal.
- A local file / `turso dev --db-file` gives durable embedded metadata with no separate service to run — the tiny tier must not quietly depend on hosted Turso Cloud.

## The distinction that must be recorded

Never say only "Turso." Each claim must name which of these is under test:

- **libSQL** — documented production-ready (the ORM/production fallback packages).
- **Turso Database** — the newer Rust rewrite, still **beta** (latest checked `v0.6.1`, 2026-05-22; release notes still heavy on MVCC correctness fixes).
- **Local file vs Turso Cloud** — Cloud has separate durability/PITR guarantees; the self-hosted tiny tier must rely only on local/embedded persistence.

## Gates before a production-default claim

Turso may not be called the production metadata default until it passes (evidence in
[../storage/metadata/turso-metadata-production-readiness.md](../storage/metadata/turso-metadata-production-readiness.md)):

1. **Crash correctness** — no metadata loss/corruption across kill/restart.
2. **Backup/restore** — a documented, tested path.
3. **Concurrency** — MVCC conflict/CDC/sync behavior adequate for Parallax's write mix.
4. **Migration** — schema-migration story for evolving metadata.

If any gate fails for the production tier, **Postgres** is the fallback (mature backup/restore and
concurrency; heavier than the tiny tier, but real). This roll-up is owned by the A5 stack decision:
[stack-decision.md](stack-decision.md).

## Evidence

- [../storage/metadata/metadata-store-benchmark-plan.md](../storage/metadata/metadata-store-benchmark-plan.md) — Turso-first benchmark plan and runnable prototype spec, with Postgres fallback gates.
- [../storage/metadata/turso-metadata-production-readiness.md](../storage/metadata/turso-metadata-production-readiness.md) — production-readiness gate: source posture, local-vs-cloud distinction, MVCC/CDC/sync constraints, backup/restore, fallback triggers.
