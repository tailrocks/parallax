# CLI Trace Safety Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

[CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md)
defines the proof gate for default-on CLI tracing. This ledger defines the
result artifacts, row schemas, claim levels, and expiry rules required before
Parallax can say CLI tracing is safe, low-overhead, or default-ready.

Current status: **not measured**. The repository has a capture policy and
benchmark design, but no dated run artifacts. Until those results exist,
Parallax should describe CLI tracing as designed but not run-proven.

The central rule:

> No "default-on CLI tracing", "redacted CLI excerpts are safe", or
> "low-overhead CLI tracing" claim without dated workload runs covering
> structural capture, redacted excerpts, raw refs, stdout/stderr, args, env,
> config, child processes, panic/error paths, CI shells, redaction canaries,
> flush/shutdown cost, and agent-visible projections.

This ledger is deliberately narrower than the
[agent session tracing ledger](agent-session-tracing-ledger.md): this one owns
CLI invocation safety. Agent-session runs consume these rows for shell commands.

## Current Source Snapshot

| Source | Current check | Parallax implication |
| --- | --- | --- |
| [OpenTelemetry semantic conventions 1.41.0](https://opentelemetry.io/docs/specs/semconv/) | CLI, process, CI/CD, exception, GenAI, and MCP areas are present in the current semantic-convention catalog. | Store `semconv_version` on every CLI-derived result and expire claims when relevant conventions change. |
| [OpenTelemetry CLI spans](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) | CLI spans are development-stage and model short-lived command execution. Executable name, exit code, and PID are required; `error.type` is required on non-zero exit. Command args are recommended by the CLI convention but should not be default-collected without sanitization. | Structural CLI rows can align with OTel, but raw argv cannot become a default product claim. |
| [OpenTelemetry process resources](https://opentelemetry.io/docs/specs/semconv/resource/process/) | `process.command_args`, `process.command_line`, working directory, parent PID, and related fields are opt-in. The docs prefer `process.command_args` over command-line strings and suggest `process.command` plus `process.args_count` when privacy blocks args. | `process.command_line` default capture is a ledger failure. Arg count and command family preserve audit value when values are denied. |
| [`tracing` `#[instrument]`](https://docs.rs/tracing/latest/tracing/attr.instrument.html) | By default, `#[instrument]` records all function arguments as span fields; arguments can be skipped explicitly. | Parallax-owned CLIs must skip raw config/args structs and emit sanitized fields intentionally. |
| [`opentelemetry_sdk` 0.32.0](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/) and [`BatchSpanProcessor`](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.BatchSpanProcessor.html) | The Rust SDK exposes batch processor queues, batch sizes, force flush, shutdown, and environment limits. Shutdown exports buffered spans and can block in some runtime shapes. | Short-lived CLI benchmarks must measure flush and shutdown separately from command work. |
| [GitHub Actions log masking](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#masking-a-value-in-a-log) | `add-mask` works per value per job and must be registered before a value is output. | CI shell output cannot be assumed safe because masking depends on timing and exact value registration. |
| [GitHub supported secret scanning patterns](https://docs.github.com/en/code-security/reference/secret-security/supported-secret-scanning-patterns) | GitHub documents generic, AI-detected, and provider-specific pattern categories, including private keys, bearer/basic auth, database connection strings, passwords, and hundreds of provider tokens. | CLI canary fixtures should use maintained pattern families and not only handmade regexes. |
| [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html) | Logs should exclude or protect access tokens, passwords, database connection strings, encryption keys, sensitive PII, payment data, and other sensitive values; logging systems should be tested for failure and resource exhaustion. | CLI traces are high-fidelity logs and need the same default-deny treatment plus overhead/failure tests. |
| [Sentry filtering docs](https://docs.sentry.io/platforms/javascript/configuration/filtering/) | Sentry recommends client-level filtering and `beforeSend` as the last chance to edit or drop data before it is sent. | Parallax CLI wrappers should scrub before ingest, not rely only on server-side cleanup after capture. |

## Claim Levels

Use these levels in `claim-ledger.jsonl`:

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No current CLI safety run exists. | "CLI tracing policy is designed but not run-proven." |
| `fixture_harness_ready` | Repeatable CLI, CI, child-process, panic, and canary workloads exist. | "CLI trace safety fixture harness prepared." |
| `structural_capture_safe` | Structural mode captures command identity, timing, exit status, policy refs, counts, and hashes without raw args/env/config/output. | "Structural CLI capture omits raw args, env, config, stdout, and stderr for the tested harness." |
| `structural_overhead_pass` | Structural mode meets wall-time, CPU, RSS, WAL, and flush budgets on the workload matrix. | "Structural CLI tracing is low-overhead for the tested workloads." |
| `redacted_excerpt_redaction_pass` | Redacted stdout/stderr excerpts leak zero seeded canaries in JSON and Markdown projections. | "Redacted CLI excerpts pass seeded redaction tests for the tested workload matrix." |
| `redacted_excerpt_overhead_pass` | Redacted excerpt mode meets normal and high-output overhead budgets. | "Redacted CLI excerpts meet overhead budgets for the tested workloads." |
| `child_process_policy_pass` | Child process spans inherit parent arg/env/output policy and report violations. | "Child-process CLI tracing inherits the tested parent policy." |
| `panic_error_capture_pass` | Rust panic/error paths capture useful structural evidence without leaking seeded canaries or exceeding failure-path overhead budgets. | "Rust CLI panic/error evidence is structurally captured for the tested workloads." |
| `ci_command_capture_pass` | CI shell workloads pass structural capture, masking-hostile canaries, and high-output checks. | "CI command tracing passes the tested structural and redaction fixtures." |
| `raw_ref_policy_pass` | Raw refs have TTL, scope, audit rows, dereference controls, and agent-visible denial checks. | "Raw CLI artifacts are retained only through scoped audited refs in the tested policy." |
| `projection_pass` | Agent-visible JSON and Markdown projections include redaction reports and never dereference raw refs by default. | "CLI trace projections pass redaction and raw-ref policy checks." |
| `cli_trace_default_ready` | Structural capture is safe, low-overhead, projected safely, and child/CI/panic surfaces are either passed or explicitly out of advertised scope. | "Parallax traces CLI invocations structurally by default for the tested workload matrix." |
| `redacted_excerpt_default_ready` | Redacted excerpt capture passes redaction, overhead, high-output, projection, child-process, and CI checks. | "Parallax can enable redacted CLI excerpts by default for the tested workload matrix." |
| `claim_expired` | Semconv, SDK, shell/platform, capture mode, redaction policy, projection, or workload coverage changed, or 90 days passed. | "CLI trace safety result expired; rerun required." |
| `claim_failed` | A required gate fails for the advertised level. | No claim for the failed mode/surface. |

Initial Parallax level: `not_measured`.

## Result Artifacts

Create these only for real CLI safety runs:

```text
docs/research/cli-trace-safety-results.md
docs/research/cli-trace-safety-runs/<run_id>/manifest.json
docs/research/cli-trace-safety-runs/<run_id>/workload-matrix.jsonl
docs/research/cli-trace-safety-runs/<run_id>/field-policy-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/redaction-canary-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/overhead-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/stdout-stderr-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/child-process-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/panic-error-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/projection-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/raw-ref-policy-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/claim-ledger.jsonl
docs/research/cli-trace-safety-runs/<run_id>/hashes.sha256
```

Do not commit raw args, raw env values, raw config, raw stdout/stderr, or real
secrets. Raw refs in research runs should point to synthetic fixtures unless
the operator explicitly approves a private retained artifact.

## Run Manifest

```json
{
  "run_id": "cli-trace-safety-YYYYMMDD-N",
  "research_date": "YYYY-MM-DD",
  "parallax_cli_commit": "<git-sha>",
  "schema_version": "cli-trace-safety-v0",
  "semconv_version": "1.41.0",
  "rust_tracing_version": "0.1.x",
  "opentelemetry_sdk_version": "0.32.0",
  "redaction_policy_version": "a6-default-deny-vN",
  "capture_modes": ["structural", "redacted_excerpt", "raw_ref"],
  "platforms": ["linux", "macos"],
  "shells": ["bash", "zsh", "pwsh"],
  "projection_formats": ["json", "markdown"],
  "workload_count": 0,
  "budgets": {
    "structural_p95_ms_under_100ms": 5,
    "structural_p95_percent_at_or_above_100ms": 5,
    "redacted_excerpt_p95_percent": 10,
    "high_output_throughput_penalty_percent": 10,
    "local_flush_p95_ms": 100
  },
  "notes": []
}
```

## Row Schemas

### Workload Matrix Row

```json
{
  "workload_id": "tiny-20ms-001",
  "command_class": "tiny_20ms|100ms|1s|high_output|many_children|rust_panic|ci_runner|agent_shell",
  "capture_mode": "off|structural|redacted_excerpt|raw_ref",
  "platform": "linux|macos|windows",
  "shell": "bash|zsh|pwsh|cmd|none",
  "baseline_command_hash": "sha256:<hex>",
  "expected_canary_count": 0,
  "expected_child_count": 0,
  "expected_output_bytes": 0,
  "notes": []
}
```

### Field Policy Result Row

```json
{
  "workload_id": "ci-runner-001",
  "capture_mode": "structural",
  "command_policy": "name-and-family",
  "arg_policy": "names-classes-and-count",
  "env_policy": "deny-values-allowlisted-names",
  "config_policy": "path-hash-and-schema",
  "cwd_policy": "repo-relative-or-hash",
  "stdout_policy": "count-only",
  "stderr_policy": "count-only",
  "child_policy": "inherit-parent-policy",
  "default_denied_field_count": 0,
  "policy_violation_count": 0,
  "pass": true
}
```

### Redaction Canary Result Row

```json
{
  "workload_id": "agent-shell-canary-001",
  "surface": "args|env|config|stdout|stderr|child_process|agent_shell",
  "capture_mode": "redacted_excerpt",
  "seeded_canaries": 20,
  "json_projection_leaks": 0,
  "markdown_projection_leaks": 0,
  "raw_ref_leaks": 0,
  "scanner_error_count": 0,
  "scanner_error_behavior": "fail_closed|strip_field|block_bundle",
  "pass": true
}
```

### Overhead Result Row

```json
{
  "workload_id": "tiny-20ms-001",
  "capture_mode": "structural",
  "baseline_p95_ms": 20,
  "observed_p95_ms": 23,
  "delta_p95_ms": 3,
  "delta_p95_percent": 15,
  "cpu_delta_p95_ms": 1,
  "rss_delta_p95_mb": 4,
  "flush_p95_ms": 18,
  "shutdown_p95_ms": 20,
  "dropped_event_count": 0,
  "wal_bytes_p95": 512,
  "budget_pass": true
}
```

For commands under 100 ms, absolute overhead decides the structural claim. For
commands at or above 100 ms, percentage overhead decides it.

### Stdout/Stderr Result Row

```json
{
  "workload_id": "high-output-001",
  "capture_mode": "redacted_excerpt",
  "stream": "stdout|stderr",
  "output_bytes": 10000000,
  "excerpt_bytes": 8192,
  "bounded_buffer": true,
  "truncated": true,
  "throughput_penalty_percent": 7,
  "seeded_canary_leaks": 0,
  "pass": true
}
```

### Child Process Result Row

```json
{
  "workload_id": "many-children-001",
  "parent_command_family": "cargo",
  "child_command_family": "rustc",
  "child_count": 24,
  "policy_inherited": true,
  "arg_violation_count": 0,
  "env_violation_count": 0,
  "output_violation_count": 0,
  "unobserved_child_count": 0,
  "pass": true
}
```

### Panic/Error Result Row

```json
{
  "workload_id": "rust-panic-001",
  "capture_mode": "structural",
  "panic_or_error": "panic|error_chain|nonzero_exit",
  "sanitized_message_captured": true,
  "backtrace_ref_policy": "none|structural|raw_ref",
  "spantrace_ref_policy": "none|structural|raw_ref",
  "symbolication_ref_policy": "none|structural|raw_ref",
  "seeded_canary_leaks": 0,
  "failure_path_overhead_p95_ms": 0,
  "pass": true
}
```

### Projection Result Row

```json
{
  "workload_id": "ci-runner-001",
  "projection_format": "json|markdown",
  "capture_mode": "structural",
  "redaction_report_present": true,
  "raw_ref_count": 0,
  "raw_ref_dereferenced": false,
  "json_projection_leaks": 0,
  "markdown_projection_leaks": 0,
  "agent_visible_pass": true
}
```

### Raw Ref Policy Result Row

```json
{
  "workload_id": "raw-ref-001",
  "raw_ref_kind": "args|env|config|stdout|stderr|panic_payload",
  "ttl_hours": 24,
  "scope": "human_debug_only|project_admin_only|break_glass",
  "audit_row_created": true,
  "agent_bundle_dereference_blocked": true,
  "access_denial_test_pass": true,
  "pass": true
}
```

### Claim Ledger Row

```json
{
  "claim_level": "structural_overhead_pass",
  "status": "pass|fail|expired",
  "run_id": "cli-trace-safety-YYYYMMDD-N",
  "mode": "structural",
  "scope": "linux-zsh-local-dev",
  "passed_workload_count": 0,
  "failed_workload_count": 0,
  "blocking_failures": [],
  "allowed_wording": "Structural CLI tracing is low-overhead for the tested workloads.",
  "expires_on": "YYYY-MM-DD"
}
```

## Counting Rules

- `structural` can become default only if it captures no raw args, env values,
  config content, stdout, stderr, or command-line strings.
- `structural` must still capture enough audit value: executable or command
  family, subcommand where safe, arg count/classes, repo/ref policy, start/end,
  exit status, duration, redaction policy, and projection refs.
- `process.command_line` default capture fails the gate.
- `process.command_args` default capture fails unless every value is sanitized
  before ingest and the sanitizer is part of the run.
- Env values are denied by default. Allowlisted env names can be visible; values
  require explicit policy and canary coverage.
- Child processes inherit parent policy. If a wrapper observes the parent but
  misses child args/env/output, the claim must say child coverage is missing.
- Scanner errors fail closed by stripping the field, blocking the bundle, or
  marking the result failed. They cannot pass the original field through.
- High-output workloads must prove bounded buffers. Unbounded stdout/stderr
  buffering fails even when redaction passes.
- `redacted_excerpt` cannot become default until canary redaction and
  high-output overhead pass for both JSON and Markdown projections.
- `raw_ref` is never a default capture mode and is never dereferenced into
  agent-visible bundles by default.
- Agent shell commands use this CLI policy plus the
  [agent session tracing ledger](agent-session-tracing-ledger.md) redaction,
  lossiness, and audit-value rows.

## Product Wording

Allowed after `not_measured`:

> CLI tracing policy is designed but not run-proven.

Allowed after `structural_overhead_pass`:

> Structural CLI tracing is low-overhead for the tested workloads.

Allowed after `cli_trace_default_ready`:

> Parallax traces CLI invocations structurally by default for the tested
> workload matrix.

Allowed after `redacted_excerpt_default_ready`:

> Parallax can enable redacted CLI excerpts by default for the tested workload
> matrix.

Avoid:

- "full command capture is safe";
- "records stdout/stderr by default";
- "safe for all CLIs";
- "raw command replay by default";
- "zero overhead";
- "CI logs are safe after GitHub masking";
- "redaction is AI-safe" without naming the run and projection formats.

## Refresh Triggers

Mark affected claims `claim_expired` when:

- OpenTelemetry CLI/process/CI semantic conventions change materially;
- Rust `tracing`, OpenTelemetry Rust SDK, exporter, or shutdown behavior
  changes;
- the Parallax CLI wrapper, shell parser, capture mode, field policy, or
  projection format changes;
- the redaction policy, detector runtime, or external scanner corpus changes;
- a supported shell, platform, CI provider, or agent shell wrapper changes;
- GitHub Actions masking, secret scanning patterns, or CI log behavior changes
  materially;
- a new workload class becomes part of product wording;
- 90 days pass since the last run during active development.

## Relationship To Other Research

- [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md)
  defines the capture modes, field policy, canary matrix, and initial budgets
  this ledger turns into result rows.
- [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md)
  defines why CLI invocations belong in the execution graph.
- [Agent and CLI OTel semantic-convention mapping](agent-cli-otel-semconv-mapping.md)
  defines how OTel CLI/process/CI spans feed stable Parallax rows.
- [Agent session tracing ledger](agent-session-tracing-ledger.md) consumes this
  ledger's shell-command safety results for coding-agent sessions.
- [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) remains the
  broader redaction veto; this ledger specializes it for CLI surfaces and
  overhead budgets.
- [Redaction detector toolchain](redaction-detector-toolchain.md) defines the
  runtime and offline scanners used by CLI canary rows.
- [Evidence bundle and open schema specification](evidence-bundle-and-schema.md)
  must expose CLI redaction reports, raw-ref policy, overhead evidence, and
  missing-evidence warnings without leaking denied fields.
- [Technical implementation concept](technical-implementation-concept.md)
  should treat this ledger as the claim boundary for default-on CLI tracing.

## Bottom Line

CLI tracing remains a strong Parallax wedge because commands are bounded,
reproducible, and directly connected to CI and coding-agent work. But the
default can only be structural until real rows prove safety and overhead. The
ledger makes that trade explicit: capture enough to reconstruct execution,
prove that secrets stay out of agent-visible projections, and require opt-in
audited refs for everything raw.
