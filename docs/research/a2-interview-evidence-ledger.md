# A2 Interview Evidence Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The [User interview and deployment intent gate](user-interview-and-deployment-intent-gate.md)
defines who to interview, what to ask, and how to score each call. This note
defines the missing execution artifact: a privacy-preserving evidence ledger
that turns those calls into auditable A2 evidence without committing raw private
notes or relying on founder memory.

A2 is not validated by "20 calls happened." It is validated only when the repo
contains enough redacted, structured evidence to show:

- which target slices were actually reached;
- what past debugging pain and current behavior were observed;
- which self-hosting, data-access, agent, budget, and commitment signals were
  present;
- which signals were weak, contradicted, or missing;
- why the resulting pass/continue/kill call follows from the evidence.

## Current Source Posture

Outside sources checked for this pass:

- Y Combinator's Startup School list includes "How To Talk To Users" as a core
  founder talk, and YC's older transcript emphasizes founder/user direct contact,
  avoiding pitches during interviews, asking about the last time the problem
  happened, and asking what the user already tried
  ([Startup School videos](https://www.ycombinator.com/blog/startup-school-videos),
  [Startup School Week 1 recap](https://www.ycombinator.com/blog/startup-school-week-1-recap-kevin-hale-and-eric-migicovsky/)).
- *The Mom Test* positions customer conversations around avoiding biased
  feedback, finding real customer pain, and figuring out whether someone is
  really going to buy, not collecting agreeable opinions
  ([The Mom Test](https://www.momtestbook.com/)).
- NIST's Privacy Framework treats privacy risk as arising from data operations
  across the full lifecycle from collection through disposal, and NIST SP 800-122
  says PII should be protected from inappropriate access, use, and disclosure
  ([NIST Privacy Framework getting started](https://www.nist.gov/privacy-framework/getting-started-0),
  [NIST SP 800-122](https://csrc.nist.gov/pubs/sp/800/122/final)).
- Stack Overflow's 2025 Developer Survey received 49,000+ responses and added a
  focus on AI agent tools and LLMs. Its agent section says 14.1% of respondents
  use AI agents at work daily and 9% weekly, while a majority either do not use
  agents or only use simpler AI tools. The same survey ranks security/privacy,
  prohibitive pricing, and better alternatives as the top three reasons
  developers reject technologies, and reports that more respondents actively
  distrust AI accuracy than trust it
  ([Stack Overflow Developer Survey 2025](https://survey.stackoverflow.co/2025/)).
- GitHub's 2025 Octoverse says generative AI is now standard in development, but
  explicitly labels its productivity/activity data as observational rather than
  causal. It also says 81.5% of contributions happen in private repositories,
  which matters because Parallax's strongest evidence lives in private runtime,
  repo, CI, and agent-session data
  ([GitHub Octoverse 2025](https://github.blog/news-insights/octoverse/octoverse-a-new-developer-joins-github-every-second-as-ai-leads-typescript-to-1/)).
- GitHub's Copilot coding-agent announcement describes draft PRs, session logs,
  human approval before CI/CD workflows run, and MCP configuration in repository
  settings. This is primary evidence that agent workflows are becoming
  auditable/control-layered, not just chat prompts
  ([GitHub Copilot coding agent](https://github.com/newsroom/press-releases/coding-agent-for-github-copilot)).
- JetBrains' 2025 Developer Ecosystem AI report says AI use is often ad hoc,
  rollout is commonly still in pilots or partial integration, and developer
  concerns center on control, code quality, reliability, and security. Its
  methodology reports 24,534 cleaned responses and notes remaining survey bias
  controls
  ([JetBrains AI report](https://devecosystem-2025.jetbrains.com/artificial-intelligence),
  [JetBrains methodology](https://lp.jetbrains.com/developer-ecosystem-2025-methedology/)).

Internal sources:

- [Risks and the bear case](risks-and-bear-case.md) makes A2 existential and
  says the GO flips if 20 interviews produce fewer than three deploy-ready teams
  and zero sustainability signal.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  puts A2 in Phase 0 before storage benchmarking.
- [Bundle-value Phase 0 runbook](bundle-value-phase0-runbook.md) can use
  interview participants who consent to provide anonymized incidents.
- [Business model validation ledger](business-model-validation-ledger.md) consumes
  redacted budget, hosted, fixer, enterprise-ops, support/services, and
  paid-pilot signals from these interviews without treating adoption alone as
  revenue evidence.

## Why The Existing Gate Needs A Ledger

The current A2 gate is strong as an interview protocol, but weak as a durable
research artifact unless the result is logged consistently. Three failure modes
matter:

| Failure mode | How it corrupts A2 | Ledger countermeasure |
| --- | --- | --- |
| Raw notes stay private and only a conclusion is committed | The repo cannot audit whether A2 passed or failed. | Commit redacted rows, aggregate counts, score distributions, and decision rationale. |
| Raw notes are committed too eagerly | Participant/company identity, customer data, or incident details leak into the public research repo. | Keep names, recordings, exact sensitive incidents, and follow-up contact data outside the repo by default. |
| Founder memory replaces evidence | Friendly calls and operator-like teams get overweighted. | Use fixed evidence classes, two-pass scoring, segment quotas, and explicit contradiction fields. |

The ledger is therefore the middle layer between private call notes and the
public A2 decision.

## Artifact Boundary

Use five artifacts during A2 execution:

| Artifact | Location | Commit? | Contents | Rule |
| --- | --- | --- | --- | --- |
| Recruiting register | Private operator workspace | No | Names, companies, emails, intros, scheduling status. | Never needed for public A2 proof. |
| Raw interview notes | Private operator workspace | No by default | Exact notes, recordings, names, incident details, proprietary tool screenshots. | Commit only if explicitly approved and redacted. |
| Redacted evidence ledger | `docs/research/a2-deployment-intent-results.md` once interviews begin | Yes | One sanitized row per interview, score vector, evidence classes, commitment type, contradictions, consent level. | Must be enough to audit the A2 decision without identifying participants. |
| Aggregate decision summary | Same results doc | Yes | Segment coverage, score distribution, commitment counts, deployment blockers, pass/continue/kill call. | Update after each batch of 5 calls and at the 20-call gate. |
| Commitment tracker | Private until consent; summarized publicly | Partial | Follow-up owner, due date, pilot/data/intro status. | Public repo gets counts and anonymized outcomes, not contact logistics. |

No A2 pass claim is valid unless the redacted ledger and aggregate summary are
committed.

## Ledger Row Schema

Each interview becomes one redacted row. Use stable IDs and buckets instead of
company names.

```yaml
interview_id: A2-001
research_date: 2026-05-25
segment: rust_self_hosted | small_saas_paid_observability | privacy_compliance | ci_flaky_test | agent_heavy | oss_infra_consultant | other
role_class: founder | staff_engineer | infra_owner | sre | security_privacy | engineering_manager | maintainer | consultant | other
company_stage_bucket: solo | 2_10 | 11_50 | 51_250 | 251_plus | oss_project | unknown
operator_network_distance: direct | warm_intro | cold_outbound | inbound | unknown
current_tooling_classes:
  - sentry_cloud
  - sentry_self_hosted
  - datadog
  - grafana_stack
  - signoz
  - openobserve
  - clickhouse_custom
  - ci_provider_only
  - scripts_custom_glue
  - none
last_incident_class: production_error | ci_failure | flaky_test | agent_change | deploy_regression | frontend_user_error | data_issue | none_recent
last_incident_recency: last_7_days | last_30_days | last_90_days | older | none
time_lost_bucket: none | under_1h | 1_4h | half_day | 1_2_days | multi_day | unknown
evidence_classes:
  - observed_past_behavior
  - current_workaround
  - paid_tool_spend
  - self_hosting_constraint
  - retention_cost_constraint
  - data_access_permission
  - redaction_blocker
  - agent_usage_observed
  - agent_workflow_maturity
  - agent_context_permission
  - agent_control_audit_need
  - budget_owner_named
  - concrete_commitment
  - contradiction
agent_workflow_maturity: none | autocomplete_only | individual_ad_hoc | team_pilot | agent_pr_workflow | agent_with_runtime_context | unknown
agent_context_permission: none | code_only | ci_only | read_only_runtime | redacted_runtime | production_runtime | unknown
agent_incident_evidence: none | near_miss | bad_patch | failed_fix | successful_agent_fix | unknown
score:
  pain_frequency: 0
  existing_behavior: 0
  self_host_deploy_fit: 0
  data_access_fit: 0
  agent_relevance: 0
  budget_sustainability: 0
  commitment: 0
total_score: 0
commitment_type: none | async_feedback | intro | anonymized_incident | local_pilot | budget_sponsor_conversation | paid_support_hosting_interest
commitment_due_bucket: none | within_7_days | within_30_days | later | unknown
permission_to_quote: none | anonymous | named
sanitized_evidence:
  - "Short paraphrase of a fact about past behavior, with no names or secrets."
  - "Short paraphrase of a blocker or contradiction."
contradictions:
  - "Example: high agent interest but no allowed production evidence access."
review_confidence: low | medium | high
reviewer: operator | second_reviewer | pair
```

Allowed `sanitized_evidence` content:

- paraphrased facts about past incidents, current tools, workflows, and blockers;
- short anonymous quotes only when permission is `anonymous` or `named`;
- score justification that does not reveal company identity.

Forbidden `sanitized_evidence` content:

- participant names, company names, customer names, employee names, email
  addresses, domains, Slack handles, ticket IDs, private repo names, exact
  incidents that would identify the company, secrets, raw logs, screenshots, or
  proprietary architecture details.

## Evidence Classes

Use these classes to separate real signal from conversational noise:

| Class | Counts as evidence | Does not count |
| --- | --- | --- |
| `observed_past_behavior` | The participant described a specific recent failure and the steps taken. | "Debugging is hard here." |
| `current_workaround` | They use scripts, manual runbooks, custom dashboards, retained logs, or incident templates to stitch context. | Generic use of Sentry/Grafana with no pain. |
| `paid_tool_spend` | They pay for Sentry, Datadog, Grafana Cloud, logging storage, CI minutes, support, or similar. | They say budget "could exist." |
| `self_hosting_constraint` | They have a policy, cost reason, data-residency reason, or operational reason to prefer self-hosting. | "Open source is nice." |
| `retention_cost_constraint` | They shortened retention, sample aggressively, or avoid storing evidence because of cost. | Abstract concern about future bills. |
| `data_access_permission` | They can provide Sentry/OTLP/CI/CLI/agent data, with named redaction constraints. | "Probably okay" without owner or policy detail. |
| `redaction_blocker` | Required evidence is legally, contractually, or culturally unavailable even after redaction. | Vague discomfort. |
| `agent_usage_observed` | Coding agents are already used in normal engineering work or incident workflows. | Curiosity about AI, autocomplete-only use, or broad survey adoption. |
| `agent_workflow_maturity` | The team can place agent use on a concrete maturity step: ad hoc, team pilot, PR workflow, or runtime-context workflow. | "We use AI" without where, how often, or with what permissions. |
| `agent_context_permission` | The team states whether agents can see code only, CI only, redacted runtime evidence, or production runtime evidence. | No owner or policy detail for what agents may access. |
| `agent_control_audit_need` | The team has or wants session logs, human approval gates, CI approval, MCP/tool access review, or post-change outcome tracking. | General fear that agents are risky. |
| `budget_owner_named` | A role or person can approve pilot/support/hosting/fixer spend. | "Someone would pay." |
| `concrete_commitment` | An intro, data share, pilot, design-partner call, budget conversation, or incident handoff has owner and due date. | Praise, stars, newsletter signup, or "keep me posted." |
| `contradiction` | A fact that weakens the score is recorded explicitly. | Softening a blocker because the call felt positive. |

## Commitment Ladder

The A2 gate should weight commitments by cost to the participant:

| Level | Commitment | How to count it |
| --- | --- | --- |
| 0 | No follow-up | Not validation. |
| 1 | Async feedback or permission to send summary | Weak interest only. |
| 2 | Intro to another relevant operator | Distribution signal, not deployment signal. |
| 3 | Shares one anonymized incident or workflow artifact | Strong product-learning signal; can feed A1 if usable. |
| 4 | Runs a local pilot or joins a recurring design-partner loop | Strong deployment signal. |
| 5 | Names budget/sponsor path or discusses paid support/hosting/fixer spend | Sustainability signal. |

For the A2 pass threshold, count only Level 3+ as "concrete deployment/data
commitments." Count Level 5 as sustainability signal only if the buyer/sponsor
path is named by role and the next step is time-boxed.

## Bias Controls

Use these controls during the 20-interview run:

- Score within 24 hours of the call, before post-call optimism rewrites memory.
- Preserve contradictions even when the total score is high.
- Use a second reviewer for any interview scored 22+ or any interview used to
  justify a pass result.
- Cap operator-like direct-network calls at 6 of the first 20 unless the
  aggregate result is reported as Rust/self-hosted-only.
- Update aggregate counts after interviews 5, 10, 15, and 20; do not wait until
  the end to notice segment skew.
- Downgrade any commitment that misses its promised follow-up window without a
  concrete artifact.
- Do not count the same company twice toward the 20-call gate unless the second
  participant owns a different decision surface, such as security/privacy or
  budget.
- Treat a privacy/data-access refusal as evidence, not as a recruiting failure.
- Do not treat broad AI coding-tool adoption as an A2 signal. It only justifies
  sampling agent-heavy teams. A2 signal requires observed agent workflow,
  permission surface, audit/control need, or an agent-caused/successful-fix
  incident.
- Autocomplete-only usage does not satisfy `agent_usage_observed`; classify it
  as `autocomplete_only` and score agent relevance no higher than 1 unless the
  team has a concrete plan to move agents into PR, CI, incident, or runtime
  workflows.

## Aggregate Summary Template

When interviews begin, create `docs/research/a2-deployment-intent-results.md`
with this structure:

```markdown
# A2 Deployment Intent Results

Research window:
Last updated:

## Gate Snapshot

| Metric | Current | Required for pass | Status |
| --- | ---: | ---: | --- |
| Completed interviews | 0 | 20 | Pending |
| Teams scoring 16+ | 0 | >=6 | Pending |
| Level 3+ commitments | 0 | >=4 | Pending |
| Level 5 sustainability signals | 0 | >=2 | Pending |
| Distinct target slices covered | 0 | >=5 | Pending |

## Segment Coverage

## Score Distribution

## Commitment Outcomes

## Contradictions And Blockers

## Interim Decision

## Redacted Ledger
```

The interim decision must use the existing A2 outcomes:

- pass A2 initial gate;
- continue but narrow;
- narrow ICP;
- pivot wedge;
- business-model shift;
- A2 fails and the GO verdict reopens.

## Relationship To Other Gates

- A1 bundle-value eval: Level 3 anonymized incident commitments become candidate
  incidents for the [Bundle-value Phase 0 runbook](bundle-value-phase0-runbook.md).
- A3 schema moat: teams willing to critique or adopt the schema become the first
  non-operator schema-adoption prospects.
- A6 redaction: every data-access refusal or redaction blocker becomes input to
  [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  and [Redaction detector toolchain](redaction-detector-toolchain.md).
- Roadmap: if the ledger cannot produce auditable A2 evidence, storage
  benchmarks remain premature no matter how attractive the technical work is.

## Bottom Line

The A2 run is a research instrument, not a set of sales calls. The public repo
should eventually contain redacted rows, aggregate counts, score distributions,
commitment outcomes, contradictions, and the explicit pass/continue/kill call.
Anything less makes A2 too easy to pass through optimism and too hard to audit
later.
