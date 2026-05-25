# Datadog Bits AI Eval Loop

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Re-check the claim that Datadog proves the industry is moving from "AI RCA
feature" toward a measured, feedback-driven investigation platform. The existing
notes cited Datadog's Bits AI SRE eval platform, but did not connect that source
back into Parallax's own A1 bundle-value gate.

## Current Verdict

Keep the claim, but make it sharper:

> Datadog is not just shipping an incident chatbot. It is industrializing an
> agent-evaluation loop around real incident labels, reconstructed investigation
> worlds, noisy distractors, score history, regression detection, model-refresh
> testing, and product feedback.

That strengthens Parallax's direction, but it does **not** validate Parallax's
bundle-value claim. Datadog's public material does not publish an open schema,
portable evidence bundles, raw-dump-vs-bundle arms, public task rows, or a
self-hosted result ledger. For Parallax, Datadog should be treated as a
methodology benchmark and a closed-platform competitor, not as proof that the
Parallax bundle moat already works.

## Source Checks

| Source | Current check | Parallax implication |
| --- | --- | --- |
| [Bits AI SRE investigation docs](https://docs.datadoghq.com/bits_ai/bits_ai_sre/investigate_issues/) | Bits AI SRE can auto-investigate supported monitors; it runs a loop of observation, reasoning, and action, forms hypotheses, queries telemetry, validates or invalidates them, and either returns an evidence-backed conclusion or marks the result inconclusive. Supported Datadog data includes metrics, APM traces, logs, dashboards, events, Change Tracking, GitHub source code, Watchdog, RUM, Network Path, Database Monitoring, and Continuous Profiler. Third-party Grafana, Dynatrace, Sentry, Splunk, ServiceNow, and Confluence integrations are Preview. Agent Trace records the steps, evidence evaluation, and decisions. | The investigation architecture matches the Parallax evidence-loop thesis, but it lives inside Datadog's data gravity and UI. |
| [Bits AI SRE eval platform engineering blog](https://www.datadoghq.com/blog/engineering/bits-ai-eval-platform/) | Datadog reconstructs investigation context as world snapshots, isolates scenarios at the data layer, adds realistic noise/red herrings, stores scores per scenario per run, segments by technology/problem/monitor/difficulty, tracks pass/fail evolution and `pass@k`, runs the full eval set weekly, and uses product feedback to grow labels. Datadog says new model releases are run against the full label set before production rollout decisions. | Parallax's A1 eval should adopt the discipline: frozen world snapshots, noise manifests, score history, segmentation, full-set regression runs, model-refresh triggers, and feedback-to-label conversion. |
| [Bits AI Dev Agent docs](https://docs.datadoghq.com/bits_ai/bits_ai_dev_agent/) | Bits AI Dev Agent is in Preview. It uses Datadog observability data to diagnose and fix code issues, creates code sessions with analysis/actions/code changes, integrates with GitHub, opens draft PRs, uses CI logs and developer feedback, and supports Error Tracking, Trace Explorer, Code Security, Test Optimization, Continuous Profiler, and Containers. Auto-push can create branches and PRs but never merges code. The docs state the Dev Agent does not support multi-repository investigations. | PR creation is commodity and Datadog is expanding beyond flaky tests into production errors, traces, security, profiling, and Kubernetes remediation. Parallax should keep the fixer separate and compete on open evidence/outcome records. |
| [Bits AI Dev Agent setup docs](https://docs.datadoghq.com/bits_ai/bits_ai_dev_agent/setup/) | Setup requires GitHub integration permissions for repository contents and pull requests, optional check/status/comment permissions for CI-log iteration, `service` and `version` telemetry tags to map runtime issues to code, service-to-repository mapping, optional auto-push, repository instruction files including `AGENTS.md`, and controlled internet access policies. | The repo-intent and deploy/release-context assumptions are not speculative. Incumbents already connect telemetry tags, source mapping, agent instructions, CI logs, and PR operations. |
| [Flaky Tests Management docs](https://docs.datadoghq.com/tests/flaky_management/) | AI-powered flaky test fixes remain Preview. Eligible tests must meet thresholds for failure rate, wasted time, failed pipelines, and default-branch flaking; eligibility also requires failure data such as `@error.message` and `@test.source.file`. Datadog also has AI-powered flaky-test categorization from execution patterns and error signals. | The old "flaky test fix" wedge is directly contested. Parallax should use flaky tests as an eval domain, not as the product's primary market position. |
| [Bits AI SRE take-action docs](https://docs.datadoghq.com/bits_ai/bits_ai_sre/take_action/) | Suggested code fixes from Bits AI Dev Agent are Preview. After Bits AI SRE determines a code-related root cause, it can hand off to Dev Agent for GitHub PR creation and iteration with CI logs and developer feedback. | Datadog is connecting incident RCA to code remediation. Parallax's durable differentiator must be the open evidence bundle plus measured fix outcome loop, not "SRE agent suggests PR." |

## Methodology Lessons For A1

Datadog exposes a stronger eval bar than the current Parallax A1 docs captured.
The Parallax result ledger should explicitly track:

| Datadog pattern | Parallax A1 control |
| --- | --- |
| Reconstructed world snapshots | Freeze an `investigation_world_snapshot` per task with normalized evidence rows, raw refs, topology links, release/deploy context, and source-field policy status. |
| Noisy evaluation worlds | Add a `noise_manifest` that records unrelated but plausible services, logs, alerts, spans, and deploys. A bundle arm must beat the raw-dump arm without receiving cleaner evidence. |
| Scenario segmentation | Segment every task by language, failure type, signal mix, monitor/source type, difficulty, provenance, and data completeness. |
| Score history per scenario per run | Keep per-task, per-arm, per-model, per-seed rows across model and bundle-template versions instead of only publishing aggregate resolved rate. |
| `pass@k` and repeated attempts | Add `pass_at_k`/`any_success` plus variance rows so a bundle win is not a lucky single attempt. |
| Full-set regression runs | Rerun the full A1 set after model, scaffold, bundle-template, retrieval, or redaction-policy changes, not only a targeted subset. |
| Product feedback to labels | Convert accepted/rejected/reverted fixer outcomes into candidate eval labels only after provenance, privacy, and source-field checks pass. |
| Model-refresh testing | Treat new model generations as a claim-expiry trigger; rerun before saying bundles still improve agents. |

## Product Positioning Impact

Datadog narrows the safe public wording for Parallax:

- Do not claim "AI RCA" as a moat. Datadog already has hypothesis-driven
  incident investigation with evidence-backed or inconclusive outcomes.
- Do not claim "agent opens PR" as a moat. Datadog Dev Agent, Sentry Seer,
  GitHub Copilot, and OpenHands all cover that direction.
- Do claim that Datadog makes the evaluation loop non-optional. If Parallax does
  not produce an auditable A1 result ledger, its bundle moat is only a theory.
- Keep the open/self-hosted, Rust-first, portable-bundle, read-only-agent-access,
  and measured outcome-loop wedge. Datadog's public sources remain closed
  platform artifacts, not a portable evidence standard.

## Remaining Uncertainty

- Datadog's eval platform details are public as an engineering blog, not a
  published schema or reproducible benchmark.
- The public docs do not show whether Dev Agent is generally available across
  all listed product areas; current docs and flaky-test pages still label it
  Preview.
- Datadog's internal score definitions, label format, world-snapshot schema, and
  model/scaffold versions are not public.
- It is unknown whether Datadog exposes enough APIs for customers to export
  investigation traces, eval labels, or code-session outcomes as portable
  artifacts.

## Falsification Criteria

Reopen this note if Datadog publishes any of the following:

- an open, versioned investigation/evidence-bundle schema with portable raw refs;
- public eval rows or a reproducible incident-agent benchmark;
- self-hosted or customer-owned Bits AI eval infrastructure;
- exportable code-session and investigation traces with enough fields to serve
  as a cross-vendor evidence standard;
- evidence that Dev Agent's Preview limitations or single-repository limitation
  have materially changed.
