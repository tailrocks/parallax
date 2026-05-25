# User Interview and Deployment Intent Gate

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This operationalizes bear-case assumption A2:

> Enough teams want self-hosted + open + low-ops Parallax to form a real user
> base.

The existing [Build roadmap](build-roadmap-and-validation-sequence.md) says to
talk to about 20 teams before investing deeply in storage and stream work. This
document turns that into a concrete gate: target segments, interview protocol,
question bank, scoring rubric, commitment tests, and kill criteria.

The goal is not to collect compliments. The goal is to discover whether teams
have the pain, have tried to solve it, can deploy a self-hosted evidence engine,
can expose the needed data safely, and will make a concrete commitment. The
companion [A2 interview evidence ledger](a2-interview-evidence-ledger.md)
defines how raw calls become redacted, auditable repo evidence.

## Source Posture

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

Internal sources:

- [Risks and the bear case](risks-and-bear-case.md) makes A2 existential and
  gives the current NO-GO trigger: 20 interviews yielding fewer than 3 teams who
  would deploy and 0 who would fund/sustain it.
- [Business model and economics](business-model-and-economics.md) says adoption,
  not revenue, is the first metric, but the business still needs a later hosting,
  fixer, enterprise ops, support, or sponsorship seam.
- [Bundle-value Phase 0 runbook](bundle-value-phase0-runbook.md) can reuse
  interview participants who are willing to provide real or anonymized incidents.
- [A2 interview evidence ledger](a2-interview-evidence-ledger.md) defines the
  redacted result artifact, commitment ladder, evidence classes, and bias
  controls for the 20-call run.

## Hypotheses To Test

Each interview should test these without pitching the answer first:

| Hypothesis | What would validate it |
| --- | --- |
| H1 Pain | The team recently spent meaningful time reconstructing a production/CI/agent failure across disconnected tools. |
| H2 Current workaround | They already pay for, self-host, script around, or manually stitch Sentry/logs/traces/CI/repo context. |
| H3 Self-host fit | They can and will run a small service if the operational footprint is lower than self-hosted Sentry. |
| H4 Data access | They can legally and culturally send Sentry/OTLP/CI/CLI/agent evidence into a self-hosted system. |
| H5 Agent relevance | They already use coding agents or expect to, and want evidence bundles for human or agent debugging. |
| H6 Commitment | They will give time, data, an intro, a pilot, sponsorship, hosted interest, or paid support interest. |

Do not count enthusiasm as validation unless it is paired with past behavior or
a next-step commitment.

## Target Interview Slices

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
[Repo-intent dependence](repo-intent-dependence.md) and should be measured
against the degraded-mode rows in
[Repo-intent value ledger](repo-intent-value-ledger.md).

## Interview Protocol

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

## Question Bank

Use these as prompts, not a rigid survey:

| Topic | Questions |
| --- | --- |
| Last incident | "Walk me through the last production, CI, or agent-caused failure that took too long to debug." "How long did it take, and who was pulled in?" |
| Context gathering | "Which tools did you open?" "What information did you wish was already connected?" "What did you have to copy/paste or reconstruct manually?" |
| Current spend/workarounds | "What do you use today for errors, logs, traces, metrics, CI failures, and deploy context?" "What have you built around those tools?" |
| Self-hosting | "Do you self-host observability today?" "If yes, why?" "If no, what would make self-hosting unacceptable?" |
| Sentry pain | "Have you used self-hosted Sentry?" "What broke or became too expensive or too operationally heavy?" |
| Retention | "How long do you keep logs/traces/errors?" "Have you shortened retention because of cost?" |
| Agents | "Which coding agents are engineers using now?" "Are they allowed to see production evidence?" "What would make that safe enough?" |
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

## Scoring Rubric

Score each interview immediately after the call:

| Dimension | 0 | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- | --- |
| Pain frequency | No relevant pain. | Rare annoyance. | Occasional but tolerable. | Monthly serious incident/debugging drag. | Weekly or acute business-impacting pain. |
| Existing behavior | No workaround/tool. | Manual habit only. | Uses standard tools. | Scripts/custom glue. | Built or bought serious internal workflow. |
| Self-host/deploy fit | Cannot deploy. | Strong blockers. | Possible with heavy review. | Can run Docker/small service. | Actively prefers self-host/open core. |
| Data access fit | Cannot expose needed data. | Major redaction/legal blockers. | Partial data only. | Most data usable with policy. | Can provide rich Sentry/OTLP/CI/CLI data. |
| Agent relevance | No agents/no interest. | Experimenting only. | Agents used individually. | Agents in normal engineering flow. | Agents touch CI/deploy/debug workflows already. |
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

## Pass, Continue, Kill

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

## Data Capture Template

Store interview notes as private or redacted Markdown until participants consent
to inclusion. Raw notes are not the public artifact; use the
[A2 interview evidence ledger](a2-interview-evidence-ledger.md) to create the
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

## How This Feeds Other Gates

- A1 bundle-value eval: recruit teams willing to share anonymized real incidents
  into [Bundle-value Phase 0](bundle-value-phase0-runbook.md).
- A3 schema moat: teams willing to adopt or critique the open bundle schema are
  the first non-operator schema signal.
- A6 redaction: objections and constraints feed
  [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md).
- Roadmap order: if A2 is weak, do not let storage benchmarks create false
  progress.

## Bottom Line

A2 is not validated by market size, stars, or friendly replies. It is validated
by teams describing recent expensive debugging pain, showing existing
workarounds, accepting the self-host/data-access model, and making concrete
commitments. Run this before mistaking engineering progress for product demand.
