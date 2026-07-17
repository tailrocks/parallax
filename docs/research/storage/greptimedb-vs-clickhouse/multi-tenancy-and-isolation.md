# Multi-Tenancy, Access Control, and Isolation

<!-- markdownlint-disable MD013 -->

Status: Run 172 (auth/isolation) + **Run 179 (2026-07-17) rate-limit / quota /
ingestion-protection surface** on pins GreptimeDB **`v1.1.3`** /
ClickHouse **`v26.6.1.1193`**. Auth source originally read at `v1.0.2` /
`26.5.1.882` (Run 172); rate-limit findings re-checked against `v1.1.3` source
tree + live Docker.

## Verdict

**ClickHouse has materially stronger engine-native tenant guardrails in OSS.** It can model tenant
isolation with database/table grants, column grants, row policies, settings profiles, and quotas.
That is useful as a backend defense layer.

**GreptimeDB OSS is too coarse for SaaS tenant isolation by itself.** The open-source auth path
supports authentication and global read/write modes; its built-in static provider authorizes every
catalog/schema. Fine-grained RBAC/ACL appears in GreptimeDB Enterprise docs, not in the OSS source path
at the pinned release. For Parallax, this means the **proxy/API layer must own tenant authorization**
if GreptimeDB is the default backend, with GreptimeDB used as storage isolation (separate schema/table)
and coarse read/write credential separation.

**Decision consequence:** this does not flip the storage-engine verdict, because Parallax already
needs a proxy for OTLP routing, ingestion shaping, bundle assembly, and product semantics. But it
does add an operational requirement: **never expose GreptimeDB directly to end users** in a SaaS
deployment. ClickHouse could be exposed to internal analysts with engine-side row policies; GreptimeDB
should stay behind Parallax unless using Enterprise RBAC/ACL or an equivalent custom auth provider.

## Mechanism comparison

| Capability | GreptimeDB OSS | ClickHouse OSS |
| --- | --- | --- |
| User auth | Static or watched-file user provider. | Full access-entity stack: users, roles, grants, row policies, quotas, settings profiles. |
| Scope of built-in privileges | Global read/write mode (`rw`, `ro`, `wo`) checked by request class. | Global, database, table, and column-level grants. |
| Row-level tenant filter | No source-confirmed OSS row policy. Must enforce in Parallax query builder or physically separate tenants. | `CREATE ROW POLICY ... USING condition ... TO role/user`; planner retrieves SELECT row-policy filters and applies them to table reads. |
| Write constraints | Global write allow/deny only in OSS default checker. | Row-policy enum has write-side placeholders, but source says only SELECT filter is currently supported; write tenant checks still belong in Parallax. |
| Quotas / rate guardrails | **Process/resource limits** (write-bytes, inflight, concurrent queries, body size, query memory pool) — not per-tenant SQL quotas. `StatusCode::RateLimited` → HTTP 429. | First-class **`CREATE QUOTA`** access entities + session **settings** (`max_execution_time`, `max_result_rows`, …). |
| Parallax shape | Proxy-enforced auth **and** ingest rate limits; optional per-tenant schema/catalog/table for blast-radius reduction. | Proxy-enforced auth plus engine-side fallback policies/quotas for internal/analyst SQL users. |

## Run 179 — rate limiting, quotas, and ingestion protection

Gap ledger residual: “Rate-limiting / quotas / ingestion protection — the proxy’s protective
layer.” Engine capabilities (what Parallax can lean on vs must own):

### ClickHouse — query quotas + settings (live-proven)

| Mechanism | Live result (26.6.1.1193) |
| --- | --- |
| `SETTINGS max_execution_time=1` on `sleep(2)` | **`TIMEOUT_EXCEEDED` (Code 159)** after ~1 s |
| `SETTINGS max_result_rows=5, result_overflow_mode='throw'` | **`TOO_MANY_ROWS_OR_BYTES` (Code 396)** |
| `CREATE QUOTA … MAX queries = N TO user` | DDL accepted; stock image also has unlimited `default` quota entity (`system.quotas`). Enforcement is real in Access stack (`EnabledQuota` throws when max exceeded — Run 172 source). Per-second burst from multiquery/`docker exec` is a weak micro-test (connection/session accounting). **Run 180 live:** a residual quota `MAX queries=2` later threw **Code 201 `QUOTA_EXCEEDED`** (`queries = 3/2`) on INSERT — enforcement confirmed under load. |
| Settings profiles / `max_concurrent_queries_for_user` | Present in `system.settings` (default 0 = unlimited) |

