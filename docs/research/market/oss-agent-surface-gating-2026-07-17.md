# OSS peer agent surface & redaction gating (2026-07-17)

<!-- markdownlint-disable MD013 -->

**Pass target:** standing market claim that "open self-hosted agent-native obs"
is table stakes — check whether **AI investigation / redaction** stay free in
core or gate to cloud/EE (affects Parallax air-gap + A6 positioning).

**Evidence class:** primary GitHub READMEs + vendor pricing/docs (2026-07-17).
**Pass 94** re-fetched OpenObserve pricing FAQ + SigNoz Noz docs.
**Pass 242** (2026-07-18) re-fetched triangle pricing + Noz docs + release pins
— agent/AI gates **hold** (see table below).

## Findings

### SigNoz

- README section **Agent-Native Observability and MCP** promotes MCP server +
  agent skills for coding agents.
- Explicit: **[Noz](https://signoz.io/docs/ai/noz/) is available only on SigNoz
  Cloud** (in-product AI investigator). Self-host gets MCP path to agents;
  **cloud-gated AI product (Noz)**.
- **Pass 94:** Noz docs still tagged **`SigNoz Cloud`** (updated 2026-06-29);
  page contrasts Noz (UI) vs **MCP Server** (IDE/agents) — does not document a
  Community/self-host Noz path.
- **Pass 242:** Noz docs still tagged **`SigNoz Cloud`**. Pricing Teams copy
  still lists **"Access to MCP Server and Noz"** under **$49/mo** Cloud
  ([signoz.io/pricing](https://signoz.io/pricing/)). Latest release **`v0.133.0`**
  (~30.3k★). Community self-host remains free software, not free Noz.
- **Pass 262:** Noz docs still tagged **`SigNoz Cloud`** only (no Community
  offline Noz path). [MCP Server docs](https://signoz.io/docs/ai/signoz-mcp-server/)
  apply to **Cloud and Self-Host** (hosted MCP URL for Cloud; self-host install
  path documented on same page). Noz vs MCP section still: Noz = in-UI Cloud
  teammate; MCP = external coding agents over telemetry. Stars **~30,316**.
  **No** portable redacted evidence-bundle export observed.
- **Implication:** air-gap teams using SigNoz OSS get **MCP tools over raw
  telemetry**, not a free in-product Noz investigator. Aligns with Parallax's
  "context engine, not the fixer" + possible HolmesGPT/Traceway as fixers.

### OpenObserve

- README markets ingest-time enrich/**redact**/reduce and
  **Sensitive Data Redaction (SDR)** listed as **Enterprise feature** (PII
  redaction during ingest/query).
- **Pass 94** ([openobserve.ai/pricing](https://openobserve.ai/pricing/)): Enterprise
  still lists **SDR + AI SRE Agent + AI Assistant + AI-Powered Observability**.
  Self-Hosted Enterprise free **≤ 50 GB/day** (FAQ, two answers — prior 200 GB
  conflict treated **resolved to 50**). Cloud AI "free during preview" with
  **20 credits** is **not** OSS-core AI.
- **Pass 242:** same pricing shape reconfirmed; release **`v0.91.2`** (~20.2k★,
  AGPL). AI still EE list + Cloud preview credits.
- **Implication:** Parallax's **bundle-path redaction (open core)** vs
  Enterprise-gated SDR is a real openness difference *if* A6 proves trustworthy.
  Do not claim unique "redaction exists" — claim **open-core agent-facing
  redaction contract** when A6 holds.

### Grafana Assistant (pass 242 pin)

- [grafana.com/pricing](https://grafana.com/pricing/) lists **Grafana Assistant**
  as its own meter: Free **3 active AI users**; Pro **$20 / active AI user** +
  **$2 / 1M tokens**.
- Reinforces: **in-product AI is a paid Cloud surface**, not free self-host OSS
  (self-managed UI still needs Cloud LLM backend — pass 238 incumbent note).

### Coroot (**pass 103** primary re-fetch)

- [MCP overview](https://docs.coroot.com/mcp/overview/): OAuth 2.0 + per-user
  **RBAC**; CE tools include full triage surface + **`resolve_alerts`** (sole CE
  mutate); EE adds `list_anomalies` + **`investigate_anomaly`**.
- [AI docs](https://docs.coroot.com/ai/): AI-powered RCA **Enterprise ($1/CPU
  core/mo)** or **Cloud integration** for CE (**10 free investigations/mo**).
- Pin still **v1.23.3 / 7,837★** (2026-07-02). **No** portable redacted evidence
  bundle; eBPF still not app-error/stack product.
- **Implication:** best-in-class **MCP safety model** among peers; **AI RCA is
  EE/Cloud-metered** (same survivor pattern as SigNoz Noz / O2 AI SRE).

### Traceway / Rustrak / GlitchTip / Bugsink

- Already deep-dived (passes 40–53): agent MCP/skills real; **no** portable
  redacted evidence bundle schema; Bugsink/GlitchTip/Rustrak error-only or
  Sentry-alt.

## Verdict

| Capability | Free OSS self-host (representative) | Gated |
| --- | --- | --- |
| MCP / agent query tools | SigNoz, Traceway, Coroot CE, Rustrak, GlitchTip | — |
| In-product AI investigator (Noz-class) | — | **SigNoz Noz = Cloud only**; **Coroot `investigate_anomaly` = EE** (or Cloud 10/mo for CE) |
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
