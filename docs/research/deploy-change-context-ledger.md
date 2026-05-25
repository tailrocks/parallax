# Deploy/Change Context Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This ledger turns "what changed?" and release-regression context into auditable
claim levels. It consumes the ingestion and edge-strength design in
[Deploy, change, and issue-tracker context](deploy-change-and-issue-context.md)
and the bundle node/edge contract in
[Evidence bundle and open schema](evidence-bundle-and-schema.md).

Current status: **not measured**. Parallax has a deploy/change data model and
edge-strength rules, but no real provider ingestion run, backfill run,
redaction run, or bundle audit. Until those exist, Parallax should describe
release/deploy/code/work-item context as planned evidence, not as proven
"what changed?" intelligence.

The central rule:

> No release-regression or "what changed?" claim without exact release,
> deployment, commit, CI/check, PR/file, and work-item evidence rates, plus
> missing-evidence rows and edge-strength audits.

## Current Source Snapshot

| Source | Current check | Why it matters |
| --- | --- | --- |
| [GitHub Deployments API](https://docs.github.com/en/rest/deployments/deployments?apiversion=2022-11-28) | The page currently renders examples with `X-GitHub-Api-Version: 2026-03-10`. Deployments record `ref`, `sha`, `task`, `payload`, `environment`, transient/production flags, creator, status URL, and repository URL; creating a deployment can require successful commit statuses unless explicitly bypassed. | Parallax must pin the API header/date used in each run and treat deployment objects as requested/deployed-ref evidence, not runtime causality. |
| [GitHub Deployment Statuses API](https://docs.github.com/en/rest/deployments/statuses?apiVersion=2022-11-28) | Status states include `error`, `failure`, `inactive`, `in_progress`, `queued`, `pending`, and `success`; `log_url` is preferred over legacy `target_url`; `environment`, `environment_url`, and `auto_inactive` affect interpretation. | A deployment without a terminal status, environment, and log/status refs is incomplete evidence. |
| [GitHub Actions variables](https://docs.github.com/actions/reference/workflows-and-actions/variables) | `GITHUB_SHA` exists, but its value depends on the workflow event that triggered the run. | CI/deploy ingestion must store event type, ref, and head/base context instead of treating `GITHUB_SHA` alone as deployed truth. |
| [GitHub PR files API](https://docs.github.com/en/rest/pulls/pulls?apiVersion=2022-11-28#list-pull-requests-files) | PR files are paginated and capped at 3000 files; PR commits endpoint caps at 250 commits before needing the commits endpoint. | File-touch and PR-contains-commit edges must include completeness flags and downgrade broad changes. |
| [GitHub compare commits API](https://docs.github.com/en/rest/commits/commits?apiVersion=2022-11-28#compare-two-commits) | Compare supports refs/SHAs and returns commits chronologically with file details, but unpaged responses are limited and large comparisons require pagination. | Release-to-release change windows need exact base/head SHAs and pagination metadata. |
| [Sentry release management CLI](https://docs.sentry.io/cli/releases/) | Sentry CLI can create/finalize releases, set commits automatically or manually, use exact full commit SHAs, configure previous/current commit ranges, and create deploys with at least an environment. | Sentry migration data is useful only when release, commit, dist, and environment semantics are preserved. |
| [Sentry Create a Deploy API](https://docs.sentry.io/api/releases/create-a-deploy/) | Sentry deploy creation requires an environment and may include name, URL, started/finished timestamps, and project slugs. | Sentry deploys can seed Parallax deploy nodes, but missing timestamps/projects should be explicit gaps. |
| [Linear Releases](https://linear.app/docs/releases) | Linear says issue `Done` is not equivalent to delivered; releases connect CI/CD, commit SHA, issues, pipelines, environments, and path filters. Business/Enterprise availability is called out. | Work-item delivery state must be separate from issue status and must carry plan/tier/source limits. |
| [Jira deployments API](https://developer.atlassian.com/cloud/jira/software/rest/api-group-deployments/) | Jira deployment rows include deployment and update sequence numbers, issue keys/associations, pipeline/environment identifiers, state, provider metadata, and accepted/rejected/unknown issue-key responses. | Jira is eventually consistent provider evidence; accepted/rejected/unknown issue keys must affect edge strength. |
| [OpenTelemetry CICD semantic conventions](https://opentelemetry.io/docs/specs/semconv/cicd/) | CICD semantic conventions are development-stage and define spans, logs, and metrics for CI/CD systems. | OTel CICD spans can feed Parallax rows but cannot be the durable schema by themselves. |
| [OpenTelemetry resource conventions](https://opentelemetry.io/docs/specs/semconv/resource/) | Version attributes such as `service.version` are stable and may be semantic versions, git hashes, or arbitrary build strings; deployment environment attributes are present under environment conventions. | Runtime telemetry release/version fields should join deploy/change rows, but arbitrary strings need normalization and missing-commit handling. |

## Claim Levels

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No provider ingestion or bundle audit run exists. | "Deploy/change context is planned." |
| `release_marker_ingest` | Release/version markers from runtime telemetry or Sentry/GitHub sources ingest and normalize with source refs. | "Release markers ingested for the tested sources." |
| `deploy_status_ingest` | Deployment and deployment-status events ingest with environment, terminal state, actor/source refs, and log/status URLs when present. | "Deployment status context for the tested providers." |
| `commit_window_reconstructed` | Release/deploy rows carry exact head and predecessor/base SHAs, and compare/PR backfill produces complete-or-flagged commit/file rows. | "Code-change windows reconstructed for tested releases." |
| `work_item_links_ingested` | GitHub/Linear/Jira work-item links ingest with machine-vs-text link strength and redacted text refs. | "Work-item links attached as context for tested providers." |
| `edge_strength_audited` | Strong/medium/weak/inferred deploy/change edges are audited against raw provider refs, completeness flags, and missing-evidence rows. | "Deploy/change evidence strengths audited for tested anchors." |
| `release_regression_context` | Release, deploy, commit-window, PR/file, CI/check, and missing-evidence rates meet thresholds for production error anchors. | "Release-regression context for the tested services and providers." |
| `what_changed_context` | The bundle can answer likely change candidates with cited release/deploy/code/work-item edges and explicit missing-evidence blockers. | "Evidence-backed 'what changed?' context for the tested subset." |
| `claim_expired` | Provider API behavior, source version/header, schema, edge rules, redaction policy, or freshness window changed. | "Deploy/change context result expired; rerun required." |
| `claim_failed` | A required gate fails for the advertised level. | No claim for the affected provider/surface/edge. |

Initial Parallax level: `not_measured`.

## Result Artifacts

Deploy/change runs should be durable and diffable:

```text
docs/research/deploy-change-context-results.md
docs/research/deploy-change-context-runs/<run_id>/manifest.json
docs/research/deploy-change-context-runs/<run_id>/raw-provider-events/<provider>/<event_id>.json
docs/research/deploy-change-context-runs/<run_id>/release-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/deploy-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/code-change-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/ci-check-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/work-item-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/edge-audit-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/missing-evidence-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/redaction-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/bundle-projection-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/claim-ledger.jsonl
docs/research/deploy-change-context-runs/<run_id>/hashes.sha256
```

Do not create run directories for hypothetical data. Add them only when a real
fixture or pilot run exists.

## Run Manifest

Each `manifest.json` should include:

```json
{
  "run_id": "deploy-change-context-YYYYMMDD-N",
  "research_date": "YYYY-MM-DD",
  "parallax_commit": "<git-sha>",
  "bundle_schema_version": "parallax-bundle-vN",
  "edge_rules_version": "deploy-change-edges-vN",
  "redaction_policy_version": "a6-default-deny-vN",
  "source_snapshot": {
    "github_rest_api_header": "2026-03-10",
    "sentry_api": "current-docs-YYYY-MM-DD",
    "linear_releases": "current-docs-YYYY-MM-DD",
    "jira_deployments": "current-docs-YYYY-MM-DD",
    "otel_semconv": "1.41.0",
    "otel_cicd_status": "development"
  },
  "providers": ["github", "sentry", "linear", "jira", "otel-cicd"],
  "anchor_types": ["issue", "error_event", "trace"],
  "environments": ["production", "staging"],
  "notes": []
}
```

The manifest must separate provider API/header/date, Parallax schema versions,
edge-rule versions, and redaction policy. A pass for one provider/environment
does not carry over to another.

## Row Schemas

### Release Result Row

```json
{
  "anchor_id": "issue_123",
  "provider": "sentry|github|otel",
  "release": "checkout@2026.05.25-4",
  "environment": "production",
  "commit_sha": "9d1f...",
  "predecessor_release": "checkout@2026.05.25-3",
  "predecessor_commit_sha": "7a2b...",
  "source_ref": "sentry:release:<id>",
  "release_context_present": true,
  "release_commit_present": true
}
```

### Deploy Result Row

```json
{
  "deployment_id": "github:deployment:42",
  "provider": "github",
  "release": "checkout@2026.05.25-4",
  "ref": "refs/heads/main",
  "commit_sha": "9d1f...",
  "environment": "production",
  "terminal_state": "success",
  "started_at": "2026-05-25T14:58:00Z",
  "finished_at": "2026-05-25T15:01:44Z",
  "log_url_present": true,
  "status_event_count": 3,
  "missing_fields": []
}
```

### Code Change Result Row

```json
{
  "change_id": "github:compare:7a2b..9d1f",
  "repo": "github.com/acme/checkout",
  "base_sha": "7a2b...",
  "head_sha": "9d1f...",
  "commit_count": 12,
  "files_count": 38,
  "files_complete": true,
  "pagination_complete": true,
  "pr_refs": ["https://github.com/acme/checkout/pull/456"],
  "truncated_reasons": []
}
```

### CI/Check Result Row

```json
{
  "check_id": "github:check_run:99",
  "provider": "github",
  "commit_sha": "9d1f...",
  "event_name": "push",
  "ref": "refs/heads/main",
  "status": "completed",
  "conclusion": "success",
  "workflow": "deploy",
  "log_ref_present": true,
  "deployed_truth": false
}
```

### Work Item Result Row

```json
{
  "work_item_id": "linear:ENG-123",
  "provider": "linear",
  "status": "Done",
  "delivery_state": "released_to_production|unknown",
  "linked_prs": ["https://github.com/acme/checkout/pull/456"],
  "linked_commits": ["9d1f..."],
  "link_strength": "machine|text|unknown",
  "description_mode": "summary_only|ref_only|denied",
  "unknown_or_rejected_links": []
}
```

### Edge Audit Result Row

```json
{
  "anchor_id": "issue_123",
  "edge_type": "deploy_preceded_issue",
  "from": "deploy_42",
  "to": "issue_123",
  "strength": "medium",
  "expected_strength": "medium",
  "raw_refs": ["github:deployment:42", "github:deployment_status:77"],
  "supporting_fields": ["environment", "commit_sha", "terminal_state"],
  "missing_evidence": [],
  "audit_status": "pass|fail"
}
```

### Missing Evidence Result Row

```json
{
  "anchor_id": "issue_123",
  "category": "missing_release_commit",
  "provider": "sentry",
  "severity": "blocks_strong_edge",
  "bundle_visible": true,
  "recommended_repair": "Set exact commit SHA in release metadata."
}
```

### Claim Ledger Row

```json
{
  "run_id": "deploy-change-context-YYYYMMDD-N",
  "claim_level": "release_regression_context",
  "claim_status": "pass|fail|expired",
  "version_matrix": {
    "github_rest_api_header": "2026-03-10",
    "otel_semconv": "1.41.0",
    "bundle_schema": "parallax-bundle-vN",
    "edge_rules": "deploy-change-edges-vN"
  },
  "product_wording": "Release-regression context for the tested services and providers.",
  "required_caveats": ["linkage is not causality", "text-only work-item links are weak"],
  "expires_at": "YYYY-MM-DD"
}
```

## Counting Rules

- No "what changed?" claim without release/deploy/code/work-item missing-evidence
  rows visible in the bundle.
- No strong deploy edge unless exact environment, terminal deployment status,
  and deployed ref/SHA/release are present.
- No code-change touched-frame claim when PR file lists or compare results are
  truncated or incomplete.
- No release-regression claim when predecessor release/base SHA is missing.
- No work-item delivery claim from issue `Done` alone. Delivery requires release
  or deployment association.
- Machine-emitted links outrank text/magic-word links; text-only links are never
  strong without provider confirmation.
- CI/check success is validation evidence, not deployed runtime truth, unless
  linked to a deployment or release.
- Provider text fields, issue descriptions, comments, deploy logs, and release
  notes are untrusted and redacted/ref-only by default.
- OTel CICD spans are adapter input. Because the CICD conventions are
  development-stage, they cannot be the only proof for a durable Parallax schema
  claim.
- Strong deploy/change edges prove linkage, not root cause. Causality still
  needs runtime evidence, first-seen/spike analysis, touched code, recurrence,
  or contradiction checks.

## Refresh Triggers

Rerun the matrix and mark affected claims `claim_expired` when any of these
change:

- GitHub REST API version/header, deployment/status API behavior, PR file/commit
  caps, compare pagination, Actions event/SHA semantics, or webhook payloads
  change;
- Sentry release/deploy API or CLI release semantics change;
- Linear release, GitHub integration, path-filter, or plan availability changes;
- Jira development/deployment information APIs, sequence semantics, or accepted
  and rejected response shapes change;
- OpenTelemetry semantic conventions, CICD conventions, resource version fields,
  or VCS fields change;
- Parallax bundle schema, deploy/change node shape, edge-strength rules,
  redaction policy, or provider adapters change;
- a new provider is included in product wording;
- 60 days pass since the last run.

## Product Wording

Allowed after `deploy_status_ingest`:

> Deployment status context for the tested providers and environments.

Allowed after `release_regression_context`:

> Release-regression context for the tested services and providers, with
> explicit missing-evidence and edge-strength reports.

Allowed after `what_changed_context`:

> Evidence-backed "what changed?" context for the tested subset, citing
> release, deploy, code-change, CI/check, and work-item records.

Avoid:

- "root cause from deploy metadata";
- "automatically finds the breaking commit";
- "knows what shipped" without environment-specific release/deploy evidence;
- "issue Done means shipped";
- "PR touched the file, so it caused the issue";
- "full GitHub/Linear/Jira context" without provider, scope, and freshness
  caveats.

## Relationship To Other Research

- [Deploy, change, and issue-tracker context](deploy-change-and-issue-context.md)
  defines ingestion and edge semantics this ledger turns into claim levels.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines the
  release, deploy, deployment-status, code-change, work-item, and check-run
  nodes this ledger audits.
- [Correlation reliability on real telemetry gate](correlation-reliability-real-telemetry-gate.md)
  and [A4 correlation reliability ledger](a4-correlation-reliability-ledger.md)
  measure whether these edges survive real telemetry.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  and [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) have veto
  power over issue text, deploy logs, release notes, and PR text in bundles.
- [Fixer component and outcome loop](fixer-component-and-outcome-loop.md)
  consumes code-change and outcome records after the evidence path is proven.

## Bottom Line

Release, deploy, code-change, CI/check, and work-item context is mandatory for
useful lifecycle reconstruction, but it is easy to overclaim. Parallax should
prove exact linkage and make missing context visible before it lets an agent say
what changed.
