# Production Database Evidence Access

> Parallax treats production databases as evidence sources, not agent control surfaces: direct database access is optional, read-only, template-driven, scoped, audited, redacted, and excluded from default bundles until a Tier 2 safety gate passes. Three tiers are decided — Tier 0 application telemetry (on by default), Tier 1 metadata/schema snapshots (optional), and Tier 2 read-only query templates (off until the gate passes) — with the first product claim limited to "Parallax can show database-related evidence," never "agents can query production." The companion ledger turns that gate into auditable claim levels (`not_measured` through `agent_visible_db_evidence`, plus `claim_expired`/`claim_failed`), defining run artifacts, row schemas, counting rules, refresh triggers, and allowed product wording. Generic SQL tools such as `run_sql` are rejected in the Parallax context server, and database evidence must never be exposed through MCP resources/resource templates — only schema-bound tool `structuredContent`. The open gates remain: until Tier 2 template fixtures prove least privilege, RLS/view scoping, read-only runtime, parser/allowlist, limits, redaction, source-field policy, projection raw-ref denial, audit, MCP resource denial, prompt-injection resistance, and failure wording, the current status is `not_measured` and direct database evidence stays marked as missing.

This note consolidates the following previously-separate research files, each preserved in full below:

- `production-database-evidence-access.md`
- `production-database-evidence-ledger.md`

## Production Database Evidence Access Gate

