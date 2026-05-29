# Business Model and Economics

> Parallax leads with the open, self-hosted, data-owned slice — structurally the least-monetizable door into a category that is nonetheless company-sized (Sentry's own Fair Source post reports 100k+ cloud customers and $100M+ annual revenue, but it monetizes overwhelmingly through cloud, not self-hosting). The decided licensing position is **Apache-2.0 for the core, kept consistent — no relicensing** (operator, 2026-05-29; a CLA may be kept for contribution provenance, not as a path to a future relicense), FSL/Fair Source rejected as ethos-incompatible at t=0; the non-negotiable is never gating the agent/evidence differentiator, because that exact gap (OpenObserve gates its AI SRE agent behind Enterprise) is the wedge Parallax exists to exploit. Four legitimate value-capture seams are named but unproven: hosted/managed Parallax, the separate fixer/agent-orchestration product, enterprise ops add-ons (SSO/SAML, RBAC, multi-tenancy, audit/compliance, retention, backup/DR, support/SLA), and support/services/certification. Adoption — not revenue — is the leading metric for the first phase, and the bear case's "no monetization path" risk is narrowed, not eliminated. The open gate: none of these seams may be called `value_capture_validated` until the validation ledger records redacted result rows showing deployment, budget/payment, conversion, or paid-pilot evidence — at least one paid pilot, signed order/contract, paid support agreement, LOI with budget owner, or hosted conversion — and none may be created by gating the open evidence/agent layer.

> **See also (2026-05-29):** [monetization-and-paying-segment.md](monetization-and-paying-segment.md)
> sizes the paying buyer (the hard-boundary air-gap/classified/sovereign self-hoster, not the
> cost-driven one) and details the monetization shape (open core + gated enterprise-ops + managed
> cloud + outcome-priced fixer). Licensing is **resolved: Apache-2.0, kept consistent — no
> relicensing** (operator, 2026-05-29); the note records the accepted trade-off (Apache gives the
> weakest hyperscaler-fork defense, so the moat leans on the corpus + cloud + best-operator position,
> not license copyleft). It also tightens the bear case: the paying base is a niche-within-a-niche,
> shrinking at the commodity end as FedRAMP-High SaaS + sovereign clouds absorb most regulated workloads.

This note consolidates the following previously-separate research files, each preserved in full below:

- `business-model-and-economics.md`
- `business-model-validation-ledger.md`

## Business Model and Economics

_Provenance: merged verbatim from `business-model-and-economics.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

This note closes a gap the [bear case](../decisions/risks-and-bear-case.md) flagged as a
top-severity, unsolved risk ("no monetization path for OSS self-hosted") and
answers the prompt's "economic/business implications" ask and strategic question
1 ("is this a company-sized opportunity?"). The [verdict](../decisions/go-no-go.md) is GO on
the *product*; this asks whether there is a *business*, and how it can exist
without betraying the operator's open-source and self-hosting ethos.

It is opinionated and current (licensing models rechecked 2026-05-25).
The companion [business model validation ledger](business-model.md)
defines the result rows and claim levels required before any of these seams can
be called validated.

### Is The Category Company-Sized?

Yes, the category is — the open-self-hosted slice is smaller and slower.

- Sentry's own Fair Source post says it had passed 100,000 cloud customers and
  $100M in annual revenue. That is enough primary evidence to treat error
  monitoring plus context as a company-sized category. Third-party ARR and
  vendor-share estimates can be useful context, but they are not primary-source
  proof and should not carry the thesis.
- But Sentry monetizes overwhelmingly through **cloud**, not self-hosting. The
  segment Parallax leads with — open, self-hosted, data-owned — is precisely the
  segment that is hardest to monetize. So "the category is big" does not imply
  "the open-self-hosted GTM is easy." It is a harder, slower path to a real but
  smaller company.

Honest framing: Parallax's wedge is a *slice* of a company-sized category,
entered through its least-monetizable door. That is survivable only if the free
tier drives the adoption that builds the schema/corpus moat, and value capture
comes from layers the self-hosting ethos does not forbid (below).

Primary sources checked on 2026-05-25: [Sentry Fair Source post](https://blog.sentry.io/sentry-is-now-fair-source/),
[Sentry FSL announcement](https://blog.sentry.io/introducing-the-functional-source-license-freedom-without-free-riding/),
[Grafana Enterprise docs](https://grafana.com/docs/grafana/latest/introduction/grafana-enterprise/),
[Grafana pricing](https://grafana.com/pricing/),
[OpenObserve pricing](https://openobserve.ai/pricing/),
[OpenObserve homepage](https://openobserve.ai/),
[OpenObserve enterprise features](https://openobserve.ai/docs/features/enterprise/),
[OpenObserve AI/MCP Enterprise recheck](../market/competitor-watch.md),
and [GitLab pricing](https://about.gitlab.com/pricing/). Secondary market
estimates such as private-company ARR breakdowns or card-spend vendor share
should be labeled as leads only.

### The Core Tension

The operator's stated values pull in two directions against revenue:

- "Open source is the key… anyone can open a pull request… agent-contributable."
  → maximize openness and embeddability.
- "I strongly prefer self-hosting, infrastructure ownership, low cost." → the
  product's lead users will run it themselves and not pay for the core.

A monetization model must therefore (a) keep the core genuinely open and
agent-contributable, and (b) capture value somewhere other than the open core —
**without gating the differentiator**.

### The Non-Negotiable: Do Not Gate The Agent/Evidence Layer

The [verdict](../decisions/go-no-go.md) and [market landscape](../market/landscape.md) found that
the entire Parallax wedge is that the **open + self-hosted + agent-native +
evidence-bundle** combination does not exist for free in one product — and that
OpenObserve's specific weakness is that it **gates its AI SRE agent behind an
Enterprise license**. SigNoz's agent-native MCP, by contrast, ships open.

Therefore the one monetization move Parallax must NOT make is the obvious one:
gating the agent surface, evidence graph, bundle format, or MCP/CLI context. That
is the exact gap Parallax exploits; closing it on ourselves would forfeit the
wedge. The open differentiator stays open. Revenue must come from elsewhere.

### Licensing Options

| Model | Example (2026) | Pro | Con for Parallax |
| --- | --- | --- | --- |
| Permissive (Apache-2.0/MIT) | many CNCF projects | Maximizes adoption, embedding, and agent/PR contribution — the exact flywheel that builds the schema/corpus moat. | No protection against a competitor running a Parallax cloud. (Low near-term threat for a small project.) |
| AGPL-3.0 | Grafana core, OpenObserve | OSI-open; network-copyleft blocks hyperscaler/SaaS free-riding. | Deters some corporate adoption and embedding; slightly at odds with "anyone can use/embed freely." |
| Fair Source / FSL | Sentry, Codecov | Non-compete now, converts to Apache/MIT after 2 years; protects the cloud business. | NOT OSI-open at t=0 — conflicts with the operator's "open source is the key / agent-contributable" ethos. |
| Open core | Grafana Enterprise | Proven revenue: gate enterprise ops features. | Dangerous if you gate the *differentiator*; safe only if you gate non-core ops features. |

**Recommendation:** license the core **Apache-2.0** to maximize the adoption and
contribution flywheel that is the moat-building mechanism, and require a
lightweight **CLA** from contributors so relicensing to AGPL stays *possible* if
SaaS free-riding ever becomes a real threat. Permissive-first is the right bet
while the priority is adoption, not defense; AGPL is the fallback lever, FSL is
rejected as ethos-incompatible at t=0.

### Where Value Capture Is Legitimate (Seams That Don't Gate The Wedge)

Four revenue seams, none of which gate the open evidence/agent differentiator:

1. **Managed/hosted Parallax.** The product self-hosters run for free; teams who
   want it but not the ops pay for a hosted version. This is how Sentry and
   Grafana actually make money. It does not betray self-hosting — self-hosting
   stays free and first-class; hosting is a convenience purchase.
2. **The fixer / agent-orchestration product.** Recall the boundary: Parallax is
   the open evidence engine; the **separate component that pulls evidence, drives
   a coding agent, and opens PRs** is where autonomous value is captured. This is
   a natural commercial layer *on top of* the open engine — it monetizes the
   outcome (fixes) without gating the evidence. Clean open-core seam. The
   [fixer component and outcome loop](../decisions/fixer-boundary.md)
   defines the technical contract and outcome records for this seam.
3. **Enterprise operations add-ons** that are not the differentiator: SSO/SAML,
   fine-grained RBAC, multi-tenancy, audit export/compliance, long-retention
   lifecycle management, backup/DR tooling, priority support/SLA. This is exactly
   what Grafana and OpenObserve charge for (OpenObserve's Self-Hosted Enterprise
   bundles SSO/RBAC/audit) — and crucially these are *ops* features, not the
   evidence/agent moat.
4. **Support, services, and certification** for teams running Parallax in
   production.

### What Stays Free Forever

To protect the wedge, these are permanently in the open core:

- Sentry-envelope + OTLP ingestion; the tiny and scalable self-host tiers.
- Deterministic grouping, correlation, the evidence graph.
- The open evidence-bundle schema and format.
- CLI and read-only MCP/HTTP context surface.
- Rust + frontend capture paths.

If a team can self-host the full evidence + agent-context capability for free,
the wedge holds and the schema spreads. Revenue is a tax on *scale, operations,
and outcomes*, not on *access to the differentiator*.

### Comparables (2026)

| Vendor | License | How it monetizes | Lesson for Parallax |
| --- | --- | --- | --- |
| Sentry | FSL / Fair Source (delayed open-source publication) | Cloud business; Sentry says it passed 100k cloud customers and $100M annual revenue. | Cloud is the real revenue; the license protects it. Parallax rejects FSL on ethos but copies "cloud is the business." |
| Grafana | AGPL core + proprietary Enterprise + Cloud | Enterprise docs list SAML/enhanced LDAP/protected roles, auditing, usage insights, recorded queries, Vault integration, and premium data sources; Cloud is separately priced. | Gate *ops* features, not the core engine. Good template. |
| OpenObserve | AGPL + Self-Hosted Enterprise | Pricing and Enterprise docs say Self-Hosted Enterprise is free up to `50 GB/day`, while the homepage FAQ says `200 GB/day`; Enterprise includes SSO/RBAC/audit/redaction, and the Enterprise plan lists AI SRE Agent, AI Assistant, and AI-powered observability. | Copy the ops gating; **do NOT copy the agent gating** — that is the weakness Parallax beats. Treat the exact free allowance as source-conflicted. |
| GitLab (reference) | Open core / buyer-based tiering | Pricing offers GitLab.com, Self-Managed, and Dedicated; paid tiers/add-ons include governance, security, compute/storage, and Duo Agent Platform credits. | Open-core works at scale, but tier boundaries are a constant fight. |

These rows are anchored to primary vendor pages where possible. Blog posts about
licensing history and third-party business estimates can remain useful leads,
but the durable business-model note should not depend on them for current
pricing, revenue, or tier claims.

### Honest Reality

- OSS infrastructure monetization is slow and hard; revenue typically lags
  adoption by years. Do not model a fast path.
- The self-hosting-first audience is structurally the least likely to pay; the
  payers are the subset that grows into enterprise ops needs, wants hosting, or
  wants the autonomous fixer. The free tier's job is reach and schema gravity,
  not revenue.
- This means **adoption is the leading metric, not revenue**, for the first
  phase — which is uncomfortable but consistent with the moat being the
  schema/corpus, not the code.

### How This Changes Earlier Conclusions

- The bear case's "no monetization path" risk is **narrowed, not eliminated**:
  there are legitimate seams (cloud, fixer, ops add-ons, support), but they all
  depend on adoption that is unproven. A2 now has a concrete
  [user interview and deployment intent gate](a2-user-demand.md);
  A3 now has a concrete
  [schema adoption and corpus moat gate](a3-schema-corpus.md).
  Payment, paid-pilot, hosted, fixer, enterprise-ops, and support claims must
  pass through the
  [business model validation ledger](business-model.md).
- It sharpens the boundary decision: the Parallax-stores / separate-fixer split
  is not only an architecture choice, it is the **primary value-capture seam** —
  another reason to keep that boundary clean.

### Falsification

- If, after meaningful adoption, neither hosting, the fixer, nor enterprise ops
  add-ons convert any team to paying, the business (not the product) is
  disproven — fold back to the bear case's NO-GO triggers.
- If adoption itself never materializes (bear case A2), monetization is moot.
  Validate that with the
  [user interview and deployment intent gate](a2-user-demand.md)
  before treating hosting, fixer, support, or enterprise ops as real seams.
- If interviews or conversion experiments show adoption interest without any
  budget, payment, support, hosted, fixer, or enterprise-ops signal, record that
  as `claim_failed` or `claim_expired` in the
  [business model validation ledger](business-model.md) and
  reopen the bear-case monetization risk.

### Relationship To Other Research

- [Verdict](../decisions/go-no-go.md) — company-sized question and the open-wedge thesis.
- [Risks and the bear case](../decisions/risks-and-bear-case.md) — the monetization and
  distribution risks this narrows.
- [Business model validation ledger](business-model.md) — the
  result contract and claim levels for turning plausible seams into measured
  value-capture claims.
- [User interview and deployment intent gate](a2-user-demand.md)
  — the A2 gate that determines whether adoption beyond the operator is real.
- [Schema adoption and corpus moat gate](a3-schema-corpus.md)
  — the A3 gate that determines whether schema/corpus gravity is real.
- [Market landscape](../market/landscape.md) — OpenObserve's agent-gating weakness
  that Parallax must not replicate.
- [Technical implementation concept](../architecture/implementation-concept.md) — the
  Parallax/fixer boundary that doubles as the value-capture seam.
- [Fixer component and outcome loop](../decisions/fixer-boundary.md) —
  separates the open evidence engine from the paid PR/fix orchestration seam.

### Bottom Line

There is a plausible business, but it is a slow open-source infrastructure
business, not a fast SaaS one. License the core Apache-2.0 (CLA to keep AGPL in
reserve), keep the evidence/agent differentiator open forever, and capture value
through hosting, the autonomous fixer, enterprise ops add-ons, and support only
after the validation ledger records specific payment or sustainability evidence.
The one fatal move would be gating the agent/evidence layer — that is the
competitor weakness Parallax exists to exploit. Adoption, not revenue, is the
metric that decides whether this becomes a company, until the ledger proves a
specific paid seam.

## Business Model Validation Ledger

_Provenance: merged verbatim from `business-model-validation-ledger.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

[Business model and economics](business-model.md) identifies
plausible value-capture seams: hosted Parallax, the separate fixer, enterprise
ops add-ons, support/services, and sponsorship. This ledger defines the missing
result contract for proving those seams without turning a plausible business
model into an unsupported claim.

The core rule is simple:

> Parallax may say it has plausible value-capture seams today. It may not say
> those seams are validated until redacted result rows show deployment,
> budget/payment, conversion, or paid-pilot evidence. The open evidence and
> agent-context layer must not be gated to create that evidence.

This ledger is a business claim-control artifact, not a pricing page.

### Current Source Snapshot

| Source | Current signal | Parallax implication |
| --- | --- | --- |
| [Sentry FSL announcement](https://blog.sentry.io/introducing-the-functional-source-license-freedom-without-free-riding/) and [Sentry Fair Source post](https://blog.sentry.io/sentry-is-now-fair-source/) | Sentry uses source-available licensing to protect a SaaS business while still permitting internal use and delayed open-source release; Sentry reported 100k+ cloud customers and $100M+ annual revenue in the Fair Source post. | Cloud revenue is a proven observability pattern, but FSL/Fair Source conflicts with Parallax's open-source-first ethos. Copy the hosted convenience seam, not the source-available license. |
| [Grafana licensing](https://grafana.com/licensing/) and [Grafana Enterprise license docs](https://grafana.com/docs/grafana/latest/administration/enterprise-licensing/) | Grafana keeps an open core while selling hosted and Enterprise capabilities such as premium plugins, advanced security, reporting, RBAC, and support through a license. | Ops and governance features are legitimate paid seams when they do not gate the core evidence engine. |
| [OpenObserve pricing](https://openobserve.ai/pricing/), [homepage](https://openobserve.ai/), [enterprise features](https://openobserve.ai/docs/features/enterprise/), and [AI/MCP Enterprise recheck](../market/competitor-watch.md) | OpenObserve monetizes cloud usage and enterprise/security/ops features, lists AI observability features in Enterprise contexts, and has source-conflicted public free Self-Hosted Enterprise allowance claims (`50 GB/day` in pricing/docs versus `200 GB/day` on the homepage FAQ). | OpenObserve reinforces the ops-feature seam, but Parallax should not copy AI/evidence gating because the open agent-context layer is the wedge. Treat self-hosted allowance figures as source-conflicted until reconciled. |
| [GitLab pricing](https://about.gitlab.com/pricing/) | GitLab publicly frames tiering around a buyer-based open-core model and supports GitLab.com, Self-Managed, and Dedicated offerings. | Open-core can scale, but tier boundaries need discipline; Parallax should tie paid tiers to buyer-owned ops/outcome problems, not the developer-facing evidence format. |
| [A2 interview evidence ledger](a2-user-demand.md), [schema adoption/corpus ledger](a3-schema-corpus.md), [fixer component](../decisions/fixer-boundary.md), and [fixer outcome ledger](../decisions/fixer-boundary.md) | The repo already has contracts for demand evidence, schema/corpus evidence, fixer boundaries, and fixer outcome rows. | Business validation should reuse those artifacts instead of inventing a parallel founder-memory process. |

### Claim Levels

Use exactly one current level in `claim_ledger.jsonl` for the business model.

| Level | Meaning | Minimum evidence |
| --- | --- | --- |
| `not_measured` | The seam is a strategic hypothesis only. | Design rationale and comparable-source snapshot. Current status. |
| `adoption_interest_signal` | Teams show concrete product/deployment interest, but no business signal yet. | A2 Level 3+ commitments or pilot/design-partner rows; no payment/budget claim allowed. |
| `deployment_intent_signal` | Target teams can plausibly deploy Parallax and expose enough evidence safely. | A2 pass/continue result, named deployment approval path by role, and no unresolved redaction blocker. |
| `hosted_payment_signal` | A team explicitly prefers hosted Parallax or managed ops over self-hosting. | Named buyer/sponsor role, budget category, and time-boxed next step for hosted or managed deployment. |
| `fixer_payment_signal` | A team may pay for the separate fixer/outcome workflow. | A1 positive or target failure-class proof, budget/sponsor signal for PR/fix orchestration or success workflow, and clear labeling of whether fixer outcome rows are proven or still pre-proof. |
| `enterprise_ops_signal` | A team may pay for SSO/RBAC/audit/retention/backup/compliance features. | Existing enterprise ops requirement, named approval path, and stated blocker that free core alone will not satisfy. |
| `support_services_signal` | A team may pay for support, implementation, certification, or consulting. | Named support owner, explicit operating need, and time-boxed next action for paid support/services. |
| `conversion_experiment_ready` | A seam is strong enough to test publicly with a landing page, hosted waitlist, paid pilot, or pricing conversation. | At least two same-seam payment signals, A2 deployment evidence, and no contradiction that the open core would need to be weakened. |
| `value_capture_validated` | At least one seam has converted into money or a high-confidence payment artifact. | Paid pilot, signed order/contract, paid support agreement, LOI with budget owner, or hosted conversion; result rows must identify seam, amount bucket, and status. |
| `claim_expired` | A prior business claim is stale. | Pricing/licensing/comparable changes, A2 result older than 90 days during discovery, or product scope changed materially. |
| `claim_failed` | The business claim failed for the measured period. | Adoption without any payment/sustainability signal, repeated no-budget outcomes, or requirement to gate the open evidence/agent layer to monetize. |

### Result Artifacts

Create these only when measurement begins:

| Artifact | Commit? | Purpose |
| --- | --- | --- |
| `docs/research/business-model-results.md` | Yes | Human-readable summary of current claim level, seam counts, conversion outcomes, contradictions, and decision. |
| `docs/research/business-model-runs/<run_id>/manifest.json` | Yes | Run metadata: date, research window, source versions, A2 result refs, pricing page refs, owner, and redaction policy. |
| `docs/research/business-model-runs/<run_id>/interview-payment-signal.jsonl` | Yes, redacted | Payment, budget, buyer, and sustainability rows derived from A2 calls. |
| `docs/research/business-model-runs/<run_id>/adoption-funnel.jsonl` | Yes | Public adoption funnel rows once there is a release: installs, pilots, active projects, schema integrations, churn. |
| `docs/research/business-model-runs/<run_id>/hosting-signal.jsonl` | Yes, redacted | Hosted/managed-deployment interest, trial, conversion, and churn rows. |
| `docs/research/business-model-runs/<run_id>/fixer-signal.jsonl` | Yes, redacted | Fixer paid-pilot, usage, outcome, and budget rows, linked to fixer outcome and agent-session linkage records. |
| `docs/research/business-model-runs/<run_id>/enterprise-ops-signal.jsonl` | Yes, redacted | SSO/RBAC/audit/export/backup/retention/compliance requirement and budget rows. |
| `docs/research/business-model-runs/<run_id>/support-services-signal.jsonl` | Yes, redacted | Support, services, certification, implementation, or consulting rows. |
| `docs/research/business-model-runs/<run_id>/conversion-experiment.jsonl` | Yes | Pricing-page, hosted waitlist, paid-pilot, or sales-call experiment rows. |
| `docs/research/business-model-runs/<run_id>/claim-ledger.jsonl` | Yes | Append-only business claim status rows. |
| `docs/research/business-model-runs/<run_id>/hashes.sha256` | Yes | Hashes for committed artifacts. |

Raw call notes, contact records, company names, recordings, sales emails,
payment details, contracts, and private screenshots stay outside the repo unless
the operator explicitly approves a redacted excerpt.

### Row Schemas

#### Payment Signal Row

```json
{
  "signal_id": "bm_pay_001",
  "source": "a2_interview|pilot|inbound|pricing_experiment|support_request",
  "source_ref": "A2-007",
  "research_date": "2026-05-25",
  "segment": "rust_self_hosted|small_saas_paid_observability|privacy_compliance|ci_flaky_test|agent_heavy|oss_infra_consultant|other",
  "seam": "hosted|fixer|enterprise_ops|support_services|sponsorship|unknown",
  "current_spend_bucket": "none|under_100_mo|100_999_mo|1k_9k_mo|10k_plus_mo|unknown",
  "buyer_role_class": "founder|engineering_manager|infra_owner|sre|security_privacy|procurement|unknown",
  "budget_path": "none|possible|role_named|owner_named|active_procurement|paid",
  "commitment_type": "none|follow_up|pricing_call|hosted_trial|paid_pilot|signed_loi|contract|paid",
  "amount_bucket": "none|under_500|500_1999|2k_9999|10k_plus|unknown",
  "timebox": "none|within_7_days|within_30_days|within_90_days|later|unknown",
  "evidence_class": "past_spend|budget_owner|procurement_step|payment|signed_document|contradiction",
  "sanitized_evidence": "Paraphrased fact with no company, person, domain, or secret.",
  "contradictions": []
}
```

#### Adoption Funnel Row

```json
{
  "funnel_id": "bm_adopt_001",
  "research_date": "2026-05-25",
  "channel": "github|docs|direct_interview|pilot|schema_integration|inbound|other",
  "stage": "visitor|star|clone|install|pilot|active_project|schema_integration|retained|churned",
  "segment": "unknown",
  "count": 0,
  "source_ref": "public_metric_or_redacted_internal_ref",
  "notes": "Counts are adoption evidence, not revenue evidence."
}
```

#### Seam Signal Row

Use this shape for hosted, fixer, enterprise ops, and support rows.

```json
{
  "seam_signal_id": "bm_seam_001",
  "research_date": "2026-05-25",
  "seam": "hosted|fixer|enterprise_ops|support_services",
  "source_ref": "A2-007|pilot_003|fixout_012",
  "need": "Managed ops, PR/fix workflow, SSO, audit export, support SLA, or similar.",
  "required_product_proof": ["a1_bundle_value", "a2_deployment", "a6_redaction", "fixer_outcome_when_seam_is_fixer"],
  "current_status": "pre_proof_interest|interest|trial|paid_pilot|converted|lost|churned",
  "amount_bucket": "none|under_500|500_1999|2k_9999|10k_plus|unknown",
  "blocking_reason": "none|no_budget|missing_feature|redaction|self_host_only|incumbent_contract|unknown",
  "sanitized_evidence": "Paraphrased, redacted signal.",
  "linked_outcome_refs": [],
  "linked_agent_session_linkage_refs": []
}
```

#### Conversion Experiment Row

```json
{
  "experiment_id": "bm_exp_001",
  "research_window": "2026-05-25/2026-06-25",
  "seam": "hosted|fixer|enterprise_ops|support_services",
  "surface": "pricing_page|hosted_waitlist|paid_pilot_offer|support_offer|sales_call",
  "audience": "target slice or public traffic source",
  "offer": "Redacted public offer or paid-pilot shape.",
  "exposures": 0,
  "qualified_actions": 0,
  "payments_or_signed_commitments": 0,
  "conversion_rate": null,
  "decision": "continue|revise|stop|inconclusive",
  "notes": "Do not count unqualified curiosity as qualified action."
}
```

#### Claim Ledger Row

```json
{
  "claim_id": "bm_claim_001",
  "research_date": "2026-05-25",
  "level": "not_measured",
  "valid_until": "2026-08-23",
  "seams_with_signal": [],
  "seams_validated": [],
  "supporting_artifacts": [],
  "contradictions": [],
  "allowed_wording": "Parallax has plausible, unvalidated value-capture seams.",
  "forbidden_wording": "Parallax has proven monetization.",
  "decision": "continue_research"
}
```

### Counting Rules

- Compliments, stars, waitlist joins, newsletter signups, and "keep me posted"
  do not count as payment signal.
- "Would pay" is weak evidence unless paired with a buyer role, budget category,
  amount bucket, and time-boxed next step.
- Free self-hosted adoption is adoption evidence, not revenue evidence.
- A hosted signal is not a fixer signal; a fixer signal is not enterprise ops
  signal. Count seams separately.
- A seam cannot be `value_capture_validated` by gating the open evidence graph,
  bundle schema, CLI/API context surface, or read-only MCP context.
- Fixer monetization cannot be claimed before the relevant A1/fixer outcome
  rows show that bundles improve or govern the target failure class. If a team
  expresses willingness to pay before `agent_session_linkage_pass`, evidence
  citation, review/CI, and recurrence rows exist, record it as
  `pre_proof_interest`, not validated fixer value.
- A paid fixer pilot counts as `value_capture_validated` only when the paid
  agreement names the measured outcome window and links to fixer outcome rows.
  Payment for custom fixture setup, implementation, or evaluation help counts
  as support/services unless the fixer outcome ledger also passes for the paid
  scope.
- Enterprise ops monetization cannot be claimed from generic "we need SSO"
  feedback unless the team also has a deployment path and budget/sponsor role.
- Support/services monetization can count before product maturity, but it must
  be labeled as services revenue, not proof of product-led SaaS conversion.
- If a team only wants free OSS and has no budget or sustaining action, record
  it as adoption evidence plus monetization contradiction.
- Any result older than 90 days during discovery becomes stale unless renewed by
  active deployment, paid pilot, contract, or current conversion data.

### Initial Results Template

When measurement begins, create `docs/research/business-model-results.md`:

```markdown
# Business Model Results

Research window:
Last updated:
Current claim level: not_measured

## Gate Snapshot

| Metric | Current | Threshold for next claim | Status |
| --- | ---: | ---: | --- |
| A2 Level 3+ deployment/data commitments | 0 | >=4 | Pending |
| A2 Level 5 sustainability signals | 0 | >=2 | Pending |
| Hosted payment signals | 0 | >=2 for experiment-ready | Pending |
| Fixer payment signals | 0 | >=2 plus A1/fixer proof | Pending |
| Enterprise ops signals | 0 | >=2 with buyer role | Pending |
| Support/services signals | 0 | >=1 paid or time-boxed | Pending |
| Paid pilots/contracts/LOIs | 0 | >=1 for value-capture validated | Pending |

## Signals By Seam

## Contradictions And Lost Reasons

## Current Allowed Wording

## Decision
```

### Product Wording Rules

| Evidence state | Allowed wording | Forbidden wording |
| --- | --- | --- |
| `not_measured` | "Plausible value-capture seams: hosted, fixer, enterprise ops, support." | "Validated business model", "proven monetization", "company-sized revenue path." |
| A2 adoption but no payment | "Adoption interest exists; business model remains unvalidated." | "Users will pay", "revenue follows adoption." |
| Payment signal rows | "Early payment/budget signals for [specific seam]." | "Validated revenue" unless converted. |
| Conversion experiment ready | "Testing paid [specific seam] with qualified teams." | "Product-market fit", "sales motion proven." |
| Value capture validated | "At least one tested seam converted into [paid pilot/contract/LOI bucket]." | Broad claims about all seams or the whole market. |
| Claim failed | "The measured period produced adoption without sustaining revenue." | Any live business-validity claim. |

### Refresh Triggers

Reopen this ledger when:

- 90 days pass without fresh A2/payment/conversion data during discovery;
- Sentry, Grafana, OpenObserve, GitLab, or a close self-hosted competitor changes
  licensing, tiering, hosted pricing, or AI/evidence gating materially;
- Parallax changes licensing, starts hosted distribution, ships a fixer, or adds
  enterprise ops features;
- A2 interviews contradict the current monetization thesis;
- any value-capture seam would require gating the open differentiator.

### Relationship To Other Research

- [Business model and economics](business-model.md) names the
  value-capture seams; this ledger controls when they become measured claims.
- [Risks and the bear case](../decisions/risks-and-bear-case.md) keeps "no monetization path"
  open until this ledger records payment or sustainability evidence.
- [User interview and deployment intent gate](a2-user-demand.md)
  and [A2 interview evidence ledger](a2-user-demand.md) provide the
  first source rows for deployment and budget evidence.
- [Fixer component and outcome loop](../decisions/fixer-boundary.md)
  defines the paid fixer seam.
- [Fixer outcome ledger](../decisions/fixer-boundary.md) defines the specific outcome
  rows needed before the fixer seam can be commercialized or counted as a
  validated value-capture claim.
- [Schema adoption and corpus moat gate](a3-schema-corpus.md)
  and [A3 schema adoption and corpus ledger](a3-schema-corpus.md)
  determine whether adoption/corpus gravity exists before broad business claims.
- [Self-hosted simplicity ledger](self-hosted-simplicity.md) provides the
  operational evidence that the free/open core is attractive enough to seed the
  funnel without hiding the differentiator behind a paid tier.

### Bottom Line

Parallax's business model is currently a disciplined hypothesis: adoption first,
paid seams later. The repo can keep the "plausible business" conclusion only if
it also keeps this ledger honest about what is still unmeasured. A real business
claim starts when redacted rows show budget, conversion, paid pilots, support
agreements, or contracts for a specific seam without weakening the open evidence
and agent-context layer.
