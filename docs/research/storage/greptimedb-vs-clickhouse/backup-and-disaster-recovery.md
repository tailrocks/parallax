# Backup & Disaster Recovery (ops surface)

<!-- markdownlint-disable MD013 -->

Status: **Run 174 (2026-07-17)** — gap-ledger item #4 (engine-layer backup/DR was
"not addressed"). Source + live Docker on pins **GT `v1.1.3`** /
**CH `26.6.1.1193`**. This is the **engine** story. **Product RPO/RTO runbook
(domains D1–D3, cadence, restore order):**
[`product-rpo-runbook.md`](product-rpo-runbook.md) (Run 222).

## One-line comparison

| Axis | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Primary backup primitive | **Export** (`COPY` SQL + `greptime cli data export[-v2]`) + **metadata snapshot** (`greptime cli meta snapshot`) | First-class **`BACKUP` / `RESTORE` SQL** to File/Disk/S3/Azure |
| Data durability model | **S3-native SSTs** (OpenDAL) → object store *is* the durable copy when deployed that way; local is cache | **Local/attached parts** first; S3 is tier or Cloud SharedMergeTree |
| Live smoke (Run 174) | `COPY spans1m TO parquet` → 100k rows / **508 KiB** / 279 ms; `COPY DATABASE public` → **1.6 MiB** multi-table parquet dump | `BACKUP TABLE spans1m TO File('backups/…')` → **1.1 MiB**, 11 files; multi-table **4.4 MiB**; `RESTORE … AS spans1m_restored` → **count=100000** match |
| Config friction | Paths are container-local unless aimed at object store | `backups.allowed_path` must allow destination (default image: relative `backups`) |
| Cluster metadata | Explicit **meta snapshot** tool (etcd/PG/MySQL → file/S3) | Keeper/ZooKeeper state is a separate recovery plane from table BACKUP |

## GreptimeDB — mechanism

**Data path (export, not “hot backup of LSM files”).**

- SQL: `COPY <table> TO '<path>' WITH (FORMAT='parquet'|…)` and
  `COPY DATABASE <db> TO '<dir>/' WITH (FORMAT='parquet')` — live-proven Run 174.
  `COPY DATABASE` failed while a broken empty `JSON2` table remained (arrow Struct
  align error); dropped empty tables → full dump succeeded (**600,002** affected-row
  counter across tables, 241 ms wall on the HTTP SQL path for this small set).
- CLI (v1.1.3 binary): `greptime cli data export|import` (legacy) and
  **`export-v2` / `import-v2`** (JSON schema + manifest). Export targets:
  `schema` / `data` / `all`; parallel DB/table jobs; connects to a running server
  (`--addr`). Data export is described as corresponding to `COPY DATABASE TO`.

**Metadata path (cluster brain).**

- `greptime cli meta snapshot {save,restore,info}` dumps/restores the **metadata
  store** (etcd / Postgres / MySQL backends) to local file or S3
  (`src/cli/src/metadata/snapshot.rs`, default `metadata_snapshot.metadata.fb`).
  Without metasrv catalog state, SST objects alone are not a self-describing cluster.

**Architectural consequence (ties to cost thesis).**

When GreptimeDB runs **object-store-native**, region SSTs already live as **1× shared
objects**. DR is less “copy every part nightly” and more:

1. continuous object-store durability + versioning/lifecycle,
2. periodic **meta snapshots**,
3. optional logical export for portability / offline analytics.

Local standalone (this Docker smoke) still needs explicit `COPY` / export because
data is on the container filesystem.

## ClickHouse — mechanism

**First-class BACKUP engine family** (`src/Backups/`, registered engines include
**File, Disk, S3, Azure** — `BackupFactory`).

- SQL: `BACKUP TABLE t TO File('backups/t')` → returns job id + `BACKUP_CREATED`.
- Multi-table: `BACKUP TABLE t1, TABLE t2, … TO File('backups/run')`.
- Restore: `RESTORE TABLE t AS t_restored FROM File('backups/t')` — live count match.
- Observability: `system.backup_log` (status, error, num_files, total_size, timings).
- Config: `<backups><allowed_path>backups</allowed_path></backups>` in server config
  (stock image). Absolute paths outside allowlist → `Code 36` BAD_ARGUMENTS.
