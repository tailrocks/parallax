# Agent how-to: point your coding agent at Parallax

Parallax's V1 agent surface is the CLI. No MCP server yet (gated decision —
see [agent-access-surface.md](../research/decisions/agent-access-surface.md));
any agent that can run shell commands already has everything it needs.

## The one command that matters

```sh
parallax issue context <fingerprint>
```

Prints an evidence bundle as Markdown: error identity (type, message, culprit
frame, occurrence counts), the trace waterfall (every service, span durations,
database query text where wrapper spans captured it), correlated logs, and
deterministic hypotheses with the evidence for each. Bounded to fit an agent
context window; `canonical hash` at the end identifies the exact evidence
state, so two agents (or one agent twice) can confirm they reasoned over the
same bundle.

### Structured output

When the agent wants the same evidence as machine-readable JSON (no Markdown
trailer), pass `--format json`:

```sh
parallax issue context <fingerprint> --format json
parallax run bundle <run_id> --format json
parallax run agent <run_id> --format json
```

Bundle JSON is the server's canonical bytes (schema `bundle-v1`); the hash is
already a field inside that object — do not re-pretty-print it. `run agent`
returns the run-scoped agent-session projection (steps, token totals) for
tool/shell reconstruction. Markdown remains the default for human eyes.

A working loop to give your agent:

```text
1. parallax issue list                      # what is broken, newest first
2. parallax issue context <fingerprint> --format json   # structured evidence
3. read the bundle; fix the code it points at
4. re-run the failing flow (parallax run start -- <cmd>)
5. parallax issue list                      # verify: no new occurrences
6. parallax issue resolve <fingerprint>
```

## Reconstructing what a human saw

When a human hands the agent a **trace id** (from an error page, a log line,
the UI) or a **run id** (from `parallax run start`):

```sh
parallax trace inspect <trace_id>          # the workflow, span by span
parallax logs --trace <trace_id>           # what the services said meanwhile
parallax logs --run <run_id> --grep error  # one run's noise, filtered
```

## Querying the API directly

Everything the CLI prints comes from `POST http://127.0.0.1:4000/graphql`.
Prefer the CLI `--format json` path above when the CLI already covers the
verb; hand-rolled GraphQL is for fields without a CLI surface yet.
`bundle(fingerprint:)` (or `bundle(runId:)` / `bundle(traceId:)`) returns the
same evidence as canonical JSON plus the Markdown projection, correlating the
trace, its logs, and the metric windows around the anchor in one artifact.
The SDL lives in the
[implementation spec §8](../research/architecture/v1-implementation-spec.md).

## Raw SQL — the power tool

When the shaped queries aren't enough, the agent gets the telemetry engine's
full read surface: GreptimeDB SQL over native observability tables plus
Parallax-derived extension tables. Raw signals stay native:
`opentelemetry_traces`, `opentelemetry_logs`, and GreptimeDB's native
per-metric tables. Derived product data can live in extension tables such as
`error_events`.

```sh
parallax sql "SELECT json_get_string(resource_attributes, 'service.name') AS service, \
              COUNT(*) FROM opentelemetry_logs \
              WHERE severity_number >= 17 GROUP BY service"
```

Same surface as the UI Logs page's SQL mode and the GraphQL `sql(query:)`
field. Read-only (SELECT/WITH/SHOW/DESCRIBE/EXPLAIN/TQL), one statement,
engine-dialect. Native GreptimeDB schemas are the contract: logs use JSON
resource/log attributes, traces use native trace columns, and metric table
names/columns are discovered through `metricNames` or `information_schema`.
Local loopback profile only; not a portable contract.

## Verifying a fix — live tail

After landing a change, watch for recurrence instead of polling:

```sh
parallax logs --follow --grep "checkout total overflowed" --for 60s
parallax traces --follow --errors --service checkout --for 60s
```

`--for <window>` tails for the window, prints the match count, and exits —
zero matches is the "fix holds" signal. The same streams back the UI Logs
page's live mode (`/v1/logs/stream`, `/v1/traces/stream` SSE).

## What the agent must know about the data

- **Redaction is pre-A6.** Bundles pass redaction-lite (key patterns, bearer
  tokens, obvious credentials). It is a seatbelt, not a guarantee — treat
  bundle contents as sensitive when forwarding beyond the local machine.
- **Hypotheses are deterministic**, derived from evidence shapes (dependency
  failure, slow span, database involvement). They rank starting points; they
  are not conclusions.
- **`insufficient_evidence` is an instruction.** When the bundle says so, the
  fix is instrumentation first: add the missing spans/logs (see
  [conventions](conventions.md)), reproduce, then reason again.
