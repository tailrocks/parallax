# Product RPO / RTO Runbook — GreptimeDB + Turso

<!-- markdownlint-disable MD013 -->

Status: **Run 222 (2026-07-17)** — closes the *product* half of gap-ledger item
#5 (engine backup surface was Run 174; this is the ops packet: **what to
protect, target RPO/RTO, backup cadence, restore order**). Not a substitute for
a practiced fire-drill. Stack policy: **GreptimeDB (telemetry) + Turso
(metadata)**; ClickHouse appears only as comparator notes.

Companion notes:

- Engine primitives: [`backup-and-disaster-recovery.md`](backup-and-disaster-recovery.md)
- Metadata decision: [`../../decisions/metadata-store.md`](../../decisions/metadata-store.md)
- Turso hardening: [`../metadata/turso-metadata-production-readiness.md`](../metadata/turso-metadata-production-readiness.md)
- Object-store cost: [`storage-cost-and-tiering.md`](storage-cost-and-tiering.md)

## Three independent durability domains

Parallax state does **not** live in one backup.

| Domain | Store | What is lost if this domain dies | Primary durability lever |
| --- | --- | --- | --- |
| **D1 — Raw telemetry** | GreptimeDB mito2 SSTs (prefer **S3-native**) | Logs / traces / metrics / profiles history | Object-store durability + versioning; optional logical `COPY` / `export-v2` |
| **D2 — Greptime catalog** | Greptime **metasrv** backend (etcd / Postgres / MySQL) | Table schemas, region routes, pipeline registry — SSTs become orphaned objects | Periodic `greptime cli meta snapshot {save,restore}` to off-cluster file **or S3** |
| **D3 — Product metadata** | **Turso** (local file / embedded for tiny tier; Cloud only if explicitly chosen) | Issues, groupings, authz/config, mutable workflow state | File copy + SQLite-compatible dump; Cloud PITR **only** if on Turso Cloud |

Neither D1 export nor D2 meta snapshot covers D3. Neither D3 dump covers D1/D2.
A “restore Parallax” procedure must sequence **all three**.

## Target envelopes (defaults to refine with product SLOs)

These are **planning defaults**, not contractual SLAs. Tighten once workload
mix and customer tiers exist.

| Class | RPO (max data loss) | RTO (max restore time) | Notes |
| --- | --- | --- | --- |
| **Tiny / single-node self-host** | Telemetry: **≤15 min** (WAL + S3 flush); Meta catalog: **≤24 h** (daily snapshot); Turso: **≤1 h** (hourly file + daily dump) | **≤4 h** to queryable evidence-bundle path | Prefer R2/S3 versioning over heroic local disks |
| **HA self-host** | Telemetry: **≤5 min** (remote WAL + multi-datanode); Meta: **≤1 h**; Turso: **≤15 min** (replicated file or frequent dump) | **≤1 h** for read path; write path may lag | Region rebalance is not a backup |
| **Managed GT + local Turso** | Telemetry/meta: vendor RPO; Turso still **operator-owned** | Vendor RTO + Turso restore | Do not assume managed GT backs Turso |

**Evidence-bundle honesty:** losing D3 (issues) while keeping D1 (raw spans/logs)
means users can still investigate by `trace_id` but **lose grouped-error
workflow state**. Losing D1 while keeping D3 leaves issue shells without raw
evidence. Both are partial disasters — document which customers care about which.

## Backup cadence (minimum viable)

### D1 — Telemetry (GreptimeDB)

**Production shape (preferred):** `[storage] type = S3` (or GCS/Azure via OpenDAL).

| Job | Cadence | Action |
| --- | --- | --- |
| Object-store durability | Continuous | Bucket **versioning** + lifecycle; block public ACL; separate backup account credentials |
| SST presence | Continuous | Rely on engine flush; alert on flush/WAL lag (product metrics) |
| Logical export (portability / ransomware) | Weekly + before major upgrades | `greptime cli data export-v2` **or** `COPY DATABASE … TO` parquet dir on offline volume (Run 174 smoke) |
| Point-in-time test restore | Monthly | Stand up empty GT → import subset → `count(*)` + one Q1 `trace_id` probe |

**Local-disk-only standalone (dev / tiny without S3):** treat the data directory
as fragile — daily `COPY DATABASE` or filesystem snapshot of the data dir **plus**
off-box copy. This is **not** the production DR model.

### D2 — Greptime meta snapshot

