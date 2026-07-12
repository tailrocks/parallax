# Storage — Metadata Store (evidence)

Evidence for the relational metadata store (product metadata, agent-session/CLI state, audit
records). The **decision** — Turso is mandatory, with no product fallback — lives in
[../../decisions/metadata-store.md](../../decisions/metadata-store.md).

- [metadata-store-benchmark-plan.md](metadata-store-benchmark-plan.md) — Turso hardening benchmark and runnable research-prototype spec; Postgres appears only as a comparison baseline.
- [turso-metadata-production-readiness.md](turso-metadata-production-readiness.md) — Turso production-readiness protocol: source posture, local-vs-cloud, MVCC/CDC/sync constraints, backup/restore, and fix-forward triggers.
