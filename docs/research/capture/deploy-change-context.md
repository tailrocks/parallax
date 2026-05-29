# Deploy, Change, and Issue Context

> Deploy/change/issue context is first-class evidence for answering "what changed?" after a production error, but it is decided that it is never root-cause proof by itself. The contract is set: Parallax ingests exact release, deployment, commit, PR, CI/check, workflow-run/job, and work-item records from GitHub/Sentry/Linear/Jira (plus OTel CICD as adapter input), attaches them to telemetry by stable identifiers first, downgrades everything else to explicit medium/weak/inferred hypotheses, and loudly reports missing-evidence categories rather than guessing. Normalized nodes, edge-strength rules, privacy defaults (issue/deploy text and logs are untrusted, redacted/ref-only by default), and the proof-gate thresholds are defined. What remains an open gate is execution: the deploy/change result ledger is currently at claim level `not_measured` — there is a data model and edge rules but no real provider ingestion, backfill, redaction, or bundle-audit run yet, so release/deploy/code/work-item context must be described as planned evidence, not proven "what changed?" intelligence. Strong deploy/change edges prove linkage only; causality still requires runtime evidence (first-seen/spike timing, trace/log/metric support, touched code, recurrence after fix, or contradiction analysis). Claims advance through explicit claim levels with allowed wording, expire on provider/schema/policy/freshness changes, and fail closed per provider, surface, and edge.

This note consolidates the following previously-separate research files, each preserved in full below:

- `deploy-change-and-issue-context.md`
- `deploy-change-context-ledger.md`

