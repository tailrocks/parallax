# Production Database Evidence Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

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
> limits, redaction, audit, prompt-injection resistance, and failure wording.

## Current Source Snapshot

| Source | Ledger consequence |
| --- | --- |
| [PostgreSQL 18 privileges](https://www.postgresql.org/docs/current/ddl-priv.html) | The connector role proof must show no owner, superuser, grant-option, DML, DDL, `MAINTAIN`, or broad schema privileges. `SELECT` alone is not enough because it can still expose sensitive rows and supports export-like paths such as `COPY TO`. |
| [PostgreSQL 18 row security](https://www.postgresql.org/docs/current/ddl-rowsecurity.html) | RLS proof must verify positive and negative tenant/project fixtures and prove the evidence role is not a table owner, superuser, or `BYPASSRLS` role. |
| [PostgreSQL 18 read-only transactions](https://www.postgresql.org/docs/current/sql-set-transaction.html) | Read-only transactions are required as a runtime guardrail, but they do not replace privilege proof, template parsing, RLS, output limits, or redaction. |
| [PostgreSQL 18 SELECT](https://www.postgresql.org/docs/current/sql-select.html) | The parser must reject lock-taking reads such as `FOR UPDATE`/`FOR SHARE`, dynamic identifiers, `SELECT *`, stacked statements, and unbounded scans. |
| [OpenTelemetry database semantic conventions](https://opentelemetry.io/docs/specs/semconv/db/database-spans/) | DB spans, operation names, summaries, row counts, errors, and latency are Tier 0 database evidence before any direct connector exists. |
| [OWASP SQL Injection Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html) | Templates must use typed parameters, allow-listed identifiers, and least privilege; free-form SQL cannot count as safe database evidence. |
| [OWASP Top 10 for LLM Applications](https://owasp.org/www-project-top-10-for-large-language-model-applications/) | Prompt injection, sensitive information disclosure, insecure tool design, and excessive agency are direct fixture categories for database evidence. |
| [NSA MCP security design considerations](https://www.nsa.gov/Portals/75/documents/Cybersecurity/CSI_MCP_SECURITY.pdf?ver=bmgiSbNQLP6Z_GiWtRt6bg%3D%3D) | Dynamic tool invocation, implicit trust, token/session handling, context sharing, and tool-description risk make direct DB templates stricter than ordinary evidence-bundle projection. |

## Claim Levels

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
| `agent_visible_db_evidence` | Agent-visible bundles may include direct DB evidence for the tested templates. | `tier2_template_safe` plus CLI/HTTP/MCP projection-equivalence and bundle-redaction checks pass. |
| `claim_expired` | A prior claim is stale. | Refresh trigger fired or max age elapsed. |
| `claim_failed` | A required fixture failed. | Any write/DDL/lock/export succeeds, any secret leaks, any scope bypass succeeds, or unsafe wording reaches a bundle. |

Initial claim level: `not_measured`.

## Result Artifacts

The durable result index lives at:

```text
docs/research/production-database-evidence-results.md
```

Each run stores immutable artifacts under:

```text
docs/research/production-database-evidence-runs/<run_id>/manifest.json
docs/research/production-database-evidence-runs/<run_id>/template-manifests/*.yaml
docs/research/production-database-evidence-runs/<run_id>/privilege-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/rls-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/parser-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/runtime-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/limit-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/redaction-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/prompt-injection-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/audit-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/bundle-projection-results.jsonl
docs/research/production-database-evidence-runs/<run_id>/claim-ledger.jsonl
docs/research/production-database-evidence-runs/<run_id>/hashes.sha256
```

## Run Manifest

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
    "template_policy_version": "db-template-vN",
    "audit_schema_version": "audit-vN",
    "bundle_schema_version": "0.1.0"
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

## Minimum Row Schemas

Template manifest row:

```json
{
  "template_id": "db.error_context.v1",
  "template_hash": "sha256:<hex>",
  "statement_class": "select_only",
  "allowed_identifiers_hash": "sha256:<hex>",
  "required_role": "parallax_evidence",
  "raw_values_allowed": false,
  "max_rows": 20,
  "max_bytes": 32768,
  "timeout_ms": 1000
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
  "missing_evidence_on_denial": true,
  "unsafe_raw_ref_exposed": false,
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

## Counting Rules

- No Tier 2 claim if any write, DDL, maintenance, lock, `COPY`, export, grant,
  revoke, migration, or rollback attempt succeeds.
- No direct database claim from read-only transactions alone.
- No free-form SQL, model-written SQL, stacked statements, dynamic identifiers,
  or `SELECT *`.
- No RLS/view claim if the tested role owns target tables, is superuser, has
  `BYPASSRLS`, or can see negative-tenant/project rows.
- No raw row exposure by default. Query output must be aggregated, bounded,
  redacted, and represented with a redaction report.
- No agent-visible DB evidence unless missing, denied, or unsafe data is
  explicitly represented as `db_evidence_missing`.
- DB evidence can support or contradict hypotheses; it does not become a root
  cause by itself.
- A pass is scoped to the database system/version, role, templates, policies,
  bundle schema, and agent surface listed in the manifest.

## Refresh Triggers

Mark the claim `claim_expired` and rerun when any of these changes:

- PostgreSQL version, privilege behavior, RLS policy behavior, transaction
  behavior, extension set, or supported database engine.
- Template parser, template manifests, identifier allowlists, role/grants,
  views/RLS policies, redaction policy, auth policy, audit schema, bundle
  schema, or agent surface.
- OWASP LLM, OWASP SQL injection, MCP, or NSA-style agent-tooling guidance that
  materially changes the threat model.
- Any seeded secret/PII canary, prompt-injection case, or negative-scope case is
  added.
- Ninety days pass for `tier2_template_safe`; sixty days pass for
  `agent_visible_db_evidence`.

## Product Wording

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
- "Read-only means safe."
- "Autonomous database debugging."
- "Free-form SQL for agents."
- "Direct root-cause analysis from database rows."
- "Production database access is enabled by default."

## Relationship To Other Research

- [Production database evidence access gate](production-database-evidence-access.md)
  defines the safety requirements that this ledger measures.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  and [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) can veto
  every database-evidence claim.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) carries
  database evidence as bounded nodes, edges, missing-evidence flags, and
  redaction refs only after this ledger permits the claim.
- [Agent access surface safety ledger](agent-access-surface-safety-ledger.md)
  ensures CLI, HTTP, and MCP expose the same redacted database evidence.
- [Deploy/change context ledger](deploy-change-context-ledger.md) supplies the
  release, deploy, commit, and work-item context that database evidence can
  support or contradict.
- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
  owns the autonomy boundary: database evidence informs hypotheses, while
  production mutation remains out of scope.

## Bottom Line

Production database evidence is claimable only as a measured, scoped capability.
Until this ledger is green, Parallax can use telemetry-derived database context
and mark direct database evidence as missing; it cannot claim that agents can
see production database evidence safely.
