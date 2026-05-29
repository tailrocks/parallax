# A2 — User Demand and Deployment Intent

> A2 — "enough teams want self-hosted + open + low-ops Parallax to form a real user base" — is operationalized as a concrete interview gate plus a privacy-preserving evidence contract, not as a market-size or star-count argument. The runbook fixes six target interview slices (with quotas), a past-behavior question bank grounded in The Mom Test and YC's "How To Talk To Users," a seven-dimension 0-4 scoring rubric, a commitment ladder, and explicit pass/continue/kill outcomes; the initial pass bar deliberately matches the bear case (>=6 teams score 16+, >=4 make Level 3+ deployment/data commitments, >=2 show a funding/sustainability path, >=5 slices covered), and A2 fails — reopening the GO verdict — if fewer than 3 teams would deploy or 0 would fund/sustain. The companion evidence ledger turns each call into one redacted YAML row (stable IDs, segment/role/stage buckets, score vector, fixed evidence classes, contradictions, consent level) committed to `docs/research/a2-deployment-intent-results.md`, while names, recordings, and contact logistics stay out of the repo by default. Bias controls (24-hour scoring, mandatory second reviewer for 22+ scores, a cap of 6 operator-network calls in the first 20, aggregate updates after calls 5/10/15/20) guard against founder optimism and operator-only signal. The decision is the protocol and the contract themselves; the gate remains OPEN — no interviews are logged yet, and no A2 pass claim is valid until the redacted ledger and aggregate summary are committed. Broad AI coding-tool adoption is explicitly not an A2 signal; only an observed agentic workflow, a context-permission surface, an audit/control need, or a concrete agent-caused/agent-fixed incident counts.

This note consolidates the following previously-separate research files, each preserved in full below:

- `user-interview-and-deployment-intent-gate.md`
- `a2-interview-evidence-ledger.md`

## User Interview and Deployment Intent Gate

