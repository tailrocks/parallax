# Fixer Component and Outcome Loop

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The prompt is explicit that Parallax stores and serves evidence while a separate
component drives a coding agent and opens pull requests. The existing research
mentions this boundary, but it does not yet specify the contract, gates, or
outcome records that make the loop defensible.

This note defines that boundary:

> Parallax core must not become the fixer. The first product contract is an
> evidence-bundle and outcome-record contract. A separate fixer component may
> consume bundles, run a coding agent, draft a patch or pull request, and write
> session/outcome evidence back to Parallax.

The strategic implication is also blunt: opening PRs is no longer scarce. Sentry
Seer, GitHub Copilot cloud agent, and OpenHands already do it. The defensible
Parallax wedge is not "agent opens PR." It is "agent opens or proposes a PR with
portable evidence, redaction reports, validation logs, and outcome feedback that
improves future evidence selection."

## Current Primary-Source Checks

| Source | What it shows | Parallax implication |
| --- | --- | --- |
| [Sentry Seer issue-fix API](https://docs.sentry.io/api/seer/start-seer-issue-fix/) | Sentry exposes an asynchronous issue-fix endpoint that can identify root cause, propose a solution, generate code changes, and create a pull request; callers can choose stopping points from root cause through open PR. | The Sentry-like "issue -> fix PR" workflow already exists. Parallax must differentiate below that workflow with open evidence bundles, self-hosted data ownership, and outcome instrumentation. |
| [Sentry Seer GA changelog](https://sentry.io/changelog/seer-sentrys-ai-debugger-is-generally-available/) | Sentry says Seer uses issue context such as stack traces, environment details, spans, commits, beta logs, and profiling data, assigns actionability scores, and can open PRs automatically based on project settings. | Actionability scoring and automatic PRs are now incumbent features. Parallax should treat "should this issue be fixed by agent?" as a measured gate, not marketing copy. |
| [GitHub Copilot cloud-agent docs](https://docs.github.com/en/copilot/concepts/agents/cloud-agent/about-cloud-agent) and [Copilot review guidance](https://docs.github.com/en/copilot/how-tos/copilot-on-github/use-copilot-agents/review-copilot-output) | Copilot can operate through the pull-request workflow. Current review guidance says Copilot PRs deserve normal review, required approval can require another reviewer, and Actions workflows do not run automatically by default on Copilot pushes unless approved or configured. | PR creation, branch pushing, review routing, and workflow approval are platform primitives. Parallax should integrate with them, record human/workflow gates, and avoid treating provider completion as fix success. |
| [GitHub Agent Tasks REST API](https://docs.github.com/en/rest/agent-tasks/agent-tasks) | GitHub exposes public-preview endpoints to start and manage Copilot cloud-agent tasks with prompt, model, `create_pull_request`, `base_ref`, task states, session counts, and pull-request artifacts. | A fixer adapter may call provider agent-task APIs instead of owning every branch/PR primitive, but outcome records must preserve provider task id, API version, preview status, model, state, permission mode, and artifacts. |
| [GitHub Agentic Workflows assign-to-copilot](https://github.github.com/gh-aw/reference/assign-to-copilot/) | Workflow automation can assign Copilot to issues/PRs, route cross-repository PR creation, and requires fine-grained PAT permissions for assignment. | The fixer needs explicit repo/branch/permission policy. Cross-repo issue-to-code routing is real and must be captured in outcome records. |
| [GitHub Copilot for Jira preview](https://github.blog/changelog/2026-03-05-github-copilot-coding-agent-for-jira-is-now-in-public-preview/) | Jira issues can be assigned to Copilot; Copilot implements changes, opens draft PRs, posts updates, asks clarifying questions, and follows existing review/approval rules. | Issue tracker -> agent -> PR is becoming a standard integration flow. Parallax should store the evidence trail across tracker, repo, CI, agent, and runtime recurrence. |
| [OpenHands GitHub Action](https://docs.openhands.dev/openhands/usage/run-openhands/github-action) | OpenHands can be triggered from issues via label or mention, attempts resolution, opens a PR, and supports iterative feedback through comments and review threads. | Open-source fixer patterns exist. Parallax should not hard-code one fixer; it should define an adapter/event contract. |
| [Where Do AI Coding Agents Fail?](https://arxiv.org/abs/2601.15195) | A 2026 study of 33k agent-authored PRs reports that documentation/CI/build tasks merge more successfully, while performance and bug-fix tasks perform worst; failed PRs often touch more files, fail CI, or suffer review/alignment problems. | Parallax should gate autonomy by task class, evidence strength, patch size, touched files, and validation status. Bug-fix PRs are exactly where evidence matters most and success is hardest. |
| [Collaborator or Assistant?](https://arxiv.org/abs/2605.08017) | Analysis of 29,585 PR lifecycles finds collaborator tools initiate and carry PR work, but terminal merge authority remains overwhelmingly human. | Parallax should model human review/merge as first-class outcome evidence. Auto-merge is not part of the MVP. |
| [Why Are AI Agent Involved Fix PRs Unmerged?](https://arxiv.org/abs/2602.00164) | Current research focuses on fix-related agent PR integration outcomes, latency, and blockers. | The moat data is not just generated patches; it is accepted/rejected/reverted/slow/unmerged outcomes with evidence context. |

## Boundary Decision

| Component | Owns | Must not own |
| --- | --- | --- |
| Parallax core | Ingest, storage, redaction, grouping, correlation, evidence bundle, raw refs, query manifest, agent-session trace ingestion, fixer outcome records. | Repository checkout, patch generation, branch push, PR creation, merge, rollback, deploy, production mutation. |
| Access surface | Read-only CLI/API/MCP bundle retrieval, deterministic hypothesis checks, scoped raw-ref reads. | Generic shell, SQL, deploy, rollback, or write tools. |
| Fixer component | Repo checkout, coding-agent orchestration, patch proposal, branch/PR creation, test execution, validation summary, human-review handoff. | Bypassing Parallax redaction/bundle limits, mutating production, merging without human policy, hiding session traces. |
| Coding agent | Code reasoning, file edits, tests, patch generation, PR text. | Deciding evidence provenance or redaction policy; writing back outcome truth without validation. |
| GitHub/GitLab/etc. | Branches, commits, PRs/MRs, review, CI, merge metadata. | Being the only source of runtime failure context or fix-effect recurrence. |

The contract is intentionally asymmetric: the fixer depends on Parallax context,
but Parallax core does not depend on one fixer implementation.

## Fixer Request Contract

The first request from fixer to Parallax should be read-only:

```json
{
  "request_id": "fixreq_01J...",
  "project_id": "proj_checkout",
  "anchor": { "type": "issue", "id": "iss_8b21" },
  "intent": "diagnose|propose_patch|open_draft_pr",
  "repo": {
    "provider": "github",
    "url": "https://github.com/acme/checkout",
    "base_ref": "main",
    "head_policy": "new_branch_only",
    "provider_agent_task_policy": "disabled|allowed_preview|allowed_stable_only"
  },
  "limits": {
    "max_bundle_tokens": 30000,
    "max_files_changed": 5,
    "max_agent_minutes": 30,
    "require_tests": true
  },
  "safety": {
    "redaction_policy": "redact-v1",
    "source_field_policy": "phase0-source-field-policy-v1",
    "projection_raw_ref_dereference": "deny",
    "raw_access": "deny",
    "production_mutation": "deny"
  }
}
```

Parallax returns a normal evidence bundle plus a `fixer_context` block:

| Field | Meaning |
| --- | --- |
| `bundle_id` | Immutable context object consumed by the fixer. |
| `bundle_hash` | Canonical hash for replay and projection equivalence. |
| `source_field_policy_status` | Provenance/source-field status for eval, corpus, benchmark, or mixed-source bundles. |
| `raw_ref_policy` | Whether raw refs are denied, listed only, or human-readable under a sensitive-read approval. |
| `evidence_strength_summary` | Counts of strong/medium/weak/inferred edges. |
| `missing_evidence` | Explicit blockers to autonomy. |
| `allowed_raw_refs` | Usually empty for Phase 1/2; raw access is human-scoped. |
| `provider_task_policy` | Whether this request may start a provider agent task such as GitHub Copilot Agent Tasks. |
| `recommended_autonomy_level` | `diagnose_only`, `propose_patch`, or `draft_pr_allowed`. |
| `required_validation` | Tests, commands, or manual checks the fixer must run before reporting success. |

## Outcome Record Contract

The fixer writes back an append-only outcome record, not a vague "fixed" flag:

```json
{
  "outcome_id": "fixout_01J...",
  "bundle_id": "bndl_01J...",
  "fixer_run_id": "fixrun_01J...",
  "agent_session_id": "ags_01J...",
  "provider_agent_task": {
    "provider": "github_agent_tasks|none",
    "api_version": "2026-03-10",
    "api_status": "public_preview|stable|not_used",
    "task_id": null,
    "model": null,
    "state": "not_used",
    "create_pull_request": false
  },
  "repo": "github.com/acme/checkout",
  "base_ref": "main",
  "head_ref": "parallax/iss-8b21-empty-discount-rules",
  "pr_url": "https://github.com/acme/checkout/pull/456",
  "patch_summary": {
    "files_changed": 2,
    "lines_added": 34,
    "lines_deleted": 6,
    "touched_symbols": ["checkout::discount::apply"]
  },
  "validation": [
    {
      "command": "cargo test discount_empty_rules",
      "status": "passed",
      "log_ref": "raw://validation/123"
    }
  ],
  "human_review": {
    "status": "pending|approved|changes_requested|closed",
    "reviewer_refs": [],
    "required_human_approval_satisfied": false
  },
  "source_field_policy_status": "pass|fail|not_applicable",
  "raw_ref_dereferenced": false,
  "merge": {
    "status": "unmerged|merged|reverted",
    "merge_commit": null,
    "revert_commit": null
  },
  "runtime_followup": {
    "issue_recurred": "unknown|yes|no",
    "window": "7d",
    "evidence_ref": null
  },
  "classification": "diagnosis_only|proposal|draft_pr|accepted|edited|rejected|reverted|inconclusive",
  "notes": "Agent cited evt_panic and span_db_lookup; recurrence not yet known."
}
```

The record is append-only because outcome truth changes over time: a PR can be
opened, edited, merged, reverted, and later associated with recurrence or
non-recurrence. Parallax should model that sequence, not collapse it.

## Autonomy Levels

| Level | Allowed action | Required evidence |
| --- | --- | --- |
| `L0 observe` | Store issue, telemetry, bundle, and human investigation notes. | Any valid bundle. |
| `L1 diagnose` | Produce evidence-cited diagnosis and next checks. | Strong or medium evidence, or explicit inconclusive output. |
| `L2 propose_patch` | Produce patch diff or PR plan, no branch push. | A1 bundle-value gate positive for this failure class; redaction passed; tests identified. |
| `L3 draft_pr` | Create branch and draft PR, or start a provider task that creates a draft PR artifact, then request human review. | Same as L2 plus repo permission policy, validation commands, provider API policy if used, patch-size limits, and no missing critical evidence. |
| `L4 auto_pr` | Automatically open PR for high-actionability issues. | Later only; requires per-project setting, actionability threshold, passing validation, and rollback/revert tracking. |
| `L5 auto_merge/deploy` | Merge or deploy without human approval. | Out of scope for Parallax MVP and should remain rejected until a separate production-control safety program exists. |

This aligns with the empirical PR lifecycle evidence: agents can carry work, but
humans retain governance. Parallax should earn L3 before it even discusses L4.

## First Fixer Gate

The fixer should not ship before these gates pass:

| Gate | Pass condition |
| --- | --- |
| A1 bundle value | Bundles beat raw telemetry dumps for the target failure class, or the product claim is narrowed to audit/retention. |
| Redaction | The [redaction pipeline](redaction-pipeline-and-secret-safety.md) and [detector toolchain](redaction-detector-toolchain.md) pass on generated bundle, raw dump, validation-log, and agent-session fixtures. |
| Source-field and projection | Eval/corpus-derived bundles preserve source-field policy status, redaction reports, missing-evidence flags, and raw-ref denial into fixer-visible inputs and PR text. |
| Evidence citation | Every material claim in the diagnosis/PR body cites bundle evidence refs or says evidence is missing. |
| Patch limits | Draft PRs obey max files/lines/touched services and reject broad rewrites by default. |
| Validation | Required tests/builds are run or explicitly reported as unavailable; logs are stored as refs. |
| Repo permission | Fixer can create a branch and draft PR only; no direct push to protected branches, merge, deploy, or production mutation. |
| Provider-task linkage | If the fixer uses GitHub Agent Tasks, or an equivalent hosted agent API, the outcome records task id, API version, preview/stability status, task state, model, session count, artifacts, and permission mode. |
| Outcome writeback | Every run writes an agent session trace and outcome record, including failure, timeout, and no-op cases. |
| Human review | Draft PRs request a human reviewer and carry a clear "agent generated" marker. |
| Recurrence tracking | Merged fixes create a follow-up watch window before Parallax marks `fix_addressed_issue` as strong. |

If these fail, keep the fixer as an offline eval harness and continue exposing
read-only bundles through CLI/API/MCP.

## Schema Implications

The existing evidence schema already has `agent_opened_pr`,
`validation_checked_patch`, `fix_addressed_issue`, and `fix_worsened_issue`
edges. This note adds stricter semantics:

| Edge / field | Rule |
| --- | --- |
| `agent_opened_pr` | Created when fixer or agent creates a branch/PR. Not evidence that the issue is fixed. |
| `provider_agent_task_started` | Created when a hosted agent task is started. Not evidence that the issue is fixed or that a PR exists. |
| `validation_checked_patch` | Created for each test/build/lint/security command. Carries command ref, status, and redaction report. |
| `fix_addressed_issue` | Strong only after human acceptance or merge plus recurrence window evidence. Medium if tests pass but runtime follow-up is unknown. |
| `fix_worsened_issue` | Created on revert, linked regression, new issue caused by PR, failed rollout, or human classification. |
| `human_review.status` | Required field for PR outcomes; pending is a valid state. |
| `runtime_followup.issue_recurred` | Required after merge for issues where recurrence can be observed. |

This prevents the classic bad metric: counting opened PRs as successful fixes.

## Competitive Implications

1. **Sentry is the direct incumbent for issue-to-fix.** Seer already combines
   Sentry context, actionability scoring, code changes, and PR creation. Parallax
   cannot win by copying that UI flow alone.
2. **GitHub owns the PR workflow.** Copilot and GitHub Agent Tasks can carry
   work into PR artifacts under GitHub permissions and review rules. Parallax
   should produce the evidence artifact and outcome trail that improves these
   agents, not fight GitHub for branch mechanics.
3. **Open-source agent runners are adequate adapters.** OpenHands shows that
   issue-label and mention-driven resolution is already open-source-shaped. The
   Parallax adapter contract should allow OpenHands, Copilot, Codex, Claude Code,
   Amp, or a custom fixer to write the same outcome records.
4. **The moat is outcome feedback.** The durable asset is a corpus of bundles,
   patches, validation logs, human review, merge/revert/recurrence outcomes, and
   evidence links. Without that, the fixer is just another PR bot.

## Implementation Order

1. Keep Phase 0 focused on hand-built bundle evals, no product fixer.
2. In Phase 1, expose bundles through CLI/API and define the outcome record
   schema even if no fixer exists.
3. In Phase 2, build a local fixer harness for evals only: consume bundle, run
   agent, produce patch, write session/outcome record.
4. In Phase 3, allow draft PR creation through one repository provider with
   explicit least-privilege permissions.
5. In Phase 4, commercialize the fixer only if A1/A2/A3 and redaction gates hold.

Do not let fixer excitement pull frontend, MCP, database evidence, or Tier-3
storage forward before the tiny evidence engine proves value.

## Relationship To Other Research

- [Business model and economics](business-model-and-economics.md) identifies the
  fixer as a value-capture seam; this note defines the technical seam.
- [Business model validation ledger](business-model-validation-ledger.md) defines
  when fixer interest, paid pilots, or conversion rows are strong enough to call
  the fixer a validated value-capture seam.
- [Fixer outcome ledger](fixer-outcome-ledger.md) defines the dated result rows,
  claim levels, and wording rules required before opened PRs, accepted fixes,
  or fixer feedback loops become claimable.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines the
  bundle and audit edges consumed by the fixer.
- [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md) defines
  the session/action trace data that a fixer must emit back to Parallax.
- [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md)
  keeps the first Parallax MCP server read-only; fixer write/proposal tools are
  later and separate.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  keeps the fixer in Phase 4 after A1/A2/A3 and redaction gates.
- [Schema adoption and corpus moat gate](schema-adoption-and-corpus-moat-gate.md)
  depends on accepted/rejected/reverted outcome records to become real.

## Bottom Line

The fixer is strategically important but should stay outside Parallax core. PR
creation is already becoming a platform commodity. Parallax's defensible role is
to make each fixer run evidence-cited, redacted, auditable, measurable, and tied
to accepted/rejected/reverted runtime outcomes.