## Deploy, Change, and Issue-Tracker Context (ingestion and edge contract)
_Provenance: merged verbatim from `deploy-change-and-issue-context.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

The prompt repeatedly names release/deploy metadata, code changes, issue
trackers, and "what changed?" as core evidence. The current schema already has
`release`, `deploy`, and `code_change` nodes, but it does not yet define how
those records enter Parallax, how strong their edges are, or when they are too
weak to support a causal claim.

This note defines the contract:

> Deploy/change/issue context is first-class evidence, but it is not root-cause
> proof by itself. Parallax should ingest exact release, deployment, commit, PR,
> CI, and issue-tracker records, attach them to telemetry by stable identifiers
> first, and downgrade everything else to explicit medium/weak hypotheses.

Results and product-claim status should be published through the
[Deploy/change context ledger](deploy-change-context.md), not inferred
from this design alone.

### Current Primary-Source Checks

| Source | What it shows | Parallax implication |
| --- | --- | --- |
| [GitHub API versions](https://docs.github.com/en/rest/about-the-rest-api/api-versions) | GitHub currently supports REST API versions `2026-03-10` and `2022-11-28`; unversioned requests default to `2022-11-28`. | Deploy/change fixtures must record the request header, docs page API version, and source-check date separately so reruns can explain API drift. |
| [GitHub Deployments API](https://docs.github.com/en/rest/deployments/deployments?apiVersion=2022-11-28) | Deployments are requests to deploy a ref, include environment/task/payload, and dispatch `deployment` events. GitHub keeps deployment execution outside GitHub. The docs page is labeled `2022-11-28`, while current examples use `X-GitHub-Api-Version: 2026-03-10`. | A GitHub deployment is a strong change marker only when Parallax records the deployed SHA/ref, environment, status, external deployment logs, and the GitHub API version/header used to collect it. |
| [GitHub Deployment Statuses API](https://docs.github.com/en/rest/deployments/statuses?apiVersion=2022-11-28) | Statuses include `queued`, `in_progress`, `success`, `failure`, `error`, and `inactive`, plus `log_url`, `target_url`, `environment_url`, and `auto_inactive`. | A deployment without final status is incomplete evidence. `success` is a runtime marker, not proof that an issue was introduced or fixed. |
| [GitHub deployment webhooks](https://docs.github.com/en/webhooks/webhook-events-and-payloads) | `deployment` and `deployment_status` webhooks carry deployment activity; deployment-status webhooks require deployment read permission and do not fire for inactive statuses. | Webhooks are the lowest-latency ingestion path, but Parallax still needs API backfill because missed/inactive transitions can matter. |
| [GitHub deployment review webhooks](https://docs.github.com/en/webhooks/webhook-events-and-payloads#deployment_review) | `deployment_review` webhook payloads represent approval activity and can include reviewer, comment, deployment callback, workflow run, and workflow job run context. | Deployment approval/protection evidence is a separate policy edge. A successful deployment status does not prove who or what approved the deploy. |
| [GitHub Actions variables](https://docs.github.com/en/actions/reference/workflows-and-actions/variables) | `GITHUB_SHA` records the commit that triggered a workflow, with value depending on the event. | CI/deploy ingestion must record event type and ref context; `GITHUB_SHA` alone is ambiguous for PR workflows. |
| [GitHub events that trigger workflows](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows) | `pull_request` workflows use the PR merge branch and merge-commit SHA, while `workflow_run` workflows use the last commit on the default branch. Deployment and deployment-status events use the commit/ref to be deployed, and inactive deployment statuses do not trigger workflows. | Actions rows need event-specific SHA semantics, PR head/base refs, and trigger lineage before they can support a change candidate. A workflow-run SHA is often automation context, not the deployed or PR head commit. |
| [GitHub workflow runs API](https://docs.github.com/en/rest/actions/workflow-runs?apiVersion=2022-11-28) | Workflow run list/backfill exposes filters for event, branch, check suite, and head SHA, and response rows include run ID, attempt, head branch/SHA, event, pull requests, jobs/logs/check-suite URLs, and head commit. | CI evidence should preserve the workflow run object separately from checks so Parallax can tell which workflow/event observed which commit. |
| [GitHub workflow jobs API](https://docs.github.com/en/rest/actions/workflow-jobs?apiVersion=2022-11-28) | Workflow jobs are fetched by run ID; the API distinguishes latest vs all attempts and returns job ID, run ID, head SHA, status/conclusion, timing, steps, check-run URL, runner labels, workflow name, and head branch. | Job-level evidence is useful for deploy steps and runner context, but it must not overwrite the parent workflow run/event interpretation. |
| [GitHub issue timeline API](https://docs.github.com/en/rest/issues/timeline?apiVersion=2022-11-28) | Timeline events cover issue/PR activity; every pull request is an issue, but not every issue is a pull request. | GitHub issues and PRs should be one normalized `work_item` family, with provider-specific event refs preserved. |
| [GitHub pull request files API](https://docs.github.com/en/rest/pulls/pulls?apiVersion=2022-11-28#list-pull-requests-files) | PR file lists are paginated and capped at 3000 files. | `code_change_touched_frame` is only reliable for bounded PRs; broad PRs must be marked incomplete or low confidence. |
| [GitHub compare commits API](https://docs.github.com/en/rest/commits/commits?apiVersion=2022-11-28#compare-two-commits) | GitHub compares base/head refs or SHAs and can return commit/file deltas. | Release-to-release change windows can be reconstructed if predecessor/head SHAs are known; missing base makes blame weak. |
| [GitHub check runs API](https://docs.github.com/en/rest/checks/runs?apiVersion=2022-11-28) | Check runs track status/conclusion for a specific `head_sha`, but the Checks API only detects pushes in the repository where the check run or suite was created; fork pushes can return an empty PR array. | CI validation is separate evidence from deployment. A green check is not a deployed runtime fact unless linked to a deploy/release, and check-to-PR edges need explicit PR/head/base evidence. |
| [Sentry releases API](https://docs.sentry.io/api/releases/) and [Create a Deploy](https://docs.sentry.io/api/releases/create-a-deploy/) | Sentry models releases and deploys; deploy creation requires environment and can include name, URL, started/finished times, and project list. | Sentry release/deploy data is a migration source for Parallax and a compatibility target for users already tagging releases. |
| [Sentry release management CLI](https://docs.sentry.io/product/cli/releases/?promo_name=hp-banner) | Sentry recommends creating a release first and then a deploy with an environment; deploys can be listed but not deleted. | Parallax should mirror the simple `release -> deploy -> environment` mental model, while keeping append-only corrections instead of destructive delete semantics. |
| [Linear GitHub integration](https://linear.app/docs/github-integration) | Linear links PRs and commits to issues through branches, titles, magic words, and commit messages; workflow automation can move issues based on PR/commit activity. | Linear is useful work-context evidence, but text/magic-word links must be treated differently from machine IDs emitted by GitHub webhooks. |
| [Linear Releases](https://linear.app/docs/releases) | Linear can connect CI/CD to know which issues ship in each release and environment; releases scan commits for issue references. | Issue delivery state is distinct from issue "Done" state. Parallax should ingest shipped-to-environment context when available. |
| [Jira development information API](https://developer.atlassian.com/cloud/jira/software/rest/api-group-development-information/) | Jira accepts repositories, commits, branches, and pull requests asynchronously; data becomes available eventually and update sequence IDs determine replacement. | Jira dev info is an eventual-consistency source. Parallax must store provider sequence/update metadata and avoid assuming immediate completeness. |
| [Jira deployments API](https://developer.atlassian.com/cloud/jira/software/rest/api-group-deployments/) | Jira deployment data is keyed by pipeline, environment, and deployment sequence; submissions are async and include accepted/rejected/unknown issue-key details. | Deployment records can carry issue associations and environment state, but unknown issue keys and rejected entities must appear as missing evidence. |

### Boundary Decision

Parallax should own the normalized evidence record and edge-strength rules, not
the external workflow system.

| Source system | Parallax uses it for | Parallax must not assume |
| --- | --- | --- |
| GitHub deployments/statuses | Exact deployed ref/SHA, environment, deploy status, actor, URLs, logs. | That deployment success caused or fixed an issue. |
| GitHub Actions/checks | CI/build/test validation by commit/ref. | That a green check was deployed to production. |
| GitHub commits/PRs | Changed files, commits, merge refs, linked issues. | That a touched file is the root cause without stack/symbol evidence. |
| Sentry releases/deploys | Migration-compatible release/deploy markers and environments. | Sentry's release heuristic is sufficient for Parallax causality. |
| Linear/Jira | Work item status, intent, linked PRs/commits, issue delivery state. | That issue state is runtime truth. "Done" is not "deployed" and not "fixed." |
| Deploy tool logs | Build/deploy command refs and rollout status. | That logs are safe to expose without redaction. |

The deploy/change layer should be append-only with corrections. External systems
mutate their state; Parallax should preserve the observed sequence.

### Normalized Nodes

Add or tighten these schema nodes:

| Node | Required fields | Notes |
| --- | --- | --- |
| `release` | `version`, `repo`, `commit_sha?`, `created_at`, `released_at?`, `source`, `project_refs[]` | Release strings are often user-defined. Commit SHA should be present for strong code-change edges. |
| `deploy` | `provider`, `deployment_id`, `release?`, `repo?`, `ref?`, `commit_sha?`, `environment`, `state`, `started_at?`, `finished_at?`, `actor?`, `task?`, `log_url?`, `source_ref` | `state` must distinguish requested/queued/in-progress/success/failure/error/inactive. |
| `deployment_status` | `deployment_id`, `state`, `created_at`, `environment?`, `log_url?`, `target_url?`, `description?` | Preserve each status event rather than only the latest state. |
| `deployment_review` | `deployment_id`, `state`, `reviewer?`, `created_at`, `comment_ref?`, `workflow_run?`, `workflow_job_run?`, `source_ref` | Approval/protection evidence is policy context, not runtime causality. Comments stay ref-only by default. |
| `code_change` | `repo`, `base_ref?`, `head_ref`, `base_sha?`, `head_sha`, `commits[]`, `files[]`, `pr_url?`, `merge_commit_sha?`, `compare_url?` | File list can be incomplete for very large PRs; store `files_complete`. |
| `work_item` | `provider`, `key`, `title`, `status`, `type`, `url`, `created_at`, `updated_at`, `labels[]`, `linked_prs[]`, `linked_commits[]` | Description/comments are high-risk and should be summarized or ref-only by default. |
| `workflow_run` | `provider`, `run_id`, `run_attempt`, `event_name`, `workflow_name`, `head_branch?`, `head_sha`, `github_ref?`, `github_sha?`, `pull_requests[]`, `check_suite_id?`, `jobs_url?`, `logs_ref?`, `source_ref` | Preserve the event-trigger identity separately from deployment and check status. For PR events, store merge SHA and PR head/base SHA separately. |
| `workflow_job` | `provider`, `job_id`, `run_id`, `run_attempt?`, `head_sha`, `status`, `conclusion?`, `started_at?`, `completed_at?`, `step_refs[]`, `check_run_url?`, `runner_ref?`, `source_ref` | Job and step names are execution evidence; logs remain refs/redacted by default. |
| `check_run` | `provider`, `check_run_id`, `head_sha`, `check_suite_id?`, `status`, `conclusion?`, `started_at?`, `completed_at?`, `workflow_run_id?`, `pull_requests[]`, `log_ref?` | CI validation should connect to code change and deploy separately. Check PR linkage can be absent, especially for fork-originated changes. |

Provider-specific raw IDs stay in `refs` so integrations can replay/backfill.

### Edge Strength Rules

Use deterministic identifiers before time windows.

| Edge | Strength | Rule |
| --- | --- | --- |
| `event_observed_in_release` | strong | Error event explicitly carries `release` matching a normalized release/version for the same project/environment. |
| `deploy_status_for_release` | strong | Deployment status references a deployment whose ref/SHA/release is known. |
| `deployment_review_for_deploy` | strong | Provider review/approval event references the deployment ID and reviewer/workflow context. This proves approval lineage, not deploy success or causality. |
| `deploy_contains_commit` | strong | Deployment or release records exact `commit_sha` and it equals the commit/change node. |
| `workflow_run_observed_commit` | strong for the recorded event context only | Workflow run records event name, head SHA, ref, run attempt, and PR head/base/merge context when applicable. This proves workflow execution context, not deployment. |
| `check_validated_commit` | strong for validation only | Check run/check suite records the same head SHA and has enough workflow/event or PR context to interpret that SHA. |
| `pr_contains_commit` | strong | GitHub PR commits endpoint or merge metadata contains the commit. |
| `work_item_linked_pr` | strong when provider emits link; medium when parsed from title/body/magic words | Machine-emitted provider link is stronger than text convention. |
| `release_contains_work_item` | medium | Release tool or issue tracker says issue shipped in the release/environment, but runtime deploy confirmation is separate. |
| `code_change_touched_frame` | medium | Changed file/symbol matches top in-app frame; symbol match is stronger than file-only match. |
| `deploy_preceded_issue` | medium | Successful deployment in same environment before first occurrence/spike, with bounded window and no stronger contradiction. |
| `issue_done_before_deploy` | weak | Tracker state changed to done before/near deploy, but no release/deploy association exists. |
| `temporal_change_near_error` | weak | Commit/PR/deploy occurred nearby but lacks exact release/environment/trace linkage. |
| `model_suggested_change_cause` | inferred | LLM hypothesis only; must cite deterministic edges and cannot stand alone. |

Strong deploy/change edges prove linkage, not causality. Causality needs runtime
evidence: first-seen timing, trace/log/metric support, touched code, recurrence
after fix, or contradiction analysis.

### Missing Evidence Categories

Bundles should report these gaps explicitly:

- `missing_release`
- `missing_release_commit`
- `missing_deploy`
- `missing_deploy_status`
- `missing_deploy_environment`
- `missing_deploy_log`
- `missing_deployment_review`
- `missing_deployment_backfill`
- `missing_webhook_delivery`
- `missing_inactive_status_backfill`
- `missing_predecessor_release`
- `missing_compare_base`
- `missing_pr_file_list`
- `pr_file_list_truncated`
- `missing_issue_tracker_link`
- `issue_tracker_link_text_only`
- `issue_tracker_eventually_consistent`
- `missing_ci_check`
- `missing_workflow_run`
- `missing_workflow_job`
- `missing_actions_event_context`
- `ambiguous_actions_sha`
- `missing_pr_head_base_sha`
- `missing_source_owner`

Agents should see these as blockers to strong claims, not as prompts to guess.

### Ingestion Contract

The first Parallax contract should be provider-neutral:

```json
{
  "event_id": "chg_01J...",
  "provider": "github",
  "kind": "deployment_status",
  "external_id": "github:deployment_status:123",
  "observed_at": "2026-05-25T15:02:11Z",
  "repo": "github.com/acme/checkout",
  "release": "checkout@2026.05.25-4",
  "ref": "refs/heads/main",
  "commit_sha": "9d1f...",
  "environment": "production",
  "state": "success",
  "started_at": "2026-05-25T14:58:00Z",
  "finished_at": "2026-05-25T15:01:44Z",
  "actor": "deploy-bot",
  "webhook_delivery_id": "github-delivery-id",
  "api_backfill_checked_at": "2026-05-25T15:05:00Z",
  "inactive_status_backfilled": true,
  "deployment_review_state": "approved|not_required|missing",
  "urls": {
    "deployment": "https://github.com/acme/checkout/deployments/42",
    "log": "https://github.com/acme/checkout/actions/runs/99"
  },
  "source": {
    "webhook_delivery_id": "github-delivery-id",
    "api_url": "https://api.github.com/repos/acme/checkout/deployments/42/statuses"
  }
}
```

For issue/work-item context:

```json
{
  "event_id": "wrk_01J...",
  "provider": "linear",
  "kind": "work_item_updated",
  "external_key": "ENG-123",
  "url": "https://linear.app/acme/issue/ENG-123",
  "status": "Done",
  "title": "Checkout panic on empty discount rules",
  "linked_prs": ["https://github.com/acme/checkout/pull/456"],
  "linked_commits": ["9d1f..."],
  "redaction": {
    "description_mode": "summary_only",
    "comments_mode": "ref_only"
  }
}
```

Idempotency key:

```text
project_id + provider + kind + external_id + observed_at/update_sequence
```

For providers with update sequence numbers, use the provider sequence for
ordering. For providers without one, preserve observed order and backfill with
API reads.

### Privacy And Safety

Issue trackers are not safe text sources. Descriptions, comments, customer
requests, support links, and deployment logs can contain secrets, customer data,
internal plans, or prompt-injection text.

Defaults:

- store raw issue descriptions/comments/logs as refs with scoped access;
- expose title, status, labels, owner, linked refs, and a short redacted summary
  in agent-visible bundles;
- treat all issue/deploy text as untrusted, never as policy;
- run the redaction pipeline on deploy logs and issue text before bundle render;
- require explicit user opt-in before agent-visible comments/customer requests;
- require source-field policy rows for synthetic/evaluation/corpus fixtures, and
  keep raw provider payloads, issue text, release notes, and deploy logs as
  non-dereferenced refs in default agent projections.

### Implementation Order

1. **V0 GitHub + Sentry release/deploy markers.** Accept GitHub deployment and
   deployment-status webhooks, GitHub Actions commit/environment fields, Sentry
   release/deploy data, and GitHub compare/PR file backfill.
2. **V0 bundle edges.** Add release/deploy/code-change edges and missing-evidence
   fields to backend error bundles. Do not add issue tracker descriptions yet.
3. **V1 issue tracker refs.** Ingest GitHub issues/PR timelines and Linear/Jira
   work-item links as metadata refs, with redacted summaries only.
4. **V1 deploy diagnostics.** Add `parallax doctor deploy-context` to check
   whether releases carry commit SHAs, deploys carry environments/statuses, and
   PR file lists, deployment reviews, inactive-status backfill, and webhook/API
   delivery coverage are complete.
5. **Later writeback.** Only after the fixer outcome loop is proven, write
   Parallax bundle/outcome links back to GitHub/Linear/Jira.

### Proof Gate

Before Parallax claims release-regression or "what changed?" intelligence:

| Gate | Target |
| --- | --- |
| `release_context_rate` | >= 90 percent for production error anchors. |
| `release_commit_rate` | >= 80 percent of release markers carry exact commit SHA or source revision. |
| `deploy_context_rate` | >= 70 percent where deploy markers are configured. |
| `deploy_success_status_rate` | >= 90 percent of deploys have terminal status within the audit window. |
| `deployment_backfill_coverage_rate` | 100 percent for providers where webhooks omit states or delivery can be missed. |
| `deployment_review_context_rate` | >= 90 percent where deployment protection/reviews are configured. |
| `actions_event_context_rate` | 100 percent for GitHub Actions rows used in deploy/change edges. |
| `compare_base_rate` | >= 80 percent of release/deploy windows have predecessor ref/commit for diff. |
| `pr_file_list_complete_rate` | >= 95 percent for PRs used in `code_change_touched_frame` edges. |
| `work_item_machine_link_rate` | >= 70 percent for issue tracker links before treating work items as more than weak context. |
| `missing_evidence_report_rate` | 100 percent for expected release/deploy/change gaps. |
| `source_field_policy_violations` | 0 for synthetic/evaluation/corpus fixtures. |
| `raw_ref_dereference_count` | 0 for default agent-visible projections. |
| `causality_overclaim_count` | 0 deploy/change rows worded as root cause without runtime support. |

Failure consequences:

- If release commit is missing, do not rank code-change hypotheses above weak.
- If deploy status is missing, say "release observed" rather than "deployed."
- If webhook/API backfill is incomplete, mark deploy evidence incomplete even
  when a success webhook exists.
- If deployment review/protection context is configured but missing, do not claim
  who approved or which gate allowed the deploy.
- If Actions event context is missing, say "workflow/check observed" rather
  than "validated this deployed change."
- If a PR workflow uses a merge-branch SHA, keep PR head/base and merge SHA
  distinct before connecting checks to code-change candidates.
- If PR file list is truncated, do not use file-touch evidence as a strong
  explanation.
- If issue-tracker links are text-only, keep them as context, not causality.

### Relationship To Other Research

- [Evidence bundle and open schema](../architecture/evidence-bundle-schema.md) defines the
  nodes, edges, source-field policy status, and redaction report fields this
  note tightens.
- [Deploy/change context ledger](deploy-change-context.md) turns provider
  ingestion, completeness, edge-strength, missing-evidence, and redaction runs
  into claim levels and allowed product wording.
- [Correlation reliability on real telemetry gate](correlation.md)
  already measures `release_context_rate` and `deploy_context_rate`; this note
  defines the ingestion and edge semantics behind those metrics.
- [Technical implementation concept](../architecture/implementation-concept.md)
  includes release/deploy context in the first useful loop.
- [Fixer component and outcome loop](../decisions/fixer-boundary.md) later
  consumes PR/commit/deploy outcome records, but does not own deploy mutation.
- [Redaction pipeline and secret safety](redaction.md)
  has veto power over issue text and deploy logs before agent exposure.

### Bottom Line

Deploy/change context is essential because "what changed?" is usually the first
question after a production error. It becomes dangerous when treated as proof.
Parallax should ingest exact release, deployment, commit, PR, CI, and work-item
records; compute deterministic edges; and loudly mark missing or weak links.
Only then can an agent reason about likely regressions without hallucinating a
causal story from a timestamp.

## Deploy/Change Context Ledger (claim levels and result artifacts)
_Provenance: merged verbatim from `deploy-change-context-ledger.md` (2026-05-29 restructure)._

_(Shared note — see the Deploy, Change, and Issue-Tracker Context (ingestion and edge contract) section above.)_

Research date: 2026-05-25

### Purpose

This ledger turns "what changed?" and release-regression context into auditable
claim levels. It consumes the ingestion and edge-strength design in
[Deploy, change, and issue-tracker context](deploy-change-context.md)
and the bundle node/edge contract in
[Evidence bundle and open schema](../architecture/evidence-bundle-schema.md).

Current status: **not measured**. Parallax has a deploy/change data model and
edge-strength rules, but no real provider ingestion run, backfill run,
redaction run, or bundle audit. Until those exist, Parallax should describe
release/deploy/code/work-item context as planned evidence, not as proven
"what changed?" intelligence.

The central rule:

> No release-regression or "what changed?" claim without exact release,
> deployment, commit, workflow-run/job, CI/check, PR/file, and work-item
> evidence rates, plus missing-evidence rows, edge-strength audits,
> source-field policy status, redaction reports, and agent-visible projection
> checks.

### Current Source Snapshot

| Source | Current check | Why it matters |
| --- | --- | --- |
| [GitHub API versions](https://docs.github.com/en/rest/about-the-rest-api/api-versions) | GitHub lists `2026-03-10` and `2022-11-28` as supported REST API versions; unversioned requests default to `2022-11-28`; unsupported versions return `410 Gone`. | Parallax runs must record the request header version, the unversioned default, and the docs/source snapshot date separately. |
| [GitHub Deployments API](https://docs.github.com/en/rest/deployments/deployments?apiVersion=2022-11-28) | The docs page is labeled `API Version: 2022-11-28`, while current examples render `X-GitHub-Api-Version: 2026-03-10`. Deployments record `ref`, `sha`, `task`, `payload`, `environment`, transient/production flags, creator, status URL, and repository URL; creating a deployment can require successful commit statuses unless explicitly bypassed. | Parallax must not collapse docs page version, request header, and source date into one field. Deployment objects are requested/deployed-ref evidence, not runtime causality. |
| [GitHub Deployment Statuses API](https://docs.github.com/en/rest/deployments/statuses?apiVersion=2022-11-28) | Status states include `error`, `failure`, `inactive`, `in_progress`, `queued`, `pending`, and `success`; `log_url` is preferred over legacy `target_url`; `environment`, `environment_url`, and `auto_inactive` affect interpretation. | A deployment without a terminal status, environment, and log/status refs is incomplete evidence. |
| [GitHub deployment webhooks](https://docs.github.com/en/webhooks/webhook-events-and-payloads#deployment_status) | `deployment_status` webhooks require deployment read permission and are not fired for statuses with state `inactive`; `deployment` webhooks cover deployment creation. | Webhooks are useful low-latency evidence, but API backfill remains mandatory for inactive transitions and missed delivery. |
| [GitHub deployment review webhooks](https://docs.github.com/en/webhooks/webhook-events-and-payloads#deployment_review) | `deployment_review` webhooks represent approval activity and can include reviewer, comment, deployment callback, workflow run, and workflow job run context. | Approval/protection context is separate from deployment status. Agent-visible "who allowed this deploy?" claims need review/gate rows, not only success status rows. |
| [GitHub Actions variables](https://docs.github.com/actions/reference/workflows-and-actions/variables) | `GITHUB_SHA` exists, but its value depends on the workflow event that triggered the run. | CI/deploy ingestion must store event type, ref, and head/base context instead of treating `GITHUB_SHA` alone as deployed truth. |
| [GitHub events that trigger workflows](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows) | GitHub's event matrix gives event-specific `GITHUB_SHA` and `GITHUB_REF` semantics: PR runs use merge-branch SHA/ref, workflow-run triggers use the default branch SHA/ref, deployment events use deployed commit/ref, and inactive deployment statuses do not trigger workflows. | Any Actions-derived deploy/change edge must name the event and preserve the interpreted SHA role. A default-branch `workflow_run` row is not evidence that the upstream PR head or deployed SHA passed. |
| [GitHub workflow runs API](https://docs.github.com/en/rest/actions/workflow-runs?apiVersion=2022-11-28) | Workflow-run listing can filter by event, branch, check suite, and head SHA, and response rows include run ID, attempt, event, head branch/SHA, PR refs, URLs for jobs/logs/check suite/artifacts, actor, triggering actor, and head commit. | Parallax needs durable workflow-run rows before it can turn check status into a change-candidate edge. |
| [GitHub workflow jobs API](https://docs.github.com/en/rest/actions/workflow-jobs?apiVersion=2022-11-28) | Workflow jobs are fetched by run ID, can include latest or all attempts, and expose job ID, run ID, head SHA, status/conclusion, step timing/status, check-run URL, runner labels, workflow name, and head branch. | Job rows can prove a deploy/test step ran in a workflow attempt, but they inherit event/SHA meaning from the parent workflow run. |
| [GitHub check runs API](https://docs.github.com/en/rest/checks/runs?apiVersion=2022-11-28) | Check runs are tied to a `head_sha`, but GitHub documents that Checks API PR association only detects pushes in the repository where the run/suite was created; fork-originated pushes can return an empty `pull_requests` array. | Check success can validate a SHA, not an inferred PR/deploy, unless workflow event and PR head/base rows remove the ambiguity. |
| [GitHub PR files API](https://docs.github.com/en/rest/pulls/pulls?apiVersion=2022-11-28#list-pull-requests-files) | PR files are paginated and capped at 3000 files; PR commits endpoint caps at 250 commits before needing the commits endpoint. | File-touch and PR-contains-commit edges must include completeness flags and downgrade broad changes. |
| [GitHub compare commits API](https://docs.github.com/en/rest/commits/commits?apiVersion=2022-11-28#compare-two-commits) | Compare supports refs/SHAs and returns commits chronologically with file details, but unpaged responses are limited and large comparisons require pagination. | Release-to-release change windows need exact base/head SHAs and pagination metadata. |
| [Sentry release management CLI](https://docs.sentry.io/cli/releases/) | Sentry CLI can create/finalize releases, set commits automatically or manually, use exact full commit SHAs, configure previous/current commit ranges, and create deploys with at least an environment. | Sentry migration data is useful only when release, commit, dist, and environment semantics are preserved. |
| [Sentry Create a Deploy API](https://docs.sentry.io/api/releases/create-a-deploy/) | Sentry deploy creation requires an environment and may include name, URL, started/finished timestamps, and project slugs. | Sentry deploys can seed Parallax deploy nodes, but missing timestamps/projects should be explicit gaps. |
| [Linear Releases](https://linear.app/docs/releases) | Linear says issue `Done` is not equivalent to delivered; releases connect CI/CD, commit SHA, issues, pipelines, environments, and path filters. Business/Enterprise availability is called out. | Work-item delivery state must be separate from issue status and must carry plan/tier/source limits. |
| [Jira deployments API](https://developer.atlassian.com/cloud/jira/software/rest/api-group-deployments/) | Jira deployment rows include deployment and update sequence numbers, issue keys/associations, pipeline/environment identifiers, state, provider metadata, and accepted/rejected/unknown issue-key responses. | Jira is eventually consistent provider evidence; accepted/rejected/unknown issue keys must affect edge strength. |
| [OpenTelemetry CICD semantic conventions](https://opentelemetry.io/docs/specs/semconv/cicd/) | CICD semantic conventions are development-stage and define spans, logs, and metrics for CI/CD systems. | OTel CICD spans can feed Parallax rows but cannot be the durable schema by themselves. |
| [OpenTelemetry resource conventions](https://opentelemetry.io/docs/specs/semconv/resource/) | Version attributes such as `service.version` are stable and may be semantic versions, git hashes, or arbitrary build strings; deployment environment attributes are present under environment conventions. | Runtime telemetry release/version fields should join deploy/change rows, but arbitrary strings need normalization and missing-commit handling. |

### Claim Levels

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No provider ingestion or bundle audit run exists. | "Deploy/change context is planned." |
| `release_marker_ingest` | Release/version markers from runtime telemetry or Sentry/GitHub sources ingest and normalize with source refs. | "Release markers ingested for the tested sources." |
| `deploy_status_ingest` | Deployment and deployment-status events ingest with environment, terminal state, actor/source refs, and log/status URLs when present. | "Deployment status context for the tested providers." |
| `deployment_gate_ingest` | Deployment review/protection events ingest with reviewer/workflow context and missing-gate rows where configured but absent. | "Deployment approval/protection context for the tested providers." |
| `commit_window_reconstructed` | Release/deploy rows carry exact head and predecessor/base SHAs, and compare/PR backfill produces complete-or-flagged commit/file rows. | "Code-change windows reconstructed for tested releases." |
| `work_item_links_ingested` | GitHub/Linear/Jira work-item links ingest with machine-vs-text link strength and redacted text refs. | "Work-item links attached as context for tested providers." |
| `edge_strength_audited` | Strong/medium/weak/inferred deploy/change edges are audited against raw provider refs, completeness flags, and missing-evidence rows. | "Deploy/change evidence strengths audited for tested anchors." |
| `release_regression_context` | Release, deploy, commit-window, PR/file, workflow-run/job, CI/check, and missing-evidence rates meet thresholds for production error anchors. | "Release-regression context for the tested services and providers." |
| `projection_safe` | Agent-visible bundle projections include edge strength, missing-evidence blockers, redaction reports, source-field policy status, and raw-ref denial. | "Deploy/change projections are safe for the tested subset." |
| `what_changed_context` | The bundle can answer likely change candidates with cited release/deploy/code/workflow/check/work-item edges, explicit missing-evidence blockers, and no raw text/log dereference. | "Evidence-backed 'what changed?' context for the tested subset." |
| `claim_expired` | Provider API behavior, source version/header, schema, edge rules, redaction/source-field/projection policy, or freshness window changed. | "Deploy/change context result expired; rerun required." |
| `claim_failed` | A required gate fails for the advertised level. | No claim for the affected provider/surface/edge. |

Initial Parallax level: `not_measured`.

### Result Artifacts

Deploy/change runs should be durable and diffable:

```text
docs/research/deploy-change-context-results.md
docs/research/deploy-change-context-runs/<run_id>/manifest.json
docs/research/deploy-change-context-runs/<run_id>/raw-provider-events/<provider>/<event_id>.json
docs/research/deploy-change-context-runs/<run_id>/release-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/deploy-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/deployment-gate-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/code-change-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/workflow-run-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/workflow-job-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/ci-check-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/work-item-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/edge-audit-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/missing-evidence-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/raw-ref-manifest.jsonl
docs/research/deploy-change-context-runs/<run_id>/redaction-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/source-field-policy-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/bundle-projection-results.jsonl
docs/research/deploy-change-context-runs/<run_id>/claim-ledger.jsonl
docs/research/deploy-change-context-runs/<run_id>/hashes.sha256
```

Do not create run directories for hypothetical data. Add them only when a real
fixture or pilot run exists.

### Run Manifest

Each `manifest.json` should include:

```json
{
  "run_id": "deploy-change-context-YYYYMMDD-N",
  "research_date": "YYYY-MM-DD",
  "parallax_commit": "<git-sha>",
  "bundle_schema_version": "parallax-bundle-vN",
  "edge_rules_version": "deploy-change-edges-vN",
  "redaction_policy_version": "a6-default-deny-vN",
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "raw_ref_policy": "provider_text_logs_notes_ref_only_by_default",
  "source_snapshot": {
    "github_rest_request_header": "2026-03-10",
    "github_rest_docs_api_version": "2022-11-28",
    "github_rest_unversioned_default": "2022-11-28",
    "github_rest_supported_versions_checked": ["2026-03-10", "2022-11-28"],
    "github_deployment_status_webhook_inactive_gap": true,
    "github_deployment_review_webhook_checked": "YYYY-MM-DD",
    "github_actions_event_sha_semantics_checked": "YYYY-MM-DD",
    "github_workflow_runs_api_checked": "YYYY-MM-DD",
    "github_workflow_jobs_api_checked": "YYYY-MM-DD",
    "github_check_runs_fork_pr_gap_checked": "YYYY-MM-DD",
    "sentry_api": "current-docs-YYYY-MM-DD",
    "linear_releases": "current-docs-YYYY-MM-DD",
    "jira_deployments": "current-docs-YYYY-MM-DD",
    "otel_semconv": "1.41.0",
    "otel_cicd_status": "development"
  },
  "providers": ["github", "sentry", "linear", "jira", "otel-cicd"],
  "anchor_types": ["issue", "error_event", "trace"],
  "environments": ["production", "staging"],
  "backfill_policy": "webhook_plus_api_backfill_for_missing_inactive_and_review_states",
  "projection_formats": ["json", "markdown"],
  "notes": []
}
```

The manifest must separate provider API/header/date, Parallax schema versions,
edge-rule versions, and redaction policy. A pass for one provider/environment
does not carry over to another.

### Row Schemas

#### Release Result Row

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

#### Deploy Result Row

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
  "webhook_delivery_id_present": true,
  "api_backfill_complete": true,
  "inactive_status_backfilled": true,
  "auto_inactive_observed": false,
  "deployment_review_required": true,
  "deployment_review_state": "approved|rejected|not_required|missing",
  "missing_fields": []
}
```

