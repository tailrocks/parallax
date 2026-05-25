# Rustrak Sentry MCP Protocol Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

Re-check Rustrak because it is the closest lightweight product-shape warning for
Parallax:

```text
Rust + Sentry-compatible ingest + low-ops self-hosting + MCP
```

This pass tests whether Rustrak has closed the Parallax wedge or only raised the
minimum bar for Phase 1.

## Short Verdict

Rustrak is a serious product-shape baseline, but not a wedge closer.

It proves that Rust-first, Sentry-compatible, low-footprint error tracking with
MCP can exist as an open project. Parallax should therefore stop treating any of
these as differentiators on their own:

- Rust server implementation;
- Sentry SDK DSN migration language;
- SQLite-first or low-process self-hosting;
- MCP availability.

The remaining Parallax gap is still meaningful:

```text
Sentry-compatible errors
+ OTLP traces/logs/metrics
+ deterministic evidence bundles
+ read-only, redacted, citable agent projections
+ CLI/coding-agent action audit
+ accepted/rejected/reverted fix outcome loop
```

## What Changed Or Was Rechecked

| Source | Current evidence | Parallax read |
| --- | --- | --- |
| [Rustrak repository](https://github.com/AbianS/rustrak) | GitHub metadata checked 2026-05-25 shows 43 stars, 6 forks, 30 open issues/PRs combined by the repo API, default branch `main`, and latest repo push on 2026-05-25. The README describes an ultra-lightweight self-hosted tracker compatible with Sentry SDKs. | Treat Rustrak as active and relevant, not as a stale toy project. |
| [Rustrak releases](https://github.com/AbianS/rustrak/releases) | Generic latest release is `docs@0.1.16` on 2026-05-21, while the server package release is [`@rustrak/server@0.2.5`](https://github.com/AbianS/rustrak/releases/tag/%40rustrak/server%400.2.5) on 2026-05-21. [`@rustrak/mcp@0.1.2`](https://github.com/AbianS/rustrak/releases/tag/%40rustrak/mcp%400.1.2) was published on 2026-05-17. | Pin component releases, not only `releases/latest`, or the benchmark will accidentally pin docs. |
| [Rustrak installation docs](https://abians.github.io/rustrak/getting-started/installation) and [database docs](https://abians.github.io/rustrak/configuration/database) | SQLite is the default image, data persists through a Docker volume, PostgreSQL uses a separate `:postgres` image, and docs recommend SQLite for personal/low-medium traffic under about 1,000 events/hour. Production can run server-only and put the UI elsewhere. | Low-ops pressure is real, but the benchmark must separate SQLite personal use, Postgres production, server-only, and full UI modes. |
| [Docker Hub server image](https://hub.docker.com/r/abians7/rustrak-server) | Docker Hub API shows `abians7/rustrak-server` last updated 2026-05-21, about 1.6k pulls, `latest`/`v0.2.5` images, and linux amd64/arm64 image sizes around 16-17 MB. | The small-image claim is currently supported by registry metadata. Do not convert that into unmeasured memory or ingestion-throughput proof. |
| [`@rustrak/mcp` docs](https://abians.github.io/rustrak/sdks/mcp), [npm](https://www.npmjs.com/package/@rustrak/mcp), and [package README](https://github.com/AbianS/rustrak/tree/main/packages/mcp) | `@rustrak/mcp` is `0.1.2`, GPL-3.0, Node >=18, stdio, and exposes 18 tools across projects, issues, events, tokens, and alerts. Docs describe it as giving AI assistants control of Rustrak. | MCP presence is table stakes. The checked surface is management/raw-event shaped, not a Parallax-style read-only evidence-bundle surface. |
| [MCP issue tools](https://github.com/AbianS/rustrak/blob/main/packages/mcp/src/tools/issues.ts), [event tools](https://github.com/AbianS/rustrak/blob/main/packages/mcp/src/tools/events.ts), [token tools](https://github.com/AbianS/rustrak/blob/main/packages/mcp/src/tools/tokens.ts), and [alert tools](https://github.com/AbianS/rustrak/blob/main/packages/mcp/src/tools/alerts.ts) | Source includes issue state changes, `delete_issue` with `destructiveHint`, raw event detail access, token creation/revocation, and alert test sends. | Parallax's first MCP should stay read-only and bundle/projection based; broad CRUD would weaken the safety distinction. |
| [Ingest route source](https://github.com/AbianS/rustrak/blob/main/apps/server/src/routes/ingest.rs) and [envelope parser source](https://github.com/AbianS/rustrak/blob/main/apps/server/src/ingest/parser.rs) | Current `main` parses Sentry envelopes, accepts `/api/{project_id}/envelope/`, validates event UUIDs, stores only the first item with type `event`, and returns an error for deprecated `/store/`. Parser size limits are present. | "Sentry compatible" is strongest for modern envelope error events, not yet full Sentry protocol coverage. |
| [Rustrak Sentry drift report](https://github.com/AbianS/rustrak/blob/main/docs/sentry-compat/2026-05-11-drift-report.md) | Rustrak's own report says core event-envelope compliance is solid, but session, transaction, client_report, and attachment items are silently discarded; standalone span data is protocol-safe to ignore but not stored. | Rustrak has a credible protocol discipline, but it still lacks broad context capture that Parallax needs for evidence bundles. |
| [`feat: sentry agent` commit](https://github.com/AbianS/rustrak/commit/b29258447523f7cdb0d3fcf763a7313b33c17830) | Current `main` includes an unreleased repo-maintenance agent workflow: `.claude/skills/agent-rusty`, `_bmad/_memory/agent-rusty`, and a `sentry-protocol-drift` skill. This appears to be maintainer workflow/tooling, not a user-facing runtime feature. | Do not count it as product AI closure, but do count it as a warning that Rustrak is actively operationalizing protocol-drift research. |
| [License](https://github.com/AbianS/rustrak/blob/main/LICENSE) | Repository license is GPL-3.0; package metadata for `@rustrak/mcp` and `@rustrak/client` also reports GPL-3.0. | Stronger open-source posture than Bugsink/Urgentry, but GPL may be less compatible with Parallax's likely Apache-2.0 business posture. |

## Implications For Parallax

1. **Rustrak is the Rust/Sentry/MCP floor.** If Parallax's first artifact only
   offers Rust ingest, DSN migration, and MCP, it is not sufficiently distinct.
2. **Protocol fixture coverage matters.** Rustrak already keeps a Sentry protocol
   drift report and E2E SDK tests. Parallax needs fixture-gated Sentry envelope
   compatibility, not vague "Sentry-compatible" language.
3. **Agent access must be safer than management MCP.** Rustrak's MCP validates
   demand for agent-side issue lookup, but also shows why first Parallax MCP
   should avoid project/token/alert/issue mutation.
4. **Context breadth is still open.** Rustrak's checked sources do not show OTLP
   traces/logs/metrics correlation, session/release health, transaction/span
   storage, evidence bundles, source/redaction policy, CLI action audit, or
   fix-outcome records.
5. **Do not over-weight vendor performance claims.** Registry image size is
   source-checkable; memory, P99 latency, and events/second should stay marked
   as vendor claims until a benchmark artifact measures them.

## Falsification Triggers

Reopen the Parallax verdict if Rustrak:

- stores transaction/span/session/client_report/attachment data and correlates it
  into issue context;
- adds OTLP logs/traces/metrics ingestion or correlation;
- replaces raw-event MCP with a read-only, redacted, citable evidence-bundle
  schema;
- adds projection-equivalence hashes across CLI/API/MCP outputs;
- records coding-agent file/command/patch/test/deploy actions and outcomes;
- proves broad SDK/envelope compatibility through published fixtures and
  conformance reports;
- gains enough adoption or release maturity that it is no longer only an early
  product-shape warning.

## Bottom Line

Rustrak narrows Parallax's safe wording:

```text
Not: "open Rust Sentry-compatible error tracking with MCP"
Yes: "open runtime evidence/context engine that starts with Sentry-compatible
errors, adds OTLP context, and gives agents bounded evidence bundles plus
action/outcome audit"
```

Rustrak is now the live Rust-first comparison row for Phase 1. Parallax should
use it as a baseline and beat it on evidence semantics, not on framework choice.