- Cloud note in stock `config.xml`: ClickHouse Cloud provides automatic backups to
  object storage — managed path differs from OSS self-host.

**Architectural consequence.**

OSS ClickHouse backup is a **deliberate part-copy** (and for S3 destinations, a
configured backup engine). HA still multiplies storage (N× replicas) unless Cloud
SharedMergeTree. Keeper metadata is **not** replaced by `BACKUP TABLE` alone.

## Live numbers (indicative, N=100k harness tables, same host as Run 173)

| Operation | Size / result | Notes |
| --- | --- | --- |
| GT `COPY` single table `spans1m` → parquet | 508 KiB | 100k rows, 279 ms |
| GT `COPY DATABASE public` → parquet dir | 1.6 MiB | all public tables after dropping empty JSON2 leftovers |
| CH `BACKUP` `spans1m` → File | 1.1 MiB, 11 files | part files + metadata SQL |
| CH multi-table BACKUP (6 tables) | 4.4 MiB | File destination under allowed_path |
| CH `RESTORE` as `spans1m_restored` | **100000** rows | correctness pass |

Absolute sizes are not cross-engine comparable (parquet dump vs native part backup
layout) — they prove **both paths work end-to-end** on current pins.

## Adopt for Parallax (product ops, not engine swap)

1. **Telemetry (GreptimeDB, mandatory stack):** prefer **object-store deployment +
   bucket versioning** as the primary durability plane; schedule **`meta snapshot`**
   off-cluster; use `COPY` / `cli data export-v2` for logical portability and test
   restores. Do **not** treat “no BACKUP SQL” as a missing engine feature in the
   S3-native design — it is a different DR shape.
2. **Comparator honesty:** ClickHouse’s `BACKUP`/`RESTORE` SQL is **more turnkey for
   local/part-centric** deployments and BI-friendly point-in-time table copies. If
   Parallax ever exposed raw CH for internal analytics, that SQL surface is nicer.
3. **Turso metadata** (issues, auth, config) is a **separate** backup domain — neither
   engine’s telemetry backup covers it. Product RPO lists Turso + Greptime meta +
   object store — see [`product-rpo-runbook.md`](product-rpo-runbook.md).
4. **JSON2 caveat:** empty/broken structured-JSON tables can break `COPY DATABASE`
   (Run 174); validate export after schema experiments.

## Reproduce

```bash
docker compose -f bench/compose.yml up -d   # GT v1.1.3 + CH 26.6.1.1193 at minimum
# GT
docker exec parallax-bench-greptimedb-1 curl -s 'http://localhost:4000/v1/sql?db=public' \
  --data-urlencode "sql=COPY spans1m TO '/tmp/spans1m.parquet' WITH (FORMAT='parquet')"
docker exec parallax-bench-greptimedb-1 curl -s 'http://localhost:4000/v1/sql?db=public' \
  --data-urlencode "sql=COPY DATABASE public TO '/tmp/pub_copy/' WITH (FORMAT='parquet')"
docker exec parallax-bench-greptimedb-1 greptime cli data --help
docker exec parallax-bench-greptimedb-1 greptime cli meta snapshot --help
# CH (allowed_path=backups in stock image)
docker exec parallax-bench-clickhouse-1 clickhouse-client -q \
  "BACKUP TABLE spans1m TO File('backups/spans1m')"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q \
  "RESTORE TABLE spans1m AS spans1m_restored FROM File('backups/spans1m')"
docker exec parallax-bench-clickhouse-1 clickhouse-client -q \
  "SELECT count() FROM spans1m_restored"
```

Source pins: GT `v1.1.3` (`63ef18a7…`) — `src/cli/src/metadata/snapshot.rs`,
`greptime cli data|meta`; CH `v26.6.1.1193-stable` (`840482cd…`) — `src/Backups/`,
`Settings` / `config.xml` `<backups>`.

## Verdict impact

No stack flip. Closes the engine-layer **backup/DR** gap in the ledger: both systems
have **working** backup/restore paths on current pins; shapes differ (export+meta+S3
vs BACKUP SQL+parts). Product still owes a combined Greptime+Turso runbook and a
managed-cloud vs self-host cost model (gap #5).