#### Deployment Gate Result Row

```json
{
  "deployment_id": "github:deployment:42",
  "provider": "github",
  "gate_id": "github:deployment_review:11",
  "gate_kind": "deployment_review|environment_protection|manual_approval|unknown",
  "state": "approved|rejected|not_required|missing",
  "reviewer_ref_present": true,
  "workflow_run_ref_present": true,
  "workflow_job_run_ref_present": true,
  "comment_mode": "ref_only|redacted_summary|denied",
  "source_ref": "github:deployment_review:11",
  "webhook_delivery_id_present": true,
  "api_backfill_complete": true,
  "missing_fields": []
}
```

#### Code Change Result Row

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

#### Workflow Run Result Row

```json
{
  "workflow_run_id": "github:actions_run:30433642",
  "provider": "github",
  "repo": "github.com/acme/checkout",
  "workflow_name": "deploy",
  "run_attempt": 1,
  "event_name": "pull_request|push|workflow_run|deployment|deployment_status|workflow_dispatch",
  "github_ref": "refs/pull/456/merge",
  "github_ref_role": "pr_merge_ref|branch|tag|deployment_ref|default_branch|unknown",
  "github_sha": "merge-sha...",
  "github_sha_role": "pr_merge_commit|head_commit|deployment_commit|default_branch_commit|unknown",
  "head_branch": "feature/checkout-rules",
  "head_sha": "9d1f...",
  "base_branch": "main",
  "base_sha": "7a2b...",
  "pull_request_refs": ["https://github.com/acme/checkout/pull/456"],
  "check_suite_id": "github:check_suite:42",
  "jobs_ref_present": true,
  "logs_ref_present": true,
  "source_ref": "github:actions_run:30433642",
  "event_context_complete": true,
  "missing_fields": []
}
```