_Provenance: merged verbatim from `production-database-evidence-access.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

The prompt says agents need runtime state, including database context, but also
calls database access one of the largest safety risks. This note operationalizes
that boundary:

> Parallax should treat production databases as evidence sources, not agent
> control surfaces. Direct database access is optional, read-only, template
> driven, scoped, audited, redacted, and excluded from default bundles until the
> gate below passes.

The first product claim should be "Parallax can show database-related evidence"
from traces, errors, logs, migrations, deploys, and safe query templates. It
should not be "agents can query production."
Results and product-claim status should be published through the
[Production database evidence ledger](production-database-evidence-ledger.md),
not inferred from this gate alone.

### Current Primary Sources

| Source | What matters for Parallax |
| --- | --- |
| [PostgreSQL 18 privileges](https://www.postgresql.org/docs/current/ddl-priv.html) | PostgreSQL separates object ownership and object privileges; `SELECT` can be granted at table or column level, while `UPDATE`, `DELETE`, `TRUNCATE`, `CREATE`, and ownership carry different blast radius. Parallax must not connect as an owner or superuser. |
| [PostgreSQL 18 row security](https://www.postgresql.org/docs/current/ddl-rowsecurity.html) | Row-Level Security can restrict which rows are visible or mutable; when enabled without policies it defaults to no visible/modifiable rows, but owners and `BYPASSRLS` roles bypass it. RLS policies can also create race/covert-channel concerns when they depend on other tables. |
| [PostgreSQL 18 read-only transactions](https://www.postgresql.org/docs/current/sql-set-transaction.html) | `READ ONLY` transactions disallow ordinary DML and DDL, but PostgreSQL explicitly treats it as a high-level notion, not a complete "no writes to disk" guarantee. It is a guardrail, not the whole safety model. |
| [PostgreSQL 18 SELECT](https://www.postgresql.org/docs/current/sql-select.html) | A `SELECT` needs column privileges, and locking forms such as `FOR UPDATE` require extra privileges. Parallax should avoid lock-taking forms and require bounded read-only statements. |
| [OpenTelemetry database semantic conventions](https://opentelemetry.io/docs/specs/semconv/db/database-spans/) | OTel database spans are stable unless otherwise specified. `db.query.summary` is intended to stay low-cardinality and avoid dynamic/sensitive data, non-parameterized `db.query.text` should be sanitized before default collection, and `db.query.parameter.<key>` is opt-in/development-stage. Tier 0 evidence should therefore prefer summaries, operation names, status codes, returned rows, and errors; query text and parameters need explicit policy before agent projection. |
| [OWASP SQL Injection Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html) | OWASP favors parameterized queries, safely implemented stored procedures, allow-list validation for identifiers, and least privilege. Parallax query templates should follow this rather than exposing free-form SQL. |
| [OWASP Top 10 for LLM Applications](https://owasp.org/www-project-top-10-for-large-language-model-applications/) | Prompt injection, sensitive information disclosure, insecure plugin design, and excessive agency directly apply when an agent can request database evidence. |
| [NSA MCP security design considerations](https://www.nsa.gov/Portals/75/documents/Cybersecurity/CSI_MCP_SECURITY.pdf?ver=bmgiSbNQLP6Z_GiWtRt6bg%3D%3D) | NSA's May 2026 guidance warns that MCP-style agent tooling depends on implementation discipline around dynamic tool invocation, implicit trust, token/session handling, and context sharing. Database evidence tools need stronger constraints than generic observability queries. |
| [MCP resources](https://modelcontextprotocol.io/specification/2025-11-25/server/resources) and [MCP tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) `2025-11-25` | Resources are application-controlled context and resource templates can expose parameterized URI spaces. Tools can return `structuredContent` validated by `outputSchema`, plus optional resource links or embedded resources. Database evidence should use schema-bound tool output, not MCP resource reads that can be auto-attached or retained by clients. |

### Decision

Database evidence has three tiers:

| Tier | Source | Default | Purpose |
| --- | --- | --- | --- |
| 0 | Application telemetry | On | DB spans, query summaries, errors, timings, row counts, migration/deploy refs. |
| 1 | Metadata/schema snapshots | Optional | Table/index/schema/version context without row values. |
| 2 | Read-only query templates | Off until gate passes | Bounded diagnostic data for one incident, never raw table exploration. |

Tier 0 is enough for the tiny tier. Tier 1 can be enabled for richer diagnosis.
Tier 2 is powerful and dangerous; it requires project-level opt-in and the gate
below.

### What Counts As Database Evidence

Safe first-class evidence:

- DB span summaries from OTel: system, operation, collection/table, latency,
  error type, returned rows, and low-cardinality query summary.
- Migration/deploy refs: migration id, schema version, release, commit, actor,
  and result.
- Query error details: SQLSTATE/error code, constraint name, deadlock/timeout
  class, lock wait summary, and sanitized stack edge.
- Aggregate diagnostics: count, min/max/avg, existence checks, explain plan
  shape without sensitive literals, and bounded sample ids when policy allows.
- Schema metadata: table names, column names, index names, foreign keys, and
  policy names when approved by the project.

Unsafe by default:

- `SELECT *` from application tables;
- arbitrary free-form SQL from an agent;
- raw customer rows;
- prompt-injected query text from logs/issues;
- unsanitized `db.query.text` in agent-visible bundles;
- query parameter values such as email, token, account id, session id, address,
  payment, or message body;
- `db.query.parameter.<key>` values in agent-visible bundles by default;
- raw explain plans or plan text with literals;
- database exports, backups, and dumps;
- lock-taking reads such as `FOR UPDATE`;
- write, DDL, migration, deploy, rollback, vacuum, copy, or extension commands.

### Access Model

Direct database evidence access must satisfy all of these:

| Control | Requirement |
| --- | --- |
| Separate role | Use a Parallax evidence role, never app owner, migration owner, admin, or superuser. |
| Read-only grants | Grant only needed `SELECT` privileges, preferably on views or specific columns. |
| RLS/views | Prefer views plus Row-Level Security for tenant/project scoping; verify the role cannot bypass RLS. |
| Read-only transaction | Wrap each query in an explicit read-only transaction with timeout and cancellation. |
| Template allowlist | Only pre-registered templates with typed parameters can run. No agent-written SQL. |
| Identifier allowlist | Table, column, index, and sort identifiers come from template metadata, not user strings. |
| Limits | Every template has row, byte, time, and cardinality limits. |
| Redaction | Query output passes the normal Parallax redaction pipeline and includes a redaction report. |
| Source-field policy | Synthetic, benchmark, evaluation, or corpus fixtures prove provenance/source status before they influence DB evidence claims. |
| Projection safety | Agent-visible JSON/Markdown carries policy status and missing-evidence flags, not dereferenced raw rows, query parameters, raw query text, or plan text. |
| MCP resource boundary | Do not expose row values, query text, parameters, plan text, or raw refs through MCP resources, resource templates, resource links, or embedded resources by default. |
| Audit | Every request records actor, tool, template id, parameters hash, result shape, redaction policy, bundle id, and denied/allowed decision. |
| No mutation | DML, DDL, locking reads, copy/export, functions with side effects, and maintenance commands are rejected before reaching the database. |

### Template Manifest

Each allowed query should be declared, reviewed, and versioned:

```yaml
id: db.error_context.v1
description: "Bounded DB evidence for one application error"
owner: "parallax"
scope:
  project: required
  environment: required
  service: required
required_permissions:
  - db:evidence:read
database:
  system: postgresql
  role: parallax_evidence
