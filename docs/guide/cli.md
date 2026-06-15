# CLI reference

One canonical GraphQL API serves the CLI, the UI, and agents alike; every
command below is a thin client of it. `--context <name>` (global) selects a
server from `~/.parallax/contexts.toml` — omitted, it targets the local one.

## Server

| Command | What it does |
| --- | --- |
| `parallax serve [--config <path>]` | Start ingest (OTLP gRPC `:4317`, HTTP `:4318`), the GraphQL API + UI (`:4000`), and the managed GreptimeDB child. Config default: `~/.parallax/config.toml`. |

## Runs

| Command | What it does |
| --- | --- |
| `parallax run start -- <cmd…>` | Wrapper mode: inject OTLP/gRPC endpoint/protocol env vars for traces, logs, metrics, and profiles plus `parallax.run.id`, run the command, propagate its exit code. |
| `parallax run start` | Bare mode: print the exports to source into your shell. |
| `parallax run finish <run_id> <exit_code>` | Close a bare-mode run. |
| `parallax run list` | Recent runs with status, exit code, relative start time. Runs whose `parallax.run.id` arrived in telemetry without a wrapper show status `external`. |
| `parallax run inspect <run_id>` | One run's record: status, exit code, trace/error counts, grouped issues. |
| `parallax run bundle <run_id>` | The run-anchored evidence bundle (Markdown + canonical hash). |
| `parallax run watch <run_id> [--level <severity>] [--grep <substr>] [--for 30s]` | Live tail of one run: new log records and finished spans interleaved (`[log]`/`[span]` prefixes) — the CLI mirror of the run page's Go live. `--for` watches a fixed window and reports per-stream match counts. |

## Issues

| Command | What it does |
| --- | --- |
| `parallax issue list [--status open\|resolved] [--run <run_id>]` | Grouped errors, newest activity first; `--run` scopes to issues whose events fell inside that run's traces. |
| `parallax issue context <fingerprint>` | **The agent handoff.** Server-rendered evidence bundle as Markdown + its canonical hash. |
| `parallax issue resolve <fingerprint>` | Mark resolved. |

## Telemetry

| Command | What it does |
| --- | --- |
| `parallax trace inspect <trace_id>` | Spans (service, kind, status, duration) + correlated logs. |
| `parallax traces [--run <id>] [--service <name>] [--min-duration 500ms] [--errors] [--grep <substr>] [--since 15m] [--limit 50]` | Browse traces newest-first with the same filters as the UI's Traces page: root-span service/name/duration plus `--errors` for traces containing an error span. `--run` anchors on one run's traces. |
| `parallax logs [--trace <id>] [--run <id>] [--service <name>] [--level <severity>] [--grep <substr>] [--since 15m] [--limit 100]` | Browse logs newest-first with the same filters as the UI's Logs page; `--trace`/`--run` scope to one trace or run. |
| `parallax logs --follow` / `parallax traces --follow` | kubectl-style live tail over SSE with the same per-row filters (no time window — it's a tail). Add `--for 30s` to watch a fixed window, then print the match count and exit: the agent verification loop ("after my fix, does it still appear?"). |
| `parallax sql "<SELECT …>"` | Raw read-only SQL straight to the GreptimeDB engine (logs, traces, metrics tables) — the same power surface as the UI's SQL mode. Single SELECT-shaped statements only. |

## Maintenance

| Command | What it does |
| --- | --- |
| `parallax doctor` | Diagnose the install: server reachable, engine health, spool and data sizes. |
| `parallax prune` | Reclaim spool space now (telemetry TTLs are engine-managed). |
| `parallax uninstall --purge --yes` | Delete `~/.parallax` (binary stays; remove it with your package manager). |

## Exit codes

`run start -- <cmd>` exits with the wrapped command's code. Everything else:
`0` success, non-zero with a message on stderr.
