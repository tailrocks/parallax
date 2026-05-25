# CLI Trace Overhead And Redaction

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note closes proof gate 9 from
[Strategic verdict and research coverage](strategic-verdict-and-research-coverage.md):

> CLI trace capture overhead and secret redaction for args, env, config, stdout,
> and stderr.

The answer is not "capture everything and redact later." CLI invocations are a
high-value Parallax surface because they are bounded, reproducible, and common
inside CI and coding-agent work. They are also one of the easiest places to leak
tokens, database URLs, config files, user paths, prompts, and generated output.

Decision: **CLI tracing is default-on only for structural capture.** Redacted
output excerpts require a separate proof gate. Full raw args, environment,
stdout, stderr, and config content are opt-in raw refs with scoped access,
audit, and retention limits.

## Current Primary-Source Checks

| Source | What matters for Parallax |
| --- | --- |
| [OpenTelemetry CLI semantic conventions](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) | OTel has a CLI span shape for short-lived command execution, including exit status semantics. Parallax should map to this vocabulary where possible instead of inventing an isolated command schema. |
| [OpenTelemetry process resource conventions](https://opentelemetry.io/docs/specs/semconv/resource/process/) | Process attributes include command, command line, and command args, but the spec warns that command args can contain sensitive information and should not be collected by default unless sanitized. This directly supports Parallax's structural-default policy. |
| [Rust `tracing`](https://docs.rs/tracing/latest/tracing/) | `tracing` gives Parallax the Rust-native span/event layer, but `#[instrument]` can record function arguments. CLI entry points and config loaders must skip raw structs and emit only sanitized fields. |
| [OpenTelemetry Rust SDK](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/) | Short-lived CLIs must handle exporter flush/shutdown explicitly. Any OTLP export path has to be measured because the last flush can dominate wall time for small commands. |
| [GitHub Actions log masking](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#masking-a-value-in-a-log) and [Actions secrets](https://docs.github.com/en/actions/concepts/security/secrets) | CI logs cannot be treated as safe. Masking depends on values being registered before output, and transformed values are not guaranteed to be masked. Parallax must scan CLI and CI output as hostile text. |
| [GitHub secret scanning patterns](https://docs.github.com/en/code-security/reference/secret-security/supported-secret-scanning-patterns) | GitHub maintains provider and generic secret-pattern categories. Parallax should use maintained pattern corpora for fixtures and canaries rather than relying on a small handmade regex list. |
| [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html) | Logs should exclude or protect passwords, access tokens, connection strings, encryption keys, sensitive PII, payment data, and data users opted out of collecting. CLI traces are logs with stronger reproducibility, so the same rule applies. |
| [Sentry sensitive-data docs](https://docs.sentry.io/platforms/javascript/guides/nextjs/data-management/sensitive-data/) | Sentry recommends source-side scrubbing so sensitive data does not leave the local environment. Parallax should apply the same principle to CLI wrappers before ingest. |

## Capture Modes

Parallax should expose four explicit CLI capture modes:

| Mode | Default? | Captured data |
| --- | --- | --- |
| `off` | No | No Parallax CLI trace. Useful for sensitive commands or debugging Parallax itself. |
| `structural` | Yes | Command identity, subcommand, exit code, duration, repo/branch/commit, policy IDs, output byte counts, redaction status, and safe field names. No raw arg/env/output values. |
| `redacted_excerpt` | Conditional | Everything in `structural`, plus bounded stdout/stderr excerpts after streaming redaction and final output scanning. This can become default only after the canary and overhead gates pass. |
| `raw_ref` | Never | Full args/env/config/stdout/stderr stored behind object refs, short TTL, audit, and scoped human access. Agent-visible bundles never dereference these by default. |

The product default should be:

```text
developer CLI wrapper: structural
Parallax-owned commands: structural, then redacted_excerpt after gate
CI command capture: structural unless project policy enables excerpts
coding-agent shell commands: structural; redacted_excerpt only after gate
raw output: opt-in raw_ref, never default
```

## Field Policy

| Surface | Default policy | Allowed by exception |
| --- | --- | --- |
| Command name | Store low-cardinality executable or logical command. | N/A |
| Subcommand | Store parsed low-cardinality subcommand path. | N/A |
| Args | Store arg keys, presence, value class, redaction rule, and optional HMAC hash for joinable safe values. Do not store `process.command_line` by default. | Allowlisted values such as enum modes, boolean flags, and bounded numeric values. |
| Environment | Deny values by default. Store allowlisted variable names and selected safe values such as `CI`, `GITHUB_SHA`, or `RUST_BACKTRACE` when policy allows. | HMAC hash values needed for joins. Never store `*_TOKEN`, `*_KEY`, `*_SECRET`, `PASSWORD`, cookies, auth headers, or connection strings. |
| Config | Store config file path policy, content hash, schema/version, and parse status. | Raw config only as `raw_ref` after opt-in. |
| CWD and repo path | Store repo root identity and relative path policy. Hash user-specific path fragments unless a project policy allows paths. | Human-visible path detail in self-hosted private projects. |
| stdout/stderr | Store byte count, line count, truncation status, exit-relevant excerpts only after scanner pass, and raw ref policy. | Bounded redacted excerpts in `redacted_excerpt`; full stream only in `raw_ref`. |
| Child processes | Apply the same command/arg/env/output policy recursively. | None. Child-process raw values are not safer than parent values. |
| Panic/error chain | Store Rust error type, panic location, sanitized message, stack/span trace refs, and redaction status. | Raw panic payload only if scanned and policy allows. |

Implementation rule: parse with structured APIs first. For Parallax-owned Rust
CLIs, use `clap` metadata to separate arg names from values. For shell-wrapped
commands, use a shell parser to identify argv tokens before detector passes.
String-splitting the full command line is not enough.

## Schema Additions

The existing `cli_invocation` node should add safety and performance fields
before CLI tracing is enabled by default:

```json
{
  "node_type": "cli_invocation",
  "capture_mode": "structural",
  "args_policy": "names-and-classes",
  "env_policy": "deny-values-allowlisted-names",
  "config_policy": "path-and-hash",
  "stdout_policy": "count-only",
  "stderr_policy": "count-only",
  "redaction_report_ref": "redact_123",
  "raw_ref_count": 0,
  "canary_scan_status": "pass",
  "overhead_budget": "structural-v1",
  "overhead_observed": {
    "wall_time_delta_ms_p95": 3,
    "rss_delta_mb_p95": 8,
    "flush_duration_ms_p95": 24
  }
}
```

This makes overhead and safety visible evidence, not a hidden implementation
claim.

## Redaction Canary Matrix

The CLI fixture suite must seed canaries across every capture surface:

| Surface | Canary examples |
| --- | --- |
| Args | `--token`, `--password`, `--api-key=...`, `-p ...`, bearer token flags, database URLs, URLs with credentials, JWT-like values, SSH key paths, repo paths with usernames. |
| Environment | `GITHUB_TOKEN`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `OPENAI_API_KEY`, `DATABASE_URL`, `COOKIE`, `SESSION`, `PASSWORD`, cloud-provider credentials, `.env` expansions. |
| Config | `.env`, TOML, YAML, JSON, npm/yarn/pnpm configs, Cargo credentials, cloud CLI configs, Docker auth files, kubeconfigs. |
| stdout | Echoed secrets, debug dumps, stack traces with env/config, test snapshot output, JSON logs, multiline private keys, base64-encoded secrets, URL-encoded secrets. |
| stderr | Compiler/test errors that include command lines, auth failures with URLs, panic messages, backtraces, child-process command echoes. |
| Child processes | Secret args/env/output from `git`, package managers, database CLIs, cloud CLIs, test runners, deploy scripts. |
| Agent-driven shell | User prompt fragments, MCP output, Parallax bundle excerpts, local file content printed by an agent command. |

The required acceptance criterion is zero known seeded canary leaks in
agent-visible JSON and Markdown renderings. A leak in either rendering fails the
gate. A detector error fails closed by stripping the affected field or blocking
the bundle.

## Overhead Benchmark Gate

Measure overhead against a no-Parallax baseline. Each benchmark run should
record wall time, CPU time, RSS, output throughput, flush/shutdown latency,
dropped spans/events, WAL bytes, redaction duration, false-positive redactions,
and canary leaks.

### Capture Matrix

| Variant | Purpose |
| --- | --- |
| Baseline command | Measures command without Parallax wrapper or SDK. |
| `structural`, no exporter | Measures parsing and local span creation only. |
| `structural`, local WAL/outbox | Measures default tiny-mode durability. |
| `structural`, OTLP batch export | Measures exporter and shutdown path. |
| `redacted_excerpt`, small output | Measures normal developer and CI command shape. |
| `redacted_excerpt`, high output | Measures ring-buffer scanning under stdout/stderr pressure. |
| `raw_ref`, high output | Measures opt-in raw artifact write overhead. |
| Panic/error path | Measures Rust panic hook, error chain, backtrace/spantrace capture. |
| Child-process fanout | Measures per-child span creation and policy recursion. |

### Workload Matrix

| Workload | Why it matters |
| --- | --- |
| Tiny 20 ms command | Flush and startup overhead dominate. This decides whether default capture feels intrusive. |
| 100 ms command | Common local helper command. Structural overhead should stay nearly invisible. |
| 1 s command | Typical build/test/setup phase. Percentage overhead matters more than absolute milliseconds. |
| High-output command | Test runners and package managers can emit large streams. Excerpt mode must not buffer unbounded text. |
| Many child processes | Build systems and scripts spawn many subprocesses. Policy recursion must stay bounded. |
| Rust panic | Parallax needs error-chain value without making failure paths significantly slower. |
| CI runner command | CI shells often echo commands and env. The canary and throughput gates both matter. |

### Initial Budgets

These budgets are hypotheses to validate, not product claims:

| Mode | Budget |
| --- | --- |
| `structural` | p95 overhead <= 5 ms for commands under 100 ms, or <= 5 percent for commands at or above 100 ms. |
| `structural` with local WAL | p95 overhead <= 8 ms for commands under 100 ms, or <= 7 percent for commands at or above 100 ms. |
| `redacted_excerpt` | p95 overhead <= 10 ms for normal commands, or <= 10 percent for commands at or above 100 ms. |
| High-output excerpts | Throughput penalty <= 10 percent while storing bounded excerpts only. |
| `raw_ref` | No default budget because it is opt-in, but the benchmark must report throughput, storage bytes, and flush time. |
| Shutdown/flush | Local/WAL flush p95 <= 100 ms. Explicit OTLP flush p95 <= 500 ms. |
| Memory | RSS delta p95 <= 16 MB in `structural`; <= 32 MB in `redacted_excerpt`. |
| Safety | Zero seeded canary leaks in agent-visible JSON/Markdown. |

If these budgets fail for common commands, CLI tracing stays opt-in until the
implementation changes. If only OTLP shutdown fails, keep local WAL as the
default and export asynchronously from the Parallax service.

## Failure Criteria

The gate fails if any of these happen:

1. A seeded canary appears in agent-visible JSON or Markdown.
2. `process.command_line` or equivalent full command text is captured by
   default.
3. Environment values are captured outside the explicit allowlist.
4. A redaction scanner error lets a field through instead of stripping or
   blocking it.
5. `redacted_excerpt` buffers unbounded stdout/stderr in memory.
6. Structural mode exceeds the overhead budget on common commands.
7. Exit-time export routinely causes user-visible lag or data loss.
8. Child-process capture bypasses parent redaction policy.

## Product Decision

CLI tracing is valuable enough to keep in the core thesis, but the product
wording must stay precise:

- "Parallax traces CLI invocations structurally by default."
- "Parallax can store redacted output excerpts after the project passes the
  redaction and overhead gates."
- "Parallax can retain raw command/output refs only when explicitly enabled."
- "Parallax does not claim full command capture is safe by default."

This protects the strongest wedge: reproducible local, CI, and agent-invoked
failures without making secret exposure part of the default value proposition.

## Relationship To Other Research

- [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md) defines
  the strategic reason and first-pass CLI trace model.
- [Agent session tracing ledger](agent-session-tracing-ledger.md) consumes
  command/edit coverage, redaction, and overhead rows for shell commands inside
  coding-agent sessions.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  defines the global default-deny redaction pipeline and red-team gate.
- [Evidence bundle and open schema specification](evidence-bundle-and-schema.md)
  must carry `redaction_report`, raw refs, and the CLI node fields used here.
- [Technical implementation concept](technical-implementation-concept.md)
  should treat this note as the CLI default-on safety gate.
- [Bundle-value Phase 0 runbook](bundle-value-phase0-runbook.md) should not use
  raw CLI output in agent arms unless this gate has passed for that fixture set.

## Bottom Line

Default CLI trace capture should be structural, low-overhead, and
default-deny. Redacted excerpts are earned by tests. Raw command/output content
is never the default. This keeps CLI tracing useful for Parallax's evidence
graph without turning the CLI into the easiest path for secret leakage.
