# Fixer Outcome Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

[Fixer component and outcome loop](fixer-component-and-outcome-loop.md) defines
the boundary: Parallax stores and serves evidence, while a separate fixer
component may use that evidence to drive a coding agent and create a patch or
pull request. This ledger defines the missing result contract for proving that
loop.

Current status: **not measured**. The repository has a boundary design and
outcome schema, but no dated result rows.

The central rule:

> No "Parallax fixes issues", "agent opens correct PRs", "accepted-fix feedback
> loop", "autonomous fix outcome learning", or "fixer business seam validated"
> claim without dated result rows linking evidence bundle -> fixer run -> agent
> session, with exact tool/version/config, adapter, capture surface, lossiness,
> redaction, source-field, projection, and raw-ref status -> provider agent task
> when used -> patch/branch/PR -> CI/checks -> review/merge/revert/recurrence
> -> human or policy verdict.

Parallax itself still does not fix. This ledger measures the separate fixer
component and the outcome records that flow back into Parallax.

## Current Primary-Source Checks

| Source | What it shows | Parallax implication |
| --- | --- | --- |
| [GitHub REST API docs](https://docs.github.com/rest/pulls/pulls) | GitHub's REST pages now identify API version `2026-03-10` as latest on the checked pages. | Fixer runs must record provider API version, not just provider name. |
| [GitHub pull request REST API](https://docs.github.com/rest/pulls/pulls) | GitHub exposes pull request creation, head/base refs, requested reviewers, diff/patch URLs, merge state, and related issue/status URLs. | PR creation is measurable plumbing, not a fix-success signal. Store branch, head SHA, PR URL, requested reviewers, and patch refs separately from outcome. |
| [GitHub pull request reviews API](https://docs.github.com/rest/pulls/reviews) | Pull request reviews are grouped review objects with states such as approved or changes requested, submitted timestamps, commit IDs, and author association. | Human review state must be a first-class outcome row. "Opened" and "approved" are different claims. |
| [GitHub Actions workflow runs API](https://docs.github.com/en/rest/actions/workflow-runs) and [check runs API](https://docs.github.com/v3/checks/runs) | Workflow/check rows include head SHA, status, conclusion, job/log/artifact URLs, pull request linkage, and rerun metadata. | CI evidence must be tied to the exact fixer head SHA and run attempt; stale, skipped, failed, timed-out, and rerun outcomes matter. |
| [GitHub webhooks](https://docs.github.com/en/webhooks/webhook-events-and-payloads) | GitHub emits events for pull requests, pull request reviews, check runs, check suites, workflow jobs, and workflow runs. | The production loop should consume webhook/event updates instead of relying only on polling or final snapshots. |
| [GitHub rulesets and required status checks](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets) | Required status checks can gate merges, and repositories can bind checks to expected sources. | Fixer success must respect repository policy. A PR that cannot satisfy required checks is not a successful fix even if the patch looks plausible. |
| [GitHub Agent Tasks REST API](https://docs.github.com/en/rest/agent-tasks/agent-tasks) | GitHub exposes public-preview endpoints to start and manage Copilot cloud-agent tasks. Task rows include states, session count, and pull-request artifacts; the start endpoint accepts a prompt, model, `create_pull_request`, and `base_ref`, with "Agent tasks" permissions. | If the fixer delegates to Copilot through GitHub, Parallax must record provider task id, task state, selected model, preview status, artifact PR refs, and permission mode. A provider task is still not a fix-success signal. |
| [GitHub Copilot coding agent](https://docs.github.com/en/copilot/concepts/about-copilot-coding-agent) and [Copilot PR review guidance](https://docs.github.com/copilot/how-tos/agents/copilot-coding-agent/reviewing-a-pull-request-created-by-copilot) | Copilot can work through the pull-request workflow. Current review guidance says Copilot PRs need thorough review, required approval may need another reviewer, and Actions workflows do not run automatically by default when Copilot pushes changes unless approved or configured. | Human governance and workflow approval are first-class outcome states. Parallax should model autonomy levels and never treat L3 draft PR creation or provider-task completion as L4/L5 completion. |
| [Sentry Seer issue-fix API](https://docs.sentry.io/api/seer/start-seer-issue-fix/) and [Seer product docs](https://docs.sentry.io/product/ai-in-sentry/seer) | Seer can run issue-fix workflows through root cause, solution, code changes, and open PR, using Sentry telemetry and connected code repositories. | "Issue -> code changes -> PR" is already incumbent behavior. The Parallax differentiator must be evidence citation, redaction, audit, and outcome feedback. |
| [Where Do AI Coding Agents Fail?](https://arxiv.org/abs/2601.15195) | A large-scale study of agent-authored PRs reports failures tied to task class, change size, CI, and review/alignment problems. | The ledger must stratify by failure class, patch size, touched files, CI status, and review outcome instead of reporting one aggregate PR-open metric. |
| [Collaborator or Assistant?](https://arxiv.org/abs/2605.08017) | A PR lifecycle study finds agents can initiate and carry PR work while merge governance remains predominantly human. | Outcome rows must separate operational agency from terminal approval authority. |
| [Why Are AI Agent Involved Fix PRs Unmerged?](https://arxiv.org/abs/2602.00164) and [Why Are Agentic Pull Requests Merged or Rejected?](https://arxiv.org/abs/2605.22534) | Recent PR studies focus on integration outcomes, unmerged fix PR blockers, review interactions, and why raw merge/rejection labels can mislead. | A useful corpus needs reviewer rationale, workflow blockers, recurrence, and inconclusive states, not just merged versus closed. |

## Claim Levels

Use exactly one current level in `claim-ledger.jsonl` for a given fixer result
scope.

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No run artifacts exist. Current status. | "Fixer outcome loop is designed but not run-proven." |
| `fixture_harness_ready` | Repeatable task matrix, fixture repos, redaction fixtures, expected checks, and scoring protocol exist. | "Fixer outcome fixture harness prepared." |
| `bundle_handoff_supported` | Fixer consumes immutable Parallax bundles with manifest, query report, redaction report, source-field policy, missing evidence, and raw-ref policy. | "Fixer bundle handoff works for the tested task set." |
| `fixer_run_record_supported` | Every run records status, policy, autonomy level, tool scopes, provider API version, optional provider task ID, agent session ID, and failure/no-op/timeout cases. | "Fixer runs are recorded for the tested task set." |
| `agent_session_linkage_pass` | Every run using an agent-session arm links to a dated agent-session ledger row with tool binary/version/config, adapter name/version, capture surface, source schema snapshot, lossiness, redaction, source-field policy, projection status, and raw-ref policy. | "Fixer runs link measured agent-session evidence for tested tasks." |
| `pr_creation_supported` | Separate fixer can create branches and draft PRs for allowed tasks with least-privilege repo policy. | "Separate fixer can open draft PRs for tested tasks." |
| `provider_agent_task_linkage_pass` | Copilot/GitHub Agent Tasks or similar provider-task runs are linked to task state, model, session count, artifacts, PR refs, and API preview/stability status. | "Provider agent-task linkage is recorded for tested tasks." |
| `source_field_projection_pass` | Eval/corpus-derived bundle projections preserve source-field policy status, redaction reports, missing-evidence flags, and raw-ref denial into fixer-visible inputs. | "Fixer-visible inputs preserve safety fields for tested tasks." |
| `evidence_citation_pass` | Every material diagnosis or PR-body claim cites bundle evidence refs or explicitly names missing evidence. | "Fixer output cites evidence for the tested tasks." |
| `ci_check_linkage_pass` | Required tests/checks are linked to exact head SHAs and conclusions, including skipped/failed/stale/rerun cases. | "Fixer PRs link CI/check evidence for the tested tasks." |
| `human_review_outcome_pass` | Human review state, edits, requested changes, close reasons, and verdict are recorded. | "Human review outcomes are recorded for tested fixer PRs." |
| `merge_revert_recurrence_tracking_pass` | Merge, revert, regression, recurrence window, and post-merge checks are recorded. | "Merged fixer PRs are followed through revert and recurrence windows." |
| `outcome_feedback_loop_pass` | Outcome rows feed A3 corpus rows and evidence-selection/agent-policy updates. | "Fixer outcomes feed the tested feedback loop." |
| `autonomy_level_1_supported` | The fixer can produce evidence-cited diagnosis or next checks, no patch. | "L1 fixer diagnosis is supported for tested tasks." |
| `autonomy_level_2_supported` | The fixer can produce a patch proposal or diff, no branch push. | "L2 patch proposals are supported for tested tasks." |
| `autonomy_level_3_supported` | The fixer can open draft PRs with passing required checks and evidence citations, no auto-merge. | "L3 draft PR workflow is supported for tested tasks." |
| `fixer_value_positive` | Against a registered baseline, fixer-bundle runs improve accepted/human-correct outcomes or reduce reviewer effort without worse leaks/regressions. | "Fixer value is positive for the tested failure class." |
| `claim_expired` | GitHub/Sentry/agent APIs, repo policy, schema, redaction policy, model, or fixer implementation changed, or 90 days passed during discovery. | "Fixer outcome result expired; rerun required." |
| `claim_failed` | Any advertised gate failed for the measured scope. | No claim for the affected task class or autonomy level. |

## Autonomy Levels

These match the fixer boundary doc, but the ledger records the measured level,
not the requested level.

| Level | Measured action | Success cannot mean |
| --- | --- | --- |
| `L0 observe` | Bundle and issue context are presented, no agent action. | Fix attempt. |
| `L1 diagnose` | Agent returns evidence-cited diagnosis or next checks. | Patch proposed or issue fixed. |
| `L2 propose_patch` | Agent returns patch/diff/plan, no branch push. | PR opened or reviewer accepted. |
| `L3 draft_pr` | Fixer creates branch/draft PR and requests review. | Fixed, merged, safe to deploy, or auto-approved. |
| `L4 auto_pr` | Fixer opens non-draft PR under explicit project policy. | Auto-merge or production deployment. |
| `L5 auto_merge_deploy` | Merge or deploy without human approval. | MVP-supported behavior. This remains rejected for the Parallax MVP. |

If a run requests L3 but policy downgrades it to L1 or L2, record both fields
and score the actual autonomy level only.

## Result Artifacts

Create these only when measurement begins:

```text
docs/research/fixer-outcome-results.md
docs/research/fixer-outcome-runs/<run_id>/manifest.json
docs/research/fixer-outcome-runs/<run_id>/task-matrix.jsonl
docs/research/fixer-outcome-runs/<run_id>/bundle-handoff-results.jsonl
docs/research/fixer-outcome-runs/<run_id>/fixer-run-results.jsonl
docs/research/fixer-outcome-runs/<run_id>/agent-session-linkage.jsonl
docs/research/fixer-outcome-runs/<run_id>/provider-agent-task-results.jsonl
docs/research/fixer-outcome-runs/<run_id>/patch-pr-results.jsonl
docs/research/fixer-outcome-runs/<run_id>/ci-check-results.jsonl
docs/research/fixer-outcome-runs/<run_id>/review-outcome-results.jsonl
docs/research/fixer-outcome-runs/<run_id>/merge-revert-recurrence-results.jsonl
docs/research/fixer-outcome-runs/<run_id>/evidence-citation-results.jsonl
docs/research/fixer-outcome-runs/<run_id>/source-field-projection-results.jsonl
docs/research/fixer-outcome-runs/<run_id>/policy-safety-results.jsonl
docs/research/fixer-outcome-runs/<run_id>/claim-ledger.jsonl
docs/research/fixer-outcome-runs/<run_id>/hashes.sha256
```

Raw repository checkouts, private code, raw model transcripts, private PR review
comments, CI logs with secrets, and runtime payloads stay outside the repo as
raw refs unless the operator explicitly approves redacted fixtures.

## Run Manifest

```json
{
  "run_id": "fixer-outcome-YYYYMMDD-N",
  "research_date": "YYYY-MM-DD",
  "parallax_context_commit": "<git-sha>",
  "fixer_component_commit": "<git-sha>",
  "agent_product": "codex|claude_code|copilot|openhands|amp|opencode|custom",
  "agent_version": "unknown",
  "repo_provider": "github|gitlab|local_fixture|other",
  "provider_api_versions": {
    "github_rest": "2026-03-10|not_used",
    "github_agent_tasks": "2026-03-10_public_preview|not_used"
  },
  "provider_agent_task_used": false,
  "source_repo_commit": "<git-sha>",
  "evidence_bundle_schema_version": "bundle-v0",
  "agent_session_schema_version": "agent-session-v0",
  "redaction_policy_version": "a6-default-deny-vN",
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "projection_schema_version": "fixer-projection-vN",
  "raw_ref_policy": "deny_dereference_by_default",
  "autonomy_level_requested": "L1|L2|L3|L4",
  "autonomy_level_max_allowed": "L1|L2|L3",
  "task_count": 0,
  "comparison_arms": [
    "repo_only",
    "raw_telemetry_dump",
    "parallax_bundle",
    "parallax_bundle_plus_session_trace"
  ],
  "outcome_window_days": 7,
  "notes": []
}
```

## Row Schemas

### Task Matrix Row

```json
{
  "task_id": "fix_task_001",
  "failure_class": "backend_error|frontend_error|ci_failure|cli_failure|agent_regression|deploy_regression|database_evidence_needed",
  "known_fix_available": true,
  "gold_patch_hash": "sha256:<hex>|none|private",
  "repo_commit": "<git-sha>",
  "bundle_id": "bndl_001",
  "expected_autonomy_level": "L1|L2|L3",
  "expected_checks": ["unit", "integration", "lint"],
  "risk_class": "low|medium|high",
  "max_files_changed": 5,
  "max_lines_changed": 250
}
```

### Bundle Handoff Row

```json
{
  "bundle_id": "bndl_001",
  "task_id": "fix_task_001",
  "bundle_schema_version": "bundle-v0",
  "query_manifest_present": true,
  "redaction_report_present": true,
  "source_field_policy_status": "pass|fail|not_applicable",
  "source_field_policy_hash": "sha256:<hex>|null",
  "missing_evidence_present": true,
  "raw_ref_count": 3,
  "raw_ref_policy": "deny_dereference_by_default",
  "raw_ref_dereferenced": false,
  "agent_visible_leak_count": 0,
  "allowed_raw_refs": [],
  "projection_equivalence_hash": "sha256:<hex>",
  "handoff_to_agent_success": true,
  "failure_reason": null
}
```

### Fixer Run Row

```json
{
  "fixer_run_id": "fixrun_001",
  "task_id": "fix_task_001",
  "bundle_id": "bndl_001",
  "agent_session_id": "ags_001",
  "provider_api_version": "github_rest:2026-03-10|not_used",
  "provider_agent_task_id": "github-agent-task-uuid|null",
  "provider_agent_task_api_status": "public_preview|stable|not_used",
  "requested_autonomy_level": "L3",
  "actual_autonomy_level": "L2",
  "policy_decision": "allow|downgrade|deny|human_required",
  "tool_scopes": ["repo_read", "branch_write", "test_run"],
  "started_at": "2026-05-25T00:00:00Z",
  "ended_at": "2026-05-25T00:10:00Z",
  "status": "success|failed|timeout|no_op|policy_denied",
  "failure_reason": null
}
```

### Provider Agent Task Row

```json
{
  "fixer_run_id": "fixrun_001",
  "provider": "github_agent_tasks|none",
  "provider_api_version": "2026-03-10",
  "api_status": "public_preview",
  "provider_agent_task_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "task_state": "queued|in_progress|completed|failed|idle|waiting_for_user|timed_out|cancelled|not_used",
  "session_count": 1,
  "model_requested": "gpt-5.3-codex|claude-sonnet-4.6|unknown",
  "create_pull_request_requested": true,
  "base_ref": "main",
  "artifact_provider": "github|null",
  "artifact_type": "pull|null",
  "artifact_pr_number": 456,
  "permission_mode": "agent_tasks_read_write|custom|unknown",
  "result": "pass|fail|not_applicable"
}
```

### Agent Session Linkage Row

```json
{
  "agent_session_id": "ags_001",
  "fixer_run_id": "fixrun_001",
  "tool": "codex|claude_code|copilot|openhands|amp|opencode|custom",
  "tool_binary_ref": "/path/to/tool|provider_task|unknown",
  "tool_version": "unknown",
  "tool_version_probe_output_ref": "raw://version-probe-or-provider-snapshot",
  "adapter_name": "parallax-codex-hooks",
  "adapter_version": "0.1.0",
  "capture_surface": "otel|hooks|plugin|run_json|stream_json|json_export|jsonl|server_api|acp|wrapper|provider_task_ref|raw_ref|unknown",
  "source_schema_snapshot": "docs-checked-YYYY-MM-DD|provider-api-YYYY-MM-DD|unknown",
  "source_config_ref": "raw://agent-config-snapshot",
  "adapter_claim_level": "codex_hooks_supported|codex_exec_json_supported|claude_otel_ingest_supported|claude_stream_json_supported|amp_plugin_supported|amp_stream_json_supported|opencode_run_json_supported|opencode_export_supported|opencode_plugin_supported|opencode_acp_supported|provider_task_link_only|unknown",
  "command_count": 4,
  "file_edit_count": 2,
  "test_count": 1,
  "agent_session_ledger_run_id": "agent-session-YYYYMMDD-N",
  "lossiness_report_present": true,
  "redaction_report_present": true,
  "source_field_policy_status": "pass|fail|not_applicable",
  "projection_safe": true,
  "raw_ref_policy": "deny_dereference_by_default",
  "raw_ref_dereferenced": false,
  "outcome_linked": true,
  "linkage_pass": true
}
```

### Patch And PR Row

```json
{
  "fixer_run_id": "fixrun_001",
  "provider": "github",
  "branch": "parallax/fix-task-001",
  "base_sha": "<base-sha>",
  "commit_sha": "<head-sha>",
  "provider_agent_task_id": "github-agent-task-uuid|null",
  "patch_hash": "sha256:<hex>",
  "pr_number": 456,
  "pr_url": "https://github.com/acme/repo/pull/456",
  "opened": true,
  "draft": true,
  "requested_reviewers": ["human_or_team_slug"],
  "maintainer_can_modify": true,
  "files_changed": 2,
  "lines_added": 34,
  "lines_deleted": 6,
  "evidence_refs_in_body": 7,
  "issue_linked": true,
  "pr_creation_error": null,
  "pass": true
}
```

### CI And Check Row

```json
{
  "fixer_run_id": "fixrun_001",
  "provider": "github_actions",
  "head_sha": "<head-sha>",
  "check_suite_id": 123,
  "workflow_run_id": 456,
  "run_attempt": 1,
  "required_checks_total": 3,
  "required_checks_passed": 3,
  "required_check_expected_source_match": true,
  "copilot_workflow_human_approved": "true|false|not_applicable",
  "workflow_auto_run_policy": "human_approval_required|auto_run_enabled|not_applicable",
  "failed_checks": [],
  "skipped_checks": [],
  "stale_checks": [],
  "flaky_rerun_count": 0,
  "status": "completed",
  "conclusion": "success",
  "logs_redacted": true
}
```

### Review Outcome Row

```json
{
  "fixer_run_id": "fixrun_001",
  "pr_number": 456,
  "reviewer_type": "human|copilot_review|external_bot|none",
  "human_reviewed": true,
  "review_decision": "approved|changes_requested|commented|closed_without_review|pending",
  "required_human_approval_satisfied": true,
  "agent_author_approval_counted": false,
  "requested_changes_count": 0,
  "accepted_without_edit": false,
  "edited_before_merge": true,
  "closed_unmerged": false,
  "label_outcome": "accepted|edited|rejected|needs_human|unsafe|inconclusive",
  "human_verdict": "correct|partially_correct|wrong|unsafe|unknown"
}
```

### Merge, Revert, And Recurrence Row

```json
{
  "fixer_run_id": "fixrun_001",
  "pr_number": 456,
  "merged": true,
  "merge_commit": "<merge-sha>",
  "reverted": false,
  "revert_commit": null,
  "time_to_merge_hours": 12.5,
  "post_merge_checks_passed": true,
  "recurrence_window_days": 7,
  "incident_recurred_within_window": false,
  "fix_regressed": false,
  "runtime_followup_ref": "raw://or_bundle_ref"
}
```

### Evidence Citation Row

```json
{
  "fixer_run_id": "fixrun_001",
  "output_ref": "pr_body|diagnosis|patch_summary",
  "material_claim_count": 8,
  "claims_with_evidence_refs": 8,
  "unsupported_claim_count": 0,
  "wrong_evidence_ref_count": 0,
  "query_manifest_cited": true,
  "missing_evidence_cited": true,
  "source_field_policy_cited": true,
  "pass": true
}
```

### Policy And Safety Row

```json
{
  "fixer_run_id": "fixrun_001",
  "production_mutation_attempted": false,
  "direct_push_to_protected_branch": false,
  "auto_merge_attempted": false,
  "raw_ref_access_requested": false,
  "raw_ref_access_granted": false,
  "raw_ref_dereferenced": false,
  "source_field_policy_status": "pass|fail|not_applicable",
  "provider_api_preview_used": false,
  "agent_visible_leak_count": 0,
  "scope_limit_violation": false,
  "policy_pass": true
}
```

### Claim Ledger Row

```json
{
  "claim_id": "fixer_claim_001",
  "research_date": "2026-05-25",
  "level": "not_measured",
  "valid_until": "2026-08-23",
  "task_scope": "none",
  "autonomy_level": "none",
  "supporting_artifacts": [],
  "contradictions": [],
  "allowed_wording": "Fixer outcome loop is designed but not run-proven.",
  "forbidden_wording": "Parallax fixes production bugs automatically.",
  "decision": "continue_research"
}
```

## Counting Rules

- Opened PR is never success by itself.
- Fix success requires human-correct or merged status, passing required checks,
  no policy/safety failure, and recurrence/revert follow-up for the configured
  window.
- Draft proposal, patch diff, draft PR, non-draft PR, merge, and deployment are
  separate autonomy levels.
- Any agent-visible secret leak fails the run regardless of PR quality.
- Eval/corpus-derived bundle handoff fails unless source-field policy status is
  `pass`; direct production telemetry may use `not_applicable` only with an
  explicit reason and no mixed source rows.
- Fixer-visible projections must not dereference raw refs by default. Raw rows,
  raw telemetry payloads, agent transcripts, CI logs, PR comments, and review
  notes stay refs unless a human-approved sensitive-read policy is recorded.
- Any missing critical evidence that is not disclosed in the diagnosis or PR
  body fails the evidence-citation gate.
- PRs generated from raw refs are not allowed unless policy and human approval
  explicitly grant that access.
- GitHub Agent Tasks or similar provider-task completion is never fix success by
  itself. It is an orchestration artifact that must link to PR, CI, review,
  merge/revert, and recurrence rows.
- Provider task `session_count` is not an agent-session trace. It supports
  `provider_agent_task_linkage_pass` only; a `parallax_bundle_plus_session_trace`
  arm requires `agent_session_linkage_pass` with a measured capture surface.
- A fixer run with `adapter_claim_level: unknown`, missing capture surface, or
  missing lossiness/redaction/projection rows can count as PR plumbing evidence
  only. It cannot support session-trace value, outcome-feedback-loop, or
  multi-agent tracing claims.
- Public-preview provider APIs can support fixture evidence only when the run
  records preview status, API version, and expiry. They cannot support broad
  stable-product wording.
- Human edits before merge are partial credit, not "agent fixed unaided".
- Reverted PRs, recurrence, or new linked regressions downgrade prior success.
- Skipped, stale, timed-out, or unrelated checks do not count as passing checks.
- Copilot-authored PR checks do not count as CI-passing unless workflow run
  approval or an explicit auto-run policy is recorded for the exact head SHA.
- Required human approval is not satisfied by the same agent or provider account
  that authored the PR.
- Outcome rows are append-only. Do not overwrite an opened PR row after review,
  merge, revert, or recurrence; append the new state.
- The business-model fixer seam cannot be validated from PR creation alone. It
  needs fixer outcome rows plus payment/budget signal rows in the
  [business model validation ledger](business-model-validation-ledger.md).
- No auto-merge belongs in early Parallax scope.

## Initial Results Template

When measurement begins, create `docs/research/fixer-outcome-results.md`:

```markdown
# Fixer Outcome Results

Research window:
Last updated:
Current claim level: not_measured

## Gate Snapshot

| Metric | Current | Threshold for L3 draft PR support | Status |
| --- | ---: | ---: | --- |
| Fixture tasks | 0 | >=10 | Pending |
| Bundle handoff pass rate | 0% | 100% | Pending |
| Agent-visible canary leaks | 0 | 0 | Pending |
| Source-field/projection pass rate | 0% | 100% | Pending |
| Agent session linkage pass rate | 0% | 100% when session-trace arm is used | Pending |
| Provider agent-task linkage rate | 0% | 100% when provider tasks are used | Pending |
| Evidence-citation pass rate | 0% | >=95% | Pending |
| Required-check linkage rate | 0% | 100% | Pending |
| Draft PR creation pass rate | 0% | >=80% for eligible tasks | Pending |
| Human review outcomes recorded | 0% | 100% | Pending |
| Merge/revert/recurrence outcomes recorded | 0% | 100% where applicable | Pending |
| Human-correct or accepted outcome lift | 0 | Positive versus baseline | Pending |

## Task Matrix

## Outcomes By Failure Class

## Evidence Citation

## CI And Review

## Reverts And Recurrence

## Current Allowed Wording

## Decision
```

## Product Wording

Allowed after `not_measured`:

> Fixer outcome loop is designed but not run-proven.

Allowed after `pr_creation_supported`:

> A separate fixer can open draft PRs for tested tasks.

Allowed after `provider_agent_task_linkage_pass`:

> Provider agent-task artifacts are linked to fixer outcomes for tested tasks.

Allowed after `agent_session_linkage_pass`:

> Fixer runs link measured agent-session evidence for tested tasks.

Allowed after `source_field_projection_pass`:

> Fixer-visible inputs preserve source-field, redaction, missing-evidence, and
> raw-ref policy fields for tested tasks.

Allowed after `human_review_outcome_pass`:

> Fixer PR review outcomes are recorded for tested tasks.

Allowed after `outcome_feedback_loop_pass`:

> Accepted, rejected, edited, reverted, and recurrent fixer outcomes feed the
> tested corpus loop.

Avoid:

- "Parallax fixes bugs";
- "autonomous production fix";
- "opened PR equals fixed";
- "self-healing";
- "agent PRs are correct";
- "Copilot task completed equals fixed";
- "GitHub Agent Tasks integration is stable" while the API is public preview;
- "validated fixer business seam";
- "auto-merge safe";
- "fixer learns from outcomes" before outcome rows alter a policy, retrieval,
  or scoring decision in a dated run.

## Refresh Triggers

Mark affected claims `claim_expired` when:

- GitHub, GitLab, Sentry, Copilot, OpenHands, Codex, Claude Code, Amp, or
  OpenCode changes the relevant PR/review/check/agent API surface materially;
- GitHub REST API version, Agent Tasks preview/stability status, or task state
  model changes;
- branch protection, rulesets, required checks, or repository permission policy
  changes;
- Parallax bundle schema, fixer outcome schema, agent-session schema,
  source-field policy, projection schema, raw-ref policy, or redaction policy
  changes;
- the fixer model, agent product, prompt, tool policy, or task matrix changes;
- 90 days pass since the last run during discovery;
- a prior accepted fixer PR is later reverted, linked to recurrence, or marked
  unsafe.

## Relationship To Other Research

- [Fixer component and outcome loop](fixer-component-and-outcome-loop.md)
  defines the boundary and outcome schema this ledger measures.
- [Evidence bundle and open schema specification](evidence-bundle-and-schema.md)
  defines `agent_opened_pr`, `validation_checked_patch`,
  `fix_addressed_issue`, and `fix_worsened_issue` edges consumed here.
- [Agent session tracing ledger](agent-session-tracing-ledger.md) measures the
  agent-session rows this ledger links to fixer runs.
- [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md)
  controls whether bundles improve fix quality versus raw context.
- [A3 schema adoption and corpus ledger](a3-schema-adoption-corpus-ledger.md)
  consumes accepted/rejected/reverted outcome rows as corpus events.
- [Business model validation ledger](business-model-validation-ledger.md)
  controls when fixer outcome evidence can become a value-capture claim.
- [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) has veto power
  before any fixer output becomes agent-visible.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  keeps the fixer in Phase 4 after A1/A2/A3/A6 gates.

## Bottom Line

The fixer loop is only defensible if it is measured past the PR-open event.
Parallax's role is to make the evidence handoff and outcome trail auditable:
bundle, agent session, patch, CI, review, merge, revert, recurrence, and human
verdict. Until those rows exist, the honest claim is design, not capability.
