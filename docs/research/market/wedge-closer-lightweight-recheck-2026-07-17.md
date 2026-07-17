# Wedge-closer recheck — lightweight + agent-first OSS (2026-07-17)

> Canonical deep-dives: [Traceway](competitors/parallax-vs-traceway.md), [Rustrak](competitors/parallax-vs-rustrak.md), [GlitchTip](competitors/parallax-vs-glitchtip.md), [Bugsink](competitors/parallax-vs-bugsink.md) — pass 50–54.

<!-- markdownlint-disable MD013 -->

**Pass target:** research-agenda item **#4** — does a wedge-closer ship the
**full Parallax combination** first?

```text
Sentry-compatible error ingest
+ OTLP traces/logs/metrics
+ low-resource self-host
+ portable versioned redacted evidence bundle
+ read-only agent-safe context
+ CLI/agent/CI action audit
+ fix-outcome / recurrence loop
```

**Prior status (theory):** "Checked 2026-06-11: not closed." Lightweight cohort
(Bugsink, Rustrak, Traceway, GoSnag, Urgentry) last deep-surveyed in
[competitor-watch.md](competitor-watch.md) (May 2026 bodies).

**Verdict (this pass):** **Combination still not closed.** No watched project
ships the full set. **Pressure increased**, especially from **Traceway**
(OTel-native multi-signal + agent-first CLI/skills) and **Rustrak** (Sentry +
shipped MCP). Parallax's remaining exclusive cells among this cohort remain
**portable redacted bundle + fix-outcome loop** (code-shipped / offline residual;
**value unproven A1**).

