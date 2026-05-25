# Production Database Evidence Access Gate

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

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

## Current Primary Sources

| Source | What matters for Parallax |
| --- | --- |
| [PostgreSQL 18 privileges](https://www.postgresql.org/docs/current/ddl-priv.html) | PostgreSQL separates object ownership and object privileges; `SELECT` can be granted at table or column level, while `UPDATE`, `DELETE`, `TRUNCATE`, `CREATE`, and ownership carry different blast radius. Parallax must not connect as an owner or superuser. |
| [PostgreSQL 18 row security](https://www.postgresql.org/docs/current/ddl-rowsecurity.html) | Row-Level Security can restrict which rows are visible or mutable; when enabled without policies it defaults to no visible/modifiable rows, but owners and `BYPASSRLS` roles bypass it. RLS policies can also create race/covert-channel concerns when they depend on other tables. |
| [PostgreSQL 18 read-only transactions](https://www.postgresql.org/docs/current/sql-set-transaction.html) | `READ ONLY` transactions disallow ordinary DML and DDL, but PostgreSQL explicitly treats it as a high-level notion, not a complete "no writes to disk" guarantee. It is a guardrail, not the whole safety model. |
| [PostgreSQL 18 SELECT](https://www.postgresql.org/docs/current/sql-select.html) | A `SELECT` needs column privileges, and locking forms such as `FOR UPDATE` require extra privileges. Parallax should avoid lock-taking forms and require bounded read-only statements. |
| [OpenTelemetry database semantic conventions](https://opentelemetry.io/docs/specs/semconv/db/database-spans/) | OTel captures database operation metadata, query summaries, returned rows, DB system names, and optional query text/parameters. This is the first database evidence path before any direct production connector. |
| [OWASP SQL Injection Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html) | OWASP favors parameterized queries, safely implemented stored procedures, allow-list validation for identifiers, and least privilege. Parallax query templates should follow this rather than exposing free-form SQL. |
| [OWASP Top 10 for LLM Applications](https://owasp.org/www-project-top-10-for-large-language-model-applications/) | Prompt injection, sensitive information disclosure, insecure plugin design, and excessive agency directly apply when an agent can request database evidence. |
| [NSA MCP security design considerations](https://www.nsa.gov/Portals/75/documents/Cybersecurity/CSI_MCP_SECURITY.pdf?ver=bmgiSbNQLP6Z_GiWtRt6bg%3D%3D) | NSA's May 2026 guidance warns that MCP-style agent tooling depends on implementation discipline around dynamic tool invocation, implicit trust, token/session handling, and context sharing. Database evidence tools need stronger constraints than generic observability queries. |

## Decision

Database evidence has three tiers:

| Tier | Source | Default | Purpose |
| --- | --- | --- | --- |
| 0 | Application telemetry | On | DB spans, query summaries, errors, timings, row counts, migration/deploy refs. |
| 1 | Metadata/schema snapshots | Optional | Table/index/schema/version context without row values. |
| 2 | Read-only query templates | Off until gate passes | Bounded diagnostic data for one incident, never raw table exploration. |

Tier 0 is enough for the tiny tier. Tier 1 can be enabled for richer diagnosis.
Tier 2 is powerful and dangerous; it requires project-level opt-in and the gate
below.

## What Counts As Database Evidence

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
- query parameter values such as email, token, account id, session id, address,
  payment, or message body;
- database exports, backups, and dumps;
- lock-taking reads such as `FOR UPDATE`;
- write, DDL, migration, deploy, rollback, vacuum, copy, or extension commands.

## Access Model

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
| Audit | Every request records actor, tool, template id, parameters hash, result shape, redaction policy, bundle id, and denied/allowed decision. |
| No mutation | DML, DDL, locking reads, copy/export, functions with side effects, and maintenance commands are rejected before reaching the database. |

## Template Manifest

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
redaction:
  output_policy: db-evidence-v1
  raw_values: deny
bundle_projection:
  include_rows: true
  include_raw_sql: false
```

The template id and version must appear in the evidence bundle so a reviewer can
replay or audit the exact diagnostic path.

## Agent Surface

The default agent-visible surface should be:

| Tool/command | Allowed? | Notes |
| --- | --- | --- |
| `parallax db evidence <issue_id>` | Yes, after Tier 2 gate | Runs approved templates selected by the bundle builder. |
| `parallax db template run <template_id>` | Yes, scoped | Requires typed parameters and project/environment scope. |
| `parallax_db_evidence` MCP tool | Yes, read-only | Returns aggregate/sanitized evidence plus refs. |
| `parallax_db_raw_ref_read` | Rare | Requires read-sensitive scope and human approval. |
| `run_sql` | No | Rejected in the Parallax context server. |
| `db_mutate`, `migration_run`, `rollback`, `grant`, `copy`, `dump` | No | Out of scope for the evidence engine. |

The model can ask for a diagnostic template by purpose, but a deterministic
policy layer decides which template, if any, is allowed.

## Evidence Bundle Fields

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
  "redaction_report_ref": "redact_456",
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

## Gate Before Tier 2

Direct database evidence access passes only if all checks pass:

| Gate | Pass condition |
| --- | --- |
| Privilege proof | The connector role cannot create, alter, drop, insert, update, delete, truncate, copy, lock rows for update, grant, revoke, or bypass RLS. |
| Template parser | Only registered templates run; free-form SQL, stacked statements, comments that alter semantics, and dynamic identifiers are rejected. |
| Read-only runtime | Every query runs in a read-only transaction with statement timeout and cancellation. |
| Limit proof | Row, byte, time, and cardinality limits are enforced even when the query would return more. |
| Redaction proof | Seeded secrets and PII in table rows, parameters, query text, errors, and plan output do not appear in agent-visible JSON or Markdown. |
| Audit proof | Allowed and denied attempts emit audit rows linked to actor, investigation, template id, bundle id, and policy version. |
| RLS proof | Tenant/project scoping works for positive and negative fixtures; owner/superuser/BYPASSRLS roles are rejected. |
| Prompt-injection proof | Malicious issue text, log lines, table names, and row values cannot change template choice, parameters, scope, or output policy. |
| Failure wording | If data is denied or redacted away, the bundle says `db_evidence_missing` instead of inventing a cause. |

Initial acceptance criterion: zero seeded write attempts succeed and zero seeded
secrets appear in agent-visible output.

## Failure Consequences

If the gate fails:

- keep Tier 2 disabled;
- rely on OTel DB spans, errors, migrations, and deploy refs;
- expose database evidence as "missing" in bundles;
- keep direct DB queries human-only outside Parallax;
- do not market database-backed autonomous debugging.

If useful DB context requires raw rows or broad free-form queries, the Parallax
agent-access claim narrows: database evidence remains a human-controlled
investigation extension, not an agent default.

## Relationship To Other Research

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
  database evidence nodes, edges, and redaction refs only after this gate passes.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  should treat direct database evidence as a safety-gated extension, not a tiny
  tier requirement.

## Bottom Line

Database context is valuable, but it is not worth turning Parallax into an
agent-controlled SQL console. The safe path is telemetry-derived DB evidence
first, schema/metadata second, and tightly scoped read-only query templates only
after privilege, redaction, audit, and prompt-injection gates pass.
