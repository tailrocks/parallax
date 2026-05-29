# Storage — Metadata Store (evidence)

Evidence for the relational metadata store (product metadata, agent-session/CLI state, audit
records). The **decision** — Turso-first, Postgres fallback — lives in
[../../decisions/metadata-store.md](../../decisions/metadata-store.md).

- [metadata-store-benchmark-plan.md](metadata-store-benchmark-plan.md) — Turso-first benchmark plan and runnable prototype spec, with Postgres fallback gates.
- [turso-metadata-production-readiness.md](turso-metadata-production-readiness.md) — Turso production-readiness gate: source posture, local-vs-cloud, MVCC/CDC/sync constraints, backup/restore, fallback triggers.