statement_class: select_only
sql: |
  SELECT query_hash, error_code, count(*) AS occurrences, max(ts) AS last_seen
  FROM app_db_errors
  WHERE service = $1 AND ts >= $2 AND ts < $3
  GROUP BY query_hash, error_code
  ORDER BY occurrences DESC
  LIMIT 20
parameters:
  - name: service
    type: service_id
  - name: start
    type: timestamp
  - name: end
    type: timestamp
limits:
  max_rows: 20
  timeout_ms: 1000
  max_bytes: 32768
telemetry:
  otel_db_semconv: "1.41.0"
  query_text_policy: static_template_or_sanitized_only
  parameter_capture_policy: hash_only
redaction:
  output_policy: db-evidence-v1
  source_field_policy: phase0-source-field-policy-v1
  raw_values: deny
bundle_projection:
  include_rows: true
  include_raw_sql: false
  include_query_parameters: false
  raw_ref_policy: deny_dereference_by_default
```

The template id and version must appear in the evidence bundle so a reviewer can
replay or audit the exact diagnostic path.

### Agent Surface

The default agent-visible surface should be:

| Tool/command | Allowed? | Notes |
| --- | --- | --- |
| `parallax db evidence <issue_id>` | Yes, after Tier 2 gate | Runs approved templates selected by the bundle builder. |
| `parallax db template run <template_id>` | Yes, scoped | Requires typed parameters and project/environment scope. |
| `parallax_db_evidence` MCP tool | Yes, read-only | Returns aggregate/sanitized `structuredContent` validated against `outputSchema`, plus refs only. |
| `db://...` MCP resources or resource templates | No by default | Metadata-only indexes may be considered later, but row/query/parameter/plan content must not be exposed as resources. |
| `parallax_db_raw_ref_read` | Rare | Requires read-sensitive scope and human approval. |
| `run_sql` | No | Rejected in the Parallax context server. |
| `db_mutate`, `migration_run`, `rollback`, `grant`, `copy`, `dump` | No | Out of scope for the evidence engine. |

The model can ask for a diagnostic template by purpose, but a deterministic
policy layer decides which template, if any, is allowed.

### Evidence Bundle Fields

Add a database evidence node only after the output is redacted:

```json
{
  "type": "db_evidence",
  "id": "db_ev_123",
  "db_system": "postgresql",
  "template_id": "db.error_context.v1",
  "role": "parallax_evidence",
  "environment": "prod",
  "query_summary": "SELECT app_db_errors",
  "parameters_hash": "sha256:...",
  "row_count": 8,
  "byte_count": 4096,
  "duration_ms": 73,
  "query_text_policy": "static_template_only",
  "parameter_capture_policy": "hash_only",
  "redaction_report_ref": "redact_456",
  "source_field_policy_ref": "source_policy_789",
  "raw_ref_policy": "deny_dereference_by_default",
  "mcp_resource_policy": "no_db_rows_query_text_parameters_or_plans_as_resources",
  "query_text_visible": false,
  "query_parameter_values_visible": false,
  "mcp_resource_visible": false,
  "raw_ref": null
}
```

Edges should be specific:

| Edge | Meaning |
| --- | --- |
| `span_executed_db_query` | Runtime span observed a DB operation. |
| `db_error_matches_issue` | Query error class/constraint matches the issue. |
| `migration_preceded_issue` | Migration/deploy happened before regression window. |
| `db_evidence_supports_hypothesis` | Template output supports a hypothesis. |
| `db_evidence_contradicts_hypothesis` | Template output contradicts a hypothesis. |
| `db_evidence_missing` | Needed DB evidence was denied, absent, or unsafe. |

### Gate Before Tier 2

Direct database evidence access passes only if all checks pass:

