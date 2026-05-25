# A6 Synthetic Canary Fixture Corpus

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The A6 ledger and Phase 0 overlay contract require seeded canaries, but they do
not yet define the first fixture corpus. That leaves two risks:

- A1 tasks might pass without testing the exact surfaces the agent sees.
- Public fixtures might accidentally commit provider-shaped fake secrets that
  trigger hosted secret scanning or look too much like real credentials.

This note defines the minimum synthetic canary corpus for A1/A6:

> Commit fixture manifests, expected findings, redacted outputs, hashes, and
> generator recipes. Do not commit raw provider-shaped canary values unless they
> are reviewed for hosted-secret-scanning impact and explicitly marked safe.

## Source Basis

| Source | What it implies for fixtures |
| --- | --- |
| [GitHub secret scanning supported patterns](https://docs.github.com/en/code-security/secret-scanning/secret-scanning-patterns) | Provider and generic pattern sets change over time; use them as a canary coverage reference, but do not blindly commit token-shaped examples to a public repo. |
| [GitHub Actions masking](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#masking-a-value-in-a-log) | Masking must be registered before output and transformed secrets are not guaranteed to be masked, so CI fixtures need pre-mask, post-transform, and encoded variants. |
| [OpenTelemetry `AnyValue`](https://opentelemetry.io/docs/specs/otel/common/#anyvalue) | OTLP canaries must cover typed strings, bytes, arrays, and key/value lists before Markdown/string rendering. |
| [Sentry sensitive-data docs](https://docs.sentry.io/platforms/javascript/guides/nextjs/data-management/sensitive-data/) | Sentry-style fixtures should cover request headers, cookies, query strings, breadcrumbs, user context, stack locals, and HTTP spans. |
| [MCP tools specification `2025-11-25`](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) | Canaries must be absent from canonical `structuredContent`, not just text projections. |
| [RFC 8785 JCS](https://www.rfc-editor.org/rfc/rfc8785.html) | Expected finding rows and projection hashes should use deterministic canonical JSON. |
| [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html) | The corpus should cover access tokens, passwords, connection strings, encryption keys, payment-like values, sensitive PII, and untrusted cross-zone data. |
| [Betterleaks recheck](redaction-toolchain-betterleaks-recheck.md) | External scanner fixtures should record network/validation/LLM mode; comparator scans must run offline by default. |

## Corpus Tiers

| Tier | Purpose | Commit policy |
| --- | --- | --- |
| `phase0_minimum_canary` | Required before any A1 task can count. Covers source-field leakage, Sentry-style event, OTLP typed value, CLI/CI text, canonical JSON, Markdown, CLI, HTTP, and MCP projection. | Commit manifests, redacted expected outputs, and hashes. Raw canary values may be committed only if they cannot trigger hosted provider alerts. |
| `surface_expansion_canary` | Adds frontend, agent session, deploy/change provider payload, database evidence, attachments/raw refs, and replay/screenshot metadata. | Commit redacted outputs and manifests; keep raw text/image/replay/database examples private or generator-only until reviewed. |
| `adversarial_encoding_canary` | Tests split-line, base64-like, URL-encoded, JSON-escaped, shell-quoted, YAML/TOML/env, Markdown table/code block, and transformed-secret cases. | Prefer generator recipes and hashes. Commit raw examples only after external scanner and hosted secret-scanning review. |
| `provider_pattern_private_canary` | Exercises realistic provider-token patterns from GitHub/Gitleaks/Betterleaks/TruffleHog references. | Do not commit raw values by default. Store in private/local red-team fixtures and commit only hashes, expected rule IDs, and redacted projections. |

## Required Phase 0 Fixture Set

The first A1 bundle-value run should require this minimum set:

| Fixture ID | Surface | Canary class | Placement | Expected action |
| --- | --- | --- | --- | --- |
| `a1_source_gold_patch_sentinel` | A1 source row | Gold patch/test patch/generator metadata leakage | `patch`, `test_patch`, `hints_text`, `all_hints_text`, `log_parser`, `meta`, or `llm_metadata` equivalent | Forbidden from Arm A/B/C/D; hash-only in private/grader manifests. |
| `sentry_auth_header_001` | Sentry event | Authorization/cookie/query token | Request headers, cookies, URL query, breadcrumb URL | Strip raw value; preserve route/header name and redaction finding. |
| `sentry_stack_local_001` | Sentry event | Stack-local secret or PII | Stack frame locals or extra context | Strip or hash; preserve frame/function/file evidence. |
| `otlp_anyvalue_nested_001` | OTLP span/log | Secret in typed map/list/bytes | Attribute value tree before text rendering | Traverse typed value; strip/hash before Markdown rendering. |
| `ci_log_transformed_secret_001` | CI log | Transformed/encoded secret | stdout/stderr/log excerpt with URL/base64/shell escaped variants | Strip canary and encoded reversible forms; preserve failing test lines. |
| `cli_env_arg_secret_001` | CLI invocation | argv/env/config secret | command args, env, cwd, stdout/stderr | Preserve executable/subcommand/exit code; strip argv/env value. |
| `raw_ref_no_deref_001` | Raw refs | Secret only in raw referenced blob | `refs/stdout.txt` or artifact ref | Agent-visible projection contains ref metadata, never raw value. |
| `projection_markdown_001` | Projection | Markdown renderer leak | table, code block, link URL, inline JSON | Canonical JSON and Markdown projection both scan clean. |
| `mcp_structured_001` | MCP | Structured output leak | `structuredContent` and text content | `structuredContent` validates against schema and contains no canary. |
| `pii_joinable_id_001` | Cross-surface | Low-entropy identifier | email, IP, user/session ID, path fragment | Keyed HMAC or strip; plain SHA/MD5 counts as leak. |

Phase 0 can use generated fixture values, but each value must have a stable
fixture ID and a `raw_value_hash`. If a value is provider-shaped, keep it out of
git unless the operator explicitly approves committing it.

## Frontend Surface Expansion Fixtures

Frontend Replay and source maps are raw/reference surfaces, so the expansion
corpus should add these fixtures before any browser capture claim can become
agent-visible:

| Fixture ID | Surface | Canary class | Placement | Expected action |
| --- | --- | --- | --- | --- |
| `frontend_replay_dom_text_001` | Replay | DOM text canary | replay segment or masked replay metadata | Mask or block; canonical/projection outputs contain no raw DOM text. |
| `frontend_replay_input_001` | Replay | Form/input canary | input event, form field, or selector path | Mask or block before replay ref or summary output. |
| `frontend_replay_network_body_001` | Replay/network detail | Request/response body canary | opt-in network detail capture fixture | Disabled by default; allowlist fixtures must still redact projections. |
| `frontend_sourcemap_sources_content_001` | Source map | Raw source content | `sourcesContent` or uploaded source artifact | Server-side symbolication only; no source content in agent-visible evidence. |
| `frontend_sourcemap_public_negative_001` | Deployed build | Public source-map access | `.js.map` URL or equivalent asset path | Public fetch/access denied; source-map claim fails if accessible. |
| `frontend_url_referrer_console_001` | Browser metadata | URL/query/referrer/console canary | request URL, query string, referrer, console breadcrumb | Strip or redact before canonical JSON, Markdown, CLI, HTTP, and MCP outputs. |

## Public Fixture Layout

Use this future layout:

```text
docs/research/redaction-fixtures/
  manifest.md
  generator-spec.json
  canary-policy.json
  fixtures/<fixture_id>/
    fixture.json
    expected-findings.json
    source-field-policy.json
    redacted-output/
      canonical-bundle.json
      bundle.md
      cli.txt
      http-response.json
      mcp-structured-content.json
    scanner-comparison.jsonl
    projection-audit.jsonl
    private-input.sha256
```

`private-input.sha256` records the hash of the unredacted generated input. It is
not the input itself. Private raw inputs can live in a gitignored local fixture
directory or an operator-controlled artifact store.

## Fixture Manifest Row

Each `fixture.json` should include:

```json
{
  "schema_version": "a6-canary-fixture-v1",
  "fixture_id": "otlp_anyvalue_nested_001",
  "tier": "phase0_minimum_canary",
  "surface": "otlp_log",
  "canary_class": "typed_anyvalue_secret",
  "source_field_zone": "agent_visible_seed",
  "commit_raw_value": false,
  "raw_value_hash": "sha256:<private-input>",
  "raw_value_public_safe": false,
  "provider_pattern_like": false,
  "generated_at": "2026-05-25T00:00:00Z",
  "placement": {
    "artifact": "normalized.jsonl",
    "json_path": "$.data.attributes[\"app.config\"].map[\"token\"]"
  },
  "expected_action": "strip",
  "expected_public_placeholder": "[REDACTED:secret.token]",
  "must_preserve": [
    "$.data.attributes[\"service.name\"]",
    "$.data.attributes[\"exception.type\"]"
  ],
  "projection_targets": [
    "canonical_json",
    "markdown",
    "cli",
    "http",
    "mcp_structuredContent"
  ],
  "external_scanner_expectations": {
    "gitleaks": "pass_after_redaction",
    "betterleaks": "pass_after_redaction_offline_no_validation",
    "detect_secrets": "pass_after_redaction"
  }
}
```

## Commit Policy

Commit:

- canary manifests;
- generator specifications;
- redacted output fixtures;
- expected finding rows;
- source-field policy rows;
- projection-audit rows;
- scanner-comparison rows;
- hashes of private raw inputs.

Do not commit by default:

- raw provider-shaped tokens;
- real or live-looking credentials;
- private keys that hosted scanners may treat as real;
- raw customer/user PII;
- screenshots, replay blobs, crash dumps, database rows, or long logs with
  unredacted canaries;
- any fixture that Betterleaks/TruffleHog/GitHub would validate over the network.

If a public fixture must include a token-like string to test exact rendering, use
a Parallax-owned synthetic prefix such as `PARALLAX_CANARY_...` and document that
it is not intended to exercise provider-specific scanner rules. Provider-pattern
coverage belongs in private/local runs unless explicitly approved.

## A1 Task Requirement

Every counted A1 task directory should include:

```text
tasks/<task_id>/
  redaction-canary-manifest.json
  redaction-report.json
  scanner-comparison.jsonl
  projection-audit.jsonl
  private-canary-inputs.sha256
```

The task can count only when:

- at least one source-field isolation canary exists or the task's real source
  fields are audited as equivalent;
- at least one secret/PII canary enters the raw overlay before redaction;
- Arm B and Arm C are derived from the same post-redaction normalized rows;
- all agent-visible projections scan clean;
- minimum debug context is preserved after redaction.

## Pass/Fail Rules

- Any exact canary value, reversible encoding, raw provider-shaped token,
  forbidden source field, or plain low-entropy hash in an agent-visible artifact
  fails the fixture.
- A fixture without `must_preserve` checks can prove safety but not usefulness.
- A scanner finding after redaction fails the run unless the finding is a known
  false positive recorded in `scanner-comparison.jsonl`.
- A private provider-pattern canary cannot be used for a public claim unless the
  public artifact includes enough hashes, scanner versions, and expected finding
  rows to audit the run without exposing the raw value.
- A hosted secret-scanning alert caused by committed synthetic data is a process
  failure; move that class to private fixtures and rotate any affected canary.

## Relationship To Other Research

- [Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md)
  requires seeded canaries before an A1 task can count.
- [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) defines the
  result rows that record canary, scanner, projection, and usefulness outcomes.
- [Redaction detector toolchain](redaction-detector-toolchain.md) defines the
  runtime detector and external scanner roles.
- [Redaction toolchain Betterleaks recheck](redaction-toolchain-betterleaks-recheck.md)
  defines the Betterleaks no-network/no-validation default.
- [A1 Hugging Face row hash procedure](a1-huggingface-row-hash-procedure.md)
  defines source-row hash and field-policy inputs that canary source-field
  fixtures must respect.
- [Frontend Replay and source-map privacy recheck](frontend-replay-sourcemap-privacy-recheck.md)
  defines Replay/source-map canaries that must exist before browser evidence
  can become agent-visible.

## Sources

- [GitHub secret scanning supported patterns](https://docs.github.com/en/code-security/secret-scanning/secret-scanning-patterns)
- [GitHub Actions masking workflow command](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#masking-a-value-in-a-log)
- [OpenTelemetry common AnyValue](https://opentelemetry.io/docs/specs/otel/common/#anyvalue)
- [Sentry sensitive data docs](https://docs.sentry.io/platforms/javascript/guides/nextjs/data-management/sensitive-data/)
- [MCP tools specification `2025-11-25`](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)
- [RFC 8785 JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785.html)
- [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html)
- [Redaction toolchain Betterleaks recheck](redaction-toolchain-betterleaks-recheck.md)
- [Frontend Replay and source-map privacy recheck](frontend-replay-sourcemap-privacy-recheck.md)

## Bottom Line

The first canary corpus should prove the product boundary without creating new
public secret-scanning noise. Keep raw provider-shaped values private or
generator-only, commit redacted outputs and hashes, and require each A1 counted
task to show both safety and preserved debugging context.
