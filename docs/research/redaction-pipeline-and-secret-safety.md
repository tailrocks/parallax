# Redaction Pipeline and Secret Safety

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note de-risks assumption A6 from
[Risks and the bear case](risks-and-bear-case.md):

> Redaction can be made trustworthy enough to expose evidence to agents and
> third-party models.

The answer is: **only with a default-deny pipeline, source-specific minimization,
and a red-team gate before any bundle reaches an agent**. Scrubbing rules are
necessary but not enough. Parallax handles too many leak-prone surfaces — Sentry
events, OTLP attributes, browser breadcrumbs, CI logs, CLI args/env,
stdout/stderr, agent prompts/tool outputs, attachments, and database query
results — to trust one regex pass.

The concrete detector/toolchain decision is now split into
[Redaction detector toolchain](redaction-detector-toolchain.md): Parallax should
own a Rust, source-aware, default-deny runtime redaction engine and use
Gitleaks, TruffleHog, detect-secrets, Presidio, and GitHub pattern references as
offline validators, not as blocking tiny-tier runtime dependencies.
The result-ledger contract for proving this gate is in
[A6 redaction red-team ledger](a6-redaction-red-team-ledger.md).

## Current Primary-Source Checks

| Source | What matters for Parallax |
| --- | --- |
| [Sentry SDK sensitive-data docs](https://docs.sentry.io/platforms/javascript/guides/nextjs/data-management/sensitive-data/) | Sentry recommends SDK-side scrubbing with `beforeSend` so sensitive data never leaves the local environment, plus server-side scrubbing as a storage safeguard. It explicitly calls out stack locals, breadcrumbs, user context, HTTP query strings, transaction names, and HTTP spans as sensitive-data paths. |
| [Sentry data scrubbing overview](https://docs.sentry.io/security-legal-pii/scrubbing/) | Sentry treats server-side scrubbing, advanced scrubbing, attachment scrubbing, and replay privacy as separate controls. Parallax should likewise split controls by surface instead of claiming one global scrubber. |
| [OpenTelemetry Collector processors](https://opentelemetry.io/docs/collector/components/processor/) | The Collector has transform/enrichment processors and a contrib/K8s Redaction Processor. As of the current docs, redaction is beta for traces and alpha for metrics/logs, so Parallax should support Collector-side redaction but not outsource bundle safety to it. |
| [OpenTelemetry redaction processor](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/processor/redactionprocessor/README.md) | The processor is designed to fail closed with `allowed_keys`, can mask blocked values, can hash with HMAC, and can emit redaction audit attributes. This maps well to Parallax ingest policy and bundle `redaction_report`, but it covers only telemetry attributes that pass through that processor. |
| [GitHub Actions log masking](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#masking-a-value-in-a-log) and [Actions secrets](https://docs.github.com/en/actions/concepts/security/secrets) | GitHub can mask values in logs, but `add-mask` must happen before output and GitHub states transformed secret redaction is not guaranteed. Parallax must treat CI logs/artifacts as hostile, even when they came from GitHub. |
| [GitHub secret scanning patterns](https://docs.github.com/en/code-security/reference/secret-security/supported-secret-scanning-patterns) | GitHub documents generic, AI-detected, and provider-specific pattern categories, with hundreds of provider patterns. Parallax should reuse a maintained pattern corpus for tests and canaries rather than inventing all patterns manually. |
| [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html) | OWASP says access tokens, passwords, connection strings, encryption keys, payment data, sensitive PII, illegal-to-collect data, and opt-out data should usually be removed, masked, sanitized, hashed, or encrypted; it also requires verification and access controls for logs. |
| [Langfuse masking docs](https://langfuse.com/docs/observability/features/masking) | LLM observability systems already expose SDK masking hooks over inputs, outputs, and metadata. Parallax should apply the same idea to agent-session capture but make masking mandatory-by-default, not an optional integration convenience. |

## Threat Model By Surface

| Surface | Leak modes | Default Parallax posture |
| --- | --- | --- |
| Sentry envelope events | `request`, headers, cookies, stack locals, breadcrumbs, query strings, raw URLs, tags, user context, attachments. | SDK-side minimization guidance, ingest allowlist, default strip of auth/cookie headers, route parameterization, metadata-only attachments until attachment scrubbing exists. |
| OTLP spans/logs/metrics | High-cardinality attributes, SQL text, HTTP URLs, user IDs, tokens in custom attributes, log body text. | Collector redaction supported, but Parallax ingest repeats policy; unknown attributes are dropped or hashed unless project policy allows them. |
| Browser/frontend capture | Form values, full DOM text, URLs, network bodies, console logs, replay media. | Error + breadcrumbs first; replay opt-in only; mask all text/block media by default; network bodies disabled unless selectors/endpoints are explicitly marked safe. |
| CI logs/artifacts | Echoed env vars, transformed secrets, debug shells, test snapshots, coverage artifacts, uploaded files. | Treat all text/artifacts as untrusted; scan before storage and before bundle; store bounded excerpts plus raw refs behind scoped access. |
| CLI invocations | Secrets in argv/env/config paths, cwd/repo path leaks, stdout/stderr dumps, child process args. | Store command/subcommand and safe structural metadata; hash or redact argv/env by default; bounded redacted stdout/stderr excerpts only. |
| Agent sessions | User prompts, model inputs/outputs, tool args, shell output, file diffs, MCP responses. | Capture hashes, refs, policy decisions, and bounded redacted excerpts; full prompts/tool output are opt-in raw refs with short-lived access. |
| Attachments and raw evidence | Crash dumps, screenshots, replay blobs, log files, test artifacts, database exports. | Metadata-only v0; later storage requires type-specific scanner, size cap, and manual/project-level enablement. |
| Database query output | Row values, customer data, credentials in config tables, export-like queries. | Read-only templates, row/column policy, aggregate/summarize first, never raw table dumps in agent bundles; see the [production database evidence access gate](production-database-evidence-access.md). |

## Pipeline Decision

Parallax redaction should be a **multi-stage safety pipeline**, not a single
function.

```text
capture source
  -> source-side minimization when available
  -> ingest normalization and default-deny field policy
  -> detector pass over accepted fields and raw refs
  -> storage with policy_version and raw_access_policy
  -> bundle builder redaction pass
  -> redaction_report and residual_risk
  -> agent/API/MCP output scanner
```

### Stage 1: Source-Side Minimization

Prefer not collecting sensitive data:

- SDK integrations should use `beforeSend`/equivalent hooks where available.
- Browser capture should disable replay and network bodies by default.
- CLI wrappers should separate command identity from argv/env values.
- Agent tracing should collect event shape, hashes, and refs before collecting
  full prompt/tool content.

This stage cannot be trusted alone because user code, framework integrations,
third-party SDKs, shells, and test tools can still emit secrets.

### Stage 2: Ingest Default-Deny Policy

The ingest gateway should normalize each source into typed fields and apply
field policy before high-volume storage:

- `allow`: known-safe structural fields such as service, trace/span IDs, release,
  status code, duration, exit code, test name, file path when project policy
  allows paths.
- `hash`: joinable but sensitive values such as user ID, session ID, IP, email,
  path fragments, CLI arg value, environment variable value.
- `strip`: passwords, tokens, cookies, auth headers, private keys, connection
  strings, payment data, and full request/response bodies.
- `ref_only`: raw logs, artifacts, prompt bodies, stdout/stderr, replay data,
  screenshots, crash dumps, and database output.

Unknown fields default to `strip` or `hash` depending on the source and whether
the field is needed for joins. Operators can loosen policy, but loosened policy
must appear in `redaction_report` and audit logs.

### Stage 3: Detector Pass

Run maintained detectors on accepted values and raw refs:

- key-name detectors: `authorization`, `cookie`, `set-cookie`, `password`,
  `secret`, `token`, `api_key`, `private_key`, `connection_string`;
- value detectors: provider tokens, private keys, bearer/basic auth headers,
  database URLs, credit cards, JWT-like strings, SSH keys, emails, phone numbers;
- structured parsers before regex where possible: URL parser, HTTP header parser,
  JSON/YAML/TOML/env parser, shell arg parser, Sentry envelope item parser, OTLP
  attribute traversal;
- HMAC hashing for low-entropy identifiers that must remain joinable.

Detector failure is a safety event. The bundle builder should fail closed when a
required detector errors on a source field intended for agent exposure.

### Stage 4: Bundle-Build Redaction

Bundle construction performs a second pass because:

- ingest policy may change between event storage and bundle generation;
- bundles join surfaces, and cross-surface combinations can re-identify users;
- agents see a more concentrated evidence artifact than any one raw event.

The bundle builder should emit deterministic excerpts only after:

1. size bounding;
2. source-specific parsing;
3. secret/PII detection;
4. replacement/hashing;
5. a final output scanner over the JSON and Markdown renderings.

### Stage 5: Output Guard

Every API, CLI, MCP, and third-party-model output must pass through the same
final scanner. This catches accidental leaks from:

- Markdown projection bugs;
- model-generated summaries that copied raw material;
- tool responses embedded inside agent-session evidence;
- newly added fields not covered by older bundle schema tests.

## Required Redaction Report

The existing [evidence bundle spec](evidence-bundle-and-schema.md) already
requires `redaction_report`. A6 needs the report to be more than a decorative
object:

```json
{
  "policy_version": "redact-v1",
  "policy_mode": "default-deny",
  "input_surfaces": ["sentry_event", "otlp_span", "ci_log", "cli_invocation"],
  "rules_applied": ["auth-header-strip", "provider-secret-detector", "pii-email", "hmac-low-entropy-id"],
  "removed": [
    { "node": "evt_1", "field": "request.headers.authorization", "rule": "auth-header-strip", "count": 1 }
  ],
  "hashed": [
    { "node": "usr_1", "field": "user.id", "rule": "hmac-low-entropy-id", "count": 1 }
  ],
  "ref_only": [
    { "node": "ci_log_1", "field": "raw_log", "reason": "unbounded_text" }
  ],
  "detector_versions": {
    "secret_patterns": "2026-05-25",
    "policy": "redact-v1"
  },
  "validation": {
    "canary_scan": "pass",
    "known_secret_matches": 0,
    "manual_review_required": false
  },
  "raw_access_policy": "scoped-read",
  "residual_risk": "medium"
}
```

The report must be machine-readable so evals can reject unsafe bundles
automatically.

## Red-Team Gate

Before any agent/third-party-model exposure, Parallax needs a reproducible
redaction eval suite. The run artifacts, row schemas, claim levels, and
freshness rules are defined in the
[A6 redaction red-team ledger](a6-redaction-red-team-ledger.md).

| Gate | Required result |
| --- | --- |
| Seeded secret corpus | Canary tokens, provider token examples, private keys, DB URLs, JWTs, cookies, auth headers, emails, phone numbers, IPs, user names, path-sensitive data, and payment-like numbers are seeded across every supported surface. |
| Dual rendering scan | Both JSON bundle and Markdown projection scan clean. |
| Raw-ref isolation | Raw refs are not dereferenced in default agent/API/MCP output. |
| Detector failure mode | If a detector errors, unsafe fields are stripped and the bundle reports `manual_review_required` or fails closed. |
| Regression fixture | The seeded corpus runs in CI for every policy/schema change. |
| Real-data pilot | Operator-owned real logs/CI/CLI/frontend sessions run through the same suite before external users. |
| False-positive review | Redaction must not erase the minimum evidence needed for A1 bundle-value evals. |

Initial acceptance criterion: **zero known seeded canary leaks** in agent-visible
JSON and Markdown. This does not prove no leaks exist, but anything weaker is
not compatible with the data-ownership value prop.

## Kill And Narrowing Criteria

A6 fails if:

1. seeded canary leaks recur after two policy iterations;
2. real-data pilots find secrets that cannot be detected without stripping most
   useful logs/CLI output;
3. frontend or replay capture requires broad raw user-content collection to be
   useful;
4. agent-session tracing requires storing full prompts/tool outputs by default;
5. teams will not enable the needed capture surfaces under the proposed policy.

If A6 weakens but does not fail, narrow the MVP:

- ship Sentry/OTLP backend errors first;
- keep frontend replay, attachments, DB query output, and full agent prompts out
  of default bundles;
- expose raw refs only to humans with scoped just-in-time access;
- let the A1 bundle-value eval use redacted excerpts, not raw dumps.

## Product Implication

Redaction is not a compliance checkbox; it is part of Parallax's product moat.
The bundle is useful only if users trust that it can be pasted into Codex,
Claude Code, Amp, Cursor, GitHub issues, or a third-party model without leaking
production secrets. Therefore:

- redaction policy belongs in the core ingest/bundle path, not a later enterprise
  add-on;
- `redaction_report` is a required schema field and a test artifact;
- unsafe surfaces stay metadata-only until the red-team gate passes;
- every future connector must define its redaction policy before it can feed
  agent-visible bundles.

## Relationship To Other Research

- [Risks and the bear case](risks-and-bear-case.md) — this is the concrete A6
  mitigation and falsification plan.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) — this note
  expands the mandatory `redaction_report`.
- [Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md)
  — frontend privacy remains the hardest PII surface and should stay opt-in.
- [Frontend capture safety ledger](frontend-capture-safety-ledger.md) — the
  claim ledger for proving browser breadcrumbs, route metadata, source-map
  status, replay refs, and agent-visible projections do not leak seeded PII.
- [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md) — CLI
  args/env/stdout/stderr and agent prompts/tool outputs require ref-first
  capture.
- [Agent session tracing across real tools](agent-session-tracing-real-tools.md)
  — extends the prompt/tool/transcript raw-ref policy across Codex, Claude Code,
  Amp, and OpenCode adapters.
- [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md) —
  expands CLI-specific args/env/config/stdout/stderr policy, canary fixtures,
  and overhead gates before default-on capture.
- [Redaction detector toolchain](redaction-detector-toolchain.md) — chooses the
  runtime detector architecture and external scanner role for A6.
- [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) — defines the
  run artifacts, fixture rows, projection audits, claim levels, and freshness
  rules that make A6 pass/fail claims auditable.
- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
  — redaction is a precondition for safe read-only agent context.
- [Production database evidence access gate](production-database-evidence-access.md)
  — turns database query output into a read-only, template-driven, redacted, and
  audited evidence source.
- [Production database evidence ledger](production-database-evidence-ledger.md)
  — defines when direct database evidence can be claimed after seeded secrets,
  prompt injection, RLS scope, limits, and audit fixtures pass.

## Bottom Line

Parallax can keep A6 alive only by treating redaction as a tested evidence
pipeline. The defensible MVP is not "we scrub secrets"; it is "agent-visible
bundles are default-deny, red-team-tested, source-specific, and self-report the
policy and residual risk that made them safe enough to expose."
