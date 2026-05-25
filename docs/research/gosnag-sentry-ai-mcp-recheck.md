# GoSnag Sentry AI MCP Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

Re-check GoSnag because it is the broadest lightweight feature-vector warning in
the Sentry-compatible competitor set:

```text
Sentry SDK ingest + issue workflow + AI RCA/triage + MCP
```

This pass tests whether GoSnag has closed the Parallax wedge or whether it only
proves that "AI over self-hosted Sentry-compatible errors" is becoming a
commodity product shape.

## Short Verdict

GoSnag is a real capability warning, not a mature wedge closer.

The source-level check supports the core shape: the repository contains Sentry
`/store/` and `/envelope/` ingest paths, stores raw Sentry event JSON, has AI
workers and manual AI endpoints for RCA/merge/deploy/ticket/priority/tag/alert
workflows, and ships a TypeScript MCP server over the management API. But the
checked project has no releases/tags, low visible traction, requires PostgreSQL
in the normal Docker Compose path, ignores Sentry transactions/sessions/client
reports, and exposes MCP as issue/project/ticket management rather than a
bounded read-only evidence-bundle projection.

The durable Parallax distinction remains:

```text
Sentry-compatible errors
+ OTLP logs/traces/metrics
+ deterministic redacted evidence bundles
+ read-only CLI/API/MCP projections with the same bundle hash
+ coding-agent and CLI action audit
+ accepted/rejected/reverted fix outcome loop
```

## What Changed Or Was Rechecked

