# Parallax vs Traceloop (OpenLLMetry)

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (pass 31;
> drift-verified 2026-07-17).
> Sources: [traceloop/openllmetry (GitHub)](https://github.com/traceloop/openllmetry)
> (7,307★ / 1,020 forks, Apache-2.0, Python, last push 2026-07-13 — active),
> [releases](https://github.com/traceloop/openllmetry/releases) (**v0.62.1**, published
> 2026-06-28), [`traceloop-sdk` on PyPI](https://pypi.org/project/traceloop-sdk/)
> (v0.62.1), [Traceloop blog](https://www.traceloop.com/blog/openllmetry),
> [Traceloop joining ServiceNow](https://traceloop.com/blog/traceloop-is-joining-servicenow),
> [cTech: $60–80M deal](https://www.calcalistech.com/ctechnews/article/sjghwiqf11e),
> [ServiceNow AI Control Tower (Traceloop acquisition completed)](https://newsroom.servicenow.com).
>
> **⚠️ Name correction (this pass):** the legacy roster listed "**Tracelo**" as an
> AI-agent-tracing competitor. **`tracelo.com` is a phone-geolocation service —
> not an observability product.** The intended tool is **Traceloop**
> (`traceloop.com` / `openllmetry`) — the real OSS OTel LLM-instrumentation SDK.
> "Tracelo" is retired from the roster as a mistaken identity; this deep-dive
> covers the real **Traceloop / OpenLLMetry**.
>
> **Bottom line up front:** Traceloop's **OpenLLMetry** is an **open-source
> (Apache-2.0) OpenTelemetry-native LLM-instrumentation SDK** — it auto-instruments
> LLM providers / frameworks / vector-DBs / MCP and exports OTLP traces (GenAI
> semantic conventions) to *any* backend. **It is a different layer from Parallax:
> an LLM instrumentation SDK, NOT a backend or context engine.** They are
> **complementary** — Parallax could *consume* OpenLLMetry's OTLP. On the
> LLM-auto-instrumentation axis, **Traceloop is decisively ahead of Parallax**
> (Parallax has no LLM-instrumentation SDK; it relies on OTel SDKs / OpenLLMetry /
> OpenInference). Parallax's edges are in the layers Traceloop does not touch
> (storage, error derivation, agent bundles, outcome loop).

## What each product is

- **Traceloop / OpenLLMetry** (`traceloop/openllmetry`) — an **open-source
  (Apache-2.0) OpenTelemetry-native LLM-instrumentation SDK**: auto-instruments
  **LLM providers** (OpenAI/Anthropic/Bedrock/Cohere/Gemini/Groq/Mistral/Ollama/
  Vertex/SageMaker/Together/Replicate/Watsonx/Aleph Alpha/HuggingFace/WRITER),
  **frameworks** (LangChain/LlamaIndex/LangGraph/LiteLLM/CrewAI/Haystack/Agno/
  OpenAI Agents/AWS Strands/Langflow), **vector DBs** (Chroma/Pinecone/Qdrant/
  Weaviate/Milvus/LanceDB/Marqo), and **MCP protocol**. Produces **OTel GenAI
  semantic-convention spans** (prompt/completion/token/cost/model) and exports
  **OTLP to any backend** — **24+ tested destinations** including Datadog,
  Honeycomb, New Relic, Grafana, Splunk, SigNoz, Sentry, HyperDX, Highlight,
  Axiom, Dynatrace, Arize/Braintrust, and the plain **OpenTelemetry Collector**.
  **Apache-2.0.** **Python primary** + JS/TS (`openllmetry-js`), Go, Ruby ports.
  It is an **instrumentation SDK** (the LLM analogue of [Odigos](parallax-vs-odigos.md)'s
  eBPF instrumentation) — not a backend, store, or context engine.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable
  **execution-context engine**: OTLP-native ingest of traces/logs/metrics +
  CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a
  typed evidence graph, serves bounded/redacted evidence bundles to humans and
  coding agents. GreptimeDB + Turso. **Pre-release.**

**Crucial framing:** OpenLLMetry is an **LLM-instrumentation SDK layer** (generate
LLM OTel traces, forward to a backend); Parallax is a **backend + context engine**
(ingest, store, derive, serve). They sit at **different stack layers** and are
**complementary** — OpenLLMetry can feed Parallax. Same layer-distinction logic as
[Odigos](parallax-vs-odigos.md) (eBPF instrumentation) — Traceloop is the
LLM-instrumentation peer.

## ServiceNow acquisition — material 2026 change

**Traceloop was acquired by ServiceNow** (announced 2025; deal valued
**~$60–80M**, ServiceNow's third Israeli acquisition in <3 months per
[CTech](https://www.calcalistech.com/ctechnews/article/sjghwiqf11e)). Traceloop's
technology is being folded into **ServiceNow's AI Control Tower**, which per
ServiceNow's newsroom "now delivers deep observability into AI agent behavior at
runtime" through the completed acquisition. The **OpenLLMetry OSS project
(60+ contributors) remains Apache-2.0 on GitHub and active** (last push
2026-07-13, latest release v0.62.1 four days before this pass).

**No-bias read of the acquisition:** this **strengthens Traceloop's enterprise
reach and durability** (ServiceNow backing, AI Control Tower distribution) — it is
not a sign of decline. The two-sided watch: (a) ServiceNow could **starve the OSS
SDK** in favor of the closed Control Tower — no evidence of that yet (release
cadence is healthy); (b) ServiceNow could **expand OpenLLMetry from
instrumentation into storage/backend/agent-governance**, which would be a direct
collision with Parallax's layers. Neither has fired. The SDK today is still a
pure instrumentation/forwarding layer.

## Signal coverage — Traceloop generates LLM traces; Parallax consumes

| Signal | Traceloop/OpenLLMetry (shipped) | Parallax (planned/shipped) |
| --- | --- | --- |
| LLM auto-instrumentation (providers/frameworks/vector-DBs/**MCP**) | ✅ **(the core — generates OTel GenAI spans)** | ❌ (relies on OTel SDKs / OpenLLMetry) |
| OTel GenAI semantic conventions | ✅ **(OpenLLMetry drove these into upstream OTel)** | 🟡 (🏗) |
| Token / cost / prompt-completion capture | ✅ | ✅ (🏗, from spans) |
| **Telemetry storage / backend** | ❌ (forwards OTLP) | ✅🧪 GreptimeDB (shipped, pre-release) |
| Error derivation / fingerprinting | ❌ | ✅ derived `error_event` (🧪 shipped) |
| Evidence bundle / agent context | ❌ | 🟡🧪 code (A1 unproven) |
| Sentry envelope / DSN | ❌ | ✅ shipped |

**Verdict:** **different layers.** OpenLLMetry excels at *generating* LLM
telemetry; Parallax excels (in design) at *deriving/serving* evidence from
telemetry. OpenLLMetry **does not store, derive errors, or serve agent context** —
exactly Parallax's layers.

## Ingestion & transport — the layer relationship

- **OpenLLMetry:** auto-instruments LLM calls → OTLP export to a backend. It is an
  **OTel-telemetry producer** (the LLM equivalent of Odigos's eBPF producer).
  **Parallax is not yet in OpenLLMetry's tested-destination list**, but because
  Parallax ingests standard OTLP, the **OpenTelemetry Collector destination**
  routes OpenLLMetry spans into Parallax with no SDK change — standard OTLP interop.
- **Parallax:** OTLP ingest gateway (consumer) + shipped Sentry-envelope adapter.

**Verdict:** OpenLLMetry is a **producer Parallax can consume.** They are
**pipeline-adjacent, not competitive.** On the LLM-auto-instrumentation axis,
**Traceloop is ahead of Parallax** (Parallax has no LLM-instrumentation SDK — it
depends on OTel SDKs / OpenLLMetry / OpenInference to generate LLM spans).

## Openness, licensing & vendor lock-in

- **OpenLLMetry:** **Apache-2.0** (OSI-open, same as Parallax). OTel-native,
  vendor-agnostic output. Zero lock-in at the SDK layer (it is a forwarding layer).
  The **Traceloop-hosted SaaS** (and now ServiceNow AI Control Tower) is closed, but
  the SDK is fully open and remains the canonical artifact.
- **Parallax:** Apache-2.0, fully open, OTLP-native, portable bundle.

**Verdict:** **tied on openness** — both Apache-2.0, OTLP-native. OpenLLMetry is a
strong OSS citizen: its GenAI semantic conventions were **upstreamed into
OpenTelemetry itself** (it helped define the standard Parallax will consume).

## Privacy posture (relevant to Parallax's redaction thesis)

OpenLLMetry **removed all SDK/instrumentation telemetry collection as of v0.49.2**
— the SDK no longer phones home (previously collected anonymous exception data to
catch provider API breakages). This is a **privacy-direction plus** for
OpenLLMetry; it does not, however, give OpenLLMetry any PII-redaction capability
(it forwards whatever spans it generates — redaction is the receiving backend's
job, which is Parallax's `REDACTION_POLICY_V1` layer, A1-unproven).

## Where Traceloop plainly wins

- **LLM auto-instrumentation breadth** — providers + frameworks + vector-DBs +
  **MCP protocol**, no manual spans (the widest OSS LLM-instrumentation coverage).
- **Drove the OTel GenAI semantic conventions** into upstream OpenTelemetry
  (Parallax will consume a standard OpenLLMetry helped write).
- **Apache-2.0 OSS** + active (v0.62.1, 7,307★, last push 2026-07-13).
- **Vendor-agnostic OTLP output** (24+ tested destinations incl. every major
  backend in this comparison set).
- **ServiceNow / AI Control Tower** enterprise backing and distribution.

## Where Parallax and Traceloop differ (not "Parallax wins")

- **Different stack layer** — OpenLLMetry generates LLM telemetry; Parallax
  stores/derives/serves. **Complementary, not competitive.**
- **Parallax's layers Traceloop doesn't touch:** storage (GreptimeDB), error
  derivation + outcome loop, agent-context bundle, Sentry-envelope, redaction.
- **Traceloop's layer Parallax doesn't touch:** LLM auto-instrumentation SDK
  (Parallax relies on OpenLLMetry/OpenInference/OTel SDKs to generate LLM spans).

> **Honest summary:** Traceloop/OpenLLMetry is **not a head-to-head competitor** —
> it is an **Apache-2.0 OTel LLM-instrumentation SDK layer** (now ServiceNow-owned,
> folded into AI Control Tower; OSS project remains Apache-2.0 and active) that
> *complements* Parallax — the LLM analogue of Odigos (eBPF instrumentation). On
> LLM auto-instrumentation, Traceloop is decisively ahead of Parallax (no
> LLM-instrumentation SDK) and helped write the GenAI OTel standard Parallax will
> consume. But OpenLLMetry does not store, derive errors, or serve agent context —
> exactly Parallax's layers. **A realistic stack: OpenLLMetry (auto-instrument LLM
> calls) → Parallax (ingest, derive errors, serve bounded agent bundles).**
> (Note: **"Tracelo" was a roster error** — tracelo.com is a phone-geo service; the
> real tool is Traceloop. "Tracelo" retired from the roster this pass.)

## Watch triggers (re-scan each pass)

- **Storage / backend expansion** — does OpenLLMetry or ServiceNow AI Control Tower
  grow into a telemetry *store* or agent-context layer? (Direct collision with
  Parallax.) **UNFIRED** — still pure instrumentation/forwarding + closed
  governance UI.
- **OSS cadence under ServiceNow** — does release cadence slow or the Apache-2.0
  license change? **Healthy so far** (v0.62.1, last push 2026-07-13).
- **GenAI-convention → Parallax mapping** — confirm OpenLLMetry's OTel GenAI spans
  map cleanly to Parallax's LLM-derive path once shipped.
- **OpenLLMetry vs OpenInference (Arize)** — two overlapping LLM-instrumentation
  standards (OpenLLMetry OTel-native vs Arize's OpenInference). Track convergence;
  Parallax should consume whichever wins (or both).

## Open questions / what would matter

- **OpenLLMetry → Parallax integration** — OpenLLMetry's OTLP feeds Parallax
  directly (via the OTel Collector destination). Worth a live spike once Parallax's
  LLM-derive path ships.
- **Parallax LLM-instrumentation gap** — Parallax relies on SDKs; OpenLLMetry is the
  canonical OSS LLM-instrumentation source. Decide whether to integrate (via
  OpenLLMetry) or build.

## Sources (accessed 2026-07-17)

- [traceloop/openllmetry (GitHub)](https://github.com/traceloop/openllmetry) — 7,307★,
  1,020 forks, Apache-2.0, Python, last push 2026-07-13 (GitHub API).
- [Releases · traceloop/openllmetry](https://github.com/traceloop/openllmetry/releases) —
  latest **v0.62.1** (2026-06-28).
- [`traceloop-sdk` PyPI](https://pypi.org/project/traceloop-sdk/) — v0.62.1.
- [Traceloop blog: Introducing OpenLLMetry](https://www.traceloop.com/blog/openllmetry);
  [Traceloop is joining ServiceNow](https://traceloop.com/blog/traceloop-is-joining-servicenow).
- [cTech: ServiceNow buys Traceloop, $60–80M](https://www.calcalistech.com/ctechnews/article/sjghwiqf11e);
  [ServiceNow AI Control Tower expansion (Traceloop completed)](https://newsroom.servicenow.com).
- Parallax side: [capture/agent-cli-tracing.md](../../capture/agent-cli-tracing.md),
  [architecture/integration-contract.md](../../architecture/integration-contract.md).
- Sibling (different-layer peers): [parallax-vs-odigos.md](parallax-vs-odigos.md)
  (eBPF instrumentation), [parallax-vs-helicone.md](parallax-vs-helicone.md)
  (LLM gateway/proxy), [parallax-vs-arize-phoenix.md](parallax-vs-arize-phoenix.md)
  (OpenInference — the competing LLM-instrumentation standard).