| Job | Cadence | Action |
| --- | --- | --- |
| Meta snapshot | **Daily** (tiny); **hourly** (HA) | `greptime cli meta snapshot save` → object store path outside the cluster VPC if possible |
| Snapshot retention | ≥30 days daily + ≥7 days hourly | Align with customer delete/legal hold policy |
| Restore drill | Quarterly | `meta snapshot restore` onto disposable metasrv → verify table list + region routes |

Without D2, D1 objects are **not** a self-describing cluster (Run 174).

#### Run 405 (2026-07-18) — live CLI surface on standalone `v1.1.3`

| Primitive | Live result | Implication |
| --- | --- | --- |
| `greptime cli meta snapshot {save,restore,info}` | Subcommands present | Documented D2 tool exists in the binary |
| `meta snapshot save --backend raft-engine-store --store-addrs <path>` | **Fails** — `Empty store addresses` without addrs; with path → `Failed to parse url` / `Invalid url scheme` / **`RelativeUrlWithoutBase`** (only etcd/postgres/mysql URL forms accepted) | **Standalone** embeds meta in local raft-engine under `/greptimedb_data/metadata/*.raftlog` — the **CLI snapshot path is for external metasrv stores**, not the standalone raftlog dir. **Run 558 re-confirm:** enum still lists `raft-engine-store`; path `/greptimedb_data/metadata` still rejected |
| Standalone D2 practical backup | Copy/stop-consistent snapshot of `/greptimedb_data/metadata/` (+ full data dir if local SSTs) | Tiny self-host runbook must **not** assume `meta snapshot save` alone covers standalone |
| HA / external metasrv D2 | Still: `meta snapshot save --backend {etcd,postgres,mysql}-store --store-addrs …` (± `--s3`) | Unchanged; full restore drill still owed on a real cluster |
| `greptime cli data export-v2 create --schema-only --to file:///…` | **OK** — exported `public` + `greptime_private` DDL + `manifest.json` (snapshot id) | Complements D1 logical portability; not a meta-route restore |

**Do not close gap “D2 drill done”** for production HA until etcd/RDS metasrv snapshot+restore is practiced. Standalone tiny-tier: treat **metadata raftlog + data dir filesystem snapshot** as the D2/D1 pair.

### D3 — Turso metadata

Name the deployment shape every time (decision rule from `metadata-store.md`):

| Shape | Cadence | Action |
| --- | --- | --- |
| **Local / embedded file** (`turso dev --db-file` or product data dir) | **Hourly** file copy off-box; **daily** logical dump | Copy `*.db` (+ WAL/SHM if hot) after `PRAGMA wal_checkpoint(FULL)` when feasible; `turso db shell <db> .dump > dump.sql` (Turso CLI docs) |
| **Turso Cloud** (only if product later opts in) | Use **Cloud** backup/PITR product features | Still keep a periodic `.dump` for vendor-exit |

Logical dump load (CLI docs pattern):

```bash
turso db shell <database-name> .dump > dump.sql
turso db shell <database-name> < dump.sql   # into empty/new DB
```

**Secrets:** dump files contain issue titles, user identifiers, tokens if
mis-stored — encrypt at rest (age/sops/KMS) and restrict IAM. Do not ship dumps
to the same bucket principal that the live GT process uses for SSTs without
separation of duties.

#### Run 409 (2026-07-18) — D3 logical dump drill (local SQLite / Turso file shape)

**Not** the product schema (product Turso DDL still evolving). Smoke of the
**runbook pattern** on a stand-in issue store:

| Step | Result |
| --- | --- |
| Create WAL file DB with `issues` + `issue_events` (2 issues, 4 events) | OK |
| `PRAGMA wal_checkpoint(FULL)` | OK |
| `sqlite3 $DB .dump > dump.sql` | 26 lines, ~1 KiB |
| `sqlite3 restored.db < dump.sql` | OK |
| Row equality (`id\|fingerprint\|status`) | **ROW_MATCH=OK** (2 issues / 4 events) |

**CLI note:** host has `sqlite3` 3.51; `turso` CLI **not** installed in this
environment. For Turso Cloud or `turso db shell`, substitute the documented
`.dump` / redirect load. Pattern is SQLite-compatible either way.

**Still owed:** product-schema D3 against real `parallax-metadata` DDL + CI
fixture; encrypt-at-rest path for dump files.

## Restore order (cold site / total loss)

Assume empty machines, intact off-box backups.