**Pass 122 recheck (2026-07-17):** Traceway GitHub API + README +
[tracewayapp.com/cloud](https://tracewayapp.com/cloud) primary re-fetch —
**still not closed**. Pins **unchanged**: **1,024★**, latest
**`backend/v1.9.1` + `cli/v1.9.1`** (2026-07-15), last push **2026-07-17**,
**MIT**. Cloud still public Free / **$12.99** / **$24.99** / **$499.99** +
overage **$0.25–$0.20/GB**; FAQ still claims **100% OSS, no feature gating**.
README still **no Sentry envelope**, **no versioned portable redacted evidence
bundle schema**, **no fix-outcome loop**. Agent path still skills + CLI + MCP.
Cohort table Traceway row below still valid; other cohort stars not re-polled
this pass (Traceway-focused).

**Pass 125 recheck (2026-07-17):** **Bugsink** + **Rustrak** primary re-poll —
combination **still not closed**.

| Product | Stars | Latest pin | Push | Combo cells |
| --- | --- | --- | --- | --- |
| **Bugsink** | **1,940** | **v2.4.0** (2026-07-10) | 2026-07-17 | Still **Sentry-compat error-only** (README); no OTLP multi-signal / portable bundle / outcome in README probe |
| **Rustrak** | **64** | **`@rustrak/server@0.9.2`** + **`@rustrak/mcp@0.2.13`** (2026-07-15) | 2026-07-17 | Still Sentry SDK path + **MCP**; no OTLP multi-signal / portable redacted bundle / outcome in README probe |

Pins match the cohort table rows (no material version/star move since table write).
Falsification for this pass: either ships OTLP full-signal **and** portable
redacted evidence bundle **and** outcome loop — **not observed**.

**Pass 134 recheck (2026-07-17):** **TMA1** + **GlitchTip** —

| Product | Pin | Watch |
| --- | --- | --- |
| **TMA1** | **109★**, **`v0.2.0-alpha12`** (2026-07-17); push 2026-07-17 | **18th UNFIRED** — recent commits = GreptimeDB **v1.1.3** min + session UI perf only. Still local-only agent loop; no Sentry envelope / portable redacted evidence bundle / fix-outcome collision |
| **GlitchTip** | GitLab `glitchtip` **161★** (last_activity 2026-07-06); backend **354★** / **v6.1.8** (2026-06-05) | Still Sentry-API error product; combo **not closed** |

**Evidence class:** primary GitHub API + README/release tags (2026-07-17). Not
a live deploy test of each product.

**Pass 173 recheck (2026-07-18):** **GlitchTip** version pin moved —

| Product | Pin | Combo |
| --- | --- | --- |
| **GlitchTip** | monorepo **161★** (act 2026-07-06); backend **354★** (act 2026-07-17); **backend tag `v6.2.1`** (2026-07-15) — was v6.1.8 | Still **Sentry-API error product** (README); MCP docs page live; **no** OTLP multi-signal / portable redacted bundle / outcome in primary probe. Combo **not closed**. |

Stars unchanged; **real release-line move** on backend only.

**Pass 179 recheck (2026-07-18):** **Bugsink** hygiene —

| Product | Pin | Combo |
| --- | --- | --- |
| **Bugsink** | **1,940★**; **v2.4.0** (2026-07-10); push 2026-07-17 | Still **Sentry-SDK error-only** (README); **no** OTLP multi-signal / portable redacted bundle / outcome. Combo **not closed**. |

**Pass 180 recheck (2026-07-18):** **Rustrak** hygiene —

| Product | Pin | Combo |
| --- | --- | --- |
| **Rustrak** | **64★**; **`@rustrak/server@0.9.2` + `@rustrak/mcp@0.2.13`** (2026-07-15); push 2026-07-17 | Still Sentry SDK + **MCP**; README **no** OTLP multi-signal / portable redacted bundle / outcome. Combo **not closed**. |

**Pass 181 recheck (2026-07-18):** **TMA1** —

| Product | Pin | Combo |
| --- | --- | --- |
| **TMA1** | **109★**; tag **`v0.2.0-alpha12`** still latest; push 2026-07-17 | **19th UNFIRED** — recent commits GreptimeDB **v1.1.3** min + session UI only. Local agent loop; **no** Sentry envelope / portable redacted prod evidence / fix-outcome collision. |

**Pass 185 recheck (2026-07-18):** **Traceway** escalator —

| Product | Pin | Combo |
| --- | --- | --- |
| **Traceway** | **1,024★**; **`backend/v1.9.1` + `cli/v1.9.1`** still latest (2026-07-15); `pushed_at` 2026-07-17 | Still **OTLP multi-signal + agent skills/CLI/MCP**. README probe: **no** Sentry envelope / portable redacted evidence bundle / fix-outcome. Combo **not closed**. |

**Pass 202 recheck (2026-07-18):** **TMA1** —

| Product | Pin | Combo |
| --- | --- | --- |
| **TMA1** | **109★**; **`v0.2.0-alpha12`** still latest; push 2026-07-17 | **20th UNFIRED** — commits still GreptimeDB **v1.1.3** min + session UI. No Sentry envelope / portable redacted prod evidence / fix-outcome collision. |

**Pass 204 recheck (2026-07-18):** **Traceway** —

| Product | Pin | Combo |
| --- | --- | --- |
| **Traceway** | **1,024★**; **`backend/v1.9.1` + `cli/v1.9.1`** still latest | Still OTLP multi-signal + agent path; README **no** Sentry/envelope/evidence-bundle/outcome. Combo **not closed**. |

**Pass 214 recheck (2026-07-18):** **Bugsink + Rustrak** hygiene —

| Product | Pin | Combo |
| --- | --- | --- |
| **Bugsink** | **1,940★** / **v2.4.0** | Sentry-SDK error-only; combo **not closed** |
| **Rustrak** | **64★** / server **0.9.2** + MCP **0.2.13** | Sentry+MCP error tracker; combo **not closed** |

**Pass 218 recheck (2026-07-18):** **Traceway** escalator —

| Product | Pin | Combo |
| --- | --- | --- |
| **Traceway** | **1,024★**; **`backend/v1.9.1` + `cli/v1.9.1`** still latest | OTLP multi-signal + agent path; **no** Sentry envelope / portable redacted bundle / outcome. Combo **not closed**. |

**Pass 219 recheck (2026-07-18):** **TMA1 + Maple** —

| Product | Pin | Watch |
| --- | --- | --- |
| **TMA1** | **109★** / **`v0.2.0-alpha12`** | **21st UNFIRED** prod-collision (no Sentry/redact/outcome) |
| **Maple** | **1,532★** / **v0.0.12** | Tinybird-decoupling **UNFIRED** |

**Pass 223 recheck (2026-07-18):** cohort star hygiene (versions unchanged) —

| Product | Stars | Note |
| --- | --- | --- |
| Bugsink | **1,940** / v2.4.0 | error-only |
| Rustrak | **64** | Sentry+MCP |
| Coroot | **7,837** / v1.23.3 | eBPF RCA |
| Maple | **1,532** / v0.0.12 | Tinybird still |
| TMA1 | **109** / alpha12 | **22nd UNFIRED** collision |
| Odigos | **3,668** / v1.31.2 | export-only |
| HolmesGPT | **2,874** / v0.36.0 | no own store |

Full wedge combination **still not closed**.

**Pass 232 recheck (2026-07-18):** **Traceway** cloud pricing + pins —

| Field | Value |
| --- | --- |
| Stars | **1,024** |
| Version | still **v1.9.1** (prior pin) |
| Cloud prices (scrape) | Free / **$12.99** / **$24.99** / **$499.99** + **$0.25–$0.20/GB** class overage still present |
| Rustrak / Holmes | **64★** / **v0.36.0** unchanged |

Combo still **not closed** (no Sentry envelope + portable redacted bundle + outcome).

**Pass 238 recheck (2026-07-18):** **Traceway** + engine/Seer watch batch —

| Field | Value |
| --- | --- |
| Traceway | still **1,024★**; **`cli/v1.9.1` + `backend/v1.9.1`** latest; README probe **no** sentry/envelope/evidence-bundle/outcome hits |
| GreptimeDB | stable **v1.1.3**; nightly release still **v1.2.0-nightly-20260706** |
| ClickHouse feature | still **v26.6.1.1193-stable** |
| Seer self-host | still **closed source** (develop.sentry.dev) |
| Grafana Assistant | still **hybrid Cloud LLM backend** |

Full wedge combination **still not closed**.

**Pass 241 recheck (2026-07-18):** cohort stars + kill-adjacent watches —

| Product | Stars (API) | Note |
| --- | --- | --- |
| Bugsink / Rustrak / TMA1 | **1,940** / **64** / **109** | error-only / Sentry+MCP / local agent |
| Maple / Coroot / Holmes | **1,532** / **7,837** / **2,874** | Tinybird / eBPF RCA / no store |
| Odigos / HyperDX / Langfuse / Phoenix | **3,668** / **9,680** / **31,341** / **10,600** | export / ClickStack / LLMOps / LLMOps |
| Bits Code | primary docs | still **never auto-merges** |
| Sentry OTLP metrics | primary docs | still **unsupported** |
| Datadog OPW | primary docs | still **route-to-destinations** Worker |

Full wedge combination **still not closed**.

**Pass 254 recheck (2026-07-18):** **HyperDX + Langfuse** neighbor pins —

| Product | Pin | Combo / note |
| --- | --- | --- |
| **HyperDX** | [hyperdxio/hyperdx](https://github.com/hyperdxio/hyperdx) **9,681★** (+1 vs pass 241 **9,680**); latest **`@hyperdx/app@2.30.1`** (2026-07-13); push 2026-07-17 | ClickStack / ClickHouse unified session+logs+metrics+traces+errors; **no** portable redacted evidence bundle + outcome in README probe. Combo **not closed**. |
| **Langfuse** | [langfuse/langfuse](https://github.com/langfuse/langfuse) **31,341★** (stable); **`v3.221.1`** (2026-07-17) | LLMOps self-host real; README still default **PostHog usage telemetry** on self-host (opt-out `TELEMETRY_ENABLED=false`). **Not** production multi-signal evidence-bundle product. Combo **not closed**. |

**Pass 255 recheck (2026-07-18):** **Phoenix + Coroot + HolmesGPT** —

| Product | Pin | Note |
| --- | --- | --- |
| **Phoenix** | [Arize-ai/phoenix](https://github.com/Arize-ai/phoenix) **10,600★**; **`arize-phoenix-v18.1.0`** (2026-07-17) | LLMOps eval/trace; **not** prod multi-signal evidence-bundle + Sentry envelope + outcome. Combo **not closed**. |
| **Coroot** | [coroot/coroot](https://github.com/coroot/coroot) **7,837★**; **`v1.23.3`** (2026-07-02) | eBPF RCA + MCP; AI RCA EE/Cloud-metered (pass 103). **No** portable redacted prod evidence bundle. Combo **not closed**. |
| **HolmesGPT** | [HolmesGPT/holmesgpt](https://github.com/HolmesGPT/holmesgpt) **2,874★**; **`0.36.0`** (2026-07-13) | Investigation agent **over external stores** (no own telemetry store). Not a wedge-closer alone. |

Full combination **still not closed**.

**Pass 278 recheck (2026-07-18):** Phoenix/Coroot/Holmes pins **unchanged**
(10,600★/v18.1.0; 7,837★/v1.23.3; 2,874★/0.36.0). Combo **not closed**.

**Pass 256 recheck (2026-07-18):** **TMA1 + Odigos** watches —

| Product | Pin | Watch |
| --- | --- | --- |
| **TMA1** | [tma1-ai/tma1](https://github.com/tma1-ai/tma1) **109★**; latest tag still **`v0.2.0-alpha12`**; push 2026-07-17 | Recent commits = GreptimeDB min **v1.1.3**, install probe, session-detail perf, launchd/codex hooks. Keyword scan last **40** commit messages: **no** sentry/redact/envelope/outcome/fingerprint/pii/evidence hits. **23rd UNFIRED** prod-collision (Sentry envelope / portable redacted prod evidence / fix-outcome). Still local-first LLM/agent loop scope. |
| **Odigos** | [odigos-io/odigos](https://github.com/odigos-io/odigos) **3,668★**; **`v1.31.2`** (2026-07-09) | eBPF auto-instrumentation **export** path — **not** own evidence store. Own-store collision **UNFIRED**. |

Full combination **still not closed**.

**Pass 270 recheck (2026-07-18):** **TMA1** only —

| Field | Value |
| --- | --- |
| Pin | still **109★** / **`v0.2.0-alpha12`**; push still **2026-07-17** |
| Trigger scan | last **25** commit messages: **no** sentry/redact/envelope/outcome/fingerprint/pii/scrub/evidence hits |
| Verdict | **24th UNFIRED** prod-collision |

Combo **still not closed**.

**Pass 279 recheck (2026-07-18):** **Odigos** — still **3,668★** / **`v1.31.2`**
(2026-07-09). Export-only eBPF auto-instrumentation; own-store collision
**UNFIRED**. Combo **not closed**.

**Pass 282 recheck (2026-07-18):** **Traceway + TMA1** dual —

| Product | Pin | Combo |
| --- | --- | --- |
| **Traceway** | still **1,024★** / **`backend/v1.9.1` + `cli/v1.9.1`**; push 2026-07-17; README **0** sentry/envelope/bundle/outcome/redact hits | **not closed** |
| **TMA1** | still **109★** / **`v0.2.0-alpha12`**; last 20 commits **no** collision keywords | **25th UNFIRED** prod-collision |

Full combination **still not closed**.

**Pass 292 recheck (2026-07-18):** **Traceway + Phoenix cohort** —

| Product | Pin | Note |
| --- | --- | --- |
| **Traceway** | still **1,024★** / **v1.9.1**; README **0** sentry/bundle/outcome | combo **not closed** |
| **Phoenix** | **10,600★** / **v18.1.0** | LLMOps; not full combo |
| **Coroot** | **7,837★** / **v1.23.3** | eBPF RCA; not full combo |
| **HolmesGPT** | **2,875★** (+1) / **0.36.0** | no own store |

Combo **still not closed**.

**Pass 293 recheck (2026-07-18):** **TMA1 + Odigos** —

| Product | Pin | Watch |
| --- | --- | --- |
| **TMA1** | **109★** / **alpha12**; 15 commits **no** collision keywords | **26th UNFIRED** |
| **Odigos** | **3,668★** / **v1.31.2** | export-only **UNFIRED** own-store |

Combo **still not closed**.

**Pass 298 recheck (2026-07-18):** **Traceway + TMA1 + Bugsink** —

| Product | Pin | Combo |
| --- | --- | --- |
| **Traceway** | still **1,024★** / **`backend/v1.9.1` + `cli/v1.9.1`** (2026-07-15); push 2026-07-17; README **0** sentry/envelope/bundle/outcome/redact | **not closed** |
| **TMA1** | still **109★** / **`v0.2.0-alpha12`**; 20 commits **no** collision keywords | **27th UNFIRED** |
| **Bugsink** | still **1,940★** / **v2.4.0** | error-only **not closed** |

Full combination **still not closed**.

**Pass 303 recheck (2026-07-18):** **Traceway + TMA1** (GO composite primaries) —

| Product | Pin | Combo |
| --- | --- | --- |
| **Traceway** | [tracewayapp/traceway](https://github.com/tracewayapp/traceway) still **1,024★**; latest releases still **`backend/v1.9.1` + `cli/v1.9.1`** (2026-07-15); `pushed_at` **2026-07-17** | Still OTLP multi-signal + agent CLI/MCP; prior README probe: **no** Sentry envelope / portable redacted evidence / fix-outcome — combo **not closed** |
| **TMA1** | [tma1-ai/tma1](https://github.com/tma1-ai/tma1) still **109★**; latest release still **`v0.2.0-alpha12`** (2026-07-17); README still local-first LLM/agent loop (Claude Code / Codex / Copilot hooks + GreptimeDB) | **28th UNFIRED** prod-incident collision (no Sentry envelope / portable redacted prod evidence / fix-outcome) |

Full combination **still not closed**.

**Pass 308 recheck (2026-07-18):** **Traceway + TMA1 + Bugsink** —

| Product | Pin | Combo |
| --- | --- | --- |
| **Traceway** | still **1,024★** / **`backend/v1.9.1` + `cli/v1.9.1`** (2026-07-15); `pushed_at` **2026-07-17**; MIT; README probe: **OTLP/OpenTelemetry** present; **0** sentry/envelope/evidence-bundle/redact/outcome | combo **not closed** |
| **TMA1** | still **109★** / **`v0.2.0-alpha12`** (2026-07-17); last 20 commits = GreptimeDB **v1.1.3** min + install probe + session UI perf — **0** collision keywords (sentry/envelope/redact/outcome/evidence/fingerprint/pii/bundle) | **29th UNFIRED** prod-incident collision |
| **Bugsink** | still **1,940★** / **v2.4.0** (2026-07-10); push 2026-07-17 | error-only peer **not closed** |

Full combination **still not closed**.

**Pass 317 recheck (2026-07-18):** **Traceway + TMA1 + Bugsink** —

| Product | Pin | Combo |
| --- | --- | --- |
| **Traceway** | still **1,024★**; latest still **`backend/v1.9.1` + `cli/v1.9.1`** (2026-07-15) | combo **not closed** |
| **TMA1** | still **109★** / **`v0.2.0-alpha12`** | **30th UNFIRED** prod-incident collision |
| **Bugsink** | still **1,940★** / **v2.4.0** | error-only **not closed** |

Full combination **still not closed**.



**Pass 258 recheck (2026-07-18):** **Maple + Uptrace** —

| Product | Pin | Watch |
| --- | --- | --- |
| **Maple** | [MapleTechLabs/maple](https://github.com/MapleTechLabs/maple) **1,532★**; latest release still **`v0.0.12`** (2026-06-18); push 2026-07-17 | Recent work = UI/trace perf, alerts v2, service-map — **not** Sentry envelope / portable redacted evidence / outcome. Code search **tinybird** still **total_count 304** → Tinybird-decoupling **UNFIRED**. Combo **not closed**. |
| **Uptrace** | [uptrace/uptrace](https://github.com/uptrace/uptrace) **4,242★**; latest **`v2.1.0-beta.7`** (2026-06-05); push 2026-06-14 | OTLP APM platform peer; **no** evidence that it ships Parallax full combo (Sentry envelope + portable redacted bundle + outcome). Combo **not closed**. |

**Pass 276 recheck (2026-07-18):** **Maple + HyperDX + Langfuse** —

| Product | Pin | Note |
| --- | --- | --- |
| **Maple** | **1,532★** / **v0.0.12**; tinybird code search still **304** | Tinybird-decoupling **UNFIRED**; combo **not closed** |
| **HyperDX** | **9,681★** / **`@hyperdx/app@2.30.1`** | ClickStack; no full Parallax combo |
| **Langfuse** | **31,342★** (+1 vs pass 254 **31,341**); **`v3.221.1`** | LLMOps; not prod multi-signal evidence-bundle |

Combo **still not closed**.

**Pass 289 recheck (2026-07-18):** Maple still **1,532★** / **v0.0.12**; tinybird
search still **304** — Tinybird-decoupling **UNFIRED**. OpenObserve still
**v0.91.2**. Combo **not closed**.

**Pass 249 recheck (2026-07-18):** **Traceway** + Sentry OTLP metrics kill —

| Field | Value |
| --- | --- |
| Traceway | [tracewayapp/traceway](https://github.com/tracewayapp/traceway) still **1,024★**; latest tags still **`backend/v1.9.1` + `cli/v1.9.1`** (2026-07-15); `pushed_at` **2026-07-17** |
| Traceway product | Still **OTLP/HTTP multi-signal** (logs/traces/metrics) + agent **skills** + exceptions/RUM/AI tracing (README) |
| Traceway gaps (README probe) | **0** hits for Sentry envelope / portable redacted evidence bundle / fix-outcome / redaction contract |
| [Sentry OTLP](https://docs.sentry.io/concepts/otlp/) | Page describes OTLP **traces and logs** (open beta). Explicit: **"Sentry does not support OTLP metrics at this time."** — kill **UNFIRED** |

Full wedge combination **still not closed** (Traceway lacks Sentry envelope + portable redacted bundle + outcome; Sentry lacks OTLP metrics).

**Pass 264 recheck (2026-07-18):** **Traceway** hygiene only —

| Field | Value |
| --- | --- |
| Stars / push | still **1,024★**; `pushed_at` still **2026-07-17** |
| Latest tags | still **`backend/v1.9.1` + `cli/v1.9.1`** (2026-07-15) |
| README probe | still **0** hits for sentry / envelope / evidence-bundle / fix-outcome / redact |

Combo **still not closed**.

**Pass 274 recheck (2026-07-18):** **Traceway** + Grafana Assistant —

| Field | Value |
| --- | --- |
| Traceway | still **1,024★** / **v1.9.1**; README **0** sentry/bundle/outcome hits — combo **not closed** |
| Grafana Assistant self-managed | still **hybrid**: backend/usage/billing in **Grafana Cloud**; prompts leave self-managed — offline BYO-LLM **UNFIRED** |

Combo **still not closed**.

**Pass 244 recheck (2026-07-18):** **Bugsink + Rustrak + GlitchTip** primary
version/README hygiene (error-tracker peer cluster) —

| Product | Pin (API / GitLab) | README / primary probe | Combo |
| --- | --- | --- | --- |
| **Bugsink** | [bugsink/bugsink](https://github.com/bugsink/bugsink) **1,940★**; **`v2.4.0`** (2026-07-10); `pushed_at` **2026-07-17** | Self-hosted **Sentry-SDK error tracking**; **0** hits for OTLP / OpenTelemetry / evidence-bundle / outcome / redact / MCP | **not closed** |
| **Rustrak** | [rustrak/rustrak](https://github.com/rustrak/rustrak) **64★**; **`@rustrak/server@0.9.2`** + **`@rustrak/mcp@0.2.13`** (2026-07-15 tags); push **2026-07-17** | Sentry SDK path + **MCP** package; architecture still Sentry→server→Postgres; **no** OTLP multi-signal / portable redacted bundle / fix-outcome | **not closed** |
| **GlitchTip** | GitLab monorepo **161★** (last_activity **2026-07-06**); [glitchtip-backend](https://gitlab.com/glitchtip/glitchtip-backend) **354★** (act **2026-07-17**); backend tag still **`v6.2.1`** (2026-07-15) | Sentry-API compatible **error tracking** (Django/Postgres); privacy/self-host messaging; **no** OTLP multi-signal / portable redacted evidence bundle / outcome in primary README | **not closed** |

**Verdict:** peer error-trackers remain **Sentry-compat (and Rustrak MCP)** —
none ship OTLP multi-signal + portable redacted evidence bundle + outcome.
Full combination **still not closed**. Version pins **unchanged** vs pass
173/179/180/214 (stable).

**Evidence class:** GitHub API + GitLab API + raw README probes (desk). Not A1.

**Pass 272 recheck (2026-07-18):** same three peers — pins **unchanged**
(Bugsink **1,940★**/v2.4.0; Rustrak **64★**/0.9.2+mcp0.2.13; GlitchTip backend
**v6.2.1**/354★). README probes still **no** OTLP multi-signal + portable
redacted bundle + outcome. Combo **not closed**.

**Pass 287 recheck (2026-07-18):** Bugsink **1,940★**/v2.4.0; Rustrak **64★** /
server **0.9.2** + MCP **0.2.13**; GlitchTip backend **v6.2.1** — **unchanged**.
SigNoz still **v0.133.0** / **~30,319★**; Noz docs still tagged **`SigNoz Cloud`**.
Combo **not closed**.

**Pass 156 recheck (2026-07-18):** **Traceway-focused** wedge re-poll + Bugsink/Rustrak
star-pin hygiene — combination **still not closed**.

| Product | Stars | Latest pin | Push | Combo cells |
| --- | --- | --- | --- | --- |
| **Traceway** | **1,024** | **`backend/v1.9.1` + `cli/v1.9.1`** (2026-07-15; still latest release tags) | `pushed_at` **2026-07-17** (API); newest commits on default branch are still **2026-07-15** release/website/widget work | Still **OTLP/HTTP multi-signal + agent skills/CLI/MCP**. README probe: **no** Sentry envelope, **no** versioned portable redacted evidence bundle, **no** fix-outcome/recurrence loop. Cloud still Free / **$12.99** / **$24.99** / **$499.99** + **$0.25–$0.20/GB** overage ([tracewayapp.com/cloud](https://tracewayapp.com/cloud)). License **MIT**. |
| **Bugsink** | **1,940** | **v2.4.0** (2026-07-10) | 2026-07-17 | Unchanged error-only Sentry-compat pin |
| **Rustrak** | **64** | **`@rustrak/server@0.9.2` + `@rustrak/mcp@0.2.13`** (2026-07-15) | 2026-07-17 | Unchanged Sentry + MCP pin |

**Evidence class (pass 156):** `gh api` repos + releases (authenticated); raw README;
public cloud HTML pricing scrape. **Not** a live deploy of each product.
**Falsification still unfired:** full combo (Sentry path **and** OTLP multi-signal
**and** portable redacted bundle **and** outcome loop) in one watched product.

---

## Cohort snapshot (pinned 2026-07-17; stars re-confirmed pass 156)

| Product | Repo | Stars | Latest pin | Stack | Sentry path | OTLP multi-signal | Agent surface | Portable redacted bundle | Outcome loop |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **Bugsink** | [bugsink/bugsink](https://github.com/bugsink/bugsink) | **1,940** | **v2.4.0** (2026-07-10); push 2026-07-17 | Python/Django | ✅ full Sentry server | ❌ error-only | ❌ (classic error product) | ❌ | ❌ |
| **Rustrak** | [rustrak/rustrak](https://github.com/rustrak/rustrak) (was AbianS/…) | **64** | **@rustrak/server@0.9.2** + **@rustrak/mcp@0.2.13** (2026-07-15) | Rust + Postgres | ✅ Sentry SDK/DSN | ❌ (error tracker) | ✅ **MCP 18 tools** ("full control" — includes mutating resolve/manage) | ❌ | ❌ |
| **Traceway** | [tracewayapp/traceway](https://github.com/tracewayapp/traceway) | **1,024** | **backend/v1.9.1** + **cli/v1.9.1** (2026-07-15) | Go + ClickHouse/Postgres or SQLite/DuckDB | ❌ (no Sentry in README/docs crawl) | ✅ **OTLP/HTTP** traces+logs+metrics + exceptions + RUM/replay + AI tracing | ✅ **CLI + skills + MCP** (local stdio `traceway mcp` **and** remote `/mcp` OAuth); mostly read-only (archive only mutates). Full deep-dive: [competitors/parallax-vs-traceway.md](competitors/parallax-vs-traceway.md) | ❌ no versioned portable schema found | ❌ |
| **GoSnag** | [darkspock/gosnag](https://github.com/darkspock/gosnag) | **9** | last push **2026-04-17** | Go Sentry-compat | ✅ claimed | ❌ | ❌ | ❌ | ❌ |
| **Urgentry** | [urgentry/urgentry](https://github.com/urgentry/urgentry) | **63** | last push **2026-07-01** | Sentry-compat self-host | ✅ claimed | ❌ | ❌ | ❌ | ❌ |
| **GlitchTip** | GitLab primary (GitHub mirrors only) | n/a here | not re-pinned to a single GitHub release this pass | Django Sentry-compat | ✅ | ❌ classic | third-party MCP exists (`vltansky/glitchtip-mcp`, low stars) | ❌ | ❌ |

Sources: GitHub REST API `repos/*` + `releases` + README bodies fetched 2026-07-17.

---

## Material deltas since May/June notes

### 1. Traceway is the cohort escalator

May notes pegged Traceway ~817★ / backend ~v1.7.x. **Now 1,024★ / backend
v1.9.1**, with an explicit **AI-First** section: agent skills installable via
`npx skills add tracewayapp/traceway`, agent-shaped CLI (JSON when piped,
stable exit codes), OTel-native multi-signal including **exceptions with
SHA-256 fingerprint grouping** and **AI tracing**.

**What this pressures:** "open self-hosted OTel + agent-native investigation"
is **no longer scarce**. Traceway is a live, growing product on that axis.

**What it does *not* close:**

- No Sentry-envelope migration path found in README/docs crawl.
- No portable, versioned, redacted, validator-backed **evidence bundle**
  artifact (agent query tools ≠ Parallax bundle contract).
- No open fix-outcome / recurrence / autonomy-budget substrate.
- Language filter: Go (in-scope) but not Rust-first; storage is not GreptimeDB.

**Falsification trigger for next pass:** Traceway ships Sentry envelope *and*
a published JSON Schema for a redacted multi-signal investigation export *and*
outcome rows.

### 2. Rustrak shipped MCP — with write power

Prior May notes treated Rustrak as ultra-light Sentry-compat (43★). **Now 64★**,
org rename to `rustrak/rustrak`, monorepo packages including
**`@rustrak/mcp` v0.2.13** marketed as giving AI assistants "**full control**"
(18 tools: issues, events, projects, **resolve**, tokens, …).

**What this pressures:** MCP on self-hosted Sentry-compat is **table stakes**,
not a Parallax differentiator. Mutating MCP also validates Parallax's
**read-only-first** safety stance as a deliberate contrast (not a lag).

**What it does *not* close:** Still error-only (no OTLP multi-signal), no
portable bundle schema, no outcome loop.

### 3. Bugsink remains the pure Sentry-replacement leader in the lightweight set

**v2.4.0 / 1,940★ / active same day.** Still error-only. Confirms the May
skeptical claim: **"simpler than Sentry" / Sentry-compat migration is not a
moat** — Bugsink owns that job more completely than Parallax's envelope adapter.

### 4. GoSnag / Urgentry / GlitchTip

- **GoSnag:** low stars (9), last push April 2026 — **not** a near-term wedge
  closer.
- **Urgentry:** 63★, active July 2026 — watch, still Sentry-compat niche.
- **GlitchTip:** still the mature Django Sentry-compat; GitHub is mirror-land;
  not re-deep-dived this pass. Third-party MCP exists; not a combination closer.

---

## Combination matrix (honest)

| Cell | Who has it among cohort | Still open for Parallax? |
| --- | --- | --- |
| Sentry-compat deep error product | Bugsink, Rustrak, Urgentry, GlitchTip | No — **not** a unique claim |
| OTLP multi-signal self-host | **Traceway** (strong), also SigNoz/OpenObserve outside this micro-cohort | No for "OTLP alone" |
| Agent access (CLI/MCP/skills) | Rustrak MCP, Traceway CLI+skills | No for "MCP/CLI alone" |
| Low-ops single/simple deploy | Bugsink, Rustrak, Traceway docker | Contested |
| **Portable redacted versioned evidence bundle** | **None found** | **Yes — unproven value (A1)** |
| **Fix-outcome / recurrence open records** | **None found** | **Yes — offline residual only** |
| Full combination above | **None** | Wedge **not closed** |

---

## Confidence and uncertainty

| Item | Confidence | Notes |
| --- | --- | --- |
| No full combination shipped | **High** | Multiple primary READMEs + release tags |
| Traceway has no Sentry path | **Medium-high** | README + docs homepage text search; could hide in unlinked docs |
| Traceway has no portable bundle schema | **Medium-high** | No schema files/marketing; live product may add without version pin |
| Rustrak MCP tool mutability | **High** | README claims "full control" + resolve tools present |
| Performance of any candidate | **Unmeasured** | Out of scope this pass |

---

## Implication for GO / positioning

1. **Do not market** "self-hosted Sentry alternative" or "agent MCP available"
   as unique. Those are occupied.
2. **Do market** (only if A1/A3 eventually hold): open **portable redacted
   production-incident evidence contract** + **outcome-fed autonomy substrate**,
   composed over OTel + Sentry grouping, serving coding agents.
3. **Watch Traceway weekly** for Sentry ingest or investigation-export schema —
   highest velocity threat in the lightweight+agent OSS lane.
4. **Watch Rustrak** only for OTLP expansion (would still miss bundle/outcome).

Canonical per-product deep-dives remain under
[`competitors/`](competitors/) (Bugsink done; Traceway/Rustrak still mostly in
legacy [competitor-watch.md](competitor-watch.md) — promote if Traceway fires
a trigger).

## Next pass candidates

1. Dedicated **Traceway deep-dive** into `competitors/parallax-vs-traceway.md`
   (missing from canonical roster).
2. A1 empirical (still #1 product gate).
3. A2 interviews (still #1 business gate).
