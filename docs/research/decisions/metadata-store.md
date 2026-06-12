# Metadata Store — Turso, Committed (No Fallback Engines)

<!-- markdownlint-disable MD013 -->

Decision date baseline: 2026-05-29; **superseded 2026-06-12 by operator decision** (evidence in [../storage/metadata/](../storage/metadata/)).

> **Decision (operator, 2026-06-12) — Turso is the metadata store, period.** The stack is
> **GreptimeDB (telemetry) + Turso (metadata), both mandatory** — the product's functionality
> is built on exactly these two engines, so neither ships with a fallback engine behind it.
> No rusqlite feature flag, no Postgres swap-out. When Turso misbehaves, the answer is
> fix-forward in our code or upstream — never an engine substitution. The in-memory
> `TelemetryStore` adapter exists for tests and dev harnesses only; it is not a product mode.
> The relational metadata store (product metadata, issue state, agent-session and
> CLI-invocation state, outcome and audit records) stays **separate from the columnar
> telemetry engine** ([storage-engine.md](storage-engine.md)) — do not put metadata in
> GreptimeDB/ClickHouse.
>
> *Historical baseline (2026-05-29, superseded):* Turso-first for the tiny tier with Postgres
> as the production/scale-out fallback. That fallback posture is retired; the gates below
> remain as a **hardening checklist on Turso itself**, not as swap triggers.

## Why Turso for the tiny tier

- **Rust-first + SQLite-compatible**, matching Parallax's low-overhead, self-hosted, one-binary tiny-tier goal.
- A local file / `turso dev --db-file` gives durable embedded metadata with no separate service to run — the tiny tier must not quietly depend on hosted Turso Cloud.

## The distinction that must be recorded

Never say only "Turso." Each claim must name which of these is under test:

- **libSQL** — documented production-ready (the ORM/production fallback packages).
- **Turso Database** — the newer Rust rewrite, still **beta** (latest checked `v0.6.1`, 2026-05-22; release notes still heavy on MVCC correctness fixes).
- **Local file vs Turso Cloud** — Cloud has separate durability/PITR guarantees; the self-hosted tiny tier must rely only on local/embedded persistence.

## Gates — now a hardening checklist on Turso itself

These remain the bar Turso must clear; with the fallback retired (operator, 2026-06-12), a
failed gate means fix-forward or upstream work, never an engine swap (evidence in
[../storage/metadata/turso-metadata-production-readiness.md](../storage/metadata/turso-metadata-production-readiness.md)):

1. **Crash correctness** — no metadata loss/corruption across kill/restart.
2. **Backup/restore** — a documented, tested path.
3. **Concurrency** — MVCC conflict/CDC/sync behavior adequate for Parallax's write mix.
   *First live lesson (2026-06-12): an `UPDATE` executed while another statement is open on
   the same connection reports success but does not persist — V1 scopes reads before writes
   and pins the behavior with a regression test
   (`parallax-storage::metadata::tests::update_with_open_statement_is_lost`).*
4. **Migration** — schema-migration story for evolving metadata.

This roll-up is owned by the A5 stack decision: [stack-decision.md](stack-decision.md).

## Evidence

- [../storage/metadata/metadata-store-benchmark-plan.md](../storage/metadata/metadata-store-benchmark-plan.md) — Turso-first benchmark plan and runnable prototype spec, with Postgres fallback gates.
- [../storage/metadata/turso-metadata-production-readiness.md](../storage/metadata/turso-metadata-production-readiness.md) — production-readiness gate: source posture, local-vs-cloud distinction, MVCC/CDC/sync constraints, backup/restore, fallback triggers.
