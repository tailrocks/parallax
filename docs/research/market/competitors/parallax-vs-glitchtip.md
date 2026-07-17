# Parallax vs GlitchTip

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (**pass 53**
> first deep-dive). Sources: [glitchtip.com](https://glitchtip.com),
> [glitchtip.com/pricing](https://glitchtip.com/pricing) (FAQ + plan structure;
> Angular SPA hard to scrape — tier dollars cross-checked against 2026 secondary
> summaries and marked where primary HTML did not yield numbers),
> [documentation/install](https://glitchtip.com/documentation/install),
> [GitHub mirror](https://github.com/burke-software/GlitchTip) (MIT, 159★, last
> push 2026-02-10 — **stale mirror**; primary development is on **GitLab**),
> [MCP docs](https://glitchtip.com/documentation/mcp/), Bugsink/Rustrak peers.
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
  SPA if quoting contracts). Primary code on **GitLab** (`glitchtip/*`); GitHub
  mirror **159★**, last push **2026-02-10** (do not treat mirror stars as
  activity). Docs include **MCP**. Stack: Django + Postgres-class self-host.
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
| Hosted paid | Event tiers (~$15/100k, $50/500k, $250/3M — **secondary-confirmed 2026**; live SPA) | n/a |
| Throttle | FAQ: after quota, throttle 10% → block at 2× | n/a |

**No public number:** exact live SPA tier matrix not machine-extracted this pass;
use pricing page for purchase decisions.

## AI / agent

- **GlitchTip:** official **MCP documentation** — agent can talk to the error
  tracker (scope: issue/product ops; not multi-signal RCA bundle).
- **Parallax:** read-only local MCP + portable bundle thesis (A1).

**Verdict:** GlitchTip wins “agent manages my error tracker” for pure Sentry-alt
users (alongside Rustrak). Parallax’s bundle claim is different and unproven.

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

- GitHub mirror staleness vs GitLab cadence.
- MCP tool mutability (read-only vs write).
- OTLP expansion (unlikely; would change layer).

## Sources (2026-07-17)

- [glitchtip.com/pricing](https://glitchtip.com/pricing) FAQ (1k free events; event definition).
- [glitchtip.com/documentation/install](https://glitchtip.com/documentation/install) (donation $5/user; resource notes).
- Blog 2026-03 support $15/user; secondary 2026 tier restatements ($15/$50/$250).
- GitHub mirror `burke-software/GlitchTip` (MIT, 159★).
- Peers: [parallax-vs-bugsink.md](parallax-vs-bugsink.md), [parallax-vs-rustrak.md](parallax-vs-rustrak.md), [parallax-vs-sentry.md](parallax-vs-sentry.md).