_Provenance: merged verbatim from `user-interview-and-deployment-intent-gate.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

This operationalizes bear-case assumption A2:

> Enough teams want self-hosted + open + low-ops Parallax to form a real user
> base.

The existing [Build roadmap](../architecture/build-roadmap.md) says to
talk to about 20 teams before investing deeply in storage and stream work. This
document turns that into a concrete gate: target segments, interview protocol,
question bank, scoring rubric, commitment tests, and kill criteria.

The goal is not to collect compliments. The goal is to discover whether teams
have the pain, have tried to solve it, can deploy a self-hosted evidence engine,
can expose the needed data safely, and will make a concrete commitment. The
companion [A2 interview evidence ledger](a2-user-demand.md)
defines how raw calls become redacted, auditable repo evidence.

### Source Posture

Two outside references shape the protocol:

- Y Combinator repeatedly frames early startup learning around staying close to
  users and talking directly to them; its Startup School library includes "How
  To Talk To Users" as one of the core founder topics
  ([YC Startup School videos](https://www.ycombinator.com/blog/startup-school-videos)).
- Rob Fitzpatrick's *The Mom Test* exists because customer conversations produce
  false confidence when founders ask for opinions, compliments, or future-tense
  hypotheticals. The useful signal is facts about past behavior and concrete
  commitment, not "I would use that"
  ([The Mom Test](https://www.momtestbook.com/)).
- Current developer-market sources make the agent slice worth testing but not
  self-validating. Stack Overflow's 2025 Developer Survey says daily/weekly AI
  agent use is still a minority behavior, security/privacy and pricing are top
  technology rejection reasons, and AI accuracy distrust is higher than trust;
  GitHub's 2025 Octoverse says AI is now standard in development but warns its
  activity signals are observational rather than causal; JetBrains' 2025 AI
  report says many teams use AI ad hoc or in partial rollouts and worry about
  control, quality, reliability, and security
  ([Stack Overflow Developer Survey 2025](https://survey.stackoverflow.co/2025/),
  [GitHub Octoverse 2025](https://github.blog/news-insights/octoverse/octoverse-a-new-developer-joins-github-every-second-as-ai-leads-typescript-to-1/),
  [JetBrains AI report](https://devecosystem-2025.jetbrains.com/artificial-intelligence)).

Internal sources:

- [Risks and the bear case](../decisions/risks-and-bear-case.md) makes A2 existential and
  gives the current NO-GO trigger: 20 interviews yielding fewer than 3 teams who
  would deploy and 0 who would fund/sustain it.
- [Business model and economics](business-model.md) says adoption,
  not revenue, is the first metric, but the business still needs a later hosting,
  fixer, enterprise ops, support, or sponsorship seam.
- [Business model validation ledger](business-model.md) defines
  how any budget, support, hosted, fixer, enterprise-ops, or paid-pilot signal
  becomes a claimable business-model result.
- [Bundle-value Phase 0 runbook](a1-bundle-value/bundle-value-phase0-runbook.md) can reuse
  interview participants who are willing to provide real or anonymized incidents.
- [A2 interview evidence ledger](a2-user-demand.md) defines the
  redacted result artifact, commitment ladder, evidence classes, and bias
  controls for the 20-call run.

### Hypotheses To Test

Each interview should test these without pitching the answer first:

| Hypothesis | What would validate it |
| --- | --- |
| H1 Pain | The team recently spent meaningful time reconstructing a production/CI/agent failure across disconnected tools. |
| H2 Current workaround | They already pay for, self-host, script around, or manually stitch Sentry/logs/traces/CI/repo context. |
| H3 Self-host fit | They can and will run a small service if the operational footprint is lower than self-hosted Sentry. |
| H4 Data access | They can legally and culturally send Sentry/OTLP/CI/CLI/agent evidence into a self-hosted system. |
| H5 Agent relevance | They already use coding agents in concrete workflows, can describe what those agents may access, and want evidence bundles or audit trails for human/agent debugging. |
| H6 Commitment | They will give time, data, an intro, a pilot, sponsorship, hosted interest, or paid support interest. |

Do not count enthusiasm as validation unless it is paired with past behavior or
a next-step commitment.

### Target Interview Slices

Do not interview 20 copies of the operator. Split the first 20 across slices:

| Slice | Minimum | Why it matters |
| --- | --- | --- |
| Rust-heavy self-hosting teams | 5 | Closest initial ICP; tests the Rust/open/self-host wedge. |
| Small SaaS teams paying for Sentry/Datadog/Grafana | 4 | Tests whether managed-tool users feel enough pain to switch or add Parallax. |
| Privacy/compliance/data-residency teams | 4 | Tests the data-ownership argument and redaction gate. |
| CI-heavy or flaky-test-heavy teams | 3 | Tests whether the first value wedge should be CI failure context before production observability. |
| Agent-heavy engineering teams | 3 | Tests whether agent audit and agent-readable context are urgent now. |
| Open-source maintainers or infra consultants | 1+ | Tests adoption and support/service channels, even if not immediate SaaS revenue. |

A strong signal from only the Rust-heavy slice is not enough. That would
reinforce the founder-market-fit risk in
[Repo-intent dependence](repo-intent.md) and should be measured
against the degraded-mode rows in
[Repo-intent value ledger](repo-intent.md).

Broad AI coding-tool usage is not an A2 signal by itself. It only justifies
including agent-heavy teams in the sample. A2 signal requires an observed
agentic workflow, production/CI/repo context permission detail, an audit/control
need, or a concrete agent-caused or agent-fixed incident.

### Interview Protocol

Run 30-45 minute calls. Split into three parts:

1. **Past behavior discovery, 20-25 minutes.** No demo, no pitch, no product
   explanation beyond "researching debugging workflows for self-hosted teams."
2. **Concept check, 5-10 minutes.** Show one short Parallax description or a
   sample evidence bundle. Do not lead them to the desired answer.
3. **Commitment ask, 5-10 minutes.** Ask for a concrete next step appropriate
   to their interest level.

Rules:

- Ask about the last concrete incident, not generic workflow opinions.
- Ask what they actually did, paid for, scripted, or avoided.
- Ask who was involved and who controls the budget or deployment approval.
- Separate "would use" from "will do next."
- End every promising call with an ask for time, data, intro, pilot, or budget
  signal.
- Record exact quotes and concrete facts; label inferences separately.

### Question Bank

Use these as prompts, not a rigid survey:

| Topic | Questions |
| --- | --- |
| Last incident | "Walk me through the last production, CI, or agent-caused failure that took too long to debug." "How long did it take, and who was pulled in?" |
| Context gathering | "Which tools did you open?" "What information did you wish was already connected?" "What did you have to copy/paste or reconstruct manually?" |
| Current spend/workarounds | "What do you use today for errors, logs, traces, metrics, CI failures, and deploy context?" "What have you built around those tools?" |
| Self-hosting | "Do you self-host observability today?" "If yes, why?" "If no, what would make self-hosting unacceptable?" |
| Sentry pain | "Have you used self-hosted Sentry?" "What broke or became too expensive or too operationally heavy?" |
| Retention | "How long do you keep logs/traces/errors?" "Have you shortened retention because of cost?" |
| Agents | "Which coding agents are engineers using now?" "Is that autocomplete/chat only, ad hoc agent work, a team pilot, or a PR/CI workflow?" "Are agents allowed to see code, CI logs, Sentry/OTLP data, database evidence, or production evidence?" "What would make that safe enough?" "What logs, session records, approvals, or PR review gates do you have today?" "Walk me through the last agent output that needed correction, rollback, or extra human investigation." |
| Data constraints | "What data can never leave your environment?" "What fields must be redacted before an agent sees them?" |
| Buying/deploy path | "Who would approve deploying this?" "Who would pay for hosting/support/fixer workflow if the open core is free?" |
| Alternatives | "If Parallax did not exist, what would you do next to improve this workflow?" |
| Commitment | "Would you share one anonymized incident for a manual bundle eval?" "Would you run a tiny local pilot?" "Can you introduce me to the person who owns observability or incident response?" |

Avoid:

- "Would you use Parallax?"
- "Would you pay for this?"
- "Do you think this is a good idea?"
- "Would AI fixing production bugs help you?"

Replace with:

- "What did you do last time?"
- "What did it cost?"
- "What have you already tried?"
- "What would stop deployment?"
- "What concrete next step can we take?"

### Scoring Rubric

Score each interview immediately after the call:

| Dimension | 0 | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- | --- |
| Pain frequency | No relevant pain. | Rare annoyance. | Occasional but tolerable. | Monthly serious incident/debugging drag. | Weekly or acute business-impacting pain. |
| Existing behavior | No workaround/tool. | Manual habit only. | Uses standard tools. | Scripts/custom glue. | Built or bought serious internal workflow. |
| Self-host/deploy fit | Cannot deploy. | Strong blockers. | Possible with heavy review. | Can run Docker/small service. | Actively prefers self-host/open core. |
| Data access fit | Cannot expose needed data. | Major redaction/legal blockers. | Partial data only. | Most data usable with policy. | Can provide rich Sentry/OTLP/CI/CLI data. |
| Agent relevance | No agents/no interest. | Autocomplete/chat only or vague plans. | Individual ad hoc agent use. | Team pilot or PR workflow with clear permissions. | Agents touch CI/deploy/debug/runtime workflows with audit or control needs. |
| Budget/sustainability | No budget path. | Only free OSS. | Possible support/hosting later. | Known owner and budget category. | Active willingness for pilot/support/hosted/fixer spend. |
| Commitment | None. | Follow-up allowed. | Intro or async feedback. | Shares incident/data or runs pilot. | Time-boxed pilot plus budget/sponsor/fixer conversation. |

Interpretation:

| Score | Meaning |
| --- | --- |
| 0-9 | Not ICP. |
| 10-15 | Weak signal; keep as learning only. |
| 16-21 | Potential design partner if one blocker is solved. |
| 22-28 | Strong design partner candidate. |

Commitment overrides praise. A low-score team that says "great idea" is still
low signal. A medium-score team that shares incident data or agrees to a pilot is
real signal.

### Pass, Continue, Kill

For the first 20 interviews:

| Outcome | Gate result | Action |
| --- | --- | --- |
| >=6 teams score 16+, >=4 make concrete deployment/data commitments, and >=2 show a plausible funding/sustainability path | Pass A2 initial gate | Continue Phase 0/1; recruit these teams into bundle-value and pilot loops. |
| 3-5 teams score 16+ but funding is weak | Continue but narrow | Keep Parallax as OSS/adoption-first; delay business claims and focus on schema/pilot adoption. |
| Strong signal only from operator-like Rust monorepo teams | Narrow ICP | Reframe as Rust/self-hosted evidence engine; do not claim broad observability market yet. |
| Strong pain but mostly CI, not production observability | Pivot wedge | Lead with CI/failure-context bundle and defer full production observability. |
| Strong pain but teams reject self-hosting | Business-model shift | Consider hosted-first packaging while keeping self-host core. |
| <3 teams would deploy or 0 would fund/sustain | A2 fails | Reopen the GO verdict per the bear case. |

The minimum bar intentionally matches the bear case: fewer than 3 deploy-ready
teams and zero sustainability signal means the research has not escaped n=1.

### Data Capture Template

Store interview notes as private or redacted Markdown until participants consent
to inclusion. Raw notes are not the public artifact; use the
[A2 interview evidence ledger](a2-user-demand.md) to create the
committed, redacted result rows and aggregate A2 decision summary. Recommended
raw-note fields:

```text
date:
participant/company:
segment:
role:
tooling_today:
last_incident_summary:
time_lost:
current_workarounds:
self_hosting_posture:
data_access_constraints:
agent_usage:
agent_workflow_maturity:
agent_context_permission:
agent_control_or_audit_need:
agent_incident_evidence:
budget_owner:
exact_quotes:
score:
commitment:
next_step:
permission_to_quote: no|anonymous|named
```

Public repo notes should avoid company names, secrets, customer data, and
incident details unless explicitly permitted. Aggregate findings can be
committed under `docs/research/`; raw notes may need to stay private. No A2 pass
claim is valid unless the redacted ledger and aggregate summary are committed.

### How This Feeds Other Gates

- A1 bundle-value eval: recruit teams willing to share anonymized real incidents
  into [Bundle-value Phase 0](a1-bundle-value/bundle-value-phase0-runbook.md).
- A3 schema moat: teams willing to adopt or critique the open bundle schema are
  the first non-operator schema signal.
- Business-model validation: any concrete budget, support, hosted, fixer,
  enterprise-ops, or paid-pilot signal feeds the
  [business model validation ledger](business-model.md), but
  adoption alone does not prove revenue.
- A6 redaction: objections and constraints feed
  [Redaction pipeline and secret safety](../capture/redaction.md).
- Roadmap order: if A2 is weak, do not let storage benchmarks create false
  progress.

### Bottom Line

A2 is not validated by market size, stars, or friendly replies. It is validated
by teams describing recent expensive debugging pain, showing existing
workarounds, accepting the self-host/data-access model, and making concrete
commitments. Run this before mistaking engineering progress for product demand.

## A2 Interview Evidence Ledger

_Provenance: merged verbatim from `a2-interview-evidence-ledger.md` (2026-05-29 restructure)._

_(Shared note — see the User Interview and Deployment Intent Gate section above.)_

Research date: 2026-05-25

### Purpose

The [User interview and deployment intent gate](a2-user-demand.md)
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

### Current Source Posture

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

- [Risks and the bear case](../decisions/risks-and-bear-case.md) makes A2 existential and
  says the GO flips if 20 interviews produce fewer than three deploy-ready teams
  and zero sustainability signal.
- [Build roadmap and validation sequence](../architecture/build-roadmap.md)
  puts A2 in Phase 0 before storage benchmarking.
- [Bundle-value Phase 0 runbook](a1-bundle-value/bundle-value-phase0-runbook.md) can use
  interview participants who consent to provide anonymized incidents.
- [Business model validation ledger](business-model.md) consumes
  redacted budget, hosted, fixer, enterprise-ops, support/services, and
  paid-pilot signals from these interviews without treating adoption alone as
  revenue evidence.

### Why The Existing Gate Needs A Ledger

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

### Artifact Boundary

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

### Ledger Row Schema

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

### Evidence Classes

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

### Commitment Ladder

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

### Bias Controls

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

### Aggregate Summary Template

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

### Relationship To Other Gates

- A1 bundle-value eval: Level 3 anonymized incident commitments become candidate
  incidents for the [Bundle-value Phase 0 runbook](a1-bundle-value/bundle-value-phase0-runbook.md).
- A3 schema moat: teams willing to critique or adopt the schema become the first
  non-operator schema-adoption prospects.
- A6 redaction: every data-access refusal or redaction blocker becomes input to
  [Redaction pipeline and secret safety](../capture/redaction.md)
  and [Redaction detector toolchain](../capture/redaction.md).
- Roadmap: if the ledger cannot produce auditable A2 evidence, storage
  benchmarks remain premature no matter how attractive the technical work is.

### Bottom Line

The A2 run is a research instrument, not a set of sales calls. The public repo
should eventually contain redacted rows, aggregate counts, score distributions,
commitment outcomes, contradictions, and the explicit pass/continue/kill call.
Anything less makes A2 too easy to pass through optimism and too hard to audit
later.