1. **Restore secrets & config** (Parallax config, S3 keys, Turso path, metasrv DSN)
   from the secrets manager — not from the DB dumps.
2. **D2 — metasrv** from latest good `meta snapshot` → cluster knows tables/regions.
3. **D1 — telemetry**
   - S3-native: point datanodes at the **existing** bucket (or version-restored
     bucket); verify region open + sample `SELECT count(*)`.
   - Logical-only: `import-v2` / `COPY FROM` parquet into fresh storage (slow;
     last-export RPO only).
4. **D3 — Turso** restore file or apply `.dump` to empty DB; run product migrations
   if schema version differs.
5. **Parallax app** start; smoke:
   - GraphQL/health ready banner
   - one known `trace_id` evidence fetch (D1)
   - one issue list / fingerprint (D3)
   - ingest canary OTLP → visible within freshness SLO
6. **Record** actual RPO (timestamp of last good event vs outage start) and RTO
   (wall clock) in the incident doc.

**Do not** restore Turso before meta if the app hard-requires both — but
prefer **meta + telemetry before app traffic**, then Turso, so raw investigate
works even if issue state lags.

## Failure mode cheat sheet

| Failure | Domain | First response |
| --- | --- | --- |
| Single datanode death | D1 | Metasrv rebalance / reopen regions from object store (no bulk copy when S3-native) |
| Metasrv corruption | D2 | Restore meta snapshot; do **not** wipe S3 |
| S3 ransomware / mass delete | D1 | Bucket versioning restore / Object Lock legal hold |
| Turso file corruption | D3 | Last hourly file or daily dump; re-link issues may desync from raw if fingerprints changed |
| Bad app migration | D3 (usually) | Roll back schema dump; telemetry usually untouched |
| Region-wide cloud outage | All | Warm standby in second region **only if** multi-region was designed (not tiny-tier default) |

## ClickHouse comparator (not product path)

If an internal analytics CH were ever added: `BACKUP`/`RESTORE` SQL is more
turnkey for table-centric PITR (Run 174). It still would **not** replace Turso
or Greptime meta. Do not invent a CH-shaped runbook for the mandatory stack.

## Verification checklist (definition of “we have DR”)

- [ ] S3 (or equivalent) versioning **on** for telemetry bucket
- [ ] Meta snapshot job green in last 24 h; artifact readable from second account
- [ ] Turso hourly file **or** dump green; encrypted off-box
- [ ] Restore drill within last 90 days with measured RPO/RTO
- [ ] Secrets restore path documented (not only “ask Alice”)
- [ ] JSON2 / experimental tables not blocking `COPY DATABASE` (Run 174 caveat)

## Live D1 drill (Run 225 — 2026-07-17)

On four-way pins, N=50k `spans1m` (product path, not CH product):

| Step | Result |
| --- | --- |
| GT `COPY spans1m TO '/tmp/…/spans.parquet'` | **50,000** rows, **~280 KiB** parquet, 74 ms |
| GT `CREATE TABLE spans1m_r225 LIKE spans1m` + `COPY FROM` parquet | **count=50,000**; JOIN on `span_id` **50,000** match |
| CH comparator `BACKUP`/`RESTORE AS spans1m_r225` | **BACKUP_CREATED** → **RESTORED**, count **50,000** |

**Status:** D1 logical export/import **works end-to-end** on current pins.
D2 meta snapshot still cluster-path (standalone has no separate metasrv backend
to snapshot in this compose). D3 Turso dump not exercised here (no product
Turso fixture in `bench/`).

## Still open after Run 222 / 225

- Concrete **customer-tier SLOs** (numbers above are defaults).
- Automated job manifests (systemd/k8s CronJob YAML) in product repo when ops
  surfaces exist.
- Multi-region active-passive design (out of tiny-tier scope).
- D2 meta snapshot drill on a **cluster** compose; D3 Turso dump/restore with
  product schema checksums (`turso-metadata-production-readiness.md` A3).

## Research date

2026-07-17 — Run 222. Revisit when Turso Cloud is adopted, when metasrv backend
changes, or when object-store vendor/region changes.

#### Run 456 (2026-07-18) — export-v2 schema-only re-verify

`greptime cli data export-v2 create --schema-only --to file:///tmp/r456_export --force`
exported `greptime_private` + `public` DDL; snapshot id
`2110c353-9ca0-446d-bb01-4352d90ff454`; `public.sql` **508 lines** (grew with bench
tables). Complements D1 portability (Run 405).

