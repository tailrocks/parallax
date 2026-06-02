# Multi-Tenancy, Access Control, and Isolation

<!-- markdownlint-disable MD013 -->

Status: Run 172 (2026-06-02). Gap-ledger item #4. Source-read at current pins:
GreptimeDB `v1.0.2` (`0ef54511f710f0ef2c05941c8c600bb4c1fd46c8`) and ClickHouse
`v26.5.1.882-stable` (`5b96a8d8a5e2f4800b43a780911a39dc5a666e1c`). Context7 docs
checked first per repo rule; source is the ground truth.

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
| Quotas / rate guardrails | Request memory limiter exists, but no per-tenant quota entity comparable to ClickHouse. | Quotas are first-class access entities, keyed by user/client key/IP/query hash; overuse throws `QUOTA_EXCEEDED`. |
| Parallax shape | Proxy-enforced auth; optional per-tenant schema/catalog/table for blast-radius reduction. | Proxy-enforced auth plus engine-side fallback policies/quotas for internal/analyst SQL users. |

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
