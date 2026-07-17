# Parallax vs GlitchTip

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (**pass 53**
> first deep-dive; **pass 59** GitLab star pin; **pass 93** re-verify;
> **pass 134** GitLab re-pin). Sources:
> [glitchtip.com](https://glitchtip.com),
> [glitchtip.com/pricing](https://glitchtip.com/pricing) (FAQ + plan structure;
> Angular SPA hard to scrape — tier dollars cross-checked against 2026 secondary
> summaries and marked where primary HTML did not yield numbers),
> [documentation/install](https://glitchtip.com/documentation/install),
> [GitLab primary](https://gitlab.com/glitchtip/glitchtip) (**161★**, last_activity
> **2026-07-06** — GitLab API pass 134; backend project **354★**, last_activity
> **2026-07-17**),
> [GitHub mirror](https://github.com/burke-software/GlitchTip) (stale — do not use
> mirror stars as activity),
> [MCP docs](https://glitchtip.com/documentation/mcp/), Bugsink/Rustrak peers.
> **Pass 134:** still Sentry-API error tracker; backend releases latest
> **v6.1.8** (2026-06-05) — **no OTLP full-signal + portable redacted bundle +
> outcome** combination.
>
> **Bottom line up front:** GlitchTip is a **mature, MIT, Sentry-API-compatible
> error-tracking product** (Django/Python) with hosted + free self-host, optional
> uptime/performance event metering, and **official MCP docs**. On **classic
> Sentry-replacement issue workflow + self-host simplicity + public hosted
> pricing, GlitchTip (like Bugsink) is a fuller error product than Parallax's
> envelope-ingest path.** It does **not** compete on multi-signal OTLP evidence
> or portable redacted bundles. Parallax edges = OTLP full-signal + Sentry
> envelope as *one input* + bundle/outcome thesis (**A1 unproven**). **Do not
> claim “self-hosted Sentry-compatible OSS” as Parallax-unique.**

## What each product is

- **GlitchTip** — open-source **Sentry API-compatible error tracking** (also
  meters uptime checks, performance transactions, and release file storage as
  “events” on hosted). **MIT.** Self-host free (suggested donation **$5/user/mo**;
  enterprise support **$15/user/mo** per blog 2026-03). Hosted Free **1,000
  events/mo**; paid tiers reported as **~$15 / 100k**, **~$50 / 500k**, **~$250 /
  3M** events/mo (2026 secondary re-statements of pricing page — re-confirm live
  SPA if quoting contracts). Primary code on **GitLab** (`glitchtip/glitchtip`
  **161★**, last_activity **2026-07-06**); GitHub mirror **159★**, last push
  **2026-02-10** (do not treat mirror stars as activity). Docs include **MCP**.
  Stack: Django + Postgres-class self-host.
- **Parallax** — Apache-2.0 Rust **execution-context engine**: OTLP + Sentry
  envelope ingest, multi-signal correlation, bounded redacted bundles
  (code-shipped; A1 unproven), local-stdio MCP. GreptimeDB + Turso. **Pre-release.**

**Layer honesty:** both can accept Sentry-shaped traffic; GlitchTip **is** a
Sentry-server substitute. Parallax **consumes** envelopes into a broader engine.

## Signal coverage

| Signal | GlitchTip | Parallax |
| --- | --- | --- |
| Errors / issues (Sentry API) | ✅ core product | ✅🧪 envelope ingest + derived errors |
| Performance transactions | 🟡 metered as “events” on hosted | 🟡 via OTLP traces (not Sentry perf product) |
| Uptime | 🟡 hosted event type | ❌ |
| OTLP traces/logs/metrics | ❌ classic error/APM-lite product | ✅🧪 all three |
| Session replay | ❌ | ❌ |
| Portable redacted bundle | ❌ | 🟡🧪 code-shipped, A1 unproven |
| Agent MCP | ✅ docs (product MCP) | ✅🧪 local-stdio read-only |

## Pricing & economics

| Mode | GlitchTip | Parallax |
| --- | --- | --- |
| Self-host | Free MIT; optional donation $5/user; support $15/user | Free Apache (pre-release) |
| Hosted Free | **1,000 events/mo** forever | n/a |
| Hosted paid | **Small $15/mo (100k events)** / **Medium $50 (500k)** / **Large $250 (3M)** — Free **1k** forever (pricing FAQ + 2026 re-statements of live page; SPA still hard to scrape machine-side) | n/a |
| Throttle | FAQ: after quota, throttle 10% → block at 2× | n/a |
| Events unit | Issues + uptime checks + perf transactions + release file MB | n/a |

Large plan: development support prioritization + BAA on request (pricing FAQ).
HIPAA hosting add-on available for Large (pricing page).

## AI / agent (**pass 54** — live [MCP docs](https://glitchtip.com/documentation/mcp/))

- **GlitchTip MCP:** built-in Streamable HTTP endpoint (`GLITCHTIP_ENABLE_MCP=True`,
  `/mcp`); OAuth 2.0 dynamic client registration **or** API token.
- **17 tools** documented:
  - Orgs/projects: `list_organizations`, `list_projects`
  - Issues: list/get/event tools + **`update_issue` (resolve / unresolve / ignore)** ← **mutating**
  - Performance (DuckDB-gated spans): transaction groups, N+1 detect, trends
  - Alerts/monitors, logs
- **Not** a portable redacted multi-signal evidence bundle.
- **Parallax:** local-stdio **read-only** MCP + portable bundle thesis (A1).

**Verdict:** GlitchTip wins “agent manages my error tracker” for pure Sentry-alt
users (alongside Rustrak’s larger write surface). **Mutating `update_issue` is
shipped** — not read-only. Parallax’s bundle claim is different and unproven.

## Where GlitchTip wins

- Mature Sentry-compatible issue product + hosted SKU.
- MIT self-host free with light resource story (docs: can run small).
- Public event-based hosted pricing (cheaper narrative vs Sentry for many).
- Official MCP docs.

## Where Parallax edges (scoped)

- Multi-signal OTLP + GreptimeDB native tables.
- Sentry envelope as input to **broader** evidence graph (not a Sentry server).
- Portable redacted versioned bundle + outcome loop (A1 / live value unproven).
- Apache-2.0 vs MIT is minor (both OSI).

## Watch

- ~~GitLab API star pin~~ → **RESOLVED pass 59:** [gitlab.com/glitchtip/glitchtip](https://gitlab.com/glitchtip/glitchtip) **161★**, last_activity **2026-07-06**, forks **14** (GitLab REST API).
- GitHub mirror **still stale** (last push **2026-02-10**, 159★); product site/docs active 2026-07. Prefer GitLab stars for primary-project health.
- ~~MCP tool mutability~~ → **RESOLVED pass 54: `update_issue` mutates** (resolve/unresolve/ignore).
- OTLP expansion (unlikely; would change layer) — **UNFIRED pass 59**.

## Sources (2026-07-17; pass 59)

- [glitchtip.com/pricing](https://glitchtip.com/pricing) FAQ (1k free events; event definition; Medium/Large support tiers).
- [glitchtip.com/documentation/mcp](https://glitchtip.com/documentation/mcp/) (**17 tools**, OAuth/token, mutating update_issue).
- [glitchtip.com/documentation/install](https://glitchtip.com/documentation/install) (donation $5/user).
- Blog 2026-03 support $15/user; 2026 secondary re-statements of $15/$50/$250 tiers.
- [GitLab primary](https://gitlab.com/glitchtip/glitchtip) **161★** (API 2026-07-17).
- GitHub mirror `burke-software/GlitchTip` (MIT, 159★, push 2026-02-10).
- Peers: [parallax-vs-bugsink.md](parallax-vs-bugsink.md), [parallax-vs-rustrak.md](parallax-vs-rustrak.md), [parallax-vs-sentry.md](parallax-vs-sentry.md).
