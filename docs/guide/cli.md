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
| `parallax run start -- <cmd…>` | Wrapper mode: inject `OTEL_EXPORTER_OTLP_ENDPOINT` + `parallax.run_id`, run the command, propagate its exit code. |
| `parallax run start` | Bare mode: print the exports to source into your shell. |
| `parallax run finish <run_id> <exit_code>` | Close a bare-mode run. |
| `parallax run list` | Recent runs with status, exit code, relative start time. Runs whose `parallax.run_id` arrived in telemetry without a wrapper show status `external`. |
| `parallax run inspect <run_id>` | One run's record: status, exit code, trace/error counts, grouped issues. |
| `parallax run bundle <run_id>` | The run-anchored evidence bundle (Markdown + canonical hash). |

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
| `parallax logs [--trace <id>] [--run <id>] [--service <name>] [--level <severity>] [--grep <substr>] [--since 15m] [--limit 100]` | Browse logs newest-first with the same filters as the UI's Logs page; `--trace`/`--run` scope to one trace or run. |
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
