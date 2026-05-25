# A6 Redaction Red-Team Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The [redaction pipeline](redaction-pipeline-and-secret-safety.md) and
[detector toolchain](redaction-detector-toolchain.md) define the A6 strategy:
default-deny capture, source-specific minimization, detector passes, and no
agent exposure until redaction has been red-team tested.

This note defines the result ledger for that red-team gate. A6 is not proven by
having a scanner, a policy document, or a `redaction_report` field. It is proven
only by a committed run artifact showing:

- seeded canaries did not leak in agent-visible JSON, Markdown, CLI, HTTP, or
  MCP output;
- detector failures failed closed;
- raw refs were not dereferenced by default;
- external scanners did not find unredacted secrets in generated outputs;
- redaction did not erase the minimum evidence needed for bundle usefulness.

## Current Primary-Source Checks

Current sources support a layered, auditable gate rather than a single magic
scrubber:

- Sentry recommends SDK-side scrubbing with hooks such as `beforeSend` so
  sensitive data does not leave the local environment; server-side scrubbing is
  a separate storage safeguard
  ([Sentry sensitive data](https://docs.sentry.io/platforms/javascript/guides/nextjs/data-management/sensitive-data/)).
- Sentry Session Replay uses privacy controls such as masking text and blocking
  media by default in the reference setup, which reinforces that replay is a
  distinct high-risk surface
  ([Sentry Session Replay](https://docs.sentry.dev/platforms/javascript/session-replay/)).
- OpenTelemetry Collector's redaction processor keeps only `allowed_keys` unless
  `allow_all_keys` is set, applies `blocked_values` to allowed keys, supports
  HMAC hash functions for low-entropy data, and can emit audit attributes
  ([OpenTelemetry redaction processor](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/processor/redactionprocessor/README.md)).
- GitHub Actions masking requires registering each value before it appears in
  logs; CI logs must still be treated as hostile text
  ([GitHub Actions masking](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#masking-a-value-in-a-log)).
- GitHub's supported secret scanning patterns separate generic, AI-detected, and
  provider patterns and currently list hundreds of provider patterns with
  varying push-protection, validity-check, metadata, and base64 support
  ([GitHub secret scanning patterns](https://docs.github.com/en/code-security/reference/secret-security/supported-secret-scanning-patterns)).
- OWASP's logging guidance says access tokens, passwords, database connection
  strings, encryption keys, sensitive PII, payment data, and similar values
  should be excluded or protected; it also says data from other trust zones must
  be treated as untrusted
  ([OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html)).
- Gitleaks can scan git history, directories/files, and stdin with configurable
  rules and baselines, making it a good fixture-output comparator
  ([Gitleaks](https://github.com/gitleaks/gitleaks)).
- detect-secrets supports baselines, plugin configuration, staged-file hooks,
  and audit workflows, making it useful for "new secret" regression checks
  ([Yelp detect-secrets](https://github.com/Yelp/detect-secrets)).
- TruffleHog can return verified credential findings across repositories and
  other stores, but verification can create network/privacy side effects and
  should stay out of the default runtime path
  ([TruffleHog](https://github.com/trufflesecurity/trufflehog)).
- Presidio explicitly warns that automated PII detection cannot guarantee that
  all sensitive information is found, so PII scanners are comparators and
  optional offline processors, not the only safety control
  ([Microsoft Presidio](https://github.com/microsoft/presidio)).

## Artifact Set

Use this public, redacted layout once A6 runs begin:

```text
docs/research/redaction-red-team-results.md
docs/research/redaction-red-team-runs/<run_id>/manifest.json
docs/research/redaction-red-team-runs/<run_id>/surface-fixture-ledger.jsonl
docs/research/redaction-red-team-runs/<run_id>/scanner-comparison.jsonl
docs/research/redaction-red-team-runs/<run_id>/projection-audit.jsonl
docs/research/redaction-red-team-runs/<run_id>/usefulness-audit.jsonl
docs/research/redaction-red-team-runs/<run_id>/repair-ledger.jsonl
docs/research/redaction-red-team-runs/<run_id>/hashes.sha256
```

Do not commit live secrets, private customer data, full prompts, raw logs,
screenshots, replay payloads, database rows, or unredacted real incident
material. Synthetic canaries should be fake but structurally realistic. If a
real pilot discovers an actual secret, rotate it and record only redacted
metadata.

## Run Manifest

Each run gets exactly one manifest:

```json
{
  "schema_version": "a6-redaction-red-team-v1",
  "run_id": "a6-2026-05-25-phase1-canary",
  "research_date": "2026-05-25",
  "dataset_kind": "synthetic_canary",
  "policy_version": "redact-v1",
  "bundle_schema_version": "evidence-bundle-v0",
  "surfaces": ["sentry_event", "otlp_log", "ci_log", "cli_invocation", "agent_session"],
  "projections": ["bundle_json", "bundle_markdown", "cli_output", "http_api", "mcp_tool_result"],
  "runtime_detector_version": "parallax-redact-rust-v0",
  "external_scanners": {
    "gitleaks": "x.y.z",
    "trufflehog": "x.y.z",
    "detect_secrets": "x.y.z",
    "presidio": "x.y.z",
    "github_pattern_snapshot": "2026-05-25"
  },
  "hmac_key_policy": "ephemeral-test-key",
  "raw_ref_policy": "metadata_only",
  "reviewer": "redacted"
}
```

`dataset_kind` must be one of:

- `synthetic_canary` - generated fixtures with fake but structurally realistic
  secrets and PII.
- `synthetic_adversarial` - encoded, fragmented, multiline, or projection-bug
  fixtures.
- `operator_real_pilot` - operator-owned real telemetry reviewed privately.
- `design_partner_real_pilot` - design-partner telemetry reviewed under consent
  and redaction agreements.

Only synthetic rows can be committed with fixture details. Real-pilot rows should
publish aggregate counters, surface labels, and redacted incident references.

## Surface Fixture Row

Every seeded fixture gets one row in `surface-fixture-ledger.jsonl`:

```json
{
  "schema_version": "a6-surface-fixture-v1",
  "run_id": "a6-2026-05-25-phase1-canary",
  "fixture_id": "cli_stdout_database_url_001",
  "surface": "cli_stdout",
  "source_shape": "bounded_stderr_excerpt",
  "canary_classes": ["postgres_connection_string", "password", "repo_path_user_fragment"],
  "encoding_variants": ["plain", "shell_quoted", "json_escaped"],
  "expected_actions": [
    {
      "class": "postgres_connection_string",
      "action": "strip",
      "rule_id": "secret.database_url.v1"
    },
    {
      "class": "repo_path_user_fragment",
      "action": "hmac",
      "rule_id": "path.user_fragment.v1"
    }
  ],
  "runtime_result": "pass",
  "bundle_json_leak_count": 0,
  "bundle_markdown_leak_count": 0,
  "cli_output_leak_count": 0,
  "http_api_leak_count": 0,
  "mcp_tool_result_leak_count": 0,
  "redaction_report_complete": true,
  "raw_ref_dereferenced": false,
  "detector_failure_mode": "not_applicable"
}
```

`leak_count` counts exact canary values, reversible encodings, and unsafe
low-entropy hashes. Keyed HMAC values are allowed when the policy marks the
field as joinable and the raw value is absent.

## Scanner Comparison Row

External scanner checks go in `scanner-comparison.jsonl`:

```json
{
  "schema_version": "a6-scanner-comparison-v1",
  "run_id": "a6-2026-05-25-phase1-canary",
  "artifact": "bundle_markdown",
  "scanner": "gitleaks",
  "scanner_version": "x.y.z",
  "findings_total": 0,
  "expected_canaries_missed_by_runtime": 0,
  "runtime_findings_missed_by_scanner": 2,
  "false_positives": 1,
  "verdict": "pass"
}
```

External scanners are comparators. If they find a canary in generated output that
the runtime missed, the run fails. If the runtime catches something they miss,
keep the runtime rule.

## Projection Audit Row

Projection rows prove that the safe internal bundle did not leak when rendered:

```json
{
  "schema_version": "a6-projection-audit-v1",
  "run_id": "a6-2026-05-25-phase1-canary",
  "bundle_id": "fixture_bundle_001",
  "projection": "bundle_markdown",
  "output_hash": "sha256:...",
  "final_scanner_status": "pass",
  "canary_leaks": 0,
  "raw_refs_expanded": 0,
  "redaction_report_present": true,
  "manual_review_required": false,
  "verdict": "pass"
}
```

Every public output path needs coverage: JSON, Markdown, CLI, HTTP API, MCP tool
result, and any model prompt wrapper.

## Usefulness Audit Row

Redaction safety can create a second failure mode: the bundle becomes too empty
to help. Track that separately in `usefulness-audit.jsonl`:

```json
{
  "schema_version": "a6-usefulness-audit-v1",
  "run_id": "a6-2026-05-25-phase1-canary",
  "fixture_id": "ci_failure_env_dump_001",
  "anchor_class": "ci_failure",
  "evidence_preserved": ["test_name", "exit_code", "stack_frame", "safe_file_path", "redacted_error_excerpt"],
  "evidence_removed": ["env_value", "database_url", "bearer_token"],
  "minimum_debug_context_preserved": true,
  "a1_eval_usable": true,
  "reviewer": "redacted",
  "verdict": "pass"
}
```

This prevents a false pass where Parallax strips all useful context and calls the
output safe.

## Repair Row

When a red-team run finds a leak or usefulness failure, record the repair:

```json
{
  "schema_version": "a6-repair-v1",
  "run_id": "a6-2026-05-25-phase1-canary",
  "repair_id": "repair_000003",
  "failure_class": "markdown_projection_leak",
  "surface": "agent_session_tool_output",
  "problem": "Markdown code block renderer bypassed final output scanner.",
  "fix": "Route rendered Markdown through final scanner before API/CLI return.",
  "before": {
    "bundle_markdown_leak_count": 1
  },
  "after": {
    "bundle_markdown_leak_count": 0
  },
  "status": "verified"
}
```

## Counting Rules

- A seeded canary leak in any agent-visible projection fails the run.
- A detector timeout, crash, or parser error passes only if the affected field is
  stripped, converted to `ref_only`, or the bundle is blocked.
- A raw ref is safe only when the default projection contains the ref metadata,
  not the raw content.
- Plain SHA/MD5 hashes of low-entropy values such as emails, IPs, user IDs, and
  path fragments count as leaks. Use keyed HMAC or strip.
- External scanner verification must not run against live customer values in the
  default path. Verified-secret checks belong in approved private red-team
  fixtures.
- A redaction report is complete only if it records policy version, rule IDs,
  action counts, detector versions, residual risk, and manual-review status.
- A pass requires both safety and usefulness: zero canary leaks and
  `minimum_debug_context_preserved = true` for required fixture classes.

## Required Fixture Classes

Cover each supported surface with at least these canary classes:

| Surface | Required canaries |
| --- | --- |
| Sentry event | auth headers, cookies, query tokens, user email, stack local value, breadcrumb URL. |
| OTLP span/log | custom token attribute, SQL text with password, URL query string, user/session ID, log body secret. |
| CI log/artifact | env dump, transformed secret, private key block, database URL, test snapshot with PII. |
| CLI invocation | argv secret, env secret, cwd user path, stdout/stderr token, child-process command line. |
| Agent session | prompt secret, tool input secret, shell output secret, MCP response secret, generated Markdown leak. |
| Frontend | form-like value, DOM text, console log, network body, replay/screenshot metadata. |
| Database evidence | row value PII, credential-like config row, SQL error with connection string, export-like result. |

Each class should appear in multiple encodings: plain text, JSON escaped,
URL-encoded, shell quoted, multiline, base64-like where realistic, Markdown
table, Markdown code block, YAML/TOML/env formats, and split-line log output.

## Claim Levels

Use these claim levels in `redaction-red-team-results.md`:

| Level | Meaning | Product wording allowed |
| --- | --- | --- |
| `not_measured` | No current A6 run exists. | "Redaction design is planned." |
| `synthetic_canary_pass` | Synthetic fixtures pass for backend/Sentry/OTLP outputs only. | "Backend bundles are tested against seeded canaries." |
| `cli_ci_bundle_pass` | CLI and CI surfaces pass safety and usefulness fixtures. | "CLI/CI excerpts are agent-visible only after redaction tests pass." |
| `frontend_metadata_only_pass` | Frontend metadata/error fixtures pass, but replay/raw DOM remains out of scope. | "Frontend metadata is redaction-tested; replay remains opt-in and gated." |
| `agent_session_metadata_pass` | Agent/session structural metadata passes, but full prompts/tool outputs remain raw refs. | "Agent traces expose metadata and redacted excerpts, not full transcripts by default." |
| `agent_visible_mixed_pass` | All claimed default surfaces pass zero-canary-leak and usefulness gates. | "Agent-visible bundles are red-team tested for the configured surfaces." |
| `fail_closed_only` | Leaks are avoided only by stripping/ref-only behavior that loses required usefulness. | "Safe metadata-only mode; no agent-visible rich excerpts." |
| `claim_expired` | A previous pass is stale or invalidated by a rerun trigger. | "Previously tested; rerun required." |

Do not claim "AI-safe redaction" generically. The allowed language must name the
surfaces and projections actually covered by the current run.

## Freshness And Rerun Triggers

Rerun A6 when any of these change:

- redaction policy version;
- evidence bundle schema or projection renderer;
- runtime detector/parser version;
- external scanner major version or GitHub pattern snapshot;
- Sentry, OpenTelemetry, CLI, frontend, agent, or database capture surface;
- raw-ref policy or authorization model;
- new MCP/HTTP/CLI output path;
- new model-prompt wrapper that embeds bundle content;
- a real pilot finds an unclassified secret or PII class;
- 90 days pass after a public agent-visible safety claim.

Until rerun, downgrade the affected claim to `claim_expired` or to the highest
unaffected narrower level.

## Relationship To Other Research

- [Risks and bear case](risks-and-bear-case.md) identifies A6 as a load-bearing
  assumption for data ownership and agent exposure.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  defines the default-deny policy and red-team gate this ledger records.
- [Redaction detector toolchain](redaction-detector-toolchain.md) defines the
  runtime and offline scanner architecture this ledger tests.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) requires
  `redaction_report` on every agent-visible bundle.
- [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md) and
  [Agent session tracing across real tools](agent-session-tracing-real-tools.md)
  provide high-risk fixture surfaces.
- [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md)
  depends on A6 because bundle-value evals should not reward unsafe raw context.

## Bottom Line

A6 can pass only with reproducible red-team artifacts. The required claim is not
"we scrub secrets"; it is "for these surfaces and projections, seeded canaries
did not leak, detector failures failed closed, raw refs stayed refs, usefulness
was preserved, and the claim expires when the capture or redaction surface
changes."
