# Parallax vs Rustrak

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (**pass 51**
> first canonical deep-dive; **pass 89 pin recheck**). Sources: primary
> [github.com/rustrak/rustrak](https://github.com/rustrak/rustrak) (README,
> LICENSE, releases, `packages/mcp`),
> [rustrak.github.io/rustrak](https://rustrak.github.io/rustrak/), pass-49
> [wedge-closer recheck](../wedge-closer-lightweight-recheck-2026-07-17.md).
>
> **Bottom line up front:** Rustrak is a **Rust, Sentry-SDK-compatible,
> self-hosted error tracker** with a **shipped MCP server that can mutate issue
> state** ("full control"). On **lightweight Rust Sentry-compat + agent MCP for
> issue management, Rustrak is a real product** (small but active). It does
> **not** threaten Parallax's multi-signal evidence-engine thesis — it is
> **error-only**, **no OTLP**, **no portable redacted bundle**, **no outcome
> loop**. It **does** kill any claim that "Rust self-hosted Sentry + MCP" is
> Parallax-unique. **License is GPL-3.0** (not Apache) — stronger copyleft than
> Parallax Apache-2.0 / Traceway MIT / Bugsink mixed.

## What each product is

- **Rustrak** (`rustrak/rustrak`, org; author Abian Suarez) — **ultra-lightweight
  self-hosted error tracking compatible with Sentry SDKs**. Point existing Sentry
  SDKs at Rustrak via DSN. **Rust + Actix-web** server, **PostgreSQL** store,
  web UI (`webview-ui`). Packages: `@rustrak/server` **v0.9.2**, `@rustrak/mcp`
  **v0.2.13**, `@rustrak/client`, docs (2026-07-15 releases — **pass 89:** still
  latest tags; npm lists client/mcp; server image `rustrak/rustrak-server`).
  **GPL-3.0** (LICENSE + badge). **64★, last push 2026-07-17.** README default
  deploy is **SQLite** (optional PostgreSQL). Homepage docs at
  `rustrak.github.io/rustrak`. **Pass 89:** still **no OTLP / portable bundle /
  outcome loop** in README; MCP remains the differentiator vs pure Sentry-compat.
- **Parallax** — Apache-2.0, Rust-first **execution-context engine**: OTLP +
  Sentry envelope, multi-signal correlation, bounded redacted evidence bundles,
  CLI/GraphQL/UI, local-stdio read-only MCP. GreptimeDB + Turso. **Pre-release.**

**Layer honesty:** both are Rust and touch Sentry protocol. Rustrak is a
**Sentry-server substitute** (issue lifecycle product). Parallax is a
**multi-signal context engine** that *also* ingests Sentry envelopes. Same
language family; different jobs.

## Signal coverage

| Signal | Rustrak (shipped) | Parallax |
| --- | --- | --- |
| Errors / Sentry events | ✅ full product (Sentry SDK DSN) | ✅ envelope ingest + derived errors |
| Issue lifecycle (resolve/mute/…) | ✅ (+ MCP can drive it) | 🟡 issue APIs exist; not Sentry-parity product |
| Traces OTLP | ❌ | ✅🧪 |
| Logs OTLP | ❌ (MCP has a `logs` tool module — product logs, not OTLP lake) | ✅🧪 |
| Metrics OTLP | ❌ | ✅🧪 |
| Transactions (APM-ish) | 🟡 MCP `transactions` tool module present — verify depth next pass | 🟡 |
| Evidence bundle | ❌ | 🟡🧪 A1 unproven |
| Fix-outcome loop | ❌ | 🟡 offline residual |

**Verdict:** Rustrak **wins pure Sentry-error self-host simplicity** among Rust
options. Parallax **wins multi-signal + designed bundle/outcome**.

## Ingestion

- **Rustrak:** Sentry SDK protocol only (documented Python/JS/Go examples).
  **No OTLP** in README.
- **Parallax:** OTLP gRPC/HTTP + Sentry envelope.

## Storage

- **Rustrak:** **PostgreSQL** (classic app DB — not an observability lake).
- **Parallax:** GreptimeDB telemetry + Turso metadata.

**Verdict:** different regimes. Postgres error store ≠ GreptimeDB evidence store.
Language-filter: both Rust servers; Rustrak still pulls a heavy RDBMS ops story
for the data plane.

## Agent surface (important contrast)

### Rustrak MCP (`@rustrak/mcp` v0.2.13) — **pass 54 inventory**

- npm package; README: gives AI assistants **"full control"** of the instance.
- **Source recount (2026-07-17 GitHub `packages/mcp/src/tools/*.ts`):** **56
  registered tools** across 11 modules — **not 18** (pass 49/51 marketing /
  wedge-closer figure was **wrong/understated**).
  - **issues (22):** list/get + **mutating** resolve/unresolve/mute/delete/
    update_status/assign/bulk_update/bulk_delete/comment/bookmark/subscribe/
    mark_seen + reports + `record_deploy`
  - **team (9):** members + **mutating** role/invite/remove + project members
  - **storage (6):** summary + **mutating** cleanup/GC (`execute_storage_cleanup`,
    `gc_storage_source_maps`)
  - **tokens (4):** list/get + **mutating** create/revoke
  - **transactions (4), projects (3 incl. create), alerts (3), events (2),
    health/logs/sessions (1 each)**
- Clients: Claude Desktop, Cursor, Continue — env `RUSTRAK_API_URL` +
  `RUSTRAK_API_TOKEN`.

### Parallax MCP

- Local-stdio **read-only** (plan 112 DONE); remote deferred.
- Bundle projection + evidence tools, not issue-state admin as the center.

**Honest verdict:** Rustrak is **far ahead on "agent manages my error tracker"**
(write-capable MCP with **dozens** of mutating admin tools — stronger write
surface than previously recorded). Parallax is **stricter on agent safety**
(read-only first) and aims at **different payload** (evidence bundle).
**Mutating MCP is not a Parallax goal to copy** — it is a safety warning.

## License / economics (no-bias)

| Axis | Rustrak | Parallax |
| --- | --- | --- |
| License | **GPL-3.0** | **Apache-2.0** |
| Self-host cost | Free (self-run Postgres) | Free (self-run engines) |
| SaaS | No public cloud product found this pass | None yet |
| Stars / maturity | 64★, pre-1.0 packages (0.9.x) | Pre-release product |

**GPL-3.0** is a real constraint for some commercial embed/SaaS redistribution
stories; Apache-2.0 is more permissive for those. That is a **Parallax edge for
enterprise redistribution**, not a product-capability edge. Do not overclaim
"openness" — both are open source; GPL is freer for end-users, stricter for
proprietary forks.

## Where each wins (scoped)

| Axis | Winner | Why |
| --- | --- | --- |
| Rust Sentry-compat error server | **Rustrak** (narrow) | Purpose-built DSN server + UI |
| Multi-signal OTLP evidence | **Parallax** | Rustrak has none |
| Agent MCP for issue admin (incl. writes) | **Rustrak** | Full-control MCP |
| Agent-safe read-only evidence | **Parallax design** | Deliberate; A1 unproven |
| License for proprietary redistribution | **Parallax** (Apache) | vs GPL-3.0 |
| Observability storage for long retention | **Parallax design** | Postgres vs GreptimeDB — unmeasured |

## Watch triggers

| Trigger | Status 2026-07-17 |
| --- | --- |
| OTLP multi-signal ingest | **UNFIRED pass 58** (recent commits: no otlp/otel hits; still Sentry-path error tracker) |
| Portable redacted evidence bundle | **UNFIRED** |
| Fix-outcome loop | **UNFIRED** |
| License change (GPL → more permissive) | Watch |
| Stars/adoption jump past niche | Watch (64★ today) |

## Falsification

- "No Rust self-hosted Sentry with MCP" → **false** (Rustrak ships both).
- "Rustrak closes Parallax combination" → **false** without OTLP + bundle +
  outcome.
- "Apache is always freer than Rustrak" → **nuanced**; GPL restricts
  proprietary redistribution more, not self-host use.

## Related

- [parallax-vs-bugsink.md](parallax-vs-bugsink.md) — deeper Sentry-replacement
  product (Python; fuller issue lifecycle maturity).
- [parallax-vs-traceway.md](parallax-vs-traceway.md) — OTel multi-signal + safer
  MCP contrast.
- [wedge-closer-lightweight-recheck-2026-07-17.md](../wedge-closer-lightweight-recheck-2026-07-17.md)
