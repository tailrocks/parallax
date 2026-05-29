# Monetization and the Paying Segment

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-29

## Purpose

Answers the #1 *business* gate the [skeptical re-assessment](../decisions/skeptical-reassessment-2026-05.md)
raised and [research-agenda](../research-agenda.md) item 2: **open self-hosted looks structurally
non-paying — so is there a paying segment, and what is the product that captures it?** Extends
[business-model.md](business-model.md) (the general economics) with a sized segment + a concrete
monetization shape grounded in 2026 primary sources.

> **Conclusion: there is a paying segment, but it resolves a paradox — it is NOT the cost-driven
> self-hoster (who self-hosts to escape SaaS bills and won't pay). It is the *hard-boundary*
> self-hoster who legally cannot use multi-tenant SaaS** — defense/intel at IL6/classified/air-gap,
> OT/critical-infra air-gapped islands under NIS2, CLOUD-Act-averse EU sovereignty hardliners, and
> finance/healthcare that geo-fence raw telemetry. These buyers **demonstrably pay** (Grafana
> Enterprise self-hosted ~$25k–150k/yr ACV; Elastic built Cloud Connect to protect a regulated
> on-prem base; paid air-gapped Splunk-on-SIPRNet and GitLab gov SKUs) **and prefer open source**
> (77% important / 61% essential, Grafana 2026 survey). The viable monetization is the survivors'
> playbook, **planned up front, not bolted on**: **Apache-2.0 for the core, kept consistent — no
> relicensing** (operator decision, 2026-05-29; accept the weaker fork-defense and lean on the
> corpus + managed cloud + best-operator position as the moat, not license copyleft), a
> production-complete open core (including evidence-bundle *generation*) + a
> **gated enterprise-ops module** + **managed cloud as the primary
> revenue motion** + an **outcome-priced fixer** add-on. The honest caveat: this paying base is a
> **niche-within-a-niche and shrinking at the commodity end** as FedRAMP-High SaaS and in-region
> sovereign clouds absorb most "regulated" workloads — so this **tightens, does not remove**, the
> bear case's distribution/monetization risk. Drop the "open self-hosted *is* the business" framing.

## 1. The paying segment (resolving the "won't pay" paradox)

There are two different self-hosters, and only one pays:

- **Cost-driven self-hoster — does NOT pay.** Self-hosts to escape Datadog bill-shock and absorbs the
  ops labor; the vendors' own ROI math defines this user by *not paying a vendor*. SaaS-only adoption
  is in fact *growing* (Grafana survey: 10%→15%→17% over 2024–2026). This is the niche the skeptical
  re-assessment correctly flagged as non-paying.
- **Hard-boundary self-hoster — PAYS, because a compliance boundary legally forbids multi-tenant SaaS:**
  - **Defense/intel:** the durable moat is **IL6 / classified / SIPRNet / SCIF / air-gap**, *not*
    FedRAMP (commercial SaaS now reaches FedRAMP High / IL5). IL6 is "a separate operational reality…
    traditional multi-tenant SaaS boundary models cannot be applied"; classified runs air-gapped with
    no vendor telemetry and no phone-home control plane. (Splunk is deployed on SIPRNet/NIPRNet for DoD,
    FOC ~June 2026.)
  - **OT / critical infrastructure** under **NIS2** (21/27 EU states transposed by May 2026; fines to
    €10M/2% turnover) — air-gapped OT/SCADA islands standard cloud tools cannot reach.
  - **EU sovereignty hardliners** where the **US CLOUD Act vs GDPR** conflict pushes data off
    US-controlled SaaS (NIS2, EU Data Act, Schrems II).
  - **Finance / healthcare** that centralize dashboards in cloud but **geo-fence raw logs/telemetry**
    on-prem (PCI-DSS, HIPAA, data-residency).

**They pay, and they prefer open:** Grafana Labs (~$400M ARR, 7,000+ customers) sells a self-hosted
**Enterprise** stack at a reported ~$25k floor to ~$150k/yr, explicitly citing public-sector + finance
"air-gapped security and compliance" as higher-ARPU verticals; Elastic's self-managed subscription
strength is attributed to "customers preferring to keep critical data within their control, especially
in regulated industries," and it built **Cloud Connect** for exactly them; Splunk and GitLab sell paid
air-gapped/gov SKUs. **77% of the 2026 Grafana survey call open source important, 61% essential** —
favourable for an open-Rust positioning.

**Rough size:** on-prem + hybrid is ~31% of a ~$3.35B 2026 observability market ⇒ a **~$1.0–1.1B
on-prem/hybrid slice** (estimated); the *compliance-only, non-cost* subset is a fraction —
**likely low hundreds of $M (asserted, no source isolates it)** — riding a large sovereign-cloud
tailwind (Gartner: sovereign-cloud IaaS $80B in 2026, +35.6% YoY).

**The squeeze (the strongest skeptical caveat):** the addressable base is being eaten from above.
**Datadog reached FedRAMP High (2026-05-27); Grafana Federal Cloud is FedRAMP High + IL5 as managed
SaaS; Elastic Cloud Hosted is FedRAMP High; AWS European Sovereign Cloud went GA (2026-01-15).** So
*most* regulated workloads up to CUI/IL5 and EU-residency now have a compliant SaaS path — leaving the
durable self-host core as the **true air-gap / classified / sovereignty-hardliner** plus the
"keep raw telemetry in-house" geo-fencers. Real and defensible, but small and not growing at the
commodity end.

## 2. Monetization shape (the survivors' playbook, applied)

1. **License: Apache-2.0 for the core, kept consistent — RESOLVED (operator, 2026-05-29).** The
   relicensing graveyard is the reason consistency matters — Elastic→SSPL forked **OpenSearch**,
   HashiCorp→BSL forked **OpenTofu**, Redis→SSPL forked **Valkey**, each within weeks and
   hyperscaler-backed, and re-adding AGPL later did **not** win users back. The operator has chosen
   **Apache-2.0 and will not relicense**, prioritizing maximal openness and keeping the evidence/agent
   differentiator maximally adoptable (the corpus precondition). **Accepted trade-off:** Apache-2.0
   gives the *weakest* defense if a hyperscaler reselling Parallax as managed SaaS is the feared
   outcome (no copyleft source-disclosure deterrent, unlike Grafana's AGPLv3). **Mitigation:** the moat
   is the **failure/fix-outcome corpus + managed cloud + being the best operator of the product**, not
   license copyleft — so the defense does not depend on the license. A CLA may be kept for contribution
   provenance, but **not** as a path to a future relicense (consistency is the operator's stated
   preference).
2. **Keep open and production-complete for one team:** full Sentry-envelope + OTLP ingest, storage,
   query, dashboards/alerting, single-node/small-cluster, and **evidence-bundle generation + the open
   schema** (the wedge must be in the open core, or adoption — the corpus's precondition — never comes).
3. **Gate a separately-licensed enterprise-ops module** (`ee/`-style, SigNoz model): SAML + SCIM SSO,
   advanced/custom RBAC, audit logs, multi-tenancy, HA/scale-out clustering, long retention, federated
   search, PII-redaction policy, ingest-cost governance, SLA support. **Keep basic OIDC open** to dodge
   the worst "SSO tax" backlash (sso.tax). This is the de-facto enterprise set every comparable gates
   without crippling the OSS core.
4. **Primary revenue = managed cloud, usage-metered on ingest.** Cloud is the growth engine for every
   OSS-first peer (Grafana Cloud growing ~2× faster than self-managed; **Elastic Cloud ≈ 49% of total
   revenue and rising**; SigNoz/OpenObserve are usage-metered, cloud-first). For the hard-boundary
   buyers who *can't* use cloud, sell **enterprise self-managed + BYOC license + support** as the
   high-ACV tail.
5. **Fixer = premium add-on, priced per successful outcome** (Datadog Bits model: ~$25–30 per
   *conclusive* investigation, inconclusive not billed). Outcome pricing **de-risks an unproven
   autonomous fixer** and converts better early than a flat seat fee; graduate to per-contributor flat
   (Sentry Seer $40/active-contributor/mo) once accuracy is trusted. Note this lives in the **separate
   fixer component** ([../decisions/fixer-boundary.md](../decisions/fixer-boundary.md)), not Parallax core.
6. **Conversion triggers to design for:** scale/ops burden → push to cloud; compliance → the gated tier;
   production-criticality → support SLA. Obsess over time-to-value (<5 min) and 48h activation
   (generic PLG data: a 3–5× conversion multiplier; no observability-specific OSS→paid rate is published).
7. **Single biggest risk: hyperscaler capture** (a cloud vendor reselling the OSS as managed SaaS).
   AGPL source-disclosure copyleft is the deterrent; a future relicense is off the table.

## 3. Strategic resolution — what it means for Parallax

- **The paying product is NOT "self-hosted OSS."** It is **open core + managed cloud + enterprise
  self-managed/support + outcome-priced fixer** — the same shape the survivors converged on. The open
  self-hosted core is simultaneously the **funnel** (adoption → corpus) and the **wedge**
  (air-gap / no-phone-home, which no incumbent offers — see
  [competitor-watch.md](../market/competitor-watch.md)). Plan this from day one; do not pretend
  self-hosted alone is the business.
- **The wedge and the paying segment align on one property: no-phone-home / air-gap.** That is the only
  observability-agent-evidence property no incumbent (Grafana on-prem phones cloud; Seer cloud-only;
  Datadog SaaS) offers, and it is exactly what the hard-boundary paying segment requires. Lead with it.
- **Honest risk update:** this *tightens* the bear case. The paying base is a niche-within-a-niche,
  shrinking at the commodity end; managed cloud (the primary revenue motion) partly contradicts the
  self-hosted ethos; and all of it is still **gated on A1** — if the bundle does not beat raw context
  ([runtime-dependence-and-raw-baseline.md](a1-bundle-value/runtime-dependence-and-raw-baseline.md)),
  there is no premium to charge for in either tier. Sequence: **prove A1, then build the cloud +
  enterprise tier for the air-gap/compliance segment.**

## Sources (primary, 2026)

- DoD IL6/air-gap reality: <https://www.secondfront.com/resources/blog/understanding-dod-cloud-computing-impact-levels/> · <https://learn.microsoft.com/en-us/azure/compliance/offerings/offering-dod-il6> · Splunk on SIPRNet/NIPRNet (Cisco): <https://www.cisco.com/c/en/us/products/collateral/security/simplifying-comply-connect-dod-stakeholders-so.html>
- NIS2 enforcement 2026: <https://www.6clicks.com/resources/blog/nis2-enforcement-2026-critical-infrastructure-government-and-defence-cant-wait> · Cisco Sovereign Critical Infrastructure (2026-04): <https://news-blogs.cisco.com/emea/2026/04/20/cisco-sovereign-critical-infrastructure-from-customer-needs-to-delivery/>
- They pay: Grafana Enterprise ACV — <https://sacra.com/c/grafana-labs/> · <https://costbench.com/software/business-intelligence/grafana-enterprise/> · Elastic regulated on-prem + Cloud Connect (Q3/Q4 FY26): <https://www.fool.com/earnings/call-transcripts/2026/05/28/elastic-estc-q4-2026-earnings-transcript/> · GitLab self-managed/air-gapped gov: <https://about.gitlab.com/blog/why-gitlab-self-managed-is-the-perfect-partner-for-the-public-sector/>
- Market sizing: observability split (Mordor): <https://www.mordorintelligence.com/industry-reports/observability-market> · sovereign-cloud IaaS $80B (Gartner): <https://www.gartner.com/en/newsroom/press-releases/2026-02-09-gartner-says-worldwide-sovereign-cloud-iaas-spending-will-total-us-dollars-80-billion-in-2026> · open-source preference (Grafana survey): <https://grafana.com/press/2026/03/18/grafana-labs-4th-annual-observability-survey-reveals-a-field-at-a-crossroads-ai-economics-complexity-and-the-enduring-power-of-open-source/>
- The squeeze: Datadog FedRAMP High (2026-05-27): <https://www.globenewswire.com/news-release/2026/05/27/3302010/0/en/datadog-and-carahsoft-announce-datadog-s-achievement-of-fedramp-high-certification-for-its-observability-and-security-platform.html> · Grafana Federal Cloud (FedRAMP High/IL5 SaaS): <https://grafana.com/products/fedramp-federal-cloud/> · AWS European Sovereign Cloud GA: <https://www.datadoghq.com/about/latest-news/press-releases/eu-region-germany/>
- Monetization playbook: Grafana AGPLv3 (no fork): <https://grafana.com/blog/grafana-loki-tempo-relicensing-to-agplv3/> · Elastic Cloud ~49% (Q2 FY26): <https://www.businesswire.com/news/home/20251119331264/en/Elastic-Reports-Second-Quarter-Fiscal-2026-Financial-Results> · SigNoz `ee/` gating + cloud pivot: <https://signoz.io/pricing/> · OpenObserve enterprise set + 50 GB/day free: <https://openobserve.ai/pricing/> · SSO tax: <https://sso.tax/> · relicensing forks (OpenTofu/Valkey/OpenSearch): <https://opentofu.org/blog/opentofu-announces-fork-of-terraform/> · <https://www.infoq.com/news/2025/05/redis-agpl-license/>
- Fixer pricing: Sentry Seer ($40/active-contributor): <https://sentry.io/product/seer/> · Datadog Bits AI SRE (per-investigation): <https://www.datadoghq.com/product/ai/bits-ai-sre/>

> Unconfirmed / flagged: the compliance-only segment $ size is estimated, not isolated by any source;
> Grafana's cloud-vs-self-managed split and OSS→paid conversion rates are not published; several
> pricing floors are from secondary teardowns.
