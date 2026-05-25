# Sentry MCP And Seer Self-Hosted Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Re-check the most load-bearing Sentry claim in the current research record:
whether Parallax can still treat self-hosted Sentry users as lacking Sentry's
hosted Seer/Autofix experience, and whether Sentry MCP has closed the
agent-access gap for self-hosted Sentry.

## Verdict

Sentry remains the strongest direct incumbent pressure, but the repository
should narrow the old claim.

Keep:

- hosted Seer/Autofix is a strong proof that production-error AI debugging,
  root-cause analysis, solution planning, code changes, PR creation, and external
  coding-agent handoff are real workflows;
- Sentry MCP makes agent access to Sentry data table stakes;
- current self-hosted docs now explicitly exclude Seer and other AI/ML features
  from self-hosted Sentry because those components are closed source;
- Parallax's sharper gap is the open, portable, redacted evidence bundle plus
  action/outcome ledger, not "Sentry has no AI."

Remove or avoid:

- treating the current self-hosted Seer exclusion as a permanent technical
  impossibility or proof that Sentry will never ship a self-hosted AI path;
- categorical wording that Sentry MCP self-hosted use always requires write
  scopes;
- any implication that "MCP exists" closes or proves the Parallax agent surface.

## Current Source Snapshot

| Source | Current check | Parallax implication |
| --- | --- | --- |
| [Sentry Seer docs](https://docs.sentry.io/product/ai-in-sentry/seer/) | Seer is described as Sentry's AI debugging agent using issue details, tracing data, logs, profiles, and code context. Seer includes Autofix, PR creation, external coding-agent handoff, Seer Agent, and code review. Seer is also an add-on to a Sentry subscription. | Hosted Sentry is directly attacking the issue-to-fix workflow. Parallax must not claim PR generation or AI RCA as a moat. |
| [Sentry Autofix docs](https://docs.sentry.io/product/ai-in-sentry/seer/autofix/) | Autofix uses Sentry context and GitHub-integrated codebases; it can stop after root cause, plan, or PR draft. The docs say Seer can only integrate with the cloud version of GitHub, and that cloud GitHub is currently the only SCM supported by Seer. Handoff agents listed are Claude Code and Cursor Cloud Agents. | Sentry's hosted path is operationally strong but cloud-GitHub-oriented. A local/open evidence engine can still win for self-hosted, air-gapped, multi-source, or schema-first users. |
| [Seer Issue Fix API](https://docs.sentry.io/api/seer/start-seer-issue-fix/) | The API can identify root cause, propose a solution, generate code changes, and create a PR. Stop points are `root_cause`, `solution`, `code_changes`, and `open_pr`; runs are asynchronous. The endpoint requires an auth token with `event:admin` or `event:write`. | The separate-fixer workflow is incumbent behavior and it is a write/admin event-scope surface. Parallax must differentiate below the fixer: evidence contract, redaction, provenance, and outcome rows. Keep the first Parallax MCP/context surface read-only; put fix orchestration in a separate control plane. |
| [sentry-mcp README](https://github.com/getsentry/sentry-mcp) and [`0.35.0` release](https://github.com/getsentry/sentry-mcp/releases/tag/0.35.0) | Latest release checked is `0.35.0`, published 2026-05-21; `/releases/latest` returned HTTP `200` at `0.35.0`, and `git ls-remote` shows tag `fc04542e24472f00b639f2d591dfc111fa855158`. The README says Sentry MCP is primarily for human-in-the-loop coding agents. It supports remote MCP, Claude Code plugin/subagent use, and stdio. The README calls stdio a work-in-progress path for self-hosted Sentry; AI-powered search needs OpenAI or Anthropic configuration; self-hosted instances may need unsupported Seer skills disabled; the README setup path lists `project:write`, `team:write`, and `event:write`. | Sentry MCP is real and important. It is not proof of self-hosted hosted-Seer parity, canonical evidence-bundle projection, redaction reports, or read-only-by-default safety. |
| [sentry-mcp stdio testing guide](https://github.com/getsentry/sentry-mcp/blob/main/docs/testing-stdio.md) | The testing guide documents full-function scopes including write scopes, but also states read-only testing can use `org:read`, `project:read`, `team:read`, and `event:read`. The guide still shows stdio self-hosted configs and example output with 20 tools available. | Narrow the old scope claim. Sentry documents a read-only testing path, but Parallax still cannot count this as a read-only-safe agent surface until tool availability, projection equivalence, redaction, and fixture behavior are proven. |
| [Self-hosted Sentry docs](https://develop.sentry.dev/self-hosted/) and [`26.5.0` release](https://github.com/getsentry/self-hosted/releases/tag/26.5.0) | Latest self-hosted release checked is `26.5.0`, published 2026-05-18; `/releases/latest` returned HTTP `200` at `26.5.0`, and `git ls-remote` shows tag `aed5b2037e74c771bfe476dbdbeb80420ef4a3d8`. Current docs describe a Docker Compose/bash setup, minimum and recommended resources, errors-only beta, feature-complete mode, single-node service caveats, FSL licensing, and a feature-complete list including traces, profiles, replays, uptime, metrics, feedback, and crons. They explicitly list Seer and other AI/ML features as unavailable on self-hosted Sentry because those components are closed source. The `26.5.0` compose file declares 72 services. | The self-hosted Seer gap is explicit in current primary docs, not merely unproven. Keep the claim tied to current docs and avoid implying Sentry cannot change this later. |

## Product Impact

The Sentry wedge should now be worded as:

> Sentry's hosted Seer/Autofix path validates the issue-to-fix workflow. Current
> self-hosted docs explicitly exclude Seer and other AI/ML features from
> self-hosted Sentry, and Sentry MCP does not publish Parallax-style portable,
> redacted, citable evidence bundles or outcome ledgers.

The issue-fix API's `event:admin`/`event:write` requirement also reinforces the
component boundary: Sentry's fix path is a privileged control surface. Parallax's
first agent-facing surface should stay read-only evidence retrieval; any
PR-opening fixer belongs above it.

This is still strategically important. It just keeps the self-hosted claim tied
to current Sentry docs instead of turning it into a future-proof guarantee.

## Falsification Criteria

Revisit the verdict and drift ledger if Sentry publishes any of the following:

- removal or reversal of the current self-hosted docs exclusion for Seer and
  other AI/ML features;
- self-hosted Seer/Autofix parity with local or customer-selected LLM providers;
- self-hosted Seer Agent support with documented access controls below
  organization-wide telemetry;
- sentry-mcp stdio graduating from work-in-progress status with read-only default
  scopes and a published tool catalog for those scopes;
- structured MCP outputs that are equivalent to a portable evidence bundle with
  redaction reports, raw refs, and source-field provenance;
- a public issue-fix outcome ledger covering accepted, rejected, reverted, and
  recurrent fixes.

Until then, keep Sentry at `wedge_under_pressure`, not `wedge_closed`.
