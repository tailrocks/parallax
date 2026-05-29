# Fixer Boundary and Outcome Loop

> Parallax core must not become the fixer: it is the context engine that stores, redacts, groups, correlates, and serves evidence, while a separate fixer component consumes evidence bundles, drives a coding agent, drafts a patch or pull request, and writes session/outcome evidence back with measured adapter provenance. The first product contract is an evidence-bundle and append-only outcome-record contract, not "agent opens PR" — PR creation is now a platform commodity (Sentry Seer, GitHub Copilot cloud agent and Agent Tasks, OpenHands), so the defensible wedge is portable evidence, redaction reports, validation logs, and outcome feedback that improves future evidence selection. The boundary and schema are designed, but the outcome loop is currently **not measured**: there are no dated result rows linking evidence bundle -> fixer run -> agent session -> patch/PR -> CI -> review/merge/revert/recurrence. Autonomy is gated by level (L0 observe through L5 auto_merge/deploy), with L5 out of scope for the MVP and L3 draft-PR creation the first goal that must be earned; opened PRs and provider-task completion are never fix success by themselves. Until the gates pass and the result rows exist, the honest claim is design, not capability, and the fixer stays an offline eval harness while Parallax exposes read-only bundles through CLI/API/MCP.

This decision record consolidates the following previously-separate research files, each preserved in full below:

- `fixer-component-and-outcome-loop.md`
- `fixer-outcome-ledger.md`

## Fixer Component and Outcome Loop