#### Workflow Job Result Row

```json
{
  "workflow_job_id": "github:actions_job:399444496",
  "provider": "github",
  "workflow_run_id": "github:actions_run:30433642",
  "run_attempt": 1,
  "job_name": "deploy",
  "head_sha": "9d1f...",
  "status": "completed",
  "conclusion": "success",
  "started_at": "2026-05-25T14:58:00Z",
  "completed_at": "2026-05-25T15:01:44Z",
  "check_run_ref": "github:check_run:399444496",
  "runner_ref_present": true,
  "step_summary_ref_present": true,
  "log_ref_present": true,
  "inherits_event_context_from_run": true,
  "missing_fields": []
}
```

#### CI/Check Result Row

```json
{
  "check_id": "github:check_run:99",
  "provider": "github",
  "head_sha": "9d1f...",
  "head_sha_role": "pr_head_commit|pr_merge_commit|push_commit|deployment_commit|unknown",
  "check_suite_id": "github:check_suite:42",
  "workflow_run_id": "github:actions_run:30433642",
  "workflow_job_id": "github:actions_job:399444496",
  "event_name": "push",
  "ref": "refs/heads/main",
  "pull_request_refs": [],
  "pr_linkage_complete": true,
  "status": "completed",
  "conclusion": "success",
  "workflow": "deploy",
  "log_ref_present": true,
  "event_context_complete": true,
  "deployed_truth": false
}
```