**Implication:** ClickHouse can enforce **query-time and analyst-path** budgets in-engine.
Ingest protection for OTLP still belongs at the collector/proxy (async_insert + app limits).

### GreptimeDB — resource/admission limits, not per-tenant quotas

Source + config (`v1.1.3` / `config/frontend.example.toml`, `standalone.example.toml`,
`datanode.example.toml`):

| Knob | Role |
| --- | --- |
| `max_in_flight_write_bytes` + `write_bytes_exhausted_policy` | Global write body memory budget; wait/fail when exhausted (`frontend.rs`, `ServerMemoryLimiter`) |
| `http.body_limit` (default 64MB) | Per-request body cap |
| `max_inflight_requests` (Prom remote-write batch path) | Backpressure before accept |
| `max_concurrent_queries` (datanode/standalone, default 0=unlimited) | Concurrent query permits + timeout |
| `query.memory_pool_size` | Agg/sort/join pool; `ResourceExhausted` when full |
| `ThrottleableRuntime` / `RuntimeRateLimiter` | Internal runtime token bucket (`common/runtime/runtime_throttleable.rs`) |
| `StatusCode::RateLimited` | Mapped to HTTP **429** / MySQL concurrent-trx / PG resource errors |

**No OSS `CREATE QUOTA` / per-tenant query budget SQL.** Multi-tenant fair-share and
per-project ingest QPS must live in **Parallax’s proxy** (or Enterprise if it adds
tenant quotas — not verified OSS).

### Parallax product implication

1. **Ingest protection (OTLP/metrics firehose):** own in proxy — batching, per-tenant token
   buckets, backpressure — both engines only offer coarse global write budgets (GT) or
   insert settings (CH), not product-tenant fairness.
2. **Query abuse (analyst/SQL/API):** CH can attach quotas/settings profiles for defense in
   depth; GT needs proxy timeouts/concurrency + optional `max_concurrent_queries` /
   memory_pool sizing.
3. Reinforces Run 172: **do not expose GT OSS as a multi-tenant user-facing SQL surface.**

## GreptimeDB source read

The OSS auth crate has a `PermissionReq` classifier and `DefaultPermissionChecker`.
It decides only whether a request is read or write, then compares that to the authenticated user's
`PermissionMode`:

- `PermissionReq::is_readonly()` classifies query/protocol operations as read vs write
  (`src/auth/src/permission.rs:41-64`, commit `0ef5451`).
- `DefaultPermissionChecker` rejects reads if `can_read()` is false and rejects writes if `can_write()`
  is false, then defaults to allow (`src/auth/src/permission.rs:109-138`).
- `PermissionMode` parses `readwrite/rw`, `readonly/ro`, and `writeonly/wo`; invalid or empty strings
  fall back to `ReadWrite` (`src/auth/src/user_info.rs:28-71`, `125-189`).

The built-in static provider authenticates users from `file:` or inline `cmd:` options, then its
`authorize(catalog, schema, user_info)` implementation is **default allow all**
(`src/auth/src/user_provider/static_user_provider.rs:31-57`, `78-90`). The watched-file provider has
the same catalog/schema-authorize shape (`src/auth/src/user_provider/watch_file_user_provider.rs:86`).
HTTP auth extracts the requested catalog/schema from header/query, authenticates, and stores the user
in `QueryContext` (`src/servers/src/http/authorize.rs:56-115`, `133-150`), but the default OSS
provider does not use that catalog/schema to restrict access.

Context7 docs check: GreptimeDB docs list static user providers for OSS and describe built-in RBAC/ACL
under Enterprise user docs. So the conservative OSS conclusion is: **coarse auth only unless Parallax
supplies a custom provider or uses Enterprise.**

## ClickHouse source read

ClickHouse carries access control as a first-class subsystem:

