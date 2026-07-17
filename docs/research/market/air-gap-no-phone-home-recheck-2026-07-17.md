# Air-gap / no-phone-home differentiator recheck (2026-07-17)

<!-- markdownlint-disable MD013 -->

**Pass target:** research-agenda comparison #5 — does the **air-gapped,
no-phone-home agent-evidence** differentiator stay unique vs incumbents?

**Prior theory:** hard-boundary buyers need self-host with no vendor control
plane / no mandatory cloud AI; Seer and Bits are cloud; Grafana on-prem may
still phone home for some features.

**Verdict (pass 56):** **Still unique for the *combination* of
air-gap-capable open core + portable agent evidence + no closed cloud AI
dependency.** Narrower claims need care:

| Claim | Status 2026-07-17 | Evidence |
| --- | --- | --- |
| Sentry Seer available self-hosted | **False — still excluded** | [develop.sentry.dev/self-hosted](https://develop.sentry.dev/self-hosted/): "Seer and other AI & ML features… currently closed source" (pass 54 fetch) |
| Datadog Bits available as self-hosted backend | **False** | Pass 42: Agent + OPW only; no self-hosted Datadog backend |
| OSS Grafana / LGTM can run offline | **True for OSS core** | Self-host OSS is real; **Enterprise plugins/license keys** and Cloud AI features are separate — do **not** claim "all Grafana phones home" without a dated primary for a specific binary feature |
| OSS SigNoz / OpenObserve / Traceway / Rustrak / GlitchTip / Bugsink can air-gap | **True (product can)** | Self-host OSS peers; **none** ship Parallax-style portable redacted evidence bundle + outcome loop (passes 49–53) |
| Full combination closed | **False** | No peer ships open portable redacted prod-incident bundle + outcome under air-gap |

**Falsification:** a major incumbent ships **self-hosted, open (or source-available)
agent evidence with no cloud AI dependency** *and* a portable redacted bundle
schema; or Seer becomes self-host GA with offline models.

**Uncertainty:** Enterprise license telemetry for Grafana/Elastic/etc. was
**not** re-instrumented this pass — mark any "X phones home" claim
**unverified** unless a primary doc is attached. Air-gap *capability* ≠ "never
phones home on every build."

**Implication for A2:** pass 54 monetization desk recheck still aligns —
hard-boundary buyers pay for boundaries incumbents refuse to open. Differentiator
is **real but niche** and does not replace A1 value proof.