#### Work Item Result Row

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
  "raw_text_ref_count": 0,
  "agent_visible_text_mode": "metadata_only|redacted_summary|denied",
  "source_field_policy_status": "pass|fail|not_applicable",
  "unknown_or_rejected_links": []
}
```

#### Edge Audit Result Row

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
  "contradicting_fields": [],
  "causality_claim_made": false,
  "missing_evidence": [],
  "audit_status": "pass|fail"
}
```

#### Missing Evidence Result Row

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

#### Redaction Result Row

```json
{
  "anchor_id": "issue_123",
  "surface": "issue_title|issue_description|issue_comment|deploy_log|release_note|pr_body|provider_payload",
  "seeded_canaries": 12,
  "json_projection_leaks": 0,
  "markdown_projection_leaks": 0,
  "raw_ref_leaks": 0,
  "redaction_policy_version": "a6-default-deny-vN",
  "redaction_report_hash": "sha256:<hex>",
  "agent_visible_pass": true
}
```

#### Source Field Policy Result Row

```json
{
  "anchor_id": "issue_123",
  "provider": "github|sentry|linear|jira|otel-cicd",
  "source_kind": "direct_provider_api|synthetic_fixture|benchmark_fixture|corpus_fixture",
  "source_field_policy_status": "pass|fail|not_applicable",
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "source_field_policy_hash": "sha256:<hex>",
  "denied_zone_count": 0,
  "violation_count": 0,
  "not_applicable_reason": "direct provider API without mixed eval/corpus source rows"
}
```