- `ASTGrantQuery` supports grants on `{db.table|db.*|*.*|table|*}` with optional column lists and role
  grants (`src/Parsers/Access/ASTGrantQuery.h:13-17`, commit `5b96a8d8`).
- `ContextAccess` mixes user and role rights (`src/Access/ContextAccess.cpp:44-48`) and checks access
  at global/database/table/column granularity (`src/Access/ContextAccess.cpp:842-872`).
- `ParserCreateRowPolicyQuery` parses `CREATE ROW POLICY ... ON [database.]table ... USING condition
  ... TO role/user` (`src/Parsers/Access/ParserCreateRowPolicyQuery.h:8-23`).
- The planner/analyzer retrieves the current user's SELECT row-policy filter for storage reads
  (`src/Planner/PlannerJoinTree.cpp:303-312`; `src/Analyzer/Resolve/QueryAnalyzer.cpp:5098-5102`).
- `ContextAccess::getRowPolicyFilter()` returns the enabled policy filter and can deny access when
  table policies exist but none match the current user (`src/Access/ContextAccess.cpp:531-574`).
- Quotas are access entities with interval limits and role/user targets; quota usage can throw
  `QUOTA_EXCEEDED` (`src/Access/Quota.h:12-19`, `35-42`; `src/Access/EnabledQuota.cpp:20-66`).

Important caveat: the row-policy enum says **only SELECT is currently supported**; INSERT/UPDATE/DELETE
checks are behind disabled placeholders (`src/Access/Common/RowPolicyDefs.h:26-45`). So ClickHouse is
strong for read-side tenant isolation and rate/resource guardrails, but **Parallax must still enforce
write-side tenant ownership before inserts**.

## Recommended Parallax design

1. **Proxy remains the source of truth for auth.** Every ingest/query request is authorized in Parallax
   before touching either database. This is mandatory for GreptimeDB OSS and still cleaner for
   ClickHouse because product permissions are not only SQL grants.
2. **Always include `tenant_id` / `project_id` in every hot table and every query template.** Treat
   engine row policies as defense-in-depth, not the primary filter.
3. **For GreptimeDB OSS:** prefer per-tenant or per-account schemas only when tenant count and table
   count stay manageable; otherwise shared tables with proxy-enforced predicates plus separate
   read/write DB credentials. Do not hand direct SQL credentials to users.
4. **For ClickHouse:** add row policies and quotas for internal SQL/BI users, e.g. one role per
   tenant/project group, `USING tenant_id IN (...)`, `GRANT SELECT(allowed columns)`, and quota keyed
   by user or client key. Still block writes through the proxy.
5. **Benchmark impact:** row policies add predicates. If Parallax ever exposes internal ClickHouse SQL
   broadly, add a harness case for `tenant_id` row policy on anchored Q1/Q6 and broad log scans to
   confirm pruning still uses the sort/skip indexes. For GreptimeDB, benchmark shared-table
   `tenant_id` predicate vs per-schema/per-table isolation only if SaaS tenant count becomes large.

## Does this change the verdict?

No immediate flip. **ClickHouse wins engine-native SaaS guardrails.** GreptimeDB's OSS limitation is a
real product/ops cost, but Parallax's architecture already requires a proxy that owns ingest,
authorization, routing, and evidence-bundle assembly. The issue is not "GreptimeDB cannot be used for
multi-tenant Parallax"; it is "GreptimeDB cannot be the user-facing authorization boundary in OSS."

Flip trigger: if Parallax needs direct customer SQL/BI access to the telemetry store early, or if proxy
authorization is deliberately minimized, ClickHouse becomes the safer default because its engine can
enforce row/column grants and quotas without waiting for Enterprise GreptimeDB or a custom provider.

## Run 262 re-verify (2026-07-17)

`CREATE QUOTA` still works on CH 26.6; GT still rejects the keyword (Code 1001). No drift from Run 179.

## Run 446 (2026-07-18) — ACCESS surface re-check

`SHOW ACCESS` on CH 26.6.1 still lists: default user, `readonly` profile, default
QUOTA keyed by user_name (TRACKING ONLY), full GRANT surface including ROW POLICY
and QUOTA admin. **No drift** vs Run 172/179: CH OSS has stronger SQL guardrails
than GT; product tenants still proxy-owned for GT.