| Gate | Pass condition |
| --- | --- |
| Privilege proof | The connector role cannot create, alter, drop, insert, update, delete, truncate, copy, lock rows for update, grant, revoke, or bypass RLS. |
| Template parser | Only registered templates run; free-form SQL, stacked statements, comments that alter semantics, and dynamic identifiers are rejected. |
| Read-only runtime | Every query runs in a read-only transaction with statement timeout and cancellation. |
| Limit proof | Row, byte, time, and cardinality limits are enforced even when the query would return more. |
| Telemetry DB span policy | Tier 0 spans use low-cardinality summaries; query text is absent, static-template-only, or sanitized; parameter values are absent from agent-visible output by default. |
| Redaction proof | Seeded secrets and PII in table rows, parameters, query text, errors, and plan output do not appear in agent-visible JSON or Markdown. |
| Source-field proof | Synthetic, benchmark, evaluation, and corpus fixture rows pass source-field policy before projection claims pass; direct telemetry/template rows record an explicit not-applicable reason when no mixed source is present. |
| Projection proof | CLI, HTTP, and MCP projections carry redaction and source-field policy status and do not dereference raw rows, query text, query parameters, plan text, transcripts, or incident-note refs by default. |
| MCP resource proof | MCP `resources/list`, `resources/read`, resource templates, tool resource links, and embedded resources cannot expose raw rows, query text, parameters, plan text, or DB raw refs in default agent-visible paths. |
| Audit proof | Allowed and denied attempts emit audit rows linked to actor, investigation, template id, bundle id, and policy version. |
| RLS proof | Tenant/project scoping works for positive and negative fixtures; owner/superuser/BYPASSRLS roles are rejected. |
| Prompt-injection proof | Malicious issue text, log lines, table names, and row values cannot change template choice, parameters, scope, or output policy. |
| Failure wording | If data is denied or redacted away, the bundle says `db_evidence_missing` instead of inventing a cause. |

Initial acceptance criterion: zero seeded write attempts succeed and zero seeded
secrets appear in agent-visible output.

### Failure Consequences

If the gate fails:

- keep Tier 2 disabled;
- rely on OTel DB spans, errors, migrations, and deploy refs;
- keep query text, query parameters, raw rows, and plan text out of
  agent-visible bundles unless a narrower policy explicitly passes;
- keep database evidence out of MCP resources/resource templates and deliver
  only schema-bound `structuredContent` for allowed tool calls;
- expose database evidence as "missing" in bundles;
- keep direct DB queries human-only outside Parallax;
- do not market database-backed autonomous debugging.

If useful DB context requires raw rows or broad free-form queries, the Parallax
agent-access claim narrows: database evidence remains a human-controlled
investigation extension, not an agent default.

### Relationship To Other Research

- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
  defines the autonomy ladder and keeps database mutation out of scope.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  owns the output policy for query results and raw refs.
- [Production database evidence ledger](production-database-evidence-ledger.md)
  defines the result artifacts, claim levels, expiry triggers, and product
  wording that make this gate measurable.
- [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md)
  rejects generic SQL tools in the context server.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) should carry
  database evidence nodes, edges, redaction refs, and source-field policy refs
  only after this gate passes.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  should treat direct database evidence as a safety-gated extension, not a tiny
  tier requirement.

### Bottom Line

Database context is valuable, but it is not worth turning Parallax into an
agent-controlled SQL console. The safe path is telemetry-derived DB evidence
with explicit query-text/parameter policy first, schema/metadata second, and
tightly scoped read-only query templates only after privilege, redaction,
source-field, projection, audit, and prompt-injection gates pass.

## Production Database Evidence Ledger