_Provenance: merged verbatim from `fixer-component-and-outcome-loop.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

The prompt is explicit that Parallax stores and serves evidence while a separate
component drives a coding agent and opens pull requests. The existing research
mentions this boundary, but it does not yet specify the contract, gates, or
outcome records that make the loop defensible.

This note defines that boundary:

> Parallax core must not become the fixer. The first product contract is an
> evidence-bundle and outcome-record contract. A separate fixer component may
> consume bundles, run a coding agent, draft a patch or pull request, and write
> session/outcome evidence back to Parallax with measured adapter provenance.

The strategic implication is also blunt: opening PRs is no longer scarce. Sentry
Seer, GitHub Copilot cloud agent, and OpenHands already do it. The defensible
Parallax wedge is not "agent opens PR." It is "agent opens or proposes a PR with
portable evidence, redaction reports, validation logs, and outcome feedback that
improves future evidence selection."

### Current Primary-Source Checks

| Source | What it shows | Parallax implication |
| --- | --- | --- |
| [Sentry Seer issue-fix API](https://docs.sentry.io/api/seer/start-seer-issue-fix/) | Sentry exposes an asynchronous issue-fix endpoint that can identify root cause, propose a solution, generate code changes, and create a pull request; callers can choose stopping points from root cause through open PR. | The Sentry-like "issue -> fix PR" workflow already exists. Parallax must differentiate below that workflow with open evidence bundles, self-hosted data ownership, and outcome instrumentation. |
| [Sentry Seer GA changelog](https://sentry.io/changelog/seer-sentrys-ai-debugger-is-generally-available/) | Sentry says Seer uses issue context such as stack traces, environment details, spans, commits, beta logs, and profiling data, assigns actionability scores, and can open PRs automatically based on project settings. | Actionability scoring and automatic PRs are now incumbent features. Parallax should treat "should this issue be fixed by agent?" as a measured gate, not marketing copy. |
| [GitHub Copilot cloud-agent docs](https://docs.github.com/en/copilot/concepts/agents/cloud-agent/about-cloud-agent) and [Copilot review guidance](https://docs.github.com/en/copilot/how-tos/copilot-on-github/use-copilot-agents/review-copilot-output) | Copilot can operate through the pull-request workflow. Current review guidance says Copilot PRs deserve normal review, required approval can require another reviewer, and Actions workflows do not run automatically by default on Copilot pushes unless approved or configured. | PR creation, branch pushing, review routing, and workflow approval are platform primitives. Parallax should integrate with them, record human/workflow gates, and avoid treating provider completion as fix success. |
| [GitHub Agent Tasks REST API](https://docs.github.com/en/rest/agent-tasks/agent-tasks) | GitHub exposes public-preview endpoints to start and manage Copilot cloud-agent tasks with prompt, model, `create_pull_request`, `base_ref`, task states, session counts, and pull-request artifacts. | A fixer adapter may call provider agent-task APIs instead of owning every branch/PR primitive, but outcome records must preserve provider task id, API version, preview status, model, state, permission mode, and artifacts. |
| [GitHub Agentic Workflows assign-to-copilot](https://github.github.com/gh-aw/reference/assign-to-copilot/) | Workflow automation can assign Copilot to issues/PRs, route cross-repository PR creation, and requires fine-grained PAT permissions for assignment. | The fixer needs explicit repo/branch/permission policy. Cross-repo issue-to-code routing is real and must be captured in outcome records. |
| [GitHub Copilot for Jira preview](https://github.blog/changelog/2026-03-05-github-copilot-coding-agent-for-jira-is-now-in-public-preview/) | Jira issues can be assigned to Copilot; Copilot implements changes, opens draft PRs, posts updates, asks clarifying questions, and follows existing review/approval rules. | Issue tracker -> agent -> PR is becoming a standard integration flow. Parallax should store the evidence trail across tracker, repo, CI, agent, and runtime recurrence. |
| [OpenHands GitHub Action](https://docs.openhands.dev/openhands/usage/run-openhands/github-action) | OpenHands can be triggered from issues via label or mention, attempts resolution, opens a PR, and supports iterative feedback through comments and review threads. | Open-source fixer patterns exist. Parallax should not hard-code one fixer; it should define an adapter/event contract. |
| [MCP 2025-11-25 tools spec](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) and [RFC 8785 JCS](https://www.rfc-editor.org/rfc/rfc8785.html) | MCP tools can return `structuredContent` validated by `outputSchema`; JCS gives JSON a deterministic hashable representation. | A read-only fixer handoff must be a schema-bound, canonical, cross-surface projection rather than an ad hoc Markdown prompt. |
| [Where Do AI Coding Agents Fail?](https://arxiv.org/abs/2601.15195) | A 2026 study of 33k agent-authored PRs reports that documentation/CI/build tasks merge more successfully, while performance and bug-fix tasks perform worst; failed PRs often touch more files, fail CI, or suffer review/alignment problems. | Parallax should gate autonomy by task class, evidence strength, patch size, touched files, and validation status. Bug-fix PRs are exactly where evidence matters most and success is hardest. |
| [Collaborator or Assistant?](https://arxiv.org/abs/2605.08017) | Analysis of 29,585 PR lifecycles finds collaborator tools initiate and carry PR work, but terminal merge authority remains overwhelmingly human. | Parallax should model human review/merge as first-class outcome evidence. Auto-merge is not part of the MVP. |
| [Why Are AI Agent Involved Fix PRs Unmerged?](https://arxiv.org/abs/2602.00164) | Current research focuses on fix-related agent PR integration outcomes, latency, and blockers. | The moat data is not just generated patches; it is accepted/rejected/reverted/slow/unmerged outcomes with evidence context. |

### Boundary Decision

| Component | Owns | Must not own |
| --- | --- | --- |
| Parallax core | Ingest, storage, redaction, grouping, correlation, evidence bundle, raw refs, query manifest, measured agent-session trace ingestion, fixer outcome records. | Repository checkout, patch generation, branch push, PR creation, merge, rollback, deploy, production mutation. |
| Access surface | Read-only CLI/API/MCP bundle retrieval, deterministic hypothesis checks, scoped raw-ref reads. | Generic shell, SQL, deploy, rollback, or write tools. |
| Fixer component | Repo checkout, coding-agent orchestration, patch proposal, branch/PR creation, test execution, validation summary, human-review handoff. | Bypassing Parallax redaction/bundle limits, mutating production, merging without human policy, hiding session traces or adapter lossiness. |
| Coding agent | Code reasoning, file edits, tests, patch generation, PR text. | Deciding evidence provenance or redaction policy; writing back outcome truth without validation. |
| GitHub/GitLab/etc. | Branches, commits, PRs/MRs, review, CI, merge metadata. | Being the only source of runtime failure context or fix-effect recurrence. |

The contract is intentionally asymmetric: the fixer depends on Parallax context,
but Parallax core does not depend on one fixer implementation.

### Fixer Request Contract

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
| `schema_ref` | Schema URI plus schema hash for the bundle contract. |
| `canonical_bundle_hash` | JCS-style canonical hash for replay and projection equivalence. |
| `projection_manifest_hash` | Hash of the projection manifest used for Markdown, CLI, HTTP, and MCP views. |
| `mcp_output_schema_ref` | Output schema used when the bundle is served through MCP `structuredContent`. |
| `access` | Raw-access policy and expiry copied from the evidence bundle. |
| `source_field_policy_status` | Provenance/source-field status for eval, corpus, benchmark, or mixed-source bundles. |
| `raw_ref_policy` | Whether raw refs are denied, listed only, or human-readable under a sensitive-read approval. |
| `evidence_strength_summary` | Counts of strong/medium/weak/inferred edges. |
| `missing_evidence` | Explicit blockers to autonomy. |
| `allowed_raw_refs` | Usually empty for Phase 1/2; raw access is human-scoped. |
| `provider_task_policy` | Whether this request may start a provider agent task such as GitHub Copilot Agent Tasks. |
| `recommended_autonomy_level` | `diagnose_only`, `propose_patch`, or `draft_pr_allowed`. |
| `required_validation` | Tests, commands, or manual checks the fixer must run before reporting success. |

### Outcome Record Contract

The fixer writes back an append-only outcome record, not a vague "fixed" flag:

```json
{
  "outcome_id": "fixout_01J...",
  "bundle_id": "bndl_01J...",
  "fixer_run_id": "fixrun_01J...",
  "agent_session_id": "ags_01J...",
  "agent_session_linkage": {
    "adapter_name": "parallax-codex-hooks",
    "adapter_version": "0.1.0",
    "capture_surface": "hooks|otel|plugin|stream_json|jsonl|provider_task_ref|unknown",
    "adapter_claim_level": "codex_hooks_supported|claude_otel_ingest_supported|provider_task_link_only|unknown",
    "canonical_session_bundle_hash": "sha256:<hex>|not_applicable",
    "projection_manifest_hash": "sha256:<hex>|not_applicable",
    "projection_equivalence_pass": true,
    "mcp_structured_content_valid": true,
    "safety_fields_only_in_meta": false,
    "lossiness_report_ref": "agent-session-run/lossiness-results.jsonl#fixrun_01J",
    "redaction_report_ref": "agent-session-run/redaction-results.jsonl#fixrun_01J",
    "projection_safe": true
  },
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

### Autonomy Levels

| Level | Allowed action | Required evidence |
| --- | --- | --- |
| `L0 observe` | Store issue, telemetry, bundle, and human investigation notes. | Any valid bundle. |
| `L1 diagnose` | Produce evidence-cited diagnosis and next checks. | Strong or medium evidence, or explicit inconclusive output. |
| `L2 propose_patch` | Produce patch diff or PR plan, no branch push. | A1 bundle-value gate positive for this failure class; A6 redaction passed; canonical bundle/projection hashes present; tests identified. |
| `L3 draft_pr` | Create branch and draft PR, or start a provider task that creates a draft PR artifact, then request human review. | Same as L2 plus repo permission policy, validation commands, provider API policy if used, patch-size limits, exact head-SHA check linkage, and no missing critical evidence. |
| `L4 auto_pr` | Automatically open PR for high-actionability issues. | Later only; requires per-project setting, actionability threshold, passing validation, and rollback/revert tracking. |
| `L5 auto_merge/deploy` | Merge or deploy without human approval. | Out of scope for Parallax MVP and should remain rejected until a separate production-control safety program exists. |

This aligns with the empirical PR lifecycle evidence: agents can carry work, but
humans retain governance. Parallax should earn L3 before it even discusses L4.

### First Fixer Gate

The fixer should not ship before these gates pass:

| Gate | Pass condition |
| --- | --- |
| A1 bundle value | Bundles beat raw telemetry dumps for the target failure class, or the product claim is narrowed to audit/retention. |
| Redaction | The [redaction pipeline](redaction-pipeline-and-secret-safety.md) and [detector toolchain](redaction-detector-toolchain.md) pass on generated bundle, raw dump, validation-log, and agent-session fixtures. |
| Source-field and projection | Eval/corpus-derived bundles preserve source-field policy status, redaction reports, missing-evidence flags, raw-ref denial, canonical bundle hash, and projection manifest across bundle JSON, Markdown, CLI, HTTP, and MCP `structuredContent`; MCP output validates against `outputSchema`; safety fields are not only in `_meta` or PR prose. |
| Evidence citation | Every material claim in the diagnosis/PR body cites bundle evidence refs or says evidence is missing. |
| Patch limits | Draft PRs obey max files/lines/touched services and reject broad rewrites by default. |
| Validation | Required tests/builds are run or explicitly reported as unavailable; logs are stored as refs. |
| Repo permission | Fixer can create a branch and draft PR only; no direct push to protected branches, merge, deploy, or production mutation. |
| Provider-task linkage | If the fixer uses GitHub Agent Tasks, or an equivalent hosted agent API, the outcome records task id, API version, preview/stability status, task state, model, session count, artifacts, and permission mode. |
| Agent-session linkage | If the fixer claims a session-trace arm, the linked agent session records exact tool/version/config, adapter, capture surface, canonical session bundle hash, projection manifest hash, lossiness, redaction, projection, MCP structured-output status, and raw-ref status. Provider task `session_count` is not enough. |
| Outcome writeback | Every run writes an outcome record, including failure, timeout, and no-op cases. Session trace claims require the agent-session linkage gate above. |
| Human review | Draft PRs request a human reviewer and carry a clear "agent generated" marker. |
| Recurrence tracking | Merged fixes create a follow-up watch window before Parallax marks `fix_addressed_issue` as strong. |

If these fail, keep the fixer as an offline eval harness and continue exposing
read-only bundles through CLI/API/MCP.

### Schema Implications

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

### Competitive Implications

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

### Implementation Order

1. Keep Phase 0 focused on hand-built bundle evals, no product fixer.
2. In Phase 1, expose bundles through CLI/API and define the outcome record
   schema even if no fixer exists.
3. In Phase 2, build a local fixer harness for evals only: consume bundle, run
   agent, produce patch, write outcome record, and link measured agent-session
   evidence when the session-trace arm is enabled.
4. In Phase 3, allow draft PR creation through one repository provider with
   explicit least-privilege permissions.
5. In Phase 4, commercialize the fixer only if A1/A2/A3 and redaction gates hold.

Do not let fixer excitement pull frontend, MCP, database evidence, or Tier-3
storage forward before the tiny evidence engine proves value.

### Relationship To Other Research

- [Business model and economics](business-model-and-economics.md) identifies the
  fixer as a value-capture seam; this note defines the technical seam.
- [Business model validation ledger](business-model-validation-ledger.md) defines
  when fixer interest, paid pilots, or conversion rows are strong enough to call
  the fixer a validated value-capture seam.
- [Fixer outcome ledger](fixer-outcome-ledger.md) defines the dated result rows,
  claim levels, and wording rules required before opened PRs, fix outcomes, or
  fixer feedback loops become claimable.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines the
  bundle and audit edges consumed by the fixer.
- [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md) defines
  the session/action trace data that a fixer must emit back to Parallax.
- [Agent session tracing ledger](agent-session-tracing-ledger.md) defines the
  per-tool, per-capture-surface evidence required before a fixer run can claim
  to include measured agent-session tracing.
- [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md)
  keeps the first Parallax MCP server read-only; fixer write/proposal tools are
  later and separate.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  keeps the fixer in Phase 4 after A1/A2/A3 and redaction gates.
- [Schema adoption and corpus moat gate](schema-adoption-and-corpus-moat-gate.md)
  depends on accepted/rejected/reverted outcome records to become real.

### Bottom Line

The fixer is strategically important but should stay outside Parallax core. PR
creation is already becoming a platform commodity. Parallax's defensible role is
to make each fixer run evidence-cited, canonical, redacted, auditable,
measurable, and tied to accepted/rejected/reverted runtime outcomes.

## Fixer Outcome Ledger

_Provenance: merged verbatim from `fixer-outcome-ledger.md` (2026-05-29 restructure)._

_(Shared note — see the Fixer Component and Outcome Loop section above.)_

### Purpose

[Fixer component and outcome loop](fixer-component-and-outcome-loop.md) defines
the boundary: Parallax stores and serves evidence, while a separate fixer
component may use that evidence to drive a coding agent and create a patch or
pull request. This ledger defines the missing result contract for proving that
loop.

Current status: **not measured**. The repository has a boundary design and
outcome schema, but no dated result rows.

The central rule:

> No "Parallax fixes issues", "agent opens correct PRs", "fixer outcome
> feedback loop", "autonomous fix outcome learning", or "fixer business seam validated"
> claim without dated result rows linking evidence bundle -> fixer run -> agent
> session, with exact tool/version/config, adapter, capture surface, lossiness,
> redaction, source-field, schema ref, canonical hash, projection manifest,
> CLI/API/MCP projection equivalence, MCP `structuredContent`/`outputSchema`
> validation, and raw-ref status -> provider agent task when used ->
> patch/branch/PR -> CI/checks -> review/merge/revert/recurrence -> human or
> policy verdict.

Parallax itself still does not fix. This ledger measures the separate fixer
component and the outcome records that flow back into Parallax.

### Current Primary-Source Checks

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
| [MCP 2025-11-25 tools spec](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) and [base protocol](https://modelcontextprotocol.io/specification/2025-11-25/basic) | Tools may define `outputSchema`; structured results are returned in `structuredContent`; servers must conform when an output schema exists and clients should validate it. `_meta` is reserved protocol metadata. | Fixer MCP handoff rows must validate structured outputs against an output schema. Safety-critical fields cannot live only in text or `_meta`. |
| [RFC 8785 JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785.html) | JCS defines a deterministic, hashable JSON representation using strict JSON serialization, I-JSON constraints, and deterministic property sorting. | Bundle, projection, and session hashes need a named canonicalization method before equality claims across CLI, HTTP, MCP, and fixture files are meaningful. |
| [Where Do AI Coding Agents Fail?](https://arxiv.org/abs/2601.15195) | A large-scale study of agent-authored PRs reports failures tied to task class, change size, CI, and review/alignment problems. | The ledger must stratify by failure class, patch size, touched files, CI status, and review outcome instead of reporting one aggregate PR-open metric. |
| [Collaborator or Assistant?](https://arxiv.org/abs/2605.08017) | A PR lifecycle study finds agents can initiate and carry PR work while merge governance remains predominantly human. | Outcome rows must separate operational agency from terminal approval authority. |
| [Why Are AI Agent Involved Fix PRs Unmerged?](https://arxiv.org/abs/2602.00164) and [Why Are Agentic Pull Requests Merged or Rejected?](https://arxiv.org/abs/2605.22534) | Recent PR studies focus on integration outcomes, unmerged fix PR blockers, review interactions, and why raw merge/rejection labels can mislead. | A useful corpus needs reviewer rationale, workflow blockers, recurrence, and inconclusive states, not just merged versus closed. |

### Claim Levels

Use exactly one current level in `claim-ledger.jsonl` for a given fixer result
scope.

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No run artifacts exist. Current status. | "Fixer outcome loop is designed but not run-proven." |
| `fixture_harness_ready` | Repeatable task matrix, fixture repos, redaction fixtures, expected checks, and scoring protocol exist. | "Fixer outcome fixture harness prepared." |
| `bundle_handoff_supported` | Fixer consumes immutable Parallax bundles with `schema_ref`, canonical bundle hash, manifest, query report, redaction report, source-field policy, missing evidence, projection manifest, access policy, and raw-ref policy. | "Fixer bundle handoff works for the tested task set." |
| `fixer_run_record_supported` | Every run records status, policy, autonomy level, tool scopes, provider API version, optional provider task ID, agent session ID, and failure/no-op/timeout cases. | "Fixer runs are recorded for the tested task set." |
| `agent_session_linkage_pass` | Every run using an agent-session arm links to a dated agent-session ledger row with tool binary/version/config, adapter name/version, capture surface, source schema snapshot, canonical session bundle hash, projection manifest hash, lossiness, redaction, source-field policy, projection status, and raw-ref policy. | "Fixer runs link measured agent-session evidence for tested tasks." |
| `pr_creation_supported` | Separate fixer can create branches and draft PRs for allowed tasks with least-privilege repo policy. | "Separate fixer can open draft PRs for tested tasks." |
| `provider_agent_task_linkage_pass` | Copilot/GitHub Agent Tasks or similar provider-task runs are linked to task state, model, session count, artifacts, PR refs, and API preview/stability status. | "Provider agent-task linkage is recorded for tested tasks." |
| `source_field_projection_pass` | Eval/corpus-derived bundle projections preserve source-field policy status, redaction reports, missing-evidence flags, raw-ref denial, canonical bundle hash, projection manifest hash, CLI/HTTP/MCP equivalence, and MCP `structuredContent` validation into fixer-visible inputs. | "Fixer-visible inputs preserve safety fields for tested tasks." |
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

### Autonomy Levels

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

### Result Artifacts

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

### Run Manifest

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
  "bundle_schema_ref": {
    "uri": "schema://parallax/evidence-bundle/v0",
    "hash": "sha256:<hex>",
    "canonicalization": "jcs-rfc8785"
  },
  "canonical_bundle_hash_required": true,
  "agent_session_schema_version": "agent-session-v0",
  "redaction_policy_version": "a6-default-deny-vN",
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "projection_schema_version": "fixer-projection-vN",
  "projection_manifest_required": true,
  "projection_surfaces_required": [
    "bundle_json",
    "bundle_markdown",
    "cli_output",
    "http_api",
    "mcp_structuredContent"
  ],
  "mcp_output_schema_required": true,
  "a1_gate_scope": "failure_class_current|not_required_for_L1|missing",
  "a4_correlation_claim_level": "not_measured|a4_gate_pass|claim_expired",
  "a6_redaction_claim_level": "not_measured|a6_gate_pass|claim_expired",
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

### Row Schemas

#### Task Matrix Row

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

#### Bundle Handoff Row

```json
{
  "bundle_id": "bndl_001",
  "task_id": "fix_task_001",
  "bundle_schema_version": "bundle-v0",
  "schema_ref": "schema://parallax/evidence-bundle/v0",
  "schema_ref_hash": "sha256:<hex>",
  "canonicalization": "jcs-rfc8785",
  "canonical_bundle_hash": "sha256:<hex>",
  "access": {
    "raw_access_policy": "deny|scoped-read",
    "expires_at": "2026-05-25T00:00:00Z|null"
  },
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
  "projection_manifest_hash": "sha256:<hex>",
  "projection_surfaces_checked": [
    "bundle_json",
    "bundle_markdown",
    "cli_output",
    "http_api",
    "mcp_structuredContent"
  ],
  "projection_equivalence_pass": true,
  "projection_equivalence_hash": "sha256:<hex>",
  "mcp_structured_content_hash": "sha256:<hex>|not_used",
  "mcp_output_schema_valid": "true|false|not_used",
  "safety_fields_only_in_meta": false,
  "handoff_to_agent_success": true,
  "failure_reason": null
}
```

#### Fixer Run Row

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

#### Provider Agent Task Row

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

#### Agent Session Linkage Row

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
  "canonical_session_bundle_hash": "sha256:<hex>|not_applicable",
  "projection_manifest_hash": "sha256:<hex>|not_applicable",
  "projection_equivalence_pass": true,
  "mcp_structured_content_valid": true,
  "safety_fields_only_in_meta": false,
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

#### Patch And PR Row

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

#### CI And Check Row

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

#### Review Outcome Row

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

#### Merge, Revert, And Recurrence Row

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

#### Evidence Citation Row

```json
{
  "fixer_run_id": "fixrun_001",
  "output_ref": "pr_body|diagnosis|patch_summary",
  "material_claim_count": 8,
  "claims_with_evidence_refs": 8,
  "unsupported_claim_count": 0,
  "wrong_evidence_ref_count": 0,
  "canonical_bundle_hash_cited": true,
  "projection_manifest_hash_cited": true,
  "query_manifest_cited": true,
  "missing_evidence_cited": true,
  "source_field_policy_cited": true,
  "pass": true
}
```

#### Policy And Safety Row

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
  "canonical_bundle_hash_present": true,
  "projection_hash_mismatch": false,
  "mcp_structured_output_invalid": false,
  "safety_fields_only_in_meta": false,
  "provider_api_preview_used": false,
  "agent_visible_leak_count": 0,
  "scope_limit_violation": false,
  "policy_pass": true
}
```

#### Claim Ledger Row

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

### Counting Rules

- Opened PR is never success by itself.
- Fix success requires human-correct or merged status, passing required checks,
  no policy/safety failure, and recurrence/revert follow-up for the configured
  window.
- Fixer-visible bundle handoff fails unless the bundle includes `schema_ref`,
  `schema_ref_hash`, `canonical_bundle_hash`, `projection_manifest_hash`,
  `access`, `redaction_report`, `source_field_policy`, and `missing_evidence`.
- CLI, HTTP, MCP, Markdown, and persisted JSON projections must carry the same
  canonical bundle hash and projection manifest hash for the same task scope.
- MCP handoff counts only when `structuredContent` validates against the
  declared `outputSchema`; text-only MCP output is PR plumbing evidence at most.
- Safety-critical fields do not count if they appear only in MCP `_meta`, PR
  body comments, descriptions, or unvalidated free text.
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
- A `parallax_bundle_plus_session_trace` arm cannot count unless the linked
  agent-session row has `projection_safe: true`, matching canonical session
  bundle and projection-manifest hashes, and no raw-transcript/tool-payload
  dereference outside policy.
- L2/L3 autonomy rows cannot count for a failure class unless A1 bundle-value
  evidence is current for that class, or the run is explicitly scoped as
  offline/experimental and barred from product claims.
- A6 redaction and A4 correlation claim levels must be current before fixer
  outputs are used as agent-visible proof or correlation-success evidence.
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

### Initial Results Template

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
| Canonical bundle handoff pass rate | 0% | 100% | Pending |
| Bundle handoff pass rate | 0% | 100% | Pending |
| Agent-visible canary leaks | 0 | 0 | Pending |
| Source-field/projection pass rate | 0% | 100% | Pending |
| Projection equivalence failures | 0 | 0 | Pending |
| MCP structured output validation failures | 0 | 0 | Pending |
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

### Product Wording

Allowed after `not_measured`:

> Fixer outcome loop is designed but not run-proven.

Allowed after `pr_creation_supported`:

> A separate fixer can open draft PRs for tested tasks.

Allowed after `provider_agent_task_linkage_pass`:

> Provider agent-task artifacts are linked to fixer outcomes for tested tasks.

Allowed after `agent_session_linkage_pass`:

> Fixer runs link measured agent-session evidence for tested tasks.

Allowed after `source_field_projection_pass`:

> Fixer-visible inputs preserve source-field, redaction, missing-evidence,
> canonical-hash, projection-manifest, MCP structured-output, and raw-ref policy
> fields for tested tasks.

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

### Refresh Triggers

Mark affected claims `claim_expired` when:

- GitHub, GitLab, Sentry, Copilot, OpenHands, Codex, Claude Code, Amp, or
  OpenCode changes the relevant PR/review/check/agent API surface materially;
- GitHub REST API version, Agent Tasks preview/stability status, or task state
  model changes;
- branch protection, rulesets, required checks, or repository permission policy
  changes;
- Parallax bundle schema, fixer outcome schema, agent-session schema,
  source-field policy, projection schema, projection surface list, MCP
  output schema, canonicalization method, raw-ref policy, or redaction policy
  changes;
- the fixer model, agent product, prompt, tool policy, or task matrix changes;
- 90 days pass since the last run during discovery;
- a prior fixer PR counted as successful is later reverted, linked to
  recurrence, or marked unsafe.

### Relationship To Other Research

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

### Bottom Line

The fixer loop is only defensible if it is measured past the PR-open event.
Parallax's role is to make the evidence handoff and outcome trail auditable:
canonical bundle, validated projection, agent session, patch, CI, review, merge,
revert, recurrence, and human verdict. Until those rows exist, the honest claim
is design, not capability.
