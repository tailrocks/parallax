# Schema Evolution and Dynamic Columns (Subsystem #10)

<!-- markdownlint-disable MD013 -->

Status: pass 38. White-box teardown of how each engine absorbs **evolving OTLP
attributes** — the checklist's "schema / dynamic columns" item. This is
decision-relevant because Parallax ingests OTLP, whose attribute set drifts (a new
span/log attribute appears whenever a customer adds one); the storage layer cannot
run a migration per new field. Three sub-questions: (1) what does a manual `ALTER …
ADD COLUMN` cost, (2) does ingest of an unknown attribute auto-evolve the schema,
(3) how is a dynamic-attribute (`JSON`) column physically stored and queried. All
three are source-confirmed **and** measured live (Docker, smoke).

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`),
re-confirmed latest stable 2026-05-25.

## 1. Manual `ADD COLUMN` — both metadata-only, no rewrite

| | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Mechanism | `handle_alter.rs`: flush the active memtable, then apply a `RegionChange` action to the **manifest** (`manifest::action::RegionChange`). Existing SSTs keep their old schema; the new column is reconciled as null/default on read. | `AlterCommands.cpp`: `ADD_COLUMN` falls through `isRequireMutationStage` → returns **false** = **no mutation**. Metadata edit only; existing parts unchanged, new column read as its `DEFAULT` until a later merge rewrites the part. |
| SST/part rewrite? | **No** (manifest edit; one memtable flush) | **No** (pure metadata) |
| Measured | — | **`ALTER … ADD COLUMN` = 0.005 s** on a 1M-row part; part `all_1_1_0` byte-identical (3.85 MiB) and same `modification_time` before/after → confirmed not rewritten. |

So promoting a hot attribute to a typed column is cheap on both — neither rewrites
data. The difference is **not** ALTER cost; it is *who issues the ALTER*.

## 2. Schema-on-write — GreptimeDB auto-evolves, ClickHouse rejects

This is the real divergence.

- **GreptimeDB: ingest auto-adds typed columns.** The row-insert path
  (`operator/src/insert.rs:257` `create_or_alter_tables_on_demand`) compares the
  incoming payload's columns to the table and **auto-issues an ALTER to add the
  missing ones** (and auto-creates the table if absent, per `AutoCreateTableType`:
  Physical/Logical/Log/LastNonNull/Trace). **Measured:** an InfluxDB-line write of
  `weather,location=us temp=82` created `weather(location, temp, greptime_timestamp)`;
  a second write `weather,location=us,city=nyc temp=80,humidity=30,wind=5`
  **auto-added `city` (tag→PK), `humidity`, `wind` (field→DOUBLE)** — the first row
  then reads `NULL` for them (schema-on-read, no rewrite). Zero operator action; the
  new attribute lands as a **typed, indexable column**.
- **ClickHouse: ingest of an unknown column is an error.** `INSERT INTO se_test
  (ts,a,c) …` with undeclared `c` → *"Received exception from server"* (rejected).
  ClickHouse never grows the schema from insert data; a new attribute must go into a
  pre-existing `JSON`/`Map` column or be added by an explicit (cheap) ALTER in the
  ingest pipeline.

**Consequence:** GreptimeDB gives **zero-touch schema evolution** for OTLP drift —
attributes become first-class typed columns automatically. ClickHouse requires
either the dynamic `JSON` type or a managed ALTER step. This is an **ingest-ergonomics
edge to GreptimeDB** (axis: operational fit), with one risk (below).

## 3. Dynamic `JSON` column — columnar subcolumns vs binary blob

When attributes are truly arbitrary, both offer a `JSON` column — but store it
differently:

| | GreptimeDB `Json` | ClickHouse `JSON` |
| --- | --- | --- |
| Physical storage | **One binary (JSONB-style) column.** The whole document is stored per row. | **Each distinct path is its own typed subcolumn** on disk (columnar). |
| Measured | `DESC` → `attrs Json`; queried `json_get_string(attrs,'k2')` (parses the blob per row). | `JSONAllPathsWithTypes` → `('k1','Int64'),('k2','String'),('k3','Bool')`; `attributes.k2` reads **only that subcolumn**. |
| Query a path | Per-row blob parse via `json_get_*` — no per-path skipping. | Reads one typed subcolumn — columnar, granule-skippable, ~native-column speed. |
| New key | absorbed (blob just grows) | absorbed as a new subcolumn, **no ALTER** (bounded by `max_dynamic_paths`; overflow paths share a structure) |

→ For *querying* dynamic attributes by path at volume, **ClickHouse's JSON is
structurally faster** (typed columnar subcolumn vs whole-blob parse). GreptimeDB's
faster path for queryable attributes is **not** its JSON type but its schema-on-write
**typed columns** (§2) — which are columnar and indexable. So the engines reach
"fast dynamic attributes" by different routes: ClickHouse via columnar JSON
subcolumns; GreptimeDB via auto-grown typed columns.

## High-cardinality consequence (the cost/ops catch)

- **GreptimeDB auto-add risk: column explosion.** If attribute *keys* are unbounded
  (user-supplied, per-request keys), schema-on-write keeps adding columns → many
  sparse columns + manifest/metadata bloat. The mitigation is to route arbitrary
  attrs into a `JSON` column and let only *stable* tags/fields become columns.
- **ClickHouse JSON cap: `max_dynamic_paths`.** The JSON type bounds how many paths
  get their own subcolumn; beyond it, extra paths fall into a shared structure
  (slower). Bounds metadata growth but needs tuning if attribute keys are wild.

Both therefore want the same discipline for Parallax: **stable attributes → typed
columns; arbitrary attributes → a JSON column.** The implementation notes already
chose `attributes JSON` on both sides — correct; this note explains why and what each
JSON column costs to query.

## Parallax implication and axis consequence

- **Ingest ergonomics (operational fit):** GreptimeDB's auto-schema-on-write means new
  OTLP attributes need no migration and no collector-side schema management — they
  appear as typed columns. ClickHouse needs the JSON column (no ALTER) or a managed
  ALTER pipeline. **Edge: GreptimeDB**, for zero-touch OTLP drift.
- **Dynamic-attribute query speed:** if Parallax queries arbitrary attributes by path,
  ClickHouse's columnar JSON subcolumns beat GreptimeDB's blob-parse `json_get_*`.
  **Edge: ClickHouse**, for path queries over a JSON attribute column.
- **Cost:** neither ALTER rewrites data (both metadata-only, measured/​confirmed), so
  schema change is not a cost axis; the cost risk is GreptimeDB column-explosion vs
  ClickHouse `max_dynamic_paths` overflow — both managed by the same "JSON for
  arbitrary keys" discipline.

Net: this subsystem does **not** move the raw-speed headline. It is an
**ingest-ergonomics edge to GreptimeDB** (auto-evolution) and a **dynamic-attr-query
edge to ClickHouse** (columnar JSON), both mechanism-confirmed and measured. It
reinforces, not flips, the standing verdict.

## Honest caveats

- Smoke scale; the column-explosion threshold and JSON-subcolumn query speed at
  millions of rows are not measured here — flagged for the harness if dynamic-attr
  queries become a Parallax hot path.
- ClickHouse `JSON` is the modern dynamic type (subcolumn model); `Map(String,String)`
  is the older alternative (one blob-ish pair of arrays, no per-path typing) — not
  re-tested this pass.
- GreptimeDB auto-ALTER on write costs a memtable flush per schema change; under a
  storm of *new keys* that is repeated flushes — another reason to cap auto-grown
  columns and prefer JSON for unbounded keys.

## Source / evidence

- GreptimeDB: `src/mito2/src/worker/handle_alter.rs` (flush-then-`RegionChange`
  manifest edit; `need_alter`, memtable-not-empty → flush at :111),
  `src/operator/src/insert.rs:257` (`create_or_alter_tables_on_demand`,
  `AutoCreateTableType`). Live: InfluxDB-line auto-add of `city/humidity/wind`;
  `Json` type + `json_get_string`.
- ClickHouse: `src/Storages/AlterCommands.cpp` (`isRequireMutationStage` → `ADD_COLUMN`
  not a mutation; `:1131` JSON type-hint changes metadata-only). Live: `ALTER ADD
  COLUMN` 5 ms no part rewrite; unknown-column insert rejected; `JSON` paths via
  `JSONAllPathsWithTypes` + `attributes.path` subcolumn read.
- Cross-refs: `write-path-and-ingestion.md` (pass 33 schema-on-write/native ingest),
  `greptimedb-implementation.md` / `clickhouse-implementation.md` (the `attributes
  JSON` choice), `local-benchmark-results.md` (Run 18).
