# Incumbent self-hosted AI recheck (2026-07-17, pass 77 + pass 126 + pass 158)

<!-- markdownlint-disable MD013 -->

**Pass target:** research-agenda standing watch — *Incumbent self-hosted AI*
(Sentry Seer self-host intent; Grafana local-inference / BYO-LLM). Either would
erode Parallax's air-gap / no-cloud-AI differentiator.

**Evidence class:** primary vendor docs (fetched 2026-07-17; pass **158** re-fetch
2026-07-18). Desk only.

---

## Verdict

| Watch item | Status 2026-07-18 (pass 158) | Erodes air-gap wedge? |
| --- | --- | --- |
| Sentry Seer on self-hosted | **Still unavailable** — closed-source AI/ML excluded from self-hosted | **No** (wedge holds) |
| Grafana local-inference / fully offline Assistant | **Not found** as a shipping product | **No** for pure offline |
| Grafana Assistant *UI* on self-managed OSS/Enterprise | **Shipped** (Grafana v13+, GrafanaCON 2026) via plugin → **Cloud backend** | **Partial pressure** — not air-gap; requires Cloud stack |

### Pass 126 (2026-07-17) — Seer primary re-fetch

Live [develop.sentry.dev/self-hosted](https://develop.sentry.dev/self-hosted/)
**Differences between self-hosted and SaaS** list still includes, verbatim:

> Seer and other AI & ML features, as these are currently closed source.

**No change** vs pass 77. Air-gap / self-host Seer wedge **still holds**.
Grafana offline BYO-LLM path **not re-probed** this pass (prior partial-pressure
claim retained).

### Pass 158 (2026-07-18) — Seer + Grafana Assistant dual re-fetch

| Source | Finding |
| --- | --- |
| [develop.sentry.dev/self-hosted](https://develop.sentry.dev/self-hosted/) | HTML still contains exact list item: **"Seer and other AI & ML features, as these are currently closed source."** — **UNFIRED** for self-host Seer GA |
| [Grafana Assistant self-managed setup](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/get-started/self-managed/) (markdown-capable docs) | Still **hybrid**: "The Assistant UI runs in your self-managed Grafana deployment. **The backend, usage limits, and billing stay in the Grafana Cloud stack** you connect during setup." Prompts + query context **sent to Grafana Cloud**. Requires Grafana **13.0.0+** + Cloud stack admin. **No offline / BYO-LLM product path** documented. |

### Pass 188 (2026-07-18) — dual re-fetch

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | Still **"Seer and other AI & ML features, as these are currently closed source."** |
| Grafana Assistant self-managed docs | Still **hybrid** Cloud backend/billing; prompts leave self-managed |

### Pass 210 (2026-07-18) — dual re-fetch

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | Still Seer **closed source** list item |
| Grafana Assistant self-managed docs | Still hybrid **Cloud backend/billing** wording |

### Pass 238 (2026-07-18) — dual re-fetch

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | Still Seer **closed source** |
| Grafana Assistant self-managed docs | Still hybrid **Cloud backend/billing** |

### Pass 245 (2026-07-18) — Seer + Assistant + Bits Code triple re-fetch

| Source | Finding |
| --- | --- |
| [develop.sentry.dev/self-hosted](https://develop.sentry.dev/self-hosted/) | Differences list still: **"Seer and other AI & ML features, as these are currently closed source."** — **UNFIRED** for self-host Seer GA |
| [Grafana Assistant self-managed setup](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/get-started/self-managed/) | Still **hybrid**: "The Assistant UI runs in your self-managed Grafana deployment. **The backend, usage limits, and billing stay in the Grafana Cloud stack** you connect during setup." Prompts + query context **sent to Grafana Cloud**. Requires Grafana **13.0.0+** + Cloud stack admin. Self-managed **does not** include Investigations / investigation memory. **No offline / BYO-LLM product path.** Pricing same as Cloud Assistant (seat/token meters — pass 242). |
| [Bits Code docs](https://docs.datadoghq.com/bits_ai/bits_ai_dev_agent/) | Explicit: **"Bits Code never auto-merges PRs or MRs."** Creates/iterates PRs; human merge remains. Self-hosted SCM (GHES, GitLab Self-Managed) **not supported**. |

**UNFIRED:** Seer self-host GA; Grafana offline/BYO-LLM Assistant; Bits app-code
auto-merge commodity.

### Pass 260 (2026-07-18) — Seer + Sentry self-host version pin

| Source | Finding |
| --- | --- |
| [getsentry/self-hosted](https://github.com/getsentry/self-hosted/releases/latest) | Latest still **`26.7.0`** (published **2026-07-16**). Compose top-level `services:` count still **64** (same method as pass 70/87). |
| [develop.sentry.dev/self-hosted](https://develop.sentry.dev/self-hosted/) | Still lists **"Seer and other AI & ML features, as these are currently closed source."** — **UNFIRED** |
| SigNoz (adjacent pin) | **`v0.133.0`** still latest; **~30,316★** (small star drift vs pass 242 ~30,309) |

### Pass 269 (2026-07-18) — Bits auto-merge + Sentry OTLP metrics kill watches

| Source | Finding |
| --- | --- |
| [Bits Code docs](https://docs.datadoghq.com/bits_ai/bits_ai_dev_agent/) | Still explicit: **"Bits Code never auto-merges PRs or MRs."** — app-code auto-merge commodity **UNFIRED** |
| [Sentry OTLP](https://docs.sentry.io/concepts/otlp/) | Still OTLP **traces and logs** (open beta). Explicit: **"Sentry does not support OTLP metrics at this time."** — OTLP metrics GA **UNFIRED** |

### Pass 274 (2026-07-18) — Grafana Assistant hybrid re-fetch

| Source | Finding |
| --- | --- |
| [Assistant self-managed setup](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/get-started/self-managed/) | Still **hybrid**: "The backend, usage limits, and billing stay in the Grafana Cloud stack"; prompts + query context **sent to Grafana Cloud**. **No** offline/BYO-LLM path documented. |

### Pass 283 (2026-07-18) — Seer + Bits + Sentry OTLP triple re-fetch

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | Still **"Seer and other AI & ML features… closed source"** |
| getsentry/self-hosted Latest | still **`26.7.0`** (2026-07-16) |
| Bits Code docs | still **"Bits Code never auto-merges PRs or MRs."** |
| docs.sentry.io/concepts/otlp | still **"Sentry does not support OTLP metrics at this time."** |

### Pass 294 (2026-07-18) — Seer + Bits + Sentry OTLP re-fetch

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | still closed-source Seer exclusion |
| Bits Code docs | still **"never auto-merges PRs or MRs"** |
| Sentry OTLP docs | still **"does not support OTLP metrics at this time"** |

### Pass 299 (2026-07-18) — Seer + Bits + Sentry OTLP re-fetch

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | still Seer **closed source** exclusion |
| getsentry/self-hosted Latest | still **`26.7.0`** |
| Bits Code docs | still **"never auto-merges PRs or MRs"** |
| Sentry OTLP docs | still **"does not support OTLP metrics at this time"** |

### Pass 303 (2026-07-18) — Seer + Assistant + Bits + Sentry OTLP + self-host pin

| Source | Finding |
| --- | --- |
| [develop.sentry.dev/self-hosted](https://develop.sentry.dev/self-hosted/) | Still: **"Seer and other AI & ML features, as these are currently closed source."** — self-host Seer GA **UNFIRED** |
| [getsentry/self-hosted Latest](https://github.com/getsentry/self-hosted/releases/latest) | still **`26.7.0`** |
| [Grafana Assistant self-managed](https://grafana.com/docs/grafana/latest/administration/assistant/) | still **hybrid**: "The Assistant backend, usage limits, and billing stay in the Grafana Cloud stack that you connect during setup." Offline/BYO-LLM **UNFIRED** |
| [Bits Code docs](https://docs.datadoghq.com/bits_ai/bits_ai_dev_agent/) | still explicit: **"Bits Code never auto-merges PRs or MRs."** — app-code auto-merge commodity **UNFIRED** |
| [Sentry OTLP](https://docs.sentry.io/concepts/otlp/) | still **"Sentry does not support OTLP metrics at this time."** — OTLP metrics GA **UNFIRED** |

**UNFIRED:** Seer self-host GA; Grafana offline/BYO-LLM Assistant; Bits auto-merge; Sentry OTLP metrics GA.

### Pass 309 (2026-07-18) — Seer + Assistant + Bits + Sentry OTLP re-fetch

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | still **"Seer and other AI & ML features, as these are currently closed source."** |
| getsentry/self-hosted Latest | still **`26.7.0`** (2026-07-16) |
| Bits Code docs | still **"Bits Code never auto-merges PRs or MRs."** |
| Sentry OTLP docs | still **"Sentry does not support OTLP metrics at this time."** |
| Grafana Assistant self-managed | still hybrid: backend/usage/billing stay in **Grafana Cloud stack** |

**UNFIRED:** Seer self-host GA; Grafana offline/BYO-LLM Assistant; Bits auto-merge; Sentry OTLP metrics GA.

### Pass 318 (2026-07-18) — Seer + Bits + Sentry OTLP re-fetch

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | still Seer **closed source** exclusion |
| getsentry/self-hosted Latest | still **`26.7.0`** |
| Bits Code docs | still **"never auto-merges PRs or MRs"** |
| Sentry OTLP docs | still **"does not support OTLP metrics at this time"** |

**UNFIRED:** Seer self-host GA; Bits auto-merge; Sentry OTLP metrics GA.

### Pass 324 (2026-07-18) — Seer + Bits re-fetch

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | still Seer **closed source** |
| Bits Code docs | still **never auto-merges** PRs/MRs |

**UNFIRED:** Seer self-host GA; Bits auto-merge.

### Pass 327 (2026-07-18) — Sentry OTLP metrics kill

| Source | Finding |
| --- | --- |
| docs.sentry.io/concepts/otlp | still **"does not support OTLP metrics at this time"** — **UNFIRED** |






**Precise claim after this pass:**

> No major incumbent ships **fully offline, open (or self-hostable without
> vendor cloud AI backend)** agent evidence for production debugging. Grafana
> *did* ship Assistant into self-managed *UIs*, but the **LLM backend remains
> Grafana Cloud**. Sentry Seer remains cloud/closed. True air-gap AI still
> routes to BYO-LLM layers (e.g. HolmesGPT + Ollama) or pure context engines
> (Parallax thesis). App-code auto-merge still **not** commodity (Bits never
> auto-merges).

---

## Sentry Seer

**Primary:** [Self-Hosted Sentry](https://develop.sentry.dev/self-hosted/)
(fetched 2026-07-17).

Unavailable list still includes:

> Seer and other AI & ML features, as these are currently closed source.

**No public GA date** for self-hosted Seer found this pass. FSL self-hosted
product remains without Seer. Prior pass-54 finding **reaffirmed**.

**Falsify:** self-hosted install ships Seer or open offline AI package with
docs and release notes.

---

## Grafana Assistant on self-managed

**Primaries (fetched 2026-07-17):**

- [Grafana Assistant (self-managed docs)](https://grafana.com/docs/grafana/latest/administration/assistant/)
- [Set up Assistant in self-managed Grafana](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/get-started/self-managed/)
- [GrafanaCON announcement blog](https://grafana.com/blog/grafana-assistant-everywhere/) (2026-04-21)

**What shipped:**

1. Starting **Grafana v13**, install **Grafana Assistant app** in self-hosted
   OSS or Enterprise (Enterprise may pre-install from 13.1 — secondary
   [whats-new](https://grafana.com/whats-new/2026-06-23-grafana-assistant-is-now-pre-installed-in-grafana-enterprise/)).
2. **Connect to a Grafana Cloud stack** (Backend URL, Instance ID, API token).
3. Docs: *“The Assistant UI runs in your self-managed Grafana deployment. The
   backend, usage limits, and billing stay in the Grafana Cloud stack.”*
4. Data path: raw datasources stay local; **request context / summaries go to
   Cloud** for processing (region of connected stack).

**What did *not* ship (relative to standing watch wording):**

- No documented **local-model / Ollama / fully air-gapped Assistant** as the
  product path.
- Assistant is **not OSS**; self-managed is a **Cloud-backed plugin**.
- Some Cloud-only features remain hidden on self-managed.

**Implication for Parallax:**

| Axis | Read |
| --- | --- |
| “Self-managed UI has AI chat” | **Grafana now occupies this** for non-air-gap teams |
| “No phone-home / no vendor AI cloud” | **Still empty for Assistant** — connection *is* phone-home to Cloud |
| Air-gap / classified / hard-boundary | **Unchanged** — Cloud-required Assistant fails the segment |
| Portable redacted evidence bundle | **Still unclaimed** by Grafana Assistant |

**Watch refinement:** replace vague “Grafana local-inference/BYO-LLM” with:

1. **FIRED (partial):** self-managed Assistant UI via Cloud backend (2026-04+).
2. **UNFIRED:** Assistant (or equivalent) with **fully offline / BYO-LLM**

### Pass 332 (2026-07-18) — Seer + Bits

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | still Seer **closed source** |
| Bits Code docs | still **never auto-merges** |

**UNFIRED:** Seer self-host GA; Bits auto-merge.

### Pass 338 (2026-07-18) — Seer + Bits + OTLP + Assistant

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | still Seer **closed source** |
| Bits Code docs | still **never auto-merges** |
| Sentry OTLP | still **no OTLP metrics** |
| Grafana Assistant self-managed | still hybrid **Cloud stack** backend |

**UNFIRED:** Seer self-host GA; Bits auto-merge; Sentry OTLP metrics; offline Assistant.

### Pass 343 (2026-07-18) — Seer + Bits (docs primaries; no GH API)

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | still Seer **closed source** |
| Bits Code docs | still **never auto-merges** |

**UNFIRED:** Seer self-host GA; Bits auto-merge.



   backend and no Grafana Cloud stack.
3. **UNFIRED:** portable versioned redacted investigation artifact.

### Pass 349 (2026-07-18) — Assistant + Sentry OTLP

| Source | Finding |
| --- | --- |
| Grafana Assistant self-managed | still hybrid **Grafana Cloud stack** backend |
| Sentry OTLP | still **does not support OTLP metrics** |

**UNFIRED:** offline Assistant; Sentry OTLP metrics GA.

### Pass 353 (2026-07-18) — Seer + Bits

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | still Seer **closed source** |
| Bits Code docs | still **never auto-merges** |

**UNFIRED:** Seer self-host GA; Bits auto-merge.

### Pass 355 (2026-07-18) — Assistant + Sentry OTLP

| Source | Finding |
| --- | --- |
| Grafana Assistant self-managed | still hybrid **Cloud stack** backend |
| Sentry OTLP | still **no OTLP metrics** |

**UNFIRED:** offline Assistant; Sentry OTLP metrics.

### Pass 359 (2026-07-18) — Seer + Bits + Assistant

| Source | Finding |
| --- | --- |
| develop.sentry.dev/self-hosted | still Seer **closed source** |
| Bits Code docs | still **never auto-merges** |
| Grafana Assistant self-managed | still hybrid **Cloud stack** backend |

**UNFIRED:** Seer self-host GA; Bits auto-merge; offline Assistant.

### Pass 364 (2026-07-18) — Assistant + Sentry OTLP

| Source | Finding |
| --- | --- |
| Grafana Assistant self-managed | still hybrid **Cloud stack** |
| Sentry OTLP | still **no OTLP metrics** |

**UNFIRED:** offline Assistant; Sentry OTLP metrics.

### Pass 373 (2026-07-18) — Seer + Bits

Seer still **closed source**; Bits still **never auto-merges**. **UNFIRED.**


### Pass 371 (2026-07-18) — Assistant + Sentry OTLP

Assistant still hybrid Cloud; Sentry still **no OTLP metrics**. **UNFIRED.**


### Pass 370 (2026-07-18) — Seer + Bits

Seer still **closed source**; Bits still **never auto-merges**. **UNFIRED.**


### Pass 368 (2026-07-18) — Assistant

Grafana Assistant still hybrid **Cloud stack** backend. Offline/BYO-LLM **UNFIRED.**


### Pass 367 (2026-07-18) — Seer + Bits

Seer still **closed source**; Bits still **never auto-merges**. **UNFIRED.**


### Pass 365 (2026-07-18) — Seer

Seer still **closed source**. **UNFIRED.**



### Pass 362 (2026-07-18) — Bits

Bits Code still **never auto-merges** PRs/MRs. **UNFIRED.**


### Pass 361 (2026-07-18) — Seer

Seer still **closed source** on self-hosted. **UNFIRED.**



### Pass 356 (2026-07-18) — Seer + Bits

Seer still **closed source**; Bits still **never auto-merges**. **UNFIRED.**




### Pass 351 (2026-07-18) — Seer

Seer still **closed source** on self-hosted. **UNFIRED.**



### Pass 348 (2026-07-18) — Bits

Bits Code docs still: **"never auto-merges PRs or MRs."** **UNFIRED.**


### Pass 346 (2026-07-18) — Seer docs primary

develop.sentry.dev/self-hosted still lists Seer as **closed source**. **UNFIRED.**


---

## Adjacent: true offline AI investigation

Not an incumbent obs platform, but relevant: **HolmesGPT** documents
**Ollama / local models** for air-gapped investigation (BYO over existing
Prometheus/K8s telemetry). That is the *fixer layer* over BYO telemetry, not a
bundle moat — still complementary to Parallax; see
[parallax-vs-holmesgpt.md](competitors/parallax-vs-holmesgpt.md).

---

## Uncertainty

| Item | Class |
| --- | --- |
| Whether Grafana will ever ship offline Assistant | Unknown — watch Enterprise + LLM plugin docs |
| Seer self-host roadmap date | No primary date; treat as open |
| gcx / Assistant CLI air-gap story | Blog mentions local-first CLI tooling; not verified as offline Assistant backend this pass |

## Related

- [air-gap-no-phone-home-recheck-2026-07-17.md](air-gap-no-phone-home-recheck-2026-07-17.md)
- [research-agenda.md](../research-agenda.md) standing watches
- [monetization-and-paying-segment.md](../validation/monetization-and-paying-segment.md)