_Provenance: merged verbatim from `production-database-evidence-ledger.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

This ledger turns the
[Production database evidence access gate](production-database-evidence-access.md)
into auditable claim levels. The gate defines what safe direct database evidence
access must look like; this ledger defines the run artifacts, row schemas,
counting rules, expiry triggers, and product wording needed before Parallax can
claim that production database evidence is safe for agent-visible bundles.

Current status: `not_measured`.

Consumes:

- [Production database evidence access gate](production-database-evidence-access.md)
  for the access model and Tier 2 safety gate.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  and [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) for output
  safety.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) for bundle
  nodes, edges, missing-evidence flags, raw refs, and redaction reports.
- [Agent access surface safety ledger](agent-access-surface-safety-ledger.md)
  for CLI/HTTP/MCP projection equivalence and negative-tool fixtures.

Central rule:

> No agent-visible direct database evidence claim until Tier 2 template fixtures
> prove least privilege, RLS/view scoping, read-only runtime, parser/allowlist,
> limits, redaction, source-field policy, projection raw-ref denial, audit,
> MCP resource denial, prompt-injection resistance, and failure wording.

### Current Source Snapshot

| Source | Ledger consequence |
| --- | --- |
| [PostgreSQL 18 current documentation](https://www.postgresql.org/docs/current/) | The current docs page is PostgreSQL 18 and the site notes PostgreSQL 18.4 was released on 2026-05-14. Run manifests must record exact server version, not only the major docs version. |
| [PostgreSQL 18 privileges](https://www.postgresql.org/docs/current/ddl-priv.html) | The connector role proof must show no owner, superuser, grant-option, DML, DDL, `MAINTAIN`, or broad schema privileges. `SELECT` alone is not enough because it can still expose sensitive rows and supports export-like paths such as `COPY TO`. |
| [PostgreSQL 18 row security](https://www.postgresql.org/docs/current/ddl-rowsecurity.html) | RLS proof must verify positive and negative tenant/project fixtures and prove the evidence role is not a table owner, superuser, or `BYPASSRLS` role. |
| [PostgreSQL 18 read-only transactions](https://www.postgresql.org/docs/current/sql-set-transaction.html) | Read-only transactions are required as a runtime guardrail, but they do not replace privilege proof, template parsing, RLS, output limits, or redaction. |
| [PostgreSQL 18 SELECT](https://www.postgresql.org/docs/current/sql-select.html) | The parser must reject lock-taking reads such as `FOR UPDATE`/`FOR SHARE`, dynamic identifiers, `SELECT *`, stacked statements, and unbounded scans. |
| [OpenTelemetry database semantic conventions](https://opentelemetry.io/docs/specs/semconv/db/database-spans/) | DB spans are stable unless otherwise specified; `db.query.summary` should be low-cardinality and not dynamic/sensitive, non-parameterized `db.query.text` should not be collected by default unless sanitized, and `db.query.parameter.<key>` is opt-in/development-stage. Tier 0 database evidence should prefer summaries, operation names, status codes, row counts, and errors; query text and parameters need explicit policy rows before agent projection. |
| [OWASP SQL Injection Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html) | Templates must use typed parameters, allow-listed identifiers, and least privilege; free-form SQL cannot count as safe database evidence. |
| [OWASP Top 10 for LLM Applications](https://owasp.org/www-project-top-10-for-large-language-model-applications/) | Prompt injection, sensitive information disclosure, insecure tool design, and excessive agency are direct fixture categories for database evidence. |
| [NSA MCP security design considerations](https://www.nsa.gov/Portals/75/documents/Cybersecurity/CSI_MCP_SECURITY.pdf?ver=bmgiSbNQLP6Z_GiWtRt6bg%3D%3D) | Dynamic tool invocation, implicit trust, token/session handling, context sharing, and tool-description risk make direct DB templates stricter than ordinary evidence-bundle projection. |
| [MCP resources](https://modelcontextprotocol.io/specification/2025-11-25/server/resources) and [MCP tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) `2025-11-25` | Resources and resource templates are application-controlled context surfaces, while tools can return schema-bound `structuredContent`, optional resource links, and embedded resources. DB evidence must prove the resource path cannot bypass the stricter tool-output projection path. |
| [Codex MCP docs](https://developers.openai.com/codex/mcp) and [Claude Code MCP docs](https://code.claude.com/docs/en/mcp) | Current clients expose MCP configuration, auth/header sources, output limits, prompts/resources, elicitation, and server/client modes differently. A safe DB-evidence result cannot assume all clients treat resources, links, tool output, or retention the same way. |

### Claim Levels

| Level | Meaning | Minimum evidence |
| --- | --- | --- |
| `not_measured` | No current database-evidence safety run exists. | Default state. |
| `telemetry_db_evidence` | Parallax can show database-related evidence from telemetry only. | OTel DB span fixture passes bundle projection and redaction. |
| `schema_snapshot_safe` | Parallax can include approved schema/metadata snapshots. | Metadata snapshot fixture proves no row values, secrets, or tenant data. |
| `template_parser_safe` | Registered templates are parsed and constrained before execution. | Parser rejects free-form SQL, stacked statements, comments that alter semantics, `SELECT *`, dynamic identifiers, writes, DDL, `COPY`, and lock-taking reads. |
| `least_privilege_proven` | The database role cannot mutate or administer the database. | Privilege proof shows no owner/superuser path, no `BYPASSRLS`, no DML/DDL/maintenance/export grants, and no grant option. |
| `rls_view_scoped` | Tenant/project scoping works for direct evidence. | Positive fixtures return allowed rows; negative fixtures return no rows and no side-channel summaries. |
| `redacted_db_output` | Direct query results can pass the redaction gate. | Seeded PII/secrets in rows, parameters, errors, and plan text are absent from JSON and Markdown output. |
| `audited_db_evidence` | Allowed and denied attempts are traceable. | Audit rows link actor, principal, investigation, template id, role, parameter hash, policy versions, bundle id, and allow/deny decision. |
| `tier2_template_safe` | Direct read-only templates pass the full Tier 2 gate for one tested policy set. | Parser, privilege, runtime, limits, RLS/view, redaction, prompt-injection, audit, and failure-wording fixtures all pass. |
| `mcp_resource_denied` | MCP resources/resource templates/resource links cannot bypass the database projection policy. | Resource-list/read/template/link/embedded-resource fixtures expose no raw rows, query text, parameters, plan text, or DB raw refs. |
| `projection_safe` | Agent-visible JSON, Markdown, CLI, HTTP, and MCP tool output carry redaction reports, source-field policy status, missing-evidence flags, and no raw row/query/parameter/resource dereference. | Bundle projection rows pass for the tested templates and surfaces. |
| `agent_visible_db_evidence` | Agent-visible bundles may include direct DB evidence for the tested templates. | `tier2_template_safe` plus CLI/HTTP/MCP projection-equivalence, MCP resource denial, source-field, and bundle-redaction/projection checks pass. |
| `claim_expired` | A prior claim is stale. | Refresh trigger fired or max age elapsed. |
| `claim_failed` | A required fixture failed. | Any write/DDL/lock/export succeeds, any secret leaks, any scope bypass succeeds, or unsafe wording reaches a bundle. |

Initial claim level: `not_measured`.

### Result Artifacts

The durable result index lives at:

```text
docs/research/production-database-evidence-results.md
```

Each run stores immutable artifacts under:

```text
docs/research/production-database-evidence-runs/<run_id>/manifest.json
docs/research/production-database-evidence-runs/<run_id>/template-manifests/*.yaml
docs/research/production-database-evidence-runs/<run_id>/privilege-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/telemetry-db-span-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/rls-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/parser-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/runtime-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/limit-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/redaction-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/source-field-policy-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/prompt-injection-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/audit-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/mcp-resource-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/raw-ref-manifest.jsonl
docs/research/production-database-evidence-runs/<run_id>/bundle-projection-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/claim-ledger.jsonl
docs/research/production-database-evidence-runs/<run_id>/hashes.sha256
```

### Run Manifest

`manifest.json` must include enough version data to make a future rerun
comparable:

```json
{
  "run_id": "db-evidence-2026-05-25T120000Z",
  "run_started_at": "2026-05-25T12:00:00Z",
  "run_finished_at": "2026-05-25T12:03:41Z",
  "runner": "parallax-db-evidence-suite",
  "suite_version": "0.1.0",
  "git_commit": "<parallax_commit_sha>",
  "database": {
    "system": "postgresql",
    "version": "18.x",
    "extensions": [],
    "evidence_role": "parallax_evidence"
  },
  "policies": {
    "auth_policy_version": "auth-vN",
    "redaction_policy_version": "a6-default-deny-vN",
    "source_field_policy_version": "phase0-source-field-policy-vN",
    "template_policy_version": "db-template-vN",
    "mcp_resource_policy_version": "db-mcp-resource-deny-vN",
    "audit_schema_version": "audit-vN",
    "bundle_schema_version": "0.1.0",
    "projection_schema_version": "db-evidence-projection-vN",
    "otel_db_semconv_version": "1.41.0",
    "raw_ref_policy": "raw_rows_query_parameters_plans_and_db_resources_not_agent_visible_by_default"
  },
  "template_manifest_hashes": ["sha256:<hex>"],
  "fixture_hashes": {
    "privilege": "sha256:<hex>",
    "rls": "sha256:<hex>",
    "redaction": "sha256:<hex>",
    "prompt_injection": "sha256:<hex>"
  },
  "result": "pass"
}
```

### Minimum Row Schemas

Template manifest row:

```json
{
  "template_id": "db.error_context.v1",
  "template_hash": "sha256:<hex>",
  "statement_class": "select_only",
  "allowed_identifiers_hash": "sha256:<hex>",
  "required_role": "parallax_evidence",
  "raw_values_allowed": false,
  "query_text_policy": "static_template_only|sanitized_summary_only",
  "parameter_capture_policy": "hash_only|deny_values",
  "projection_policy": "aggregate_rows_only",
  "max_rows": 20,
  "max_bytes": 32768,
  "timeout_ms": 1000
}
```

Telemetry DB span result row:

```json
{
  "check": "telemetry_db_span_policy",
  "span_id": "span_123",
  "db_system_name": "postgresql",
  "db_query_summary_present": true,
  "db_query_text_present": false,
  "db_query_text_sanitized": true,
  "db_query_parameter_values_present": false,
  "db_response_status_code_present": true,
  "db_response_returned_rows_present": true,
  "redaction_report_hash": "sha256:<hex>",
  "result": "pass"
}
```

Privilege proof row:

```json
{
  "check": "privilege_no_mutation",
  "role": "parallax_evidence",
  "object": "app_db_errors",
  "owner": false,
  "superuser": false,
  "bypass_rls": false,
  "attempted": ["insert", "update", "delete", "truncate", "alter", "copy_to", "lock_table"],
  "unexpected_successes": [],
  "result": "pass"
}
```

RLS/view scope row:

```json
{
  "check": "tenant_negative_scope",
  "template_id": "db.error_context.v1",
  "fixture_tenant": "tenant_a",
  "requested_tenant": "tenant_b",
  "row_count": 0,
  "side_channel_fields_present": false,
  "result": "pass"
}
```

Parser result row:

```json
{
  "check": "parser_rejects_unsafe_sql",
  "case_id": "stacked_statement",
  "input_hash": "sha256:<hex>",
  "expected": "deny",
  "actual": "deny",
  "reason": "stacked_statement",
  "result": "pass"
}
```

Runtime/limit row:

```json
{
  "check": "runtime_limits",
  "template_id": "db.error_context.v1",
  "read_only_transaction": true,
  "statement_timeout_ms": 1000,
  "cancelled_on_timeout": true,
  "row_limit_enforced": true,
  "byte_limit_enforced": true,
  "result": "pass"
}
```

Redaction result row:

```json
{
  "check": "db_output_redaction",
  "surface": "markdown_projection",
  "seeded_secret_count": 12,
  "visible_secret_count": 0,
  "visible_pii_count": 0,
  "redaction_report_hash": "sha256:<hex>",
  "result": "pass"
}
```

Source-field policy result row:

```json
{
  "check": "db_source_field_policy",
  "source_kind": "direct_database_template|telemetry_span|synthetic_fixture|benchmark_fixture|corpus_fixture",
  "source_field_policy_status": "pass|fail|not_applicable",
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "source_field_policy_hash": "sha256:<hex>",
  "denied_zone_count": 0,
  "violation_count": 0,
  "not_applicable_reason": "direct telemetry/template source without mixed eval/corpus source rows",
  "result": "pass"
}
```

Prompt-injection result row:

```json
{
  "check": "prompt_injection_resistance",
  "case_id": "row_value_requests_free_form_sql",
  "injection_surface": "row_value",
  "template_changed": false,
  "scope_changed": false,
  "output_policy_changed": false,
  "result": "pass"
}
```

Audit result row:

```json
{
  "check": "audit_allowed_and_denied",
  "request_id": "dbreq_123",
  "actor_id_hash": "sha256:<hex>",
  "template_id": "db.error_context.v1",
  "decision": "deny",
  "audit_row_present": true,
  "bundle_id": "bndl_123",
  "result": "pass"
}
```

Bundle projection row:

```json
{
  "check": "bundle_projection",
  "template_id": "db.error_context.v1",
  "json_hash": "sha256:<hex>",
  "markdown_hash": "sha256:<hex>",
  "redaction_report_present": true,
  "source_field_policy_status": "pass|fail|not_applicable",
  "missing_evidence_on_denial": true,
  "unsafe_raw_ref_exposed": false,
  "raw_ref_dereferenced": false,
  "raw_row_values_visible": false,
  "query_parameter_values_visible": false,
  "plan_text_visible": false,
  "mcp_structured_content_valid": true,
  "mcp_resource_link_count": 0,
  "mcp_embedded_resource_count": 0,
  "result": "pass"
}
```

MCP resource result row:

```json
{
  "check": "mcp_resource_denial",
  "surface": "resources_list|resources_read|resource_template|tool_resource_link|embedded_resource",
  "client": "codex|claude_code|generic_mcp",
  "template_id": "db.error_context.v1",
  "resource_uri": "db://prod/app_db_errors/tenant_a",
  "listed": false,
  "read_allowed": false,
  "raw_row_values_visible": false,
  "query_text_visible": false,
  "query_parameter_values_visible": false,
  "plan_text_visible": false,
  "raw_ref_visible": false,
  "structured_content_equivalent_hash": "sha256:<hex>|null",
  "result": "pass"
}
```

Claim ledger row:

```json
{
  "claim_level": "tier2_template_safe",
  "run_id": "db-evidence-2026-05-25T120000Z",
  "policy_set": "postgresql-18-default-deny-vN",
  "templates": ["db.error_context.v1"],
  "granted_at": "2026-05-25T12:05:00Z",
  "expires_at": "2026-08-23T12:05:00Z",
  "result": "pass"
}
```

### Counting Rules

- No Tier 2 claim if any write, DDL, maintenance, lock, `COPY`, export, grant,
  revoke, migration, or rollback attempt succeeds.
- No direct database claim from read-only transactions alone.
- No free-form SQL, model-written SQL, stacked statements, dynamic identifiers,
  or `SELECT *`.
- No RLS/view claim if the tested role owns target tables, is superuser, has
  `BYPASSRLS`, or can see negative-tenant/project rows.
- No raw row exposure by default. Query output must be aggregated, bounded,
  redacted, and represented with a redaction report.
- No `db.query.parameter.<key>` values or unsanitized `db.query.text` in
  agent-visible bundles by default. Tier 0 DB telemetry claims must record
  whether query text is absent, static, sanitized, or denied.
- Synthetic, benchmark, or corpus-derived DB evidence runs require
  `source_field_policy_status: pass` before projection claims can pass. Direct
  telemetry/template sources may use `not_applicable` only when no mixed
  eval/corpus source rows are present.
- No agent-visible DB evidence unless missing, denied, or unsafe data is
  explicitly represented as `db_evidence_missing`.
- Agent-visible projections must not dereference raw row, raw query, raw
  parameter, plan-text, transcript, or incident-note refs by default.
- Direct database evidence must be delivered through canonical bundle fields or
  MCP tool `structuredContent` that validates against an `outputSchema`.
- No DB row values, query text, parameter values, plan text, or raw DB refs may
  appear in MCP `resources/list`, `resources/read`, resource templates, tool
  resource links, or embedded resources by default.
- Client-specific MCP rows must record Codex/Claude/generic client settings
  that affect resource reads, auth/header sources, output limits, elicitation,
  and retention before cross-client DB-evidence wording can pass.
- DB evidence can support or contradict hypotheses; it does not become a root
  cause by itself.
- A pass is scoped to the database system/version, role, templates, policies,
  bundle schema, and agent surface listed in the manifest.

### Refresh Triggers

Mark the claim `claim_expired` and rerun when any of these changes:

- PostgreSQL version, privilege behavior, RLS policy behavior, transaction
  behavior, extension set, or supported database engine.
- Template parser, template manifests, identifier allowlists, role/grants,
  views/RLS policies, redaction policy, source-field policy, auth policy, audit
  schema, bundle schema, projection schema, or agent surface.
- OpenTelemetry DB semantic conventions, database query-text/parameter capture
  guidance, or instrumentation stability mode changes.
- OWASP LLM, OWASP SQL injection, MCP, or NSA-style agent-tooling guidance that
  materially changes the threat model.
- Any seeded secret/PII canary, prompt-injection case, or negative-scope case is
  added.
- Ninety days pass for `tier2_template_safe`; sixty days pass for
  `agent_visible_db_evidence`.

### Product Wording

Allowed after `telemetry_db_evidence`:

> Parallax includes database-related telemetry such as DB span summaries,
> errors, timings, and migration/deploy context.

Allowed after `tier2_template_safe`:

> Parallax can run approved, bounded, read-only database evidence templates for
> the tested PostgreSQL policy set.

Allowed after `agent_visible_db_evidence`:

> Parallax can include redacted, audited, template-derived production database
> evidence in agent-visible bundles for the tested templates and policies.

Avoid:

- "Agents can query production."
- "Safe production SQL."
- "Database evidence as MCP resources."
- "Read-only means safe."
- "Autonomous database debugging."
- "Free-form SQL for agents."
- "Direct root-cause analysis from database rows."
- "Production database access is enabled by default."
- "Safe DB query text/parameters" without query-text, parameter, redaction, and
  projection rows.
- "Safe DB MCP output" without `structuredContent` schema validation and MCP
  resource-denial rows.

### Relationship To Other Research

- [Production database evidence access gate](production-database-evidence-access.md)
  defines the safety requirements that this ledger measures.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  and [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) can veto
  every database-evidence claim.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) carries
  database evidence as bounded nodes, edges, missing-evidence flags, and
  redaction/source-field refs only after this ledger permits the claim.
- [Agent access surface safety ledger](agent-access-surface-safety-ledger.md)
  ensures CLI, HTTP, and MCP expose the same redacted database evidence.
- [Deploy/change context ledger](deploy-change-context-ledger.md) supplies the
  release, deploy, commit, and work-item context that database evidence can
  support or contradict.
- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
  owns the autonomy boundary: database evidence informs hypotheses, while
  production mutation remains out of scope.

### Bottom Line

Production database evidence is claimable only as a measured, scoped capability.
Until this ledger is green, Parallax can use telemetry-derived database context
with strict query-text/parameter policy and mark direct database evidence as
missing; it cannot claim that agents can see production database evidence
safely.
