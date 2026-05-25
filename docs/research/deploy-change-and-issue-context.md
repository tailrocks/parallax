# Deploy, Change, and Issue-Tracker Context

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

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
[Deploy/change context ledger](deploy-change-context-ledger.md), not inferred
from this design alone.

## Current Primary-Source Checks

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

## Boundary Decision

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

## Normalized Nodes

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

## Edge Strength Rules

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

## Missing Evidence Categories

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

## Ingestion Contract

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

## Privacy And Safety

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

## Implementation Order

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

## Proof Gate

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

## Relationship To Other Research

- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines the
  nodes, edges, source-field policy status, and redaction report fields this
  note tightens.
- [Deploy/change context ledger](deploy-change-context-ledger.md) turns provider
  ingestion, completeness, edge-strength, missing-evidence, and redaction runs
  into claim levels and allowed product wording.
- [Correlation reliability on real telemetry gate](correlation-reliability-real-telemetry-gate.md)
  already measures `release_context_rate` and `deploy_context_rate`; this note
  defines the ingestion and edge semantics behind those metrics.
- [Technical implementation concept](technical-implementation-concept.md)
  includes release/deploy context in the first useful loop.
- [Fixer component and outcome loop](fixer-component-and-outcome-loop.md) later
  consumes PR/commit/deploy outcome records, but does not own deploy mutation.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  has veto power over issue text and deploy logs before agent exposure.

## Bottom Line

Deploy/change context is essential because "what changed?" is usually the first
question after a production error. It becomes dangerous when treated as proof.
Parallax should ingest exact release, deployment, commit, PR, CI, and work-item
records; compute deterministic edges; and loudly mark missing or weak links.
Only then can an agent reason about likely regressions without hallucinating a
causal story from a timestamp.
