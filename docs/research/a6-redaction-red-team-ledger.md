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
- source-field policies kept runner-private, grader-private, and default
  triage-private fields out of agent-visible projections;
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
- OpenTelemetry common values can contain typed scalars, bytes, arrays, and
  key/value lists; Parallax must inspect the typed value tree before string or
  Markdown rendering
  ([OpenTelemetry common `AnyValue`](https://opentelemetry.io/docs/specs/otel/common/#anyvalue)).
- MCP `2025-11-25` tool results can return JSON `structuredContent` alongside
  text content, and an advertised `outputSchema` makes server conformance and
  client validation part of the contract. A6 must therefore test the canonical
  structured output, not only the human-readable text projection
  ([MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)).
- RFC 8785/JCS constrains JSON to a deterministic, hashable representation.
  A6 projection rows should bind scanner results to the canonical bundle hash so
  JSON, Markdown, CLI, HTTP, and MCP outputs can be compared without trusting
  renderer-specific formatting
  ([RFC 8785](https://www.rfc-editor.org/rfc/rfc8785.html)).
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
- Gitleaks `v8.30.1` is the latest release checked on 2026-05-25, published
  2026-03-21, and can scan git history, directories/files, and stdin with
  configurable rules and baselines. Its README now says Gitleaks is
  feature-complete and future releases are security patches only, making it a
  stable fixture-output comparator but not a sufficient current-provider-pattern
  source
  ([Gitleaks v8.30.1](https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1)).
- detect-secrets `v1.5.0` is the latest release checked on 2026-05-25,
  published 2024-05-06, and supports baselines, plugin configuration,
  staged-file hooks, verification settings, and audit workflows, making it
  useful for "new secret" regression checks but stale for provider-token churn
  ([Yelp detect-secrets v1.5.0](https://github.com/Yelp/detect-secrets/releases/tag/v1.5.0)).
- TruffleHog `v3.95.3` is the latest release checked on 2026-05-25, published
  2026-05-11, and can return verified credential findings across repositories
  and other stores, but verification and credential analysis can create
  network/privacy side effects and should stay out of the default runtime path
  ([TruffleHog v3.95.3](https://github.com/trufflesecurity/trufflehog/releases/tag/v3.95.3)).
- Presidio `2.2.362` is the latest release checked on 2026-05-25, published
  2026-03-18 on GitHub with `presidio-analyzer` and `presidio-anonymizer`
  uploaded to PyPI on 2026-03-15. It explicitly warns that automated PII
  detection cannot guarantee that all sensitive information is found, so PII
  scanners are comparators and optional offline processors, not the only safety
  control
  ([Microsoft Presidio 2.2.362](https://github.com/microsoft/presidio/releases/tag/2.2.362)).

## Artifact Set

Use this public, redacted layout once A6 runs begin:

```text
docs/research/redaction-red-team-results.md
docs/research/redaction-red-team-runs/<run_id>/manifest.json
docs/research/redaction-red-team-runs/<run_id>/surface-fixture-ledger.jsonl
docs/research/redaction-red-team-runs/<run_id>/scanner-comparison.jsonl
docs/research/redaction-red-team-runs/<run_id>/projection-audit.jsonl
docs/research/redaction-red-team-runs/<run_id>/source-field-policy-audit.jsonl
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
  "source_field_policy_version": "phase0-source-field-policy-v1",
  "source_field_policy_refs": [
    "docs/research/bundle-value-eval/tasks/<task_id>/source-field-policy.json"
  ],
  "bundle_schema_version": "evidence-bundle-v0",
  "bundle_schema_ref": {
    "uri": "https://parallax.dev/schemas/evidence-bundle/v0.json",
    "hash": "sha256:...",
    "canonicalization": "jcs-rfc8785"
  },
  "canonical_bundle_hash_algorithm": "sha256 over RFC8785 canonical JSON after redaction",
  "projection_manifest_required": true,
  "mcp_output_schema_required": true,
  "surfaces": [
    "sentry_event",
    "otlp_log",
    "otlp_anyvalue",
    "ci_log",
    "cli_invocation",
    "agent_session",
    "frontend_metadata",
    "baggage",
    "deploy_provider_payload",
    "database_evidence"
  ],
  "projections": ["bundle_json", "bundle_markdown", "cli_output", "http_api", "mcp_tool_result"],
  "runtime_detector_version": "parallax-redact-rust-v0",
  "external_scanners": {
    "gitleaks": "8.30.1",
    "trufflehog": "3.95.3",
    "detect_secrets": "1.5.0",
    "presidio": "2.2.362",
    "github_pattern_snapshot": "2026-05-25"
  },
  "scanner_release_metadata": {
    "gitleaks": {
      "published_at": "2026-03-21T02:17:58Z",
      "development_posture": "feature_complete_security_patches_only",
      "replacement_watch": "betterleaks_unvetted"
    },
    "trufflehog": {
      "published_at": "2026-05-11T18:38:34Z",
      "license": "AGPL-3.0",
      "verification_policy": "disabled_unless_approved_private_fixture"
    },
    "detect_secrets": {
      "published_at": "2024-05-06T18:05:06Z",
      "role": "baseline_and_new_secret_regression_comparator"
    },
    "presidio": {
      "published_at": "2026-03-18T05:32:57Z",
      "pypi_uploads_checked": [
        "presidio-analyzer 2026-03-15T12:40:43.801880Z",
        "presidio-anonymizer 2026-03-15T12:40:38.651984Z"
      ]
    }
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
  "structured_path": "stdout.excerpt|otlp.body.map.key|provider_payload.deployment_status.log_url",
  "source_field_zone": "agent_visible_seed",
  "agent_visible_expected": true,
  "raw_ref_policy": "metadata_only|ref_only|deny_dereference",
  "projection_targets": ["bundle_json", "bundle_markdown", "cli_output", "http_api", "mcp_tool_result"],
  "schema_ref_hash": "sha256:...",
  "canonical_bundle_hash": "sha256:...",
  "projection_manifest_hashes": {
    "bundle_json": "sha256:...",
    "bundle_markdown": "sha256:...",
    "cli_output": "sha256:...",
    "http_api": "sha256:...",
    "mcp_structuredContent": "sha256:..."
  },
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
  "mcp_structured_content_hash": "sha256:...",
  "mcp_output_schema_valid": true,
  "safety_fields_only_in_meta": false,
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
  "scanner_release_published_at": "YYYY-MM-DDTHH:MM:SSZ",
  "scanner_execution_mode": "offline_no_network|network_verification_disabled|network_verification_approved_private_fixture",
  "scanner_development_posture": "active|feature_complete_security_patches_only|stale_baseline",
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
  "schema_ref_hash": "sha256:...",
  "canonical_bundle_hash": "sha256:...",
  "projection_manifest_hash": "sha256:...",
  "projection_derives_from_canonical": true,
  "source_field_policy_hash": "sha256:...",
  "source_field_policy_violations": 0,
  "mcp_structured_content_hash": null,
  "mcp_output_schema_valid": null,
  "safety_fields_only_in_meta": false,
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

## Source Field Policy Audit Row

A6 also records semantic field-policy isolation for eval/corpus sources. These
rows catch leaks that secret scanners cannot classify, such as gold patches,
hidden verifier IDs, generated hints, parser source, resolving commit URLs, or
LLM metadata.

```json
{
  "schema_version": "a6-source-field-policy-audit-v1",
  "run_id": "a6-2026-05-25-phase1-canary",
  "task_id": "swe-live-rust-example",
  "policy_ref": "tasks/swe-live-rust-example/source-field-policy.json",
  "policy_hash": "sha256:...",
  "projection": "bundle_json",
  "checked_zones": ["runner_private", "grader_private", "triage_private"],
  "agent_visible_denied_field_count": 0,
  "denied_fields_found": [],
  "allowed_private_derivations": [
    {
      "field": "test_cmds",
      "zone": "runner_private",
      "derivation": "sanitized_command_identity",
      "reason": "Harness command identity needed for failure reproduction."
    }
  ],
  "verdict": "pass"
}
```

`allowed_private_derivations` must describe derived, sanitized facts, not raw
field inclusion. A row with any raw `grader_private` content, any raw
`runner_private` harness internals beyond pre-approved sanitized identity, or
any default `triage_private` field in agent-visible output fails the run even if
all canary and external scanner checks pass.

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
- A run cannot pass unless the canonical bundle includes `schema_ref`,
  `canonical_hash`, `projection_manifest`, `redaction_report`,
  `source_field_policy`, `access`, and raw-ref policy fields.
- `canonical_bundle_hash` is computed only after source-field filtering,
  redaction, residual-risk labeling, and schema validation.
- Every projection must derive from the canonical bundle and match its
  `projection_manifest` hash. Hash mismatch, missing projection row, or an
  unscanned projection fails the run.
- MCP bundle output counts only when `structuredContent` validates against the
  evidence-bundle `outputSchema` and carries the same canonical hash as the CLI
  and HTTP result. Text-only MCP JSON or Markdown is a projection, not proof of
  schema-safe redaction.
- Safety fields that appear only in MCP `_meta`, tool annotations, descriptions,
  or model-prompt wrapper metadata do not count. They must be present in the
  canonical bundle JSON.
- A source-field policy violation in any agent-visible projection fails the run.
  Redaction cannot rescue a forbidden field that should never have entered the
  projection.
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
| Sentry event | auth headers, cookies, query tokens, user email, stack local value, breadcrumb URL, request/response headers, referrer URL. |
| OTLP span/log | custom token attribute, SQL text with password, URL query string, user/session ID, log body secret, typed `AnyValue` map/list/bytes secrets, resource/scope attribute secret. |
| CI log/artifact | env dump, transformed secret, private key block, database URL, test snapshot with PII. |
| CLI invocation | argv secret, env secret, cwd user path, stdout/stderr token, child-process command line. |
| Agent session | prompt secret, tool input secret, shell output secret, MCP response secret, generated Markdown leak. |
| Frontend | form-like value, DOM text, full request URL/query, referrer URL, request/response header, console log, baggage value, network body, replay/screenshot metadata, source-map/source-content ref. |
| Deploy/change provider payload | deployment status payload, deployment review comment, deploy log URL/body, release note, PR body, issue/comment text, environment URL, webhook delivery metadata. |
| Database evidence | row value PII, credential-like config row, SQL query text, query parameter, plan text, SQL error with connection string, export-like result. |
| A1/eval source fields | gold patch, test patch, fail/pass verifier IDs, generated hints, parser source, resolving commit URL, LLM metadata. |

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
| `structured_provider_projection_pass` | OTLP typed values, deploy/change provider payloads, deployment review comments, database query text/parameters, and raw-ref projections pass for the tested subset. | "Structured telemetry and provider evidence are redaction-tested for the configured projections." |
| `agent_visible_mixed_pass` | All claimed default surfaces pass zero-canary-leak, source-field-isolation, canonical-hash, projection-manifest, MCP structured-output, and usefulness gates. | "Agent-visible bundles are red-team tested for the configured surfaces." |
| `fail_closed_only` | Leaks are avoided only by stripping/ref-only behavior that loses required usefulness. | "Safe metadata-only mode; no agent-visible rich excerpts." |
| `claim_expired` | A previous pass is stale or invalidated by a rerun trigger. | "Previously tested; rerun required." |

Do not claim "AI-safe redaction" generically. The allowed language must name the
surfaces and projections actually covered by the current run.

## Freshness And Rerun Triggers

Rerun A6 when any of these change:

- redaction policy version;
- source-field policy version or source task field schema;
- evidence bundle schema, canonicalization method, canonical hash procedure, or
  projection renderer;
- runtime detector/parser version;
- external scanner major version or GitHub pattern snapshot;
- Sentry, OpenTelemetry, CLI, frontend, agent, or database capture surface;
- OTLP typed-value handling, provider webhook/API payload shape, deployment
  review/comment capture, browser URL/header/referrer/console capture, baggage
  policy, or database query-text/parameter policy;
- raw-ref policy or authorization model;
- new MCP/HTTP/CLI output path;
- MCP `outputSchema`, `structuredContent`, or `_meta` behavior changes;
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
- [Frontend capture safety ledger](frontend-capture-safety-ledger.md) defines
  browser/replay-specific privacy rows that A6 can veto before frontend evidence
  becomes agent-visible.
- [Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md)
  defines source-field zones that A6 must audit before A1 bundles can count.
- [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md)
  depends on A6 because bundle-value evals should not reward unsafe raw context.

## Bottom Line

A6 can pass only with reproducible red-team artifacts. The required claim is not
"we scrub secrets"; it is "for these canonical bundles, surfaces, and
projections, seeded canaries did not leak, forbidden source fields stayed out,
detector failures failed closed, raw refs stayed refs, MCP structured output
validated against the bundle schema, usefulness was preserved, and the claim
expires when the capture, source-field, schema, projection, or redaction surface
changes."
