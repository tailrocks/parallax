# Incumbent self-hosted AI recheck (2026-07-17, pass 77)

<!-- markdownlint-disable MD013 -->

**Pass target:** research-agenda standing watch — *Incumbent self-hosted AI*
(Sentry Seer self-host intent; Grafana local-inference / BYO-LLM). Either would
erode Parallax's air-gap / no-cloud-AI differentiator.

**Evidence class:** primary vendor docs (fetched 2026-07-17). Desk only.

---

## Verdict

| Watch item | Status 2026-07-17 | Erodes air-gap wedge? |
| --- | --- | --- |
| Sentry Seer on self-hosted | **Still unavailable** — closed-source AI/ML excluded from self-hosted | **No** (wedge holds) |
| Grafana local-inference / fully offline Assistant | **Not found** as a shipping product | **No** for pure offline |
| Grafana Assistant *UI* on self-managed OSS/Enterprise | **Shipped** (Grafana v13+, GrafanaCON 2026) via plugin → **Cloud backend** | **Partial pressure** — not air-gap; requires Cloud stack |

**Precise claim after this pass:**

> No major incumbent ships **fully offline, open (or self-hostable without
> vendor cloud AI backend)** agent evidence for production debugging. Grafana
> *did* ship Assistant into self-managed *UIs*, but the **LLM backend remains
> Grafana Cloud**. Sentry Seer remains cloud/closed. True air-gap AI still
> routes to BYO-LLM layers (e.g. HolmesGPT + Ollama) or pure context engines
> (Parallax thesis).

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
   backend and no Grafana Cloud stack.
3. **UNFIRED:** portable versioned redacted investigation artifact.

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
