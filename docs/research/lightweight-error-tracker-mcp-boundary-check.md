# Lightweight Error-Tracker MCP Boundary Check

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The [lightweight Sentry-compatible competitor watch](lightweight-sentry-compatible-competitor-watch.md)
already shows that "simpler than self-hosted Sentry" is crowded. This note
checks a narrower question:

> Have lightweight Sentry-compatible or OTLP-native challengers closed
> Parallax's agent-access wedge by shipping MCP or agent tools?

Short answer: **no, but the wedge is under more pressure**. Rustrak and GoSnag
now prove that MCP can appear inside very small error-tracking projects. Their
checked MCP surfaces are management and raw-event tools, not Parallax-style
read-only, redacted, canonical evidence-bundle projections.

## Boundary Verdict

| Project | Agent/MCP posture checked | Boundary read |
| --- | --- | --- |
| Bugsink | No first-party MCP or AI agent surface found in the checked official README, self-hosting page, or Sentry-SDK compatibility page. Current release: `2.2.1` on 2026-05-22 adds canonical API issue actions/comments and OpenAPI docs. Small third-party MCP adapters now exist (`bugsink-mcp` on npm / `draded/bugsink-mcp`, plus `j-shelfwood/bugsink-mcp`) with Bugsink project/team/issue/event/stacktrace/release query tools. License file uses PolyForm Shield for most repository content, with noted third-party exceptions. | Strong low-ops Sentry-compatible baseline, and ecosystem-level MCP pressure exists, but no first-party or mature Parallax-style read-only evidence-bundle surface is proven. |
| Rustrak | `@rustrak/mcp` `0.1.2` is live on npm. Its README says it gives AI assistants "full control" and exposes 18 tools across projects, issues, events, tokens, and alerts. Source/docs include `create_project`, `resolve_issue`, `unresolve_issue`, `mute_issue`, `delete_issue` with `destructiveHint`, `get_event` with full Sentry-envelope data, `create_token`, `revoke_token`, and `test_alert_channel`. A focused recheck also found an unreleased repo-maintenance Sentry protocol agent workflow in `main`, not a product-facing runtime feature. | MCP trigger is hit. The surface is management/write/raw-event shaped, not a bounded evidence-bundle contract. Parallax should not compete by adding more MCP CRUD; it should keep first MCP read-only and bundle/projection based. |
| Traceway | Focused recheck confirms stronger-than-README OTLP/context pressure: source registers `/api/otel/v1/{traces,metrics,logs}` plus native `/api/report`; converters map spans, exceptions, logs, metrics, and `gen_ai.*` AI traces; docs cover SQLite/all-in-one/minimal/embedded modes and integration skills. Current backend release: `backend/v1.7.27` on 2026-05-22. Checks found integration skills but no MCP server or Sentry-compatible ingest path; `sentry`/`dsn`/`envelope` content hits were comparison/design/test/framework references. | Strong OTLP/context/replay pressure, but not an MCP or Sentry-migration closure. Integration skills are assistant guidance for instrumentation, not a read-only data-access evidence surface. |
| GoSnag | Focused source recheck confirms a TypeScript stdio MCP server using `GOSNAG_URL`, `GOSNAG_TOKEN`, and Bearer-token `/api/v1` calls. Tools include `list_projects`, `get_project`, `create_project`, `update_project`, `delete_project`, `list_issues`, `get_issue`, `update_issue_status`, `get_issue_events`, `get_issue_counts`, `list_alerts`, `create_alert`, `list_issue_tags`, `add_issue_tag`, `list_users`, `create_ticket`, `get_ticket`, `update_ticket`, `list_tickets`, and `get_ticket_counts`. GitHub has no tagged release; latest checked push is 2026-04-17. | Capability warning. The AI/MCP feature vector is broad, but maturity is weak and the MCP surface is management/write/raw-event-list shaped, not read-only evidence context. |
| Urgentry | Checked README shows DSN migration, one-binary Tiny mode, split self-hosted mode over PostgreSQL/MinIO/Valkey/NATS, traces/replay/profiling/logs surfaces, and benchmark claims. Current release: `v0.2.12` on 2026-05-22. License is FSL-1.1-ALv2. No MCP surface found in the checked README. | Strong Sentry-compatible simplicity pressure, but not an open-source or agent-access closure in checked sources. |

## What This Changes

The old comparison "Parallax has MCP and lightweight competitors do not" is no
longer safe. The correct comparison is:

```text
management MCP / raw issue access
vs
read-only, redacted, citable evidence bundle with hashes, source policy,
missing-evidence fields, and outcome writeback outside the read path
```

Rustrak and GoSnag make MCP table stakes even at the lightweight end of the
market. They do **not** remove the Parallax wedge because the checked surfaces
do not publish:

- canonical evidence bundle schema;
- projection-equivalence hashes across CLI/API/MCP/Markdown/JSON;
- redaction manifest and source-field policy rows;
- missing-evidence model;
- read-only-by-default least-privilege bundle access;
- coding-agent action audit;
- accepted/rejected/reverted fix-outcome loop.

## Source Snapshot