#### Bundle Projection Result Row

```json
{
  "anchor_id": "issue_123",
  "projection_format": "json|markdown",
  "redaction_report_present": true,
  "source_field_policy_status": "pass|fail|not_applicable",
  "missing_evidence_present": true,
  "edge_strength_visible": true,
  "raw_issue_text_ref_count": 1,
  "raw_deploy_log_ref_count": 1,
  "raw_provider_payload_ref_count": 2,
  "raw_ref_dereferenced": false,
  "seeded_canary_leaks": 0,
  "causality_overclaim_count": 0,
  "agent_visible_pass": true
}
```

#### Claim Ledger Row

```json
{
  "run_id": "deploy-change-context-YYYYMMDD-N",
  "claim_level": "release_regression_context",
  "claim_status": "pass|fail|expired",
  "version_matrix": {
    "github_rest_request_header": "2026-03-10",
    "github_rest_docs_api_version": "2022-11-28",
    "github_rest_unversioned_default": "2022-11-28",
    "github_actions_event_sha_semantics_checked": "YYYY-MM-DD",
    "github_workflow_run_schema_checked": "YYYY-MM-DD",
    "github_workflow_job_schema_checked": "YYYY-MM-DD",
    "github_check_run_pr_linkage_checked": "YYYY-MM-DD",
    "otel_semconv": "1.41.0",
    "bundle_schema": "parallax-bundle-vN",
    "edge_rules": "deploy-change-edges-vN",
    "redaction_policy": "a6-default-deny-vN",
    "source_field_policy": "phase0-source-field-policy-vN",
    "projection_schema": "bundle-projection-vN"
  },
  "product_wording": "Release-regression context for the tested services and providers.",
  "required_caveats": ["linkage is not causality", "text-only work-item links are weak"],
  "expires_at": "YYYY-MM-DD"
}
```

