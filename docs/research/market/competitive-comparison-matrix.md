# Competitive Comparison Matrix — Parallax vs Alternatives

> Research date: 2026-05-31
> Quick-reference matrix for decision-making. Full analysis in
> [alternatives-deep-analysis.md](alternatives-deep-analysis.md).

## How to read this

Each tool is rated against Parallax's 8 wedge dimensions:
- `+` = has it
- `~` = partial / announced but incomplete
- `-` = absent

**Threat level** = how likely this tool is to close the gap within 12 months.

---

## Tier 1: Direct Platform Competitors (could become Parallax)

| Tool | ★ Stars | Lang | Sentry Ingest | OTLP | Evidence Bundles | Outcome Tracking | Agent/CLI/CI Capture | Air-gapped | Rust | Single Binary | **Threat** |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Parallax (planned)** | 0 | Rust | + | + | + | + | + | + | + | + | — |
| **OpenObserve** | 4,000+ | Rust | - | + | ~ | - | - | + | + | + | ★★★★ Very High |
| **SigNoz** | 27,151 | Go | - | + | ~ | - | - | + | - | - | ★★★★ High |
| **Maple** | 382 | TS/Go | - | + | - | - | - | + | - | + | ★★★ High |
| **Traceway** | 830 | Go | - | + | - | - | - | + | - | + | ★★★ High |
| **Coroot** | 7,675 | Go | - | + | - | - | - | + | - | ~ | ★★★ High |

## Tier 2: Sentry-Compatible Error Trackers

| Tool | ★ Stars | Lang | Sentry Ingest | OTLP | Evidence Bundles | Outcome Tracking | Agent/CLI/CI Capture | Air-gapped | Rust | Single Binary | **Threat** |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Rustrak** | 44 | Rust | + | - | - | - | - | + | + | ~ | ★★★★ Very High |
| **edde746/bugs** | <10 | Rust | + | - | - | - | - | + | + | + | ★★★ High |
| **Errex** | 4 | Rust | + | - | - | - | - | + | + | + | ★★★ High |
| **Urgentry** | 55 | - | + | ~ | - | - | - | + | - | + | ★★★ High |
| **Bugsink** | - | Python | + | - | - | - | - | + | - | ~ | ★★★ High |
| **GlitchTip** | - | Python | + | - | - | - | - | + | - | - | ★★ Moderate |

## Tier 3: AI SRE Agents & Investigation Tools

| Tool | ★ Stars | Lang | Sentry Ingest | OTLP | Evidence Bundles | Outcome Tracking | Agent/CLI/CI Capture | Air-gapped | Rust | Single Binary | **Threat** |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **HolmesGPT** | 2,538 | Python | - | - | + | ~ | - | + | - | - | ★★★ High |
| **Aurora** | 257 | Python | - | - | + | ~ | - | + | - | - | ★★★ High |
| **Syncause** | - | - | - | - | + | - | - | + | - | - | ★★ Moderate |

## Tier 4: Emerging Agent-Context Tools (new in 2026)

| Tool | ★ Stars | Lang | Sentry Ingest | OTLP | Evidence Bundles | Outcome Tracking | Agent/CLI/CI Capture | Air-gapped | Rust | Single Binary | **Threat** |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **opentrace** | 15 | Go | - | ~ | - | - | + | + | - | + | ★★ Moderate |
| **sentro** | 1 | TS | - | - | - | - | + | + | - | - | ★ Low |
| **OTel MCP Server** | 189 | Python | - | + | - | - | ~ | + | - | - | ★★ Moderate |

---

## Key Takeaways

### What Parallax uniquely offers (no competitor has all of these):

1. **Sentry-envelope + OTLP ingest** in one engine
2. **Portable, versioned, redacted evidence bundles** as a typed artifact
3. **Fix-outcome tracking** (accepted/rejected/reverted/recurred)
4. **CLI/agent/CI session capture** as first-class signals
5. **Rust-first** capture quality in a single binary

### What Parallax's biggest risks are:

1. **OpenObserve** adding Sentry ingest (medium likelihood, would kill the wedge)
2. **A1 gate failure** — bundles don't improve agent outcomes (existential)
3. **Market too small** — niche³ of niche² of niche
4. **Distribution** — OSS observability monetization is historically hard
5. **Window closing** — 6-12 months before competitive closure accelerates

### The stack-it-yourself alternative:

For teams that don't want Parallax, the alternative is:
```
Sentry SDK + Bugsink/Rustrak (errors)
+ OTel Collector + Jaeger/Tempo (traces)
+ Loki/Quickwit (logs)
+ Prometheus/VictoriaMetrics (metrics)
+ HolmesGPT (AI investigation)
+ OTel MCP Server (agent access)
= 5-7 services, no unified evidence, no outcomes
```

This works. It's more operational overhead. But it exists today.
