# Monetization and the Paying Segment

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-29 · **Desk recheck 2026-07-17 (pass 54 + 94 + 106 + 117 + 119 + 131 + 132)**

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

### Pass 54 desk recheck (2026-07-17) — what still holds / what moved

| Claim from this note | Recheck | Evidence class |
| --- | --- | --- |
| Open self-host OSS core is structurally non-paying; survivors use cloud + EE gates | **Holds** | Concurrent market pins (pass 41–48): OpenObserve Enterprise free ≤50GB/day then gate; SigNoz Cloud + EE; Grafana Cloud Pro usage; Bugsink free self-host + paid Hosted | primary pricing pages in competitor deep-dives |
| Hard-boundary air-gap still lacks Seer-class self-hosted AI | **Holds** | [Sentry self-hosted develop docs](https://develop.sentry.dev/self-hosted/) still list **"Seer and other AI & ML features… currently closed source"** among unavailable items (fetched 2026-07-17) | primary docs |
| Datadog FedRAMP High squeezes mid-tier "regulated" | **Holds** | Pass 42 competitor note: FedRAMP High on US1-FED (2026-05-06) | prior primary; not re-fetched this pass |
| Paying segment exists but niche | **Unchanged theory** | No new primary isolation of air-gap ACV this pass; **A2 interviews still open** | desk only |

### Pass 94 desk recheck (2026-07-17) — survivor cloud/EE gates (primary re-fetch)

| Survivor pattern | Live primary (2026-07-17) | Holds? |
| --- | --- | --- |
| **OpenObserve** gates AI + SDR to Enterprise; free EE allowance | [openobserve.ai/pricing](https://openobserve.ai/pricing/): Cloud Professional **$0.50/GB ingest**, **$0.01/GB query**; Enterprise lists **Sensitive Data Redaction**, **AI-Powered Observability**, **Incident Management & AI SRE Agent**, **AI Assistant**, SSO/RBAC/audit. FAQ **twice**: Self-Hosted Enterprise free forever **≤ 50 GB/day** (above → sales). | **Holds.** Prior business-model note of **50 vs 200 GB FAQ conflict** appears **resolved toward 50 GB** on this page (both FAQ answers say 50). Treat **50 GB/day** as current primary; mark any residual 200 GB cite **stale**. |
| **SigNoz Noz** cloud-gated AI teammate | [signoz.io/docs/ai/noz](https://signoz.io/docs/ai/noz/): page tagged **`SigNoz Cloud`**; last updated **2026-06-29**; distinguishes **Noz** (in-UI) vs **MCP Server** (external agents). No self-host Noz install path on this doc. Aligns pass 59/77 "Noz = Cloud". | **Holds** (Cloud product surface; MCP remains the OSS/self-host agent path). |
| **Sentry Seer** not on self-host | develop.sentry.dev/self-hosted still excludes Seer (pass 77/87 reconfirm). | **Holds** |
| **Traceway** MIT full-box + cheap managed cloud | Pass 84: public Free→Enterprise cloud tiers; self-host free. | **Holds** — cloud is convenience revenue, not open-core agent gate. |
| Hard-boundary ACV size | Still **not** isolated by a primary source this pass. | **A2 interviews still open** |

**Nuance (not a falsification):** OpenObserve lists AI features as **"free during preview"** with **20 credits** on Cloud — that is a **preview meter**, not "AI forever free in OSS core." Self-host Enterprise still packages AI SRE + Assistant with the EE gate above free GB.

**Falsify pass-54/94 holds:** Sentry ships self-hosted Seer/open AI stack; SigNoz ships **offline Noz** free in Community; OpenObserve moves **AI SRE + SDR** fully into free OSS core *and* ships portable redacted evidence bundles + outcome records; or a peer ships that combination free. **Does not falsify:** more SaaS FedRAMP/IL5 — already priced into the squeeze.

**Implication:** monetization shape (Apache core + EE ops + managed cloud + outcome-priced fixer;
**do not** gate the evidence-bundle differentiator the way OpenObserve gates AI SRE) **still the
least-bad desk design**. Empirical A2 (interviews) remains the only way to size the hard-boundary
buyer; desk rechecks cannot close A2.

### Pass 106 (2026-07-17) — Grafana Cloud pricing shape (comparison #2)

Live [grafana.com/pricing](https://grafana.com/pricing/) (usage-based, multi-signal
meters; extract is **order-of-magnitude**, not a quote):

| Surface | Primary signal (2026-07-17) |
| --- | --- |
| **Cloud Free** | Limited free tier (e.g. metrics/logs allowances cited on page — free + community support) |
| **Cloud Pro (self-serve)** | **From ~$19/mo platform fee + usage** (examples on page: e.g. **10k active series** then PAYG; **50 GB** log ingest included then PAYG at ~**$0.05/GB** class; host hours ~**$0.07/host-hour** class; users ~**$8/active user** class). Volume discounts on series/GB/users. |
| **Enterprise plugins** | Paid add-on on Cloud Pro PAYG (page cites ~**$55/active user/mo** class with volume tiers) |
| **Enterprise (full service)** | **Starts at ~$25,000/year** spend commit (matches self-managed EE floor order-of-magnitude used elsewhere in this note) |

**Parity with Parallax monetization design:** Grafana sells **usage Cloud** +
**ops/EE add-ons** + high-ACV Enterprise — **not** the open dashboard engine as
the paid product. Reinforces survivors' playbook (pass 54/94). **A2 still open**
for whether hard-boundary buyers pay *Parallax* at EE/cloud prices.

**Falsify:** Grafana puts core observability (or AI assistant offline) only behind
a seat tax that makes free OSS self-host non-viable for all but hobby — already
partially true for Cloud, not for OSS Grafana CE.

### Pass 117 (2026-07-17) — SigNoz Cloud pricing shape (comparison #2)

Live [signoz.io/pricing](https://signoz.io/pricing/) (usage-based, no host/seat
meters; extract is **order-of-magnitude**, not a quote):

| Surface | Primary signal (2026-07-17) |
| --- | --- |
| **Community (self-host)** | **$0** software; self-managed ops |
| **Teams Cloud** | **$49/mo** base (page still shows struck **$199** → **$49** promo framing) + usage: logs/traces **~$0.30/GB ingested**; metrics **~$0.10 / million samples**; retention tiers 15d–1yr (logs/traces) |
| **Enterprise** | Custom; page cites **starts at ~$4,000/mo** class (HIPAA/BAA, volume discounts, dedicated support; Cloud or self-managed EE options) |

**Cross-check with product peer note** [parallax-vs-signoz.md](../market/competitors/parallax-vs-signoz.md)
(pass 101): same rate card; **Noz = Cloud only**; MCP available to self-host.
Reinforces survivors' playbook: **free OSS core + usage Cloud + EE ops/compliance**
— not selling the agent/evidence differentiator as the only paid gate (Noz is
Cloud AI, not the open MCP path).

**Parity for Parallax design:** managed Cloud as primary revenue motion; EE for
ops/compliance; keep evidence-bundle generation open (do **not** copy Noz-style
*in-product AI only on Cloud* for the *core context artifact* — MCP over open
telemetry is the OSS peer pattern).

**Falsify:** SigNoz moves Noz fully offline free in Community; or Teams Cloud
drops below free-self-host economics so hard that OSS self-host dies (ops cost
is separate).

### Pass 119 (2026-07-17) — OpenObserve Cloud/EE pricing (comparison #2)

Live primary re-fetch of [openobserve.ai/pricing](https://openobserve.ai/pricing/)
(Cloud / Self Hosted tabs; extract is **order-of-magnitude**, not a quote). Completes
the Grafana (pass 106) + SigNoz (pass 117) + OpenObserve triangle for survivor
usage-cloud pricing.

| Surface | Primary signal (2026-07-17) |
| --- | --- |
| **OSS Community (self-host)** | **$0**, no ingest caps on AGPL core (per FAQ + downloads framing) |
| **Cloud Professional (PAYG)** | Ingest **$0.50/GB** (page notes \*includes **~30% annual commitment discount**); query **$0.01/GB**; metrics ret. **15 months**; non-metrics (logs/traces/etc.) **30 days**; extra non-metrics ret. **$0.02/GB per +30 days**; **unlimited users** / no seat or host meters; **14-day free trial** (no card) |
| **Cloud Enterprise** | Custom; lists **Sensitive Data Redaction**, **AI-Powered Observability**, **Incident Management & AI SRE Agent**, **AI Assistant**, pipelines, audit, SSO/RBAC, BYOC, SLAs, volume discounts |
| **Self-Hosted Enterprise** | FAQ (twice): **free ≤ 50 GB/day** ingestion; includes SSO, RBAC, federated search, query/workload QoS, audit trail, **sensitive data redaction**; **above 50 GB/day or paid support → sales** |
| **AI meter (Cloud)** | FAQ: AI features **free during preview** with **20 credits** (AI SRE Agent + AI Assistant); not “AI forever free in OSS core” |

**Holds vs pass 94:** unit rates, EE feature list, and **50 GB/day** free self-host
Enterprise allowance are **unchanged**. No reappearance of the historical **200 GB**
FAQ conflict on this page.

**Parity vs peers (same day desk):**

| Vendor | Cloud entry shape | AI / agent gate | Self-host EE gate |
| --- | --- | --- | --- |
| **OpenObserve** | **$0.50/GB** ingest + **$0.01/GB** query | EE list + Cloud **preview credits** | Free ≤**50 GB/day** then sales |
| **SigNoz** (pass 117) | **$49/mo** + **~$0.30/GB** logs/traces | **Noz = Cloud**; MCP open | Community free; EE custom |
| **Grafana Cloud** (pass 106) | **~$19/mo** + multi-signal usage | Assistant needs Cloud LLM backend | EE high-ACV / plugins |

**Implication for Parallax monetization design:** OpenObserve is the **nearest
architecture peer** *and* a clean **usage-Cloud + EE ops/AI gate** template.
Parallax should still **not** gate the portable redacted evidence bundle the way
O2 gates AI SRE / SDR — keep that differentiator open for adoption (A1 corpus
precondition). Cloud usage + EE ops/compliance remains the least-bad desk shape.
**A2 interviews still open** (no ACV isolation this pass).

### Pass 216 (2026-07-18) — triangle primary re-scrape (no interviews)

Live HTML scrape of the three public pricing surfaces (order-of-magnitude; not
quotes). **Desk playbook holds**; no material price-shape flip.

| Vendor | Primary signal (pass 216) | vs prior pin |
| --- | --- | --- |
| **Grafana Cloud** | Free tier; **Pro ~$19/mo + usage** (includes e.g. **10k active series** class messaging); Advanced/enterprise **~$25k/yr spend commit** class still visible | **Holds** pass 106 |
| **SigNoz Cloud** | Teams still **$49/mo** (struck **$199** promo framing) + **~$0.30/GB** logs/traces class; Enterprise **~$4000** class | **Holds** pass 117 |
| **OpenObserve** | Cloud ingest **$0.50/GB** + query **$0.01/GB**; Self-Host Enterprise free ≤**50 GB/day** still present | **Holds** pass 94/119 |

**Still cannot close A2:** zero interview rows (pass 214). Pricing survivors still
point to **usage cloud + EE gates**, not pure free OSS self-host as paying product.

**Falsify:** OpenObserve moves AI SRE + SDR into free unlimited OSS core *and*
ships portable redacted evidence bundles + outcome records; or public Cloud rates
collapse so that managed cloud is no longer a viable peer revenue motion (unlikely
from this re-fetch).

### Pass 131 (2026-07-17) — Datadog Bits / AI Credits pricing (fixer reference)

Live primary: [datadoghq.com/pricing/?product=ai-credits](https://www.datadoghq.com/pricing/?product=ai-credits#products)
(fetched 2026-07-17).

| Meter | Primary signal |
| --- | --- |
| **AI Credits (bundle)** | Starting **$500 / 500 credits / month** (annual billing note on page) |
| **On-demand** | **$1.30 per credit** |
| **Bits Investigate** (autonomous investigation) | Est. **~6.5 credits** per use (page table; “average… may vary”) |
| **Bits Chat** message | ~**0.5** credits |
| **Bits Code** fix | ~**5** credits |
| **Bits Agent Builder** run | ~**3** credits |
| **Rollover** | Unused credits **do not** roll over (reset monthly) |

**Implied investigation unit cost (order-of-magnitude, not a quote):**

- Annual bundle: ~6.5 × ($500/500) ≈ **~$6.50 / investigation** average
- On-demand: ~6.5 × $1.30 ≈ **~$8.45 / investigation** average

**Stale claim correction:** older monetization prose used **~$25–30 per conclusive
investigation** (secondary teardowns / prior Bits packaging). **Do not treat that
as current primary.** Live packaging is **shared AI Credits** across Bits Chat /
Investigation / Code / Agent Builder — closer to a **compute/work unit** than a
binary “conclusive-only” SKU. Secondary blogs still cite “$500 for 20 investigations”
(~$25) as narrative; that may describe older or marketing math, not the credit table
above.

**Parity for Parallax fixer design:** outcome- or work-unit metering remains a
valid reference class vs Sentry Seer **per-contributor** seats. Prefer **primary
credit math** when comparing unit economics; keep inconclusive-not-billed as a
*product design option*, not a Datadog primary fact unless re-verified in Bits
billing docs.

**Falsify:** Datadog publishes a different exclusive per-investigation SKU that
replaces AI Credits for Bits Investigation only.

### Pass 132 (2026-07-17) — Sentry Seer seat pricing (fixer reference)

Live primaries: [sentry.io/pricing](https://sentry.io/pricing/) (marketing) +
[docs.sentry.io/pricing/#seer-pricing](https://docs.sentry.io/pricing/#seer-pricing)
(billing detail). Completes the **Seer seat** vs **Bits credits** (pass 131)
fixer-model pair.

| Surface | Primary signal (2026-07-17) |
| --- | --- |
| **Marketing page** | Seer listed as add-on: **$40/active contributor/month** (Team / Business / Enterprise); “subscription required” on Team feature list |
| **Current Seer SKU** | **$40 / active contributor / month** on Team **and** Business (same rate) |
| **Active contributor definition** | User who makes **≥2 PRs** to a **Seer-Enabled** repository (repo connected + ≥1 Seer feature enabled). Counts **reset monthly** |
| **GitHub counting** | Repo members counted except GitHub bots marked `[bot]` |
| **GitLab counting** | All group members counted (even with Autofix/bots) |
| **vs PAYG** | Seer is a **separate monthly charge**; **does not** consume the shared PAYG budget |
| **Legacy Seer** (pre-Jan 2026 only) | **$20/mo** per Sentry subscription + **$25** Seer event credits; overage via PAYG. Issue Scan ~**$0.003**/run tiered; Issue Fix **$1.00**/run. **No longer offered** as new add-on after Jan 2026 |
| **Self-host** | Seer still **unavailable** (pass 126 / develop.sentry.dev) |

**Holds:** desk claim “Seer ≈ $40/active-contributor” is **still current primary**
for new customers. Legacy per-event / $1 fix-run path is **legacy-only**.

**Fixer-model comparison (same day desk):**

| Vendor | Meter | Unit economics (order-of-magnitude) | Self-host AI |
| --- | --- | --- | --- |
| **Sentry Seer** | Seat (active contributor) | **$40/contributor/mo** unlimited usage within SKU | **No** |
| **Datadog Bits** (pass 131) | AI Credits | **~$6.50–$8.45**/Investigate avg (credit math) | **No** (SaaS) |

**Implication for Parallax fixer design:** two live incumbent patterns —

1. **Seat / contributor** (Seer) — simple ACV math once adoption is broad; expensive for large PR teams; no self-host.
2. **Work-unit / credit** (Bits AI Credits) — variable cost with usage spikes; still SaaS-only.

Parallax desk design (outcome-priced fixer + open evidence core + optional EE/cloud)
still sits in the **work-unit / outcome** family early, with optional graduate to
contributor seats once accuracy is trusted — **unchanged**. Self-host Seer absence
still supports hard-boundary air-gap narrative (pass 126).

**Falsify:** Seer drops to free on Team; or ships self-hosted open AI; or
primary rate leaves $40/contributor for a pure per-fix PAYG only.

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
5. **Fixer = premium add-on, priced per successful outcome** (or credit-metered
   investigation work — see **pass 131** Datadog AI Credits primary). Historical
   desk cited ~$25–30 per *conclusive* Bits investigation; **live primary is now
   credit-based**, not a pure per-investigation SKU. Outcome/credit pricing still
   **de-risks an unproven autonomous fixer** better than a flat seat fee early;
   graduate to per-contributor flat
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
- Fixer pricing: Sentry Seer **$40/active contributor** (pass 132 primary docs): <https://docs.sentry.io/pricing/#seer-pricing> · marketing: <https://sentry.io/pricing/> · Datadog **AI Credits** (pass 131): <https://www.datadoghq.com/pricing/?product=ai-credits#products>

> Unconfirmed / flagged: the compliance-only segment $ size is estimated, not isolated by any source;
> Grafana's cloud-vs-self-managed split and OSS→paid conversion rates are not published; several
> pricing floors are from secondary teardowns.