### Counting Rules

- No "what changed?" claim without release/deploy/code/work-item missing-evidence
  rows visible in the bundle.
- No strong deploy edge unless exact environment, terminal deployment status,
  and deployed ref/SHA/release are present.
- No complete deployment-status claim from webhooks alone for GitHub. The run
  must record API backfill coverage for inactive statuses and missed delivery.
- No deployment approval/protection claim unless deployment-gate rows show the
  provider review/protection state or an explicit `not_required` result.
- No code-change touched-frame claim when PR file lists or compare results are
  truncated or incomplete.
- No release-regression claim when predecessor release/base SHA is missing.
- No work-item delivery claim from issue `Done` alone. Delivery requires release
  or deployment association.
- Machine-emitted links outrank text/magic-word links; text-only links are never
  strong without provider confirmation.
- CI/check success is validation evidence, not deployed runtime truth, unless
  linked to a deployment or release.
- No Actions-derived change candidate unless workflow-run rows record
  `event_name`, `github_sha_role`, `github_ref_role`, run attempt, and PR
  head/base/merge refs where applicable.
- No check-to-PR edge when the check run has an empty PR array or unknown
  head/base context; this is especially important for fork-originated changes.
- No workflow-job deploy-step claim unless the parent workflow-run row supplies
  complete event/SHA context and the job row is tied to the run attempt.