| Source | Evidence checked | Freshness |
| --- | --- | --- |
| [Bugsink release](https://github.com/bugsink/bugsink/releases/tag/2.2.1), [self-hosting page](https://www.bugsink.com/built-to-self-host/), [Sentry-SDK compatibility page](https://www.bugsink.com/sentry-sdk-compatible/), [license](https://github.com/bugsink/bugsink/blob/main/LICENSE), [`bugsink-mcp` package](https://www.npmjs.com/package/bugsink-mcp), [`draded/bugsink-mcp`](https://github.com/draded/bugsink-mcp), and [`j-shelfwood/bugsink-mcp`](https://github.com/j-shelfwood/bugsink-mcp) | Single-container/SQLite/no-queue posture, Sentry SDK compatibility, no first-party MCP/AI agent surface in official docs, PolyForm Shield license posture, and small third-party Bugsink MCP adapters with issue/event/stacktrace query tools. | Bugsink release `2.2.1` published 2026-05-22; `bugsink-mcp` npm package checked at `1.0.0`; `draded/bugsink-mcp` has 0 stars/no releases; `j-shelfwood/bugsink-mcp` has 6 stars/no releases and last push 2026-01-12. |
| [Rustrak repository](https://github.com/AbianS/rustrak), [`@rustrak/mcp` npm package](https://www.npmjs.com/package/@rustrak/mcp), [MCP package README](https://github.com/AbianS/rustrak/tree/main/packages/mcp), [MCP issue tools source](https://github.com/AbianS/rustrak/blob/main/packages/mcp/src/tools/issues.ts), and [Rustrak recheck](rustrak-sentry-mcp-protocol-recheck.md) | Rust/Actix Sentry-compatible tracker, SQLite/Postgres deployment, `@rustrak/mcp` `0.1.2`, project/issue/event/token/alert tools, destructive issue and token operations, raw Sentry-envelope event access, and unreleased repo-maintenance Sentry protocol agent workflow. | Repo pushed 2026-05-25; generic latest release `docs@0.1.16` and server release `@rustrak/server@0.2.5` published 2026-05-21; npm `@rustrak/mcp` checked at `0.1.2`. |
| [Traceway repository](https://github.com/tracewayapp/traceway), [OTLP route source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/routes.go), [integration skills](https://github.com/tracewayapp/traceway/tree/main/skills), and [Traceway recheck](traceway-otlp-ai-replay-recheck.md) | MIT, direct OTLP/HTTP logs/traces/metrics, native `/api/report` for sessions/recordings, AI trace conversion from `gen_ai.*`, SQLite/all-in-one/minimal/embedded deployment modes, integration-instruction skills, and no checked MCP or Sentry-compatible ingest path. | Backend release `backend/v1.7.27` published 2026-05-22; repo pushed 2026-05-25; latest checked `main` commit `38b8d385`. |
| [GoSnag repository](https://github.com/darkspock/gosnag), [GoSnag MCP source](https://github.com/darkspock/gosnag/blob/main/mcp/src/index.ts), [GoSnag MCP package file](https://github.com/darkspock/gosnag/blob/main/mcp/package.json), and [GoSnag recheck](gosnag-sentry-ai-mcp-recheck.md) | Sentry `/store/` and `/envelope/` error-event ingest, AI RCA/triage source, Bearer-token MCP server, management tools for projects/issues/alerts/tags/tickets/users, no tagged GitHub release, and no Parallax-style evidence-bundle contract. | Repo pushed 2026-04-17; no latest release found in GitHub API; `mcp/package.json` version `1.0.0`; latest checked `main` commit `418b8b1`. |
| [Urgentry repository](https://github.com/urgentry/urgentry), [release](https://github.com/urgentry/urgentry/releases/tag/v0.2.12), and [license](https://github.com/urgentry/urgentry/blob/main/LICENSE) | DSN migration, Tiny one-binary SQLite mode, split PostgreSQL/MinIO/Valkey/NATS mode, benchmark claims, FSL source-available license, no checked MCP surface in README. | Release `v0.2.12` published 2026-05-22; repo pushed 2026-05-22. |

## Counting Rules

- Count lightweight MCP as a watch trigger, not a moat closure.
- Count write/destructive tools as safety pressure against Parallax's MCP
  design, not as evidence-bundle parity.
- Do not count raw Sentry event access as agent-ready context unless it is
  redacted, source-labeled, bounded, and projected through the same canonical
  bundle contract as CLI/API output.
- Keep license posture separate from deployment simplicity: Bugsink and
  Urgentry are relevant self-hosting baselines even though their checked licenses
  do not satisfy Parallax's open-source thesis.
- Treat no-release projects as capability warnings until release cadence,
  install path, and fixture behavior become reproducible.

## Parallax Impact

This pass strengthens the current product boundary:

- CLI and HTTP can remain day-one access surfaces.
- MCP should ship only after projection-equivalence and redaction fixtures pass.
- First MCP server should be read-only evidence context, not alert/dashboard/
  user/token/project/ticket CRUD and not issue resolution.
- Outcome records belong in a separate append-only write path after the core
  bundle contract is tested.

## Falsification Triggers

Reopen this note and the GO verdict if any lightweight challenger publishes:

- Sentry SDK migration plus OTLP traces/logs/metrics correlation;
- a versioned portable evidence-bundle schema with redaction/source policy;
- read-only MCP bundle tools with `structuredContent`/schema validation and
  projection-equivalence hashes;
- coding-agent command/file/approval/patch/test audit;
- accepted/rejected/reverted fix outcome rows;
- reproducible benchmark artifacts that beat Parallax's tiny-tier first-use
  target while covering a comparable evidence surface.

## Bottom Line

Lightweight competitors and their ecosystems have crossed the "has MCP" threshold. They have not
crossed the "safe evidence contract for agents" threshold. Parallax should use
that distinction aggressively: no broad management MCP in the first context
server, and no "agent-ready" wording without canonical bundle, redaction,
projection, and outcome-ledger proof.
