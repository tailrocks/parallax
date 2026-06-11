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

A working loop to give your agent:

```text
1. parallax issue list                      # what is broken, newest first
2. parallax issue context <fingerprint>     # full evidence for one issue
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
Agents that prefer structured data over rendered Markdown can query it
directly — `bundle(fingerprint:)` returns the same evidence as canonical JSON
plus the Markdown projection. The SDL lives in the
[implementation spec §8](../research/architecture/v1-implementation-spec.md).

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
