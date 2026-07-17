# Air-gap / no-phone-home differentiator recheck (2026-07-17)

<!-- markdownlint-disable MD013 -->

**Pass target:** research-agenda comparison #5 — does the **air-gapped,
no-phone-home agent-evidence** differentiator stay unique vs incumbents?

**Prior theory:** hard-boundary buyers need self-host with no vendor control
plane / no mandatory cloud AI; Seer and Bits are cloud; Grafana on-prem may
still phone home for some features.

**Verdict (pass 56 + pass 151):** **Still unique for the *combination* of
air-gap-capable open core + portable agent evidence + no closed cloud AI
dependency.** Narrower claims need care:

| Claim | Status 2026-07-17 | Evidence |
| --- | --- | --- |
| Sentry Seer available self-hosted | **False — still excluded** | [develop.sentry.dev/self-hosted](https://develop.sentry.dev/self-hosted/): "Seer and other AI & ML features… currently closed source" (**pass 126 + pass 151** primary re-fetch) |
| Datadog Bits available as self-hosted backend | **False** | Pass 42 + **pass 111:** only OSS Agent + Observability Pipelines Worker (routes into SaaS); **no self-hosted Datadog store/UI**. Secondary market sources still “SaaS-only” (2026); primary product remains cloud backend. |
| OSS Grafana / LGTM can run offline | **True for OSS core** | Self-host OSS is real; **Enterprise plugins/license keys** and Cloud AI features are separate — do **not** claim "all Grafana phones home" without a dated primary for a specific binary feature |
| Grafana Assistant on self-managed | **UI yes / AI backend Cloud** | **Pass 77:** plugin + connect to Grafana Cloud stack required; not offline BYO-LLM — [incumbent-self-hosted-ai-recheck-2026-07-17.md](incumbent-self-hosted-ai-recheck-2026-07-17.md) |
| OSS SigNoz / OpenObserve / Traceway / Rustrak / GlitchTip / Bugsink can air-gap | **True (product can)** | Self-host OSS peers; **none** ship Parallax-style portable redacted evidence bundle + outcome loop (passes 49–53 + 122–125 + 134) |
| Langfuse self-host phone-home | **Default usage telemetry ON** | **Pass 107:** README — self-hosted instances report basic usage stats to centralized PostHog by default (not raw traces); opt-out documented. Not air-gap-clean without config. |
| Full combination closed | **False** | No peer ships open portable redacted prod-incident bundle + outcome under air-gap |

**Pass 151:** Seer exclusion re-confirmed (same primary sentence as pass 126).
Wedge combination rechecks (Traceway/Bugsink/Rustrak/TMA1/GlitchTip) still show
**no** portable redacted prod-incident bundle + outcome under pure air-gap.
Differentiator **holds as combination claim**; A1 value still unproven.

**Falsification:** a major incumbent ships **self-hosted, open (or source-available)
agent evidence with no cloud AI dependency** *and* a portable redacted bundle
schema; or Seer becomes self-host GA with offline models; or Grafana Assistant
runs fully offline with BYO-LLM (would fire pass-77 UNFIRED item).

**Uncertainty:** Enterprise license telemetry for Grafana/Elastic/etc. was
**not** re-instrumented this pass — mark any "X phones home" claim
**unverified** unless a primary doc is attached. Air-gap *capability* ≠ "never
phones home on every build."

**Implication for A2:** pass 54 monetization desk recheck still aligns —
hard-boundary buyers pay for boundaries incumbents refuse to open. Differentiator
is **real but niche** and does not replace A1 value proof.