- Provider text fields, issue descriptions, comments, deploy logs, and release
  notes are untrusted and redacted/ref-only by default.
- Raw provider events, issue text, PR text, deploy logs, and release notes may
  be retained as raw refs for audit, but agent-visible projections must not
  dereference them by default.
- Synthetic, benchmark, or corpus-derived deploy/change runs require
  `source_field_policy_status: pass` before redaction or projection claims can
  pass. Direct provider API runs may use `not_applicable` only when no mixed
  eval/corpus source rows are present.
- OTel CICD spans are adapter input. Because the CICD conventions are
  development-stage, they cannot be the only proof for a durable Parallax schema
  claim.
- Strong deploy/change edges prove linkage, not root cause. Causality still
  needs runtime evidence, first-seen/spike analysis, touched code, recurrence,
  or contradiction checks.

### Refresh Triggers

Rerun the matrix and mark affected claims `claim_expired` when any of these
change:

- GitHub REST supported versions, unversioned default, request-header guidance,
  deployment/status API behavior, PR file/commit caps, compare pagination,
  Actions event/SHA semantics, workflow-run/job response fields, check-run PR
  association behavior, deployment-review behavior, inactive-status webhook
  behavior, or webhook payloads change;
- Sentry release/deploy API or CLI release semantics change;
- Linear release, GitHub integration, path-filter, or plan availability changes;
- Jira development/deployment information APIs, sequence semantics, or accepted
  and rejected response shapes change;
- OpenTelemetry semantic conventions, CICD conventions, resource version fields,
  or VCS fields change;
- Parallax bundle schema, deploy/change node shape, edge-strength rules,
  redaction policy, source-field policy, projection schema, or provider adapters
  change;
- a new provider is included in product wording;
- 60 days pass since the last run.

### Product Wording

Allowed after `deploy_status_ingest`:

> Deployment status context for the tested providers and environments.

Allowed after `deployment_gate_ingest`:

> Deployment approval/protection context for the tested providers and
> environments.

Allowed after `release_regression_context`:

> Release-regression context for the tested services and providers, with
> explicit missing-evidence and edge-strength reports.

Allowed after `what_changed_context`:

> Evidence-backed "what changed?" context for the tested subset, citing
> release, deploy, code-change, workflow-run/job, CI/check, and work-item
> records.

Avoid:

- "root cause from deploy metadata";
- "automatically finds the breaking commit";
- "knows what shipped" without environment-specific release/deploy evidence;
- "knows who approved the deploy" without deployment review/protection rows;
- "CI proved the deployed change" without workflow event/SHA role rows and
  release/deploy linkage;
- "issue Done means shipped";
- "PR touched the file, so it caused the issue";
- "full GitHub/Linear/Jira context" without provider, scope, and freshness
  caveats.
- "safe issue/deploy text" without redaction, source-field, and projection rows.

### Relationship To Other Research

- [Deploy, change, and issue-tracker context](deploy-change-context.md)
  defines ingestion and edge semantics this ledger turns into claim levels.
- [Evidence bundle and open schema](../architecture/evidence-bundle-schema.md) defines the
  release, deploy, deployment-status, code-change, work-item, check-run,
  redaction report, and source-field policy fields this ledger audits.
- [Correlation reliability on real telemetry gate](correlation.md)
  and [A4 correlation reliability ledger](correlation.md)
  measure whether these edges survive real telemetry.
- [Redaction pipeline and secret safety](redaction.md)
  and [A6 redaction red-team ledger](redaction.md) have veto
  power over issue text, deploy logs, release notes, and PR text in bundles.
- [Fixer component and outcome loop](../decisions/fixer-boundary.md)
  consumes code-change and outcome records after the evidence path is proven.

### Bottom Line

Release, deploy, code-change, workflow-run/job, CI/check, and work-item context
is mandatory for useful lifecycle reconstruction, but it is easy to overclaim.
Parallax should prove exact linkage and make missing context visible before it
lets an agent say what changed.