| Source | Current evidence | Parallax read |
| --- | --- | --- |
| [GoSnag repository](https://github.com/darkspock/gosnag) | GitHub page and API checks on 2026-05-25 show `darkspock/gosnag`, MIT license, roughly 8 stars and 4 forks, default branch `main`, 136 commits, no published releases, and no tags. The latest checked `main` commit is [`418b8b1`](https://github.com/darkspock/gosnag/commit/418b8b107e274bfaab3f905510ddd274173d216b), dated 2026-04-17. | Treat GoSnag as a moving-target capability warning, not a stable benchmark release. Pin a commit if it is used in comparisons. |
| [README](https://github.com/darkspock/gosnag) and [router source](https://github.com/darkspock/gosnag/blob/main/cmd/gosnag/router.go) | README claims Sentry SDK compatibility, legacy `/store/` and modern `/envelope/`, single Go binary with embedded React UI/migrations, issue workflow, GitHub/Jira, tickets, AI, and MCP. Router source confirms `POST /api/{project_id}/store/` and `POST /api/{project_id}/envelope/`, plus a broad `/api/v1` management API. | The Sentry-compatible issue-tracker posture is real enough to count. The source does not make it an OTLP evidence engine. |
| [Ingest handler](https://github.com/darkspock/gosnag/blob/main/internal/ingest/handler.go), [event parser](https://github.com/darkspock/gosnag/blob/main/internal/ingest/event.go), [envelope parser](https://github.com/darkspock/gosnag/blob/main/internal/ingest/envelope.go), and [auth helper](https://github.com/darkspock/gosnag/blob/main/internal/ingest/auth.go) | `/store/` parses one JSON event; `/envelope/` loops items and stores `event` items. `transaction` is explicitly out of scope; `session`, `sessions`, and `client_report` are silently ignored. Event parsing covers exception, stack frames, tags, extra, user, request, contexts, breadcrumbs, SDK, modules, release, environment, and stores raw JSON. Auth extracts a Sentry public key from `X-Sentry-Auth` or `sentry_key`. Payload reads are gzip/deflate-aware and capped at 1 MiB. | "Sentry-compatible" should be narrowed to error-event ingest. It is not evidence of transaction/span/session coverage or full Sentry protocol parity. |
| [AI provider source](https://github.com/darkspock/gosnag/blob/main/internal/ai/provider.go), [AI service](https://github.com/darkspock/gosnag/blob/main/internal/ai/service.go), [RCA source](https://github.com/darkspock/gosnag/blob/main/internal/ai/rca.go), and [deploy analyzer](https://github.com/darkspock/gosnag/blob/main/internal/ai/deploy.go) | Implemented provider switch covers OpenAI-compatible OpenAI, OpenAI-compatible Groq, and AWS Bedrock. The README/config text lists Claude and Ollama too, but no direct Anthropic or Ollama provider implementation was found in the checked provider switch. AI calls have daily token budgets, calls-per-minute limits, prompt-hash caching, and usage logging. RCA prompts combine issue metadata, latest stack trace, breadcrumbs, tags, similar issues, and recent deploys. Deploy analysis waits 15 minutes and compares pre/post windows. | GoSnag has real AI-assisted triage mechanics, but its "evidence" is model-generated strings over selected DB context, not a canonical, citable evidence bundle with raw references, redaction manifest, missing-evidence fields, and projection hashes. |
| [Merge source](https://github.com/darkspock/gosnag/blob/main/internal/ai/merge.go), [priority evaluator](https://github.com/darkspock/gosnag/blob/main/internal/priority/evaluator.go), and [ticket description source](https://github.com/darkspock/gosnag/blob/main/internal/ai/description.go) | AI can suggest or auto-merge duplicate issues, evaluate custom priority rules once per issue/rule, and generate sanitized HTML ticket descriptions from issue context. Auto-merge can mutate issue/event records when enabled. | This is issue-workflow automation, not the Parallax fixer boundary. It reinforces the need to keep Parallax core read-only and move outcome writes into a separate append-only path. |
| [MCP source](https://github.com/darkspock/gosnag/blob/main/mcp/src/index.ts) and [MCP package](https://github.com/darkspock/gosnag/blob/main/mcp/package.json) | `gosnag-mcp` is version `1.0.0`, TypeScript, stdio, and depends on `@modelcontextprotocol/sdk`. It calls `/api/v1` with `Authorization: Bearer ${GOSNAG_TOKEN}`. Tools include `list_projects`, `get_project`, `create_project`, `update_project`, `delete_project`, `list_issues`, `get_issue`, `update_issue_status`, `get_issue_events`, `get_issue_counts`, `list_alerts`, `create_alert`, `list_issue_tags`, `add_issue_tag`, `list_users`, `create_ticket`, `get_ticket`, `update_ticket`, `list_tickets`, and `get_ticket_counts`. | MCP is table stakes. GoSnag's checked MCP is a management/write surface over issues, alerts, projects, tags, tickets, and users, not a read-only evidence bundle with least-privilege schema and redaction proof. |
| [Docker Compose](https://github.com/darkspock/gosnag/blob/main/docker-compose.yml), [Dockerfile](https://github.com/darkspock/gosnag/blob/main/Dockerfile), [go.mod](https://github.com/darkspock/gosnag/blob/main/go.mod), and [.env example](https://github.com/darkspock/gosnag/blob/main/.env.example) | Compose declares `gosnag` and `db` services, Postgres 16, resource limits, and required `DATABASE_URL`. Dockerfile builds frontend with Node 20 and backend with Go 1.25 into an Alpine runtime. `.env.example` includes core/auth/SMTP/Slack variables but omits the AI variables listed in README/config source. | Low process count is real, but the default persistent path is Postgres-backed. The config/docs gap around AI setup is another maturity warning. |

## Implications For Parallax

1. **"AI over errors" is not enough.** GoSnag already has AI RCA, deploy
   anomaly analysis, priority/tag/alert suggestions, ticket descriptions, and
   merge suggestions in a lightweight Sentry-compatible tracker.
2. **Sentry compatibility must be fixture-scoped.** GoSnag currently stores
   error events and raw event JSON, while transactions, sessions, and
   client reports are ignored. Parallax should name exactly which Sentry
   envelope items it accepts, stores, correlates, and exposes in bundles.
3. **MCP power must stay bounded.** GoSnag proves MCP can arrive early in a
   small error tracker, but its tool surface mutates projects, statuses, alerts,
   tags, and tickets. Parallax's first context MCP should avoid this shape.
4. **Evidence semantics remain the wedge.** GoSnag RCA is a generated answer
   from issue context. Parallax must produce a portable failure dossier with
   source labels, redaction results, raw references, missing-data warnings, and
   stable hashes before asking an agent to reason over it.
5. **Maturity weighting matters.** No releases/tags and low traction mean
   GoSnag should not be weighted like Bugsink or Traceway in market adoption
   evidence, but it still shows where small competitors can move.

## Falsification Triggers

Reopen the Parallax verdict if GoSnag:

- publishes stable releases and fixture-backed Sentry protocol compatibility;
- stores transactions, spans, sessions, client reports, and attachments instead
  of ignoring them;
- adds OTLP traces/logs/metrics correlation or an OTel-native ingest path;
- replaces management MCP with read-only, redacted, citable evidence bundles;
- publishes projection-equivalence hashes across API/MCP/CLI/bundle outputs;
- records coding-agent commands/files/patches/tests/approvals and PR outcomes;
- makes AI RCA cite raw evidence fields and missing-evidence records by schema.

## Bottom Line

GoSnag narrows Parallax wording:

```text
Not: "self-hosted Sentry-compatible errors with AI and MCP"
Yes: "runtime evidence/context engine that uses Sentry-compatible errors as one
input, correlates OTLP and execution traces, and gives agents bounded citable
context plus an action/outcome audit trail"
```

Until GoSnag has releases and broader protocol/context coverage, it remains a
feature-vector warning. Its main value for Parallax is negative space: do not
build a GoSnag-style issue tracker and call it an evidence engine.
