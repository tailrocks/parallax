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
