# OSS peer agent surface & redaction gating (2026-07-17)

<!-- markdownlint-disable MD013 -->

**Pass target:** standing market claim that "open self-hosted agent-native obs"
is table stakes — check whether **AI investigation / redaction** stay free in
core or gate to cloud/EE (affects Parallax air-gap + A6 positioning).

**Evidence class:** primary GitHub READMEs (2026-07-17), not live deploys.

## Findings

### SigNoz

- README section **Agent-Native Observability and MCP** promotes MCP server +
  agent skills for coding agents.
- Explicit: **[Noz](https://signoz.io/docs/ai/noz/) is available only on SigNoz
  Cloud** (in-product AI investigator). Self-host gets MCP path to agents;
  **cloud-gated AI product (Noz)**.
- **Implication:** air-gap teams using SigNoz OSS get **MCP tools over raw
  telemetry**, not a free in-product Noz investigator. Aligns with Parallax's
  "context engine, not the fixer" + possible HolmesGPT/Traceway as fixers.

### OpenObserve

- README markets ingest-time enrich/**redact**/reduce and
  **Sensitive Data Redaction (SDR)** listed as **Enterprise feature** (PII
  redaction during ingest/query).
- **Implication:** Parallax's **bundle-path redaction (open core)** vs
  Enterprise-gated SDR is a real openness difference *if* A6 proves trustworthy.
  Do not claim unique "redaction exists" — claim **open-core agent-facing
  redaction contract** when A6 holds.

### Coroot / others

- Coroot README fetch incomplete this pass; prior deep-dive still owns
  OAuth+RBAC MCP (1 mutate). Re-fetch next pass if citing.

### Traceway / Rustrak / GlitchTip / Bugsink

- Already deep-dived (passes 40–53): agent MCP/skills real; **no** portable
  redacted evidence bundle schema; Bugsink/GlitchTip/Rustrak error-only or
  Sentry-alt.

## Verdict

| Capability | Free OSS self-host (representative) | Gated |
| --- | --- | --- |
| MCP / agent query tools | SigNoz, Traceway, Coroot (prior), Rustrak, GlitchTip | — |
| In-product AI investigator (Noz-class) | — | **SigNoz Noz = Cloud only** |
| Advanced PII SDR | partial | **OpenObserve Enterprise** |
| Portable redacted versioned evidence bundle | **none found** | n/a |
| Fix-outcome open records | **none found** | n/a |

**Parallax positioning (honest):** free local MCP + open bundle redaction is
**still rare for the full evidence artifact**, but **MCP alone is not rare**.
Air-gap buyers lose SigNoz Noz; they keep MCP-over-telemetry options (Traceway,
SigNoz MCP, HolmesGPT). Bundle+outcome remains the unproven differentiator.

## Falsification

- SigNoz ships Noz (or equivalent) fully offline free in OSS.
- OpenObserve moves SDR to free core *and* ships portable investigation export.
- Any peer ships JSON-Schema evidence bundle with redaction report + outcomes.

## Related

- [air-gap-no-phone-home-recheck-2026-07-17.md](air-gap-no-phone-home-recheck-2026-07-17.md)
- [parallax-vs-signoz.md](competitors/parallax-vs-signoz.md)
- [parallax-vs-openobserve.md](competitors/parallax-vs-openobserve.md)
- [parallax-vs-traceway.md](competitors/parallax-vs-traceway.md)
