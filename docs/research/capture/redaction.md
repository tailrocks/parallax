# A6 — Redaction Pipeline and Secret Safety

> A6 (one of the A1–A7 assumptions — "redaction can be made trustworthy enough to expose evidence to agents and third-party models") is kept alive only with a default-deny pipeline: source-specific minimization, a source-field policy gate before scanner-based redaction, detector and output passes over the canonical bundle plus every projection, a machine-readable `redaction_report`, and a reproducible red-team gate before any bundle reaches an agent or third-party model. The detector/toolchain decision is settled: Parallax owns a small Rust, source-aware, default-deny runtime redaction engine that fails closed, and uses Gitleaks (`v8.30.1`), Betterleaks (`v1.3.1`), TruffleHog (`v3.95.3`), detect-secrets (`v1.5.0`), Presidio (`2.2.362`), and the GitHub secret-scanning pattern corpus as offline validators and red-team comparators, never as blocking tiny-tier runtime dependencies. The Betterleaks drift recheck moves it from "unvetted future candidate" to a tracked `experimental_active` offline comparator, with network/credential-verification/LLM validation disabled by default; Gitleaks is now feature-complete (security patches only) so it stays the stable comparator but not the current-provider-pattern source. The synthetic canary fixture corpus is specified: commit manifests, expected findings, redacted outputs, hashes, and generator recipes, but keep raw provider-shaped values private or generator-only to avoid hosted-secret-scanning noise. The open gate is execution: A6 is proven only by committed red-team run artifacts showing zero seeded-canary leaks across canonical JSON, MCP `structuredContent`, and every agent-visible projection, fail-closed detector behavior, raw-ref isolation, source-field isolation, and preserved debugging usefulness — no current run exists yet, so the claim level remains `not_measured` until the ledger is populated.

This note consolidates the following previously-separate research files, each preserved in full below:

- `redaction-pipeline-and-secret-safety.md`
- `redaction-detector-toolchain.md`
- `redaction-toolchain-betterleaks-recheck.md`
- `a6-synthetic-canary-fixture-corpus.md`
- `a6-redaction-red-team-ledger.md`

## Redaction Pipeline and Secret Safety

_Provenance: merged verbatim from `redaction-pipeline-and-secret-safety.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

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

2026-05-25 correction: redaction is also not enough to protect A1 eval
integrity. The [Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md)
now requires `source-field-policy.json` to separate `agent_visible_seed`,
`runner_private`, `grader_private`, `triage_private`, and `public_audit`
source fields. A6 must consume that policy as a hard pre-redaction gate. A
scanner can miss no secrets and still fail if an agent-visible projection
contains a gold patch, hidden verifier ID, generated hint, parser source, or
resolving commit URL that the source-field policy forbids.

The concrete detector/toolchain decision is now split into
[Redaction detector toolchain](redaction-detector-toolchain.md): Parallax should
own a Rust, source-aware, default-deny runtime redaction engine and use
Gitleaks, Betterleaks, TruffleHog, detect-secrets, Presidio, and GitHub pattern
references as offline validators, not as blocking tiny-tier runtime
dependencies.
The result-ledger contract for proving this gate is in
[A6 redaction red-team ledger](a6-redaction-red-team-ledger.md).
The first public/private canary corpus boundary is specified in
[A6 synthetic canary fixture corpus](a6-synthetic-canary-fixture-corpus.md).

### Current Primary-Source Checks

| Source | What matters for Parallax |
| --- | --- |
| [Sentry SDK sensitive-data docs](https://docs.sentry.io/platforms/javascript/guides/nextjs/data-management/sensitive-data/) | Sentry recommends SDK-side scrubbing with `beforeSend` so sensitive data never leaves the local environment, plus server-side scrubbing as a storage safeguard. It explicitly calls out stack locals, breadcrumbs, user context, HTTP query strings, transaction names, and HTTP spans as sensitive-data paths. |
| [Sentry data scrubbing overview](https://docs.sentry.io/security-legal-pii/scrubbing/) | Sentry treats server-side scrubbing, advanced scrubbing, attachment scrubbing, and replay privacy as separate controls. Parallax should likewise split controls by surface instead of claiming one global scrubber. |
| [OpenTelemetry Collector processors](https://opentelemetry.io/docs/collector/components/processor/) | The Collector has transform/enrichment processors and a contrib/K8s Redaction Processor. As of the current docs, redaction is beta for traces and alpha for metrics/logs, so Parallax should support Collector-side redaction but not outsource bundle safety to it. |
| [OpenTelemetry redaction processor](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/processor/redactionprocessor/README.md) | The processor is designed to fail closed with `allowed_keys`, can mask blocked values, recommends HMAC for low-entropy data, can emit redaction audit attributes, and now documents URL/database-query sanitizers. Its `allowed_values` setting takes precedence over `blocked_values`, so Parallax should avoid broad allowlists in default policies. This maps well to Parallax ingest policy and bundle `redaction_report`, but it covers only telemetry attributes that pass through that processor. |
| [OpenTelemetry common `AnyValue`](https://opentelemetry.io/docs/specs/otel/common/#anyvalue) | OTLP values can be scalars, bytes, arrays, and key/value lists. Parallax must redact the typed tree before converting a value to text or Markdown. |
| [MCP tools specification `2025-11-25`](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) | MCP tool results can contain both text content and JSON `structuredContent`; when an `outputSchema` is provided, servers must conform to it and clients should validate it. Parallax redaction safety must therefore scan and hash the canonical `structuredContent`, not only the text block. |
| [MCP resources specification `2025-11-25`](https://modelcontextprotocol.io/specification/2025-11-25/server/resources) | MCP resources expose context through list/read APIs and host applications decide how resources enter model context. Sensitive resources need access controls. Parallax must scan `resources/read` output and resource templates as agent-visible projections, not only `tools/call` results. |
| [Claude Code MCP resources](https://code.claude.com/docs/en/mcp) | Claude Code lets users reference MCP resources with `@` mentions; referenced resources are automatically fetched and included as attachments, paths are fuzzy-searchable, and resource contents can be text, JSON, structured data, or other content types. A6 must treat client resource attachment as a projection path. |
| [Codex config reference](https://developers.openai.com/codex/config-reference) | Codex exposes `approval_policy.granular.mcp_elicitations` and memory controls including `features.memories`, `memories.generate_memories`, `memories.use_memories`, and `memories.disable_on_external_context`. A6 must record whether MCP evidence can be retained into memory or whether external-context memory generation is disabled. |
| [RFC 8785 JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785.html) | JCS defines a hashable JSON representation using I-JSON constraints and deterministic property sorting. Redaction reports and projection hashes should be computed after redaction over canonical JSON so CLI, HTTP, file, and MCP outputs can be compared. |
| [GitHub Actions log masking](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#masking-a-value-in-a-log) and [Actions secrets](https://docs.github.com/en/actions/concepts/security/secrets) | GitHub can mask values in logs, but `add-mask` must happen before output and GitHub states transformed secret redaction is not guaranteed. Parallax must treat CI logs/artifacts as hostile, even when they came from GitHub. |
| [GitHub secret scanning patterns](https://docs.github.com/en/code-security/reference/secret-security/supported-secret-scanning-patterns) | GitHub documents generic, AI-detected, and provider-specific pattern categories, with hundreds of provider patterns. Parallax should reuse a maintained pattern corpus for tests and canaries rather than inventing all patterns manually. |
| [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html) | OWASP says access tokens, passwords, connection strings, encryption keys, payment data, sensitive PII, illegal-to-collect data, and opt-out data should usually be removed, masked, sanitized, hashed, or encrypted; it also requires verification and access controls for logs. |
| [Langfuse masking docs](https://langfuse.com/docs/observability/features/masking) | LLM observability systems already expose SDK masking hooks over inputs, outputs, and metadata. Parallax should apply the same idea to agent-session capture but make masking mandatory-by-default, not an optional integration convenience. |

### Threat Model By Surface

| Surface | Leak modes | Default Parallax posture |
| --- | --- | --- |
| Sentry envelope events | `request`, headers, cookies, stack locals, breadcrumbs, query strings, raw URLs, tags, user context, attachments. | SDK-side minimization guidance, ingest allowlist, default strip of auth/cookie headers, route parameterization, metadata-only attachments until attachment scrubbing exists. |
| OTLP spans/logs/metrics | High-cardinality attributes, typed `AnyValue` maps/lists/bytes, SQL text, HTTP URLs, user IDs, tokens in custom attributes, log body text. | Collector redaction supported, but Parallax ingest repeats policy; unknown attributes are dropped or hashed unless project policy allows them. Redact typed values before text rendering. |
| Browser/frontend capture | Form values, full DOM text, URLs, network bodies, console logs, replay media. | Error + breadcrumbs first; replay opt-in only; mask all text/block media by default; network bodies disabled unless selectors/endpoints are explicitly marked safe. |
| CI logs/artifacts | Echoed env vars, transformed secrets, debug shells, test snapshots, coverage artifacts, uploaded files. | Treat all text/artifacts as untrusted; scan before storage and before bundle; store bounded excerpts plus raw refs behind scoped access. |
| CLI invocations | Secrets in argv/env/config paths, cwd/repo path leaks, stdout/stderr dumps, child process args. | Store command/subcommand and safe structural metadata; hash or redact argv/env by default; bounded redacted stdout/stderr excerpts only. |
| Agent sessions | User prompts, model inputs/outputs, tool args, shell output, file diffs, MCP responses. | Capture hashes, refs, policy decisions, and bounded redacted excerpts; full prompts/tool output are opt-in raw refs with short-lived access. |
| MCP resources and client retention | Resource list/read output, `@` resource attachments, fuzzy-searchable resource paths, client-persisted oversized output, local attachments, and memory generation from MCP context. | Treat resources as agent-visible projections; raw refs deny without sensitive scope; high-sensitivity evidence must be excluded from client memory or persisted only as redacted bounded artifacts. |
| Deploy/change provider records | Webhook/API payloads, deployment review comments, release notes, PR/issue text, deploy logs, environment URLs. | Store provider payloads and long text as raw refs by default; project structural fields and redacted summaries only after A6 provider-payload fixtures pass. |
| Attachments and raw evidence | Crash dumps, screenshots, replay blobs, log files, test artifacts, database exports. | Metadata-only v0; later storage requires type-specific scanner, size cap, and manual/project-level enablement. |
| Database query output | Row values, customer data, credentials in config tables, query text, parameters, plan text, export-like queries. | Read-only templates, row/column policy, aggregate/summarize first, never raw table dumps in agent bundles; see the [production database evidence access gate](production-database-evidence-access.md). |

### Pipeline Decision

Parallax redaction should be a **multi-stage safety pipeline**, not a single
function.

```text
capture source
  -> source-side minimization when available
  -> source-field policy gate for eval/corpus rows
  -> ingest normalization and default-deny field policy
  -> detector pass over accepted fields and raw refs
  -> storage with policy_version and raw_access_policy
  -> bundle builder redaction pass
  -> redaction_report and residual_risk
  -> schema validation and canonical bundle hashing
  -> projection_manifest hashing for JSON/Markdown/CLI/HTTP/MCP
  -> agent/API/MCP output scanner
```

#### Stage 1: Source-Side Minimization

Prefer not collecting sensitive data:

- SDK integrations should use `beforeSend`/equivalent hooks where available.
- Browser capture should disable replay and network bodies by default.
- CLI wrappers should separate command identity from argv/env values.
- Agent tracing should collect event shape, hashes, and refs before collecting
  full prompt/tool content.

This stage cannot be trusted alone because user code, framework integrations,
third-party SDKs, shells, and test tools can still emit secrets.

#### Stage 2: Source-Field And Ingest Default-Deny Policy

For generated eval/corpus inputs, apply the source-field policy before normal
redaction:

- `agent_visible_seed` fields can enter agent-visible bundle candidates after
  leakage review.
- `runner_private` fields can drive harness execution and parsing, but only
  sanitized command identity or aggregate status can appear in agent context.
- `grader_private` fields can be hashed for audit, but patch, test patch,
  hidden verifier, fixed commit, and resolving PR/commit content cannot enter
  Arm A/B/C or model prompts.
- `triage_private` fields are denied by default. Promotion requires a recorded
  reason and contamination-tier downgrade.
- `public_audit` fields can appear in manifests and hashes, but prompt/bundle
  inclusion still needs an arm-specific reason.

This policy is semantic isolation, not PII detection. Treat a violation as a
bundle-safety failure even when detectors, canary scans, and external scanners
all pass.

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

#### Stage 3: Detector Pass

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

#### Stage 4: Bundle-Build Redaction

Bundle construction performs a second pass because:

- ingest policy may change between event storage and bundle generation;
- bundles join surfaces, and cross-surface combinations can re-identify users;
- agents see a more concentrated evidence artifact than any one raw event.

The bundle builder should emit deterministic excerpts only after:

1. size bounding;
2. source-specific parsing;
3. secret/PII detection;
4. replacement/hashing;
5. schema validation;
6. canonical JSON hashing;
7. a final output scanner over the canonical JSON and every projection.

#### Stage 5: Output Guard

Every API, CLI, MCP, and third-party-model output must pass through the same
final scanner. The scanner target is the exact canonical JSON object plus the
exact projection bytes that the caller or model receives. This catches
accidental leaks from:

- Markdown projection bugs;
- model-generated summaries that copied raw material;
- tool responses embedded inside agent-session evidence;
- MCP `resources/read` output, resource attachments, and fuzzy-search resource
  labels;
- client-side persisted files, attachments, or memories derived from MCP
  context;
- newly added fields not covered by older bundle schema tests.

#### Stage 6: Canonical Projection Gate

The newer [evidence bundle schema contract](evidence-bundle-and-schema.md)
turns redaction from a text-rendering property into a canonical-bundle property.
Before a bundle can be exposed to an agent, API caller, CLI user, MCP client, or
eval/corpus row:

- the canonical bundle JSON must include `schema_ref`, `canonical_hash`,
  `projection_manifest`, `redaction_report`, `source_field_policy`, `access`,
  and raw-ref policy fields;
- `canonical_hash` must be computed after source-field filtering, redaction,
  and residual-risk labeling;
- every JSON, Markdown, CLI, HTTP, ZIP, model-prompt, and MCP projection must be
  derived from the canonical JSON and recorded in `projection_manifest`;
- MCP bundle tools must declare an `outputSchema` matching `schema_ref.uri` and
  return the canonical object in `structuredContent`;
- MCP resource paths, `resources/read` responses, and resource-template outputs
  must either return redacted bounded evidence derived from the same canonical
  object or deny access to raw refs without the matching sensitive scope;
- client-retention paths, including persisted oversized MCP output, resource
  attachments, and memory generation from MCP context, must be recorded in the
  projection or retention manifest when a client is part of the claim;
- `_meta`, tool annotations, descriptions, or text-only JSON can duplicate
  hashes or give hints, but cannot be the only place safety fields appear;
- if a projection hash differs from the manifest, if `structuredContent` is
  missing for a bundle-returning MCP tool, if a resource read bypasses the
  canonical/redacted path, or if a leak appears only in a rendered projection,
  client attachment, or retained client artifact, the A6 run fails.

This is stricter than "scan JSON and Markdown." Redaction must hold for the
canonical object and for every consumer-visible projection, otherwise a safe
stored bundle can still become an unsafe agent prompt.

### Required Redaction Report

The existing [evidence bundle spec](evidence-bundle-and-schema.md) already
requires `redaction_report`. A6 needs the report to be more than a decorative
object:

```json
{
  "policy_version": "redact-v1",
  "policy_mode": "default-deny",
  "source_field_policy": {
    "version": "phase0-source-field-policy-v1",
    "hash": "sha256:...",
    "violations": 0
  },
  "bundle_schema_ref": {
    "uri": "https://parallax.dev/schemas/evidence-bundle/v0.json",
    "hash": "sha256:...",
    "canonicalization": "jcs-rfc8785"
  },
  "canonical_bundle_hash": "sha256:...",
  "projection_manifest": {
    "bundle_json": {
      "hash": "sha256:...",
      "scanner_status": "pass"
    },
    "bundle_markdown": {
      "hash": "sha256:...",
      "scanner_status": "pass"
    },
    "mcp_structuredContent": {
      "hash": "sha256:...",
      "output_schema_id": "https://parallax.dev/schemas/evidence-bundle/v0.json",
      "scanner_status": "pass"
    }
  },
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
automatically. For A1-style eval rows, `source_field_policy.violations > 0`
blocks the result even when `known_secret_matches = 0`. A report that lacks
`bundle_schema_ref`, `canonical_bundle_hash`, or per-projection scanner status
is incomplete for agent-visible claims.

### Red-Team Gate

Before any agent/third-party-model exposure, Parallax needs a reproducible
redaction eval suite. The run artifacts, row schemas, claim levels, and
freshness rules are defined in the
[A6 redaction red-team ledger](a6-redaction-red-team-ledger.md).

| Gate | Required result |
| --- | --- |
| Seeded secret corpus | Canary tokens, provider token examples, private keys, DB URLs, JWTs, cookies, auth headers, emails, phone numbers, IPs, user names, path-sensitive data, and payment-like numbers are seeded across every supported surface. |
| Dual rendering scan | Canonical bundle JSON and Markdown projection scan clean. |
| Projection manifest | Every CLI, HTTP, MCP, model-prompt, and file projection has a hash in `projection_manifest` and derives from the canonical bundle hash. |
| MCP structured output | Bundle-returning MCP tools validate `structuredContent` against the bundle `outputSchema`; text-only MCP output is a projection, not proof of schema-safe redaction. |
| MCP resource output | `resources/list`, `resources/read`, resource templates, and client resource attachments contain no canaries and enforce raw-ref scope denial. |
| Client retention | Codex memory settings, Claude persisted-file behavior, resource attachments, and any client-side output persistence are recorded; sensitive evidence is excluded from memory or stored only as redacted bounded artifacts. |
| Raw-ref isolation | Raw refs are not dereferenced in default agent/API/MCP output. |
| Detector failure mode | If a detector errors, unsafe fields are stripped and the bundle reports `manual_review_required` or fails closed. |
| Source-field isolation | `runner_private`, `grader_private`, and default `triage_private` fields do not appear in agent-visible projections. |
| Regression fixture | The seeded corpus runs in CI for every policy/schema change. |
| Real-data pilot | Operator-owned real logs/CI/CLI/frontend sessions run through the same suite before external users. |
| False-positive review | Redaction must not erase the minimum evidence needed for A1 bundle-value evals. |

Initial acceptance criterion: **zero known seeded canary leaks** in canonical
bundle JSON, MCP `structuredContent`, and every agent-visible projection. This
does not prove no leaks exist, but anything weaker is not compatible with the
data-ownership value prop.

### Kill And Narrowing Criteria

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

### Product Implication

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

### Relationship To Other Research

- [Risks and the bear case](risks-and-bear-case.md) — this is the concrete A6
  mitigation and falsification plan.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) — this note
  expands the mandatory `redaction_report`.
- [Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md) —
  defines the A1 source-field policy that A6 must enforce before scanner-based
  redaction.
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
- [Redaction toolchain Betterleaks recheck](redaction-toolchain-betterleaks-recheck.md)
  — updates the external scanner set now that Betterleaks is active, while
  keeping network and LLM validation out of the default path.
- [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) — defines the
  run artifacts, fixture rows, projection audits, claim levels, and freshness
  rules that make A6 pass/fail claims auditable.
- [A6 synthetic canary fixture corpus](a6-synthetic-canary-fixture-corpus.md)
  — defines which canary manifests, redacted outputs, hashes, and generator
  recipes are public, and which provider-shaped raw values stay private.
- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
  — redaction is a precondition for safe read-only agent context.
- [Production database evidence access gate](production-database-evidence-access.md)
  — turns database query output into a read-only, template-driven, redacted, and
  audited evidence source.
- [Production database evidence ledger](production-database-evidence-ledger.md)
  — defines when direct database evidence can be claimed after seeded secrets,
  prompt injection, RLS scope, limits, and audit fixtures pass.

### Bottom Line

Parallax can keep A6 alive only by treating redaction as a tested evidence
pipeline. The defensible MVP is not "we scrub secrets"; it is "agent-visible
bundles are default-deny, red-team-tested, source-specific, and self-report the
policy and residual risk that made them safe enough to expose."

## Redaction Detector Toolchain

_Provenance: merged verbatim from `redaction-detector-toolchain.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

The [redaction pipeline](redaction-pipeline-and-secret-safety.md) establishes the
A6 policy: default-deny capture, source-specific minimization, detector passes,
and a red-team gate before agent exposure. This note answers the implementation
question it leaves open:

> Which detector engines should Parallax actually use, and where are they
> allowed in the runtime path?

Decision:

> Build the runtime redaction path as a small Rust policy engine with typed
> parsers, allowlists, deny rules, HMAC hashing, and streaming output scanning.
> Use Gitleaks, Betterleaks, TruffleHog, detect-secrets, Presidio, and GitHub
> patterns as reference corpora, offline validators, and CI/red-team
> comparators, not as blocking runtime dependencies in the tiny tier.

The reason is operational. Parallax must redact Sentry events, OTLP attributes,
CI logs, CLI output, agent transcripts, frontend breadcrumbs, and database
evidence while producing bounded bundles quickly. Git-repo secret scanners and
PII anonymizers are useful references, but they do not directly provide a
low-latency, source-aware, evidence-bundle-safe runtime.

The companion [redaction toolchain Betterleaks recheck](redaction-toolchain-betterleaks-recheck.md)
updates the external scanner boundary: Betterleaks is now an active comparator
candidate, but its network, credential-verification, and LLM-validation features
must be disabled by default for A6 fixture runs.

### Current Primary-Source Checks (Detector Toolchain)

| Source | What it provides | Parallax implication |
| --- | --- | --- |
| [OpenTelemetry Collector redaction processor](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/processor/redactionprocessor/README.md) | Checked 2026-05-25. Current README lists stability as alpha for logs/metrics and beta for traces. It supports `allowed_keys`, `blocked_values`, `allowed_values`, masking, HMAC hash functions, redaction audit attributes, URL sanitization, and database-query sanitization. `allowed_values` takes precedence over `blocked_values`. Follow-up release API checks showed OpenTelemetry Collector core source at `v0.153.0` on 2026-05-25, collector-releases stable binaries/images at `v0.152.1`, and collector-contrib latest still `v0.152.0`. | Good model for OTLP attribute policy and audit fields, but broad allowlists can intentionally bypass blocked-value masking. Not enough for bundle safety because it only covers data that flowed through that Collector processor. Pin redaction evidence to the collector-contrib processor and tested distribution version, not just the Collector core source version. |
| [OpenTelemetry Collector processor catalog](https://opentelemetry.io/docs/collector/components/processor/) | Checked 2026-05-25. The catalog was last modified 2026-03-16 and links Redaction Processor at contrib `v0.152.0`, included in contrib/K8s with traces beta and metrics/logs alpha. | Parallax should support Collector-side redaction, but still repeat policy in ingest and bundle building. Treat upstream Collector redaction as a useful pre-filter, not an agent-safety boundary. |
| [OpenTelemetry common `AnyValue`](https://opentelemetry.io/docs/specs/otel/common/#anyvalue) | OTLP values can be scalars, byte arrays, arrays, and key/value lists. | Redaction must traverse typed values directly; string rendering of maps/lists is only a projection and cannot be the detector input. |
| [GitHub secret scanning supported patterns](https://docs.github.com/en/code-security/reference/secret-security/supported-secret-scanning-patterns) | Current pattern taxonomy includes generic, AI-detected, and provider patterns, with 500+ provider entries and notes on precision, validity checks, partner alerts, and token-version churn. | Use as a maintained external reference for canary coverage and provider-pattern watchlists. Do not assume Parallax can copy GitHub's proprietary AI/validity checks. |
| [Gitleaks v8.30.1](https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1) | Latest release checked 2026-05-25; GitHub API shows `v8.30.1` published 2026-03-21. Mature open-source scanner for git history, directories, files, and stdin; supports default/custom config, pre-commit, GitHub Action, JSON/CSV/JUnit/SARIF-style workflows, archive/decode scanning, redacted output, and baseline/ignore behavior. Its README now says Gitleaks is feature-complete and future releases are security patches only while focus shifts to Betterleaks. | Best stable comparator for Phase 0 fixture scanning, but no longer a strong signal for new provider-pattern coverage. Its Go binary can validate redaction fixtures in CI, but a blocking runtime shell-out would hurt tiny-tier latency and failure modes. |
| [Betterleaks v1.3.1](https://github.com/betterleaks/betterleaks/releases/tag/v1.3.1) | Latest release checked 2026-05-25; GitHub API shows `v1.3.1` published 2026-05-22, public MIT repo, current push activity, JSON/SARIF reports, `dir`/`git`/`github`/`s3`/`stdin` sources, checksums, Sigstore metadata, CEL filters, and live validation hooks. Config docs include HTTP, AWS, and LLM-assisted validation examples. | Move from unvetted future candidate to experimental active comparator. Useful for red-team fixture comparison once binary/checksum/report schema are pinned. Validation/network/LLM features must be disabled by default and never run in the Parallax runtime path. |
| [TruffleHog v3.95.3](https://github.com/trufflesecurity/trufflehog/releases/tag/v3.95.3) | Latest release checked 2026-05-25; GitHub API shows `v3.95.3` published 2026-05-11. AGPL-3.0 scanner focused on finding, verifying, and analyzing leaked credentials across git, GitHub, S3/GCS, Docker, Hugging Face, stdin, and multi-source configs. Verified findings are confirmed by testing against provider APIs, and JSON output is available. | Useful red-team comparator for live/verified secrets and historical/source scans. Verification and credential analysis can create network, privacy, and rate-limit side effects, so A6 must record whether verification was disabled or approved for a private fixture run and must not run it in the default runtime path. |
| [Yelp detect-secrets v1.5.0](https://github.com/Yelp/detect-secrets/releases/tag/v1.5.0) | Latest release checked 2026-05-25; GitHub API shows `v1.5.0` published 2024-05-06. Baseline-driven secret detection with configurable plugins, allowlists, entropy detectors, filters, verification settings, pre-commit hooks, and audit workflow. | Useful for repository baselines, "new secret" regression checks, and human review workflows. Its older release cadence and Python/plugin model make it a secondary comparator, not a provider-churn source and not a fit for the Parallax hot path. |
| [Microsoft Presidio 2.2.362](https://github.com/microsoft/presidio/releases/tag/2.2.362) | Latest release checked 2026-05-25; GitHub API shows `2.2.362` published 2026-03-18, and PyPI shows `presidio-analyzer`/`presidio-anonymizer` `2.2.362` uploaded 2026-03-15. Open-source PII detection/anonymization framework with NER, regex, rule logic, image redaction, custom recognizers, and an explicit warning that automated detection cannot guarantee all sensitive information is found. | Best reference for PII/anonymization and optional offline processing. Too heavy and probabilistic to be the only runtime guarantee for agent-visible bundles. |
| [IssueGuard paper](https://arxiv.org/abs/2602.08072) | Current research on real-time secret leak prevention in issue reports; targets unstructured collaborative text and separates real secrets from false positives. | Confirms Parallax's risk surface: secrets leak in issue-like/debug text, not only git repos. Good design reference for pre-submit warning UX, not yet a runtime dependency. |

### What Existing Tools Are Good At

| Tool class | Strong fit | Weak fit |
| --- | --- | --- |
| Git/repo secret scanners | Historical git scans, fixture corpus checks, PR/pre-commit validation, SARIF/CI reporting, provider token pattern coverage. | Runtime Sentry/OTLP/CLI/frontend bundle building; they expect files/repos more than typed telemetry nodes. |
| Verified secret scanners | Prioritizing real credential leaks and reducing false positives when provider verification is available. | Agent-bundle redaction where making network verification calls leaks candidate secrets and slows the request. |
| Baseline scanners | Letting teams suppress known findings and detect newly introduced secrets. | Default-deny runtime output, where "known existing secret" still must not be shown to an agent. |
| PII anonymizers | Person names, emails, locations, phone numbers, structured and unstructured text, image redaction, custom recognizers. | Deterministic low-latency safety guarantees for all secret classes; PII tools warn that misses remain possible. |
| Collector redaction processors | Early OTLP attribute minimization before data reaches Parallax. | Surfaces outside OTLP, raw refs, Markdown projections, database outputs, and post-correlation bundle joins. |

The conclusion is not "pick Gitleaks" or "pick Presidio." The correct design is
layered: Parallax owns the final runtime decision and uses external tools to
stress-test that decision.

### 2026-05-25 Freshness Findings

This pass narrowed the trust boundary rather than changing the architecture:

- Gitleaks remains the best simple stable comparator for generated fixture
  output, but its current README says active feature work has moved elsewhere.
  Parallax should not depend on Gitleaks alone for current provider-token churn.
- Betterleaks is now active enough to track as an experimental comparator, with
  MIT licensing, JSON/SARIF output, stdin support, and release checksums. Its
  CEL validation can make network or LLM calls, so A6 must default it to
  offline/no-validation mode and record any exception.
- TruffleHog is fresher and stronger for verified credential evidence, but that
  strength comes from provider/API verification and optional credential
  analysis. A6 fixture runs must record network policy, verification mode, and
  whether the corpus was synthetic or private.
- detect-secrets is still useful for baseline and audit workflows, but a 2024
  latest release makes it weak evidence for current provider coverage.
- Presidio is current enough to remain the PII reference, but its own warning
  means PII detection cannot replace default-deny source policy.
- The OpenTelemetry redaction processor is a good upstream minimizer, especially
  for HMAC hashing and query/URL sanitization, but alpha signal stability and
  `allowed_values` precedence make it unsafe as the final agent-output boundary.
- OpenTelemetry Collector version numbers need source/distribution precision:
  core source reached `v0.153.0`, while collector-contrib redaction and the
  latest stable collector-releases distribution checked here remain on the
  `v0.152.x` axis. A6 rows should store both the processor source version and
  the actual Collector binary/image used in a fixture.

Falsification criteria:

- If Betterleaks changes license, report schema, default validation behavior,
  checksum/signing contract, or offline/no-network behavior, re-run the scanner
  comparison rows before using it in an A6 gate.
- If OpenTelemetry redaction reaches stable status for logs/metrics and removes
  or changes allowlist precedence, revisit how much work must be duplicated at
  Parallax ingest.
- If TruffleHog changes verification defaults, license, output schema, or
  detector/source coverage materially, re-run the A6 scanner comparison rows.
- If detect-secrets remains stale while GitHub's provider taxonomy changes
  substantially, demote it further to legacy-baseline checks only.

### Runtime Architecture

The runtime detector should be an internal Rust library used by ingest, bundle
building, CLI/API/MCP rendering, and tests:

```text
typed source parser
  -> field policy compiler
  -> key-name rules
  -> structured value parsers
  -> provider/secret pattern matchers
  -> PII recognizers where cheap and deterministic
  -> HMAC/hash/strip/ref-only action
  -> final JSON + Markdown output scanner
  -> machine-readable redaction report
```

Implementation requirements:

| Layer | Requirement |
| --- | --- |
| Policy compiler | Compile project policy into per-source allow/hash/strip/ref-only decisions. Unknown fields default to strip or hash. |
| Structured parsers | Parse URLs, referrers, headers, cookies, baggage, JSON, YAML, TOML, env files, shell argv, Sentry envelope items, OTLP typed `AnyValue` values, provider webhook/API payloads, database URLs, SQL text/parameters, JWT-like strings, and PEM/private-key blocks before regex matching. |
| Pattern engine | Use anchored provider patterns and high-confidence generic patterns for tokens, private keys, auth headers, connection strings, cookies, and bearer/basic credentials. |
| Streaming scanner | Scan stdout/stderr/log excerpts incrementally with bounded buffers; never buffer unbounded output just to redact it. |
| HMAC hashing | Use keyed HMAC for joinable low-entropy values such as IPs, user IDs, session IDs, emails, and path fragments. Do not use plain SHA/MD5 for values an attacker can enumerate. |
| Raw-ref guard | Full raw logs, prompts, attachments, screenshots, database rows, and terminal output remain refs unless a scoped human read-sensitive grant exists. |
| Output scanner | Re-scan final JSON and Markdown renderings so projection bugs cannot leak values that were safe in the internal graph. |
| Deterministic report | Emit rule IDs, action, counts, detector versions, policy version, residual risk, and manual-review flag for every bundle. |

### Recommended Toolchain By Phase

| Phase | Runtime dependency | CI / red-team dependency | Rationale |
| --- | --- | --- | --- |
| Phase 0 bundle eval | Internal Rust sanitizer plus a small curated canary corpus. | Run Gitleaks and/or Betterleaks plus detect-secrets over generated fixtures; manually inspect Presidio-style PII cases. | Keep the eval lightweight, but prevent obvious secret leaks in arms B/C. Betterleaks validation stays disabled. |
| Phase 1 tiny tier | Internal Rust redaction library is mandatory. No Python/Go sidecar in the request path. | Gitleaks/Betterleaks on fixture dirs and generated bundles; TruffleHog on private local red-team corpus when network verification is safe. | Tiny tier cannot depend on multi-runtime scanner services to claim simplicity. |
| Phase 2 red-team | Internal library plus optional offline Presidio adapter for PII-heavy text/image fixtures. | Gitleaks, Betterleaks, TruffleHog, detect-secrets, Presidio, GitHub pattern watchlist, and custom canary corpus. | This is where A6 can earn broader source coverage and decide whether Betterleaks replaces Gitleaks as the primary static comparator. |
| Phase 3 enterprise/pilots | Same runtime library; optional policy pack imports from customer secret-scanning tools. | Customer scanners can validate exported bundle fixtures. | Enterprise integration should validate Parallax, not replace its default-deny core. |

### Detector Catalog

Start with a small explicit catalog. Add provider patterns only when they are
fixture-tested and source-mapped to a redaction action.

| Category | Minimum runtime action |
| --- | --- |
| Private keys and cert material | Strip entire PEM block; preserve type and count only. |
| Auth headers and cookies | Strip value; keep header/cookie name only when policy allows. |
| Bearer/basic credentials | Strip value; report rule and field. |
| Cloud/provider tokens | Strip value; keep provider/rule ID when detected. |
| Database and service URLs | Parse; strip password/token; HMAC host/user/db if joins need them. |
| JWT-like values | Strip raw token; optionally decode header claims only if policy allows and signature/payload are not exposed. |
| Emails/user IDs/session IDs/IPs | HMAC by default for joins; strip when not needed. |
| File paths | Preserve repo-relative safe path; HMAC or truncate user/home/customer fragments. |
| HTTP URLs/query strings | Normalize route/path; strip query values; HMAC selected stable identifiers. |
| Referrer URLs and browser console logs | Strip or redact raw values; preserve event shape and route class only. |
| OTLP typed maps/lists/bytes | Traverse as typed values; redact before string rendering or Markdown projection. |
| Baggage/session context | Allowlist opaque keys; HMAC or strip values; raw user/account/session values fail the fixture. |
| Provider payloads and deploy logs | Keep raw payload/log refs scoped; project only structural fields and redacted summaries. |
| Prompt/tool output snippets | Ref-only by default; redacted excerpt only after final output scanner passes. |
| Frontend DOM/replay/screenshot text | Metadata-only or opt-in masked replay until replay red-team passes. |

### Test Corpus

The redaction test corpus should be separate from production data and committed
as synthetic fixtures. It should include:

- provider-like tokens from GitHub's public pattern taxonomy;
- generated fake private keys, JWTs, cookies, bearer/basic headers, database
  URLs, and webhook URLs;
- emails, phone numbers, IP addresses, names, path fragments, and customer-like
  identifiers;
- encoded variants: base64, URL-encoded, JSON-escaped, shell-quoted, multiline,
  YAML/TOML/env formats;
- adversarial context: tokens split across log lines, stack traces containing
  env dumps, command echoes, prompt transcripts, Markdown tables, code blocks,
  Sentry request contexts, OTLP typed `AnyValue` maps/lists/bytes, browser
  request URLs/referrers/headers/console logs, baggage values, provider webhook
  payloads, deploy logs, deployment review comments, release notes, PR/issue
  text, SQL text/parameters, and database error messages;
- safe near-misses and false-positive controls so useful evidence is not erased.

Every fixture should declare expected findings:

```json
{
  "fixture": "cli_stdout_database_url",
  "surface": "cli_stdout",
  "expected": [
    {
      "kind": "database_url",
      "action": "strip",
      "rule_id": "secret.database_url.v1",
      "field": "stdout.excerpt"
    }
  ],
  "must_not_redact": ["postgres error code 23505", "src/users.rs:118"]
}
```

### Pass / Fail Gate

The A6 detector gate passes only when:

| Gate | Pass condition |
| --- | --- |
| Canary leaks | Zero expected canary values appear in agent-visible JSON or Markdown. |
| Raw refs | No default agent/API/MCP response dereferences raw refs. |
| Cross-tool check | Gitleaks and/or Betterleaks plus detect-secrets do not find unredacted secrets in committed generated fixtures. |
| Verification check | TruffleHog is run only on approved local/private red-team fixtures; any verified live secret in output fails the gate and triggers rotation. |
| PII check | Presidio-style fixtures pass for the project-supported PII classes, but misses do not override default-deny source policy. |
| False positives | Redaction does not remove stack frame, span, issue, route, test, or command evidence needed by the A1 bundle eval. |
| Failure mode | Any detector crash or timeout strips the affected field or blocks the bundle; it never lets the field through. |
| Report | The bundle's `redaction_report` records detector versions, rule IDs, actions, counts, and manual-review status. |

If the cross-tool scanners catch something the runtime library missed, the
runtime rules must be updated before the bundle is considered agent-safe. If the
runtime library catches a value external scanners miss, keep the Parallax rule:
external tools are comparators, not the source of truth.

### Product Decision

Parallax should not market "AI-safe redaction" as a generic solved problem.
The honest claim is narrower:

- runtime capture is default-deny by source and field;
- final bundle outputs are scanned twice, once at source normalization and once
  at projection;
- external scanners validate fixtures and red-team corpora;
- raw sensitive data is retained only as scoped refs, not model-visible content;
- every bundle carries a redaction report that a test or agent can reject.

This keeps the product buildable and avoids outsourcing trust to tools built for
a different surface.

### Relationship To Other Research (Detector Toolchain)

- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  defines the A6 policy and red-team gate this toolchain implements.
- [Redaction toolchain Betterleaks recheck](redaction-toolchain-betterleaks-recheck.md)
  moves Betterleaks into the tracked comparator set while keeping validation
  disabled by default.
- [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) defines the
  run artifacts that prove the toolchain does not leak seeded canaries and still
  preserves minimum debug usefulness.
- [A6 synthetic canary fixture corpus](a6-synthetic-canary-fixture-corpus.md)
  defines the minimum fixture classes and commit policy for public redaction
  tests.
- [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md)
  supplies the highest-risk streaming output fixture surface.
- [CLI trace safety ledger](cli-trace-safety-ledger.md) records the CLI-specific
  canary, projection, and overhead rows that make that surface claimable.
- [Agent session tracing across real tools](agent-session-tracing-real-tools.md)
  supplies prompt/tool-output redaction fixtures.
- [Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md)
  supplies browser and replay privacy fixtures.
- [Frontend capture safety ledger](frontend-capture-safety-ledger.md) records
  browser-specific privacy canary, breadcrumb, replay, projection, and overhead
  rows for those fixtures.
- [Production database evidence access gate](production-database-evidence-access.md)
  supplies database-output and SQL-error fixtures.
- [Deploy/change context ledger](deploy-change-context-ledger.md) supplies
  provider payload, deployment review, release note, PR/issue text, and deploy
  log fixtures.
- [Bundle-value Phase 0 runbook](bundle-value-phase0-runbook.md) must use this
  toolchain before exposing any generated bundle or raw dump to an agent.

### Bottom Line (Detector Toolchain)

Use external secret and PII scanners to keep Parallax honest, but do not put
them on the critical request path. The critical request path needs a Rust,
source-aware, default-deny redaction engine that fails closed and proves its work
through `redaction_report`, fixture snapshots, and cross-tool red-team checks.

## Redaction Toolchain Betterleaks Recheck

_Provenance: merged verbatim from `redaction-toolchain-betterleaks-recheck.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

The redaction detector note said to watch Betterleaks only after its license,
CLI, and output contracts were checked. That claim was too stale for an A6
toolchain decision because Gitleaks now states that it is feature-complete while
Betterleaks has active releases.

This pass rechecks that boundary:

> Betterleaks should move from "unvetted future candidate" to "tracked offline
> comparator candidate." It still must not become Parallax's runtime redaction
> boundary, and any network, credential-verification, or LLM validation features
> must be disabled by default for A6 fixture runs.

### Current Primary-Source Snapshot

| Source | Current facts | Parallax impact |
| --- | --- | --- |
| [Gitleaks `v8.30.1`](https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1) and [README](https://github.com/gitleaks/gitleaks) | Latest GitHub release remains `v8.30.1`, published `2026-03-21T02:17:58Z`. The README states Gitleaks is feature-complete and future releases are security patches only while focus shifts to Betterleaks. It still supports git, dir, stdin, JSON/CSV/JUnit/SARIF reports, redacted output, archive scanning, decode depth, and baselines. | Keep as the stable static comparator for now, but do not rely on it as the best current provider-pattern source. |
| [Betterleaks repo](https://github.com/betterleaks/betterleaks), [release `v1.3.1`](https://github.com/betterleaks/betterleaks/releases/tag/v1.3.1), and [scanning docs](https://github.com/betterleaks/betterleaks/blob/main/docs/scanning.md) | Repository is public, MIT-licensed, pushed on `2026-05-25T13:15:48Z`, and has latest release `v1.3.1` published `2026-05-22T16:18:18Z`. The scanning docs cover `dir`, `git`, `github`, `s3`, and `stdin`; reports include JSON and SARIF; releases ship checksums and Sigstore metadata. | Add to A6 as an experimental offline comparator candidate for generated fixtures. Require pinned binary/checksum and report-format snapshot before it can replace Gitleaks in a gate. |
| [Betterleaks config docs](https://github.com/betterleaks/betterleaks/blob/main/docs/config.md) | Betterleaks uses CEL filters and validation expressions. Validation can make HTTP requests, AWS validation calls, and even LLM-based validation examples that call OpenAI with obfuscated candidate/context. | Validation is powerful but unsafe as a default A6 comparator mode. A6 must record `network_calls_allowed=false` and `llm_validation_allowed=false` unless a private, approved red-team run explicitly opts in. |
| [TruffleHog `v3.95.3`](https://github.com/trufflesecurity/trufflehog/releases/tag/v3.95.3) | Latest release remains `v3.95.3`, published `2026-05-11T18:38:34Z`; repository license API reports AGPL-3.0. | Still useful for private verified-secret checks, not runtime. |
| [detect-secrets `v1.5.0`](https://github.com/Yelp/detect-secrets/releases/tag/v1.5.0) | Latest release remains `v1.5.0`, published `2024-05-06T18:05:06Z`; docs emphasize baseline/audit/plugin workflows. | Keep as legacy-baseline/new-secret comparator, not provider-churn source. |
| [Presidio `2.2.362`](https://github.com/microsoft/presidio/releases/tag/2.2.362), [PyPI analyzer](https://pypi.org/project/presidio-analyzer/), and [PyPI anonymizer](https://pypi.org/project/presidio-anonymizer/) | Latest release remains `2.2.362`, published `2026-03-18T05:32:57Z`; PyPI analyzer/anonymizer latest versions are `2.2.362`. README warns automated detection cannot guarantee all sensitive information is found. | Keep as PII reference and optional offline comparator; default-deny source policy still carries the safety guarantee. |
| [OpenTelemetry Collector redaction processor](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/processor/redactionprocessor/README.md), [contrib `v0.152.0`](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.152.0), [collector core `v0.153.0`](https://github.com/open-telemetry/opentelemetry-collector/releases/tag/v0.153.0), and [collector-releases `v0.152.1`](https://github.com/open-telemetry/opentelemetry-collector-releases/releases/tag/v0.152.1) | Current README still lists alpha logs/metrics and beta traces. It supports fail-closed `allowed_keys`, `blocked_values`, `allowed_values`, HMAC hashes, audit attributes, URL sanitization, and database-query sanitization; `allowed_values` takes precedence over `blocked_values`. Follow-up release API checks found collector core source latest `v0.153.0`, collector-releases stable binaries/images latest `v0.152.1`, and collector-contrib latest `v0.152.0`. | Good upstream minimizer and policy model, but still not the final bundle-output boundary. A6 fixture rows must pin the contrib processor/source and the actual Collector distribution used instead of inheriting the newer core-source version number. |

### Decision Update

The A6 architecture does **not** change:

- Parallax still needs an internal Rust default-deny redaction engine in the
  ingest, bundle, CLI, HTTP, and MCP output paths.
- External tools remain comparators over generated fixtures and private red-team
  corpora.
- The public A6 gate still requires zero canary leaks, source-field isolation,
  projection-hash equivalence, and a complete `redaction_report`.

The scanner set changes:

- `gitleaks` remains the stable, known-output comparator for Phase 0.
- `betterleaks` joins the tracked comparator set as `experimental_active`.
- `trufflehog` remains private/approved verification only.
- `detect-secrets` remains a baseline/audit comparator.
- `presidio` remains the PII/anonymization reference, not the safety boundary.

### Betterleaks Guardrails

Before Betterleaks can replace or sit alongside Gitleaks in a pass/fail A6 run,
the run manifest must record:

```json
{
  "scanner": "betterleaks",
  "scanner_version": "1.3.1",
  "scanner_license": "MIT",
  "scanner_release_published_at": "2026-05-22T16:18:18Z",
  "scanner_development_posture": "active_experimental_comparator",
  "scanner_execution_mode": "offline_no_network",
  "network_calls_allowed": false,
  "secret_validation_allowed": false,
  "llm_validation_allowed": false,
  "report_formats_checked": ["json", "sarif"],
  "binary_integrity": "release_checksum_or_sigstore_verified"
}
```

If a run enables Betterleaks validation, it must be a private red-team run with
synthetic or operator-approved data. The result row must record which validation
endpoints were allowed, whether candidate values or obfuscated context left the
machine, and why that does not apply to the default runtime path.

### Impact On A6

- Phase 0 should still be lightweight: internal sanitizer plus Gitleaks and/or
  Betterleaks over generated fixtures, detect-secrets as a baseline comparator,
  and manual/Presidio-style PII checks.
- Phase 1 tiny tier must not shell out to Betterleaks in the request path. Use it
  in CI/red-team checks only.
- Phase 2 should compare Gitleaks and Betterleaks outputs on the same committed
  synthetic fixtures and record false positives, misses, and output-schema
  stability.
- GitHub's provider-pattern taxonomy remains a separate watchlist; Betterleaks
  does not replace the need to track provider token churn.
- Collector-side redaction is also a versioned fixture input. Store the
  collector-contrib redaction processor version and the tested Collector
  distribution separately because core source, contrib, and released
  binaries/images can move on different dates.

### Falsification Triggers

Revisit this decision when:

- Betterleaks changes license, report schema, default validation behavior, or
  release signing/checksum contract;
- Gitleaks receives new feature releases rather than security patches only;
- Betterleaks red-team output proves less stable or less useful than Gitleaks on
  Parallax synthetic fixtures;
- Betterleaks validation or LLM-based examples become enabled by default;
- A6 cannot run Betterleaks in an offline, no-network, no-LLM mode.

### Sources (Betterleaks Recheck)

- [GitHub API: Gitleaks latest release](https://api.github.com/repos/gitleaks/gitleaks/releases/latest)
- [Gitleaks README](https://github.com/gitleaks/gitleaks)
- [GitHub API: Betterleaks repository](https://api.github.com/repos/betterleaks/betterleaks)
- [GitHub API: Betterleaks latest release](https://api.github.com/repos/betterleaks/betterleaks/releases/latest)
- [Betterleaks README](https://github.com/betterleaks/betterleaks)
- [Betterleaks scanning docs](https://github.com/betterleaks/betterleaks/blob/main/docs/scanning.md)
- [Betterleaks config docs](https://github.com/betterleaks/betterleaks/blob/main/docs/config.md)
- [GitHub API: Betterleaks license](https://api.github.com/repos/betterleaks/betterleaks/license)
- [GitHub API: TruffleHog latest release](https://api.github.com/repos/trufflesecurity/trufflehog/releases/latest)
- [GitHub API: TruffleHog license](https://api.github.com/repos/trufflesecurity/trufflehog/license)
- [GitHub API: detect-secrets latest release](https://api.github.com/repos/Yelp/detect-secrets/releases/latest)
- [GitHub API: Presidio latest release](https://api.github.com/repos/microsoft/presidio/releases/latest)
- [PyPI: presidio-analyzer](https://pypi.org/pypi/presidio-analyzer/json)
- [PyPI: presidio-anonymizer](https://pypi.org/pypi/presidio-anonymizer/json)
- [OpenTelemetry redaction processor README](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/processor/redactionprocessor/README.md)
- [OpenTelemetry Collector Contrib latest release](https://api.github.com/repos/open-telemetry/opentelemetry-collector-contrib/releases/latest)
- [GitHub secret scanning supported patterns](https://docs.github.com/en/code-security/secret-scanning/secret-scanning-patterns)

### Bottom Line (Betterleaks Recheck)

Betterleaks is now relevant enough to track in A6, but it strengthens the
offline comparator story, not the runtime safety boundary. The safest near-term
position is Gitleaks for stable fixture comparison, Betterleaks as an active
experimental comparator with validation disabled, and Parallax's Rust
source-aware policy engine as the only default agent-output gate.

## A6 Synthetic Canary Fixture Corpus

_Provenance: merged verbatim from `a6-synthetic-canary-fixture-corpus.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

The A6 ledger and Phase 0 overlay contract require seeded canaries, but they do
not yet define the first fixture corpus. That leaves two risks:

- A1 tasks might pass without testing the exact surfaces the agent sees.
- Public fixtures might accidentally commit provider-shaped fake secrets that
  trigger hosted secret scanning or look too much like real credentials.

This note defines the minimum synthetic canary corpus for A1/A6:

> Commit fixture manifests, expected findings, redacted outputs, hashes, and
> generator recipes. Do not commit raw provider-shaped canary values unless they
> are reviewed for hosted-secret-scanning impact and explicitly marked safe.

### Source Basis

| Source | What it implies for fixtures |
| --- | --- |
| [GitHub secret scanning supported patterns](https://docs.github.com/en/code-security/secret-scanning/secret-scanning-patterns) | Provider and generic pattern sets change over time; use them as a canary coverage reference, but do not blindly commit token-shaped examples to a public repo. |
| [GitHub Actions masking](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#masking-a-value-in-a-log) | Masking must be registered before output and transformed secrets are not guaranteed to be masked, so CI fixtures need pre-mask, post-transform, and encoded variants. |
| [OpenTelemetry `AnyValue`](https://opentelemetry.io/docs/specs/otel/common/#anyvalue) | OTLP canaries must cover typed strings, bytes, arrays, and key/value lists before Markdown/string rendering. |
| [Sentry sensitive-data docs](https://docs.sentry.io/platforms/javascript/guides/nextjs/data-management/sensitive-data/) | Sentry-style fixtures should cover request headers, cookies, query strings, breadcrumbs, user context, stack locals, and HTTP spans. |
| [MCP tools specification `2025-11-25`](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) | Canaries must be absent from canonical `structuredContent`, not just text projections. |
| [MCP resources specification `2025-11-25`](https://modelcontextprotocol.io/specification/2025-11-25/server/resources) and [Claude Code MCP resources](https://code.claude.com/docs/en/mcp) | Canaries must also be absent from `resources/read` output and client resource attachments, because Claude Code can auto-fetch `@` resources into context. |
| [Codex config reference](https://developers.openai.com/codex/config-reference) | Fixture manifests should record Codex MCP elicitation and memory settings when Codex is part of the tested client matrix, because MCP-derived context may otherwise become retained client state. |
| [RFC 8785 JCS](https://www.rfc-editor.org/rfc/rfc8785.html) | Expected finding rows and projection hashes should use deterministic canonical JSON. |
| [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html) | The corpus should cover access tokens, passwords, connection strings, encryption keys, payment-like values, sensitive PII, and untrusted cross-zone data. |
| [Betterleaks recheck](redaction-toolchain-betterleaks-recheck.md) | External scanner fixtures should record network/validation/LLM mode; comparator scans must run offline by default. |

### Corpus Tiers

| Tier | Purpose | Commit policy |
| --- | --- | --- |
| `phase0_minimum_canary` | Required before any A1 task can count. Covers source-field leakage, Sentry-style event, OTLP typed value, CLI/CI text, canonical JSON, Markdown, CLI, HTTP, and MCP projection. | Commit manifests, redacted expected outputs, and hashes. Raw canary values may be committed only if they cannot trigger hosted provider alerts. |
| `surface_expansion_canary` | Adds frontend, agent session, deploy/change provider payload, database evidence, attachments/raw refs, and replay/screenshot metadata. | Commit redacted outputs and manifests; keep raw text/image/replay/database examples private or generator-only until reviewed. |
| `adversarial_encoding_canary` | Tests split-line, base64-like, URL-encoded, JSON-escaped, shell-quoted, YAML/TOML/env, Markdown table/code block, and transformed-secret cases. | Prefer generator recipes and hashes. Commit raw examples only after external scanner and hosted secret-scanning review. |
| `provider_pattern_private_canary` | Exercises realistic provider-token patterns from GitHub/Gitleaks/Betterleaks/TruffleHog references. | Do not commit raw values by default. Store in private/local red-team fixtures and commit only hashes, expected rule IDs, and redacted projections. |

### Required Phase 0 Fixture Set

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
| `mcp_resource_read_001` | MCP resource | Resource-read or resource-attachment leak | `resources/read` response, resource URI/path, Claude Code `@` attachment | Resource output derives from redacted canonical bundle; raw refs deny without sensitive scope. |
| `client_retention_001` | Agent client | Persisted output or memory leak | oversized MCP output file ref, resource attachment, Codex memory/external-context path | No raw canary persists; client settings and persisted artifact scanner status are recorded. |
| `pii_joinable_id_001` | Cross-surface | Low-entropy identifier | email, IP, user/session ID, path fragment | Keyed HMAC or strip; plain SHA/MD5 counts as leak. |

Phase 0 can use generated fixture values, but each value must have a stable
fixture ID and a `raw_value_hash`. If a value is provider-shaped, keep it out of
git unless the operator explicitly approves committing it.

### Frontend Surface Expansion Fixtures

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

### Public Fixture Layout

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
      mcp-resource-read.json
      client-retention.json
    scanner-comparison.jsonl
    projection-audit.jsonl
    private-input.sha256
```

`private-input.sha256` records the hash of the unredacted generated input. It is
not the input itself. Private raw inputs can live in a gitignored local fixture
directory or an operator-controlled artifact store.

### Fixture Manifest Row

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

### Commit Policy

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

### A1 Task Requirement

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

### Pass/Fail Rules

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

### Relationship To Other Research (Canary Fixture Corpus)

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

### Sources (Canary Fixture Corpus)

- [GitHub secret scanning supported patterns](https://docs.github.com/en/code-security/secret-scanning/secret-scanning-patterns)
- [GitHub Actions masking workflow command](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#masking-a-value-in-a-log)
- [OpenTelemetry common AnyValue](https://opentelemetry.io/docs/specs/otel/common/#anyvalue)
- [Sentry sensitive data docs](https://docs.sentry.io/platforms/javascript/guides/nextjs/data-management/sensitive-data/)
- [MCP tools specification `2025-11-25`](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)
- [RFC 8785 JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785.html)
- [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html)
- [Redaction toolchain Betterleaks recheck](redaction-toolchain-betterleaks-recheck.md)
- [Frontend Replay and source-map privacy recheck](frontend-replay-sourcemap-privacy-recheck.md)

### Bottom Line (Canary Fixture Corpus)

The first canary corpus should prove the product boundary without creating new
public secret-scanning noise. Keep raw provider-shaped values private or
generator-only, commit redacted outputs and hashes, and require each A1 counted
task to show both safety and preserved debugging context.

## A6 Redaction Red-Team Ledger

_Provenance: merged verbatim from `a6-redaction-red-team-ledger.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

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
- MCP resources, resource templates, and client resource-attachment paths did
  not bypass the tool-output scanner;
- client retention paths such as persisted oversized outputs, attachments, or
  Codex memories did not keep sensitive MCP evidence after the turn;
- source-field policies kept runner-private, grader-private, and default
  triage-private fields out of agent-visible projections;
- external scanners did not find unredacted secrets in generated outputs;
- redaction did not erase the minimum evidence needed for bundle usefulness.

The companion [A6 synthetic canary fixture corpus](a6-synthetic-canary-fixture-corpus.md)
defines the minimum Phase 0 fixture set and the public/private boundary for raw
provider-shaped canary values.

### Current Primary-Source Checks (Red-Team Ledger)

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
  Follow-up release API checks on 2026-05-25 showed Collector core source at
  `v0.153.0`, collector-releases stable binaries/images at `v0.152.1`, and
  collector-contrib at `v0.152.0`, so redaction fixtures must pin the processor
  source and tested distribution separately.
- OpenTelemetry common values can contain typed scalars, bytes, arrays, and
  key/value lists; Parallax must inspect the typed value tree before string or
  Markdown rendering
  ([OpenTelemetry common `AnyValue`](https://opentelemetry.io/docs/specs/otel/common/#anyvalue)).
- MCP `2025-11-25` tool results can return JSON `structuredContent` alongside
  text content, and an advertised `outputSchema` makes server conformance and
  client validation part of the contract. A6 must therefore test the canonical
  structured output, not only the human-readable text projection
  ([MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)).
- MCP resources expose context through `resources/list`, `resources/read`,
  resource templates, annotations, and text/binary content; host applications
  decide how resources enter model context, and the spec calls for access
  controls on sensitive resources. A6 must therefore test resource reads and
  resource templates as projection paths
  ([MCP resources specification](https://modelcontextprotocol.io/specification/2025-11-25/server/resources)).
- Claude Code's current MCP docs say resource `@` mentions auto-fetch resources
  as attachments, resources are fuzzy-searchable, and resource contents can be
  text, JSON, structured data, or other content types. That makes client-side
  resource attachment a redaction target, not just a UX detail
  ([Claude Code MCP docs](https://code.claude.com/docs/en/mcp)).
- Codex's current config reference exposes granular MCP elicitation approval and
  memory controls for whether threads with external context such as MCP, web, or
  tool search enter memory generation. A6 runs that claim Codex client safety
  must record those settings
  ([Codex config reference](https://developers.openai.com/codex/config-reference)).
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
- Betterleaks `v1.3.1` is the latest release checked on 2026-05-25, published
  2026-05-22, and its public repository is MIT-licensed and actively pushed.
  It supports git, dir, GitHub, S3, and stdin sources plus JSON/SARIF reports,
  but its CEL validation can make network or LLM calls. Use it as an
  experimental active comparator with validation disabled by default
  ([Betterleaks v1.3.1](https://github.com/betterleaks/betterleaks/releases/tag/v1.3.1)).
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

### Artifact Set

Use this public, redacted layout once A6 runs begin:

```text
docs/research/redaction-red-team-results.md
docs/research/redaction-red-team-runs/<run_id>/manifest.json
docs/research/redaction-red-team-runs/<run_id>/surface-fixture-ledger.jsonl
docs/research/redaction-red-team-runs/<run_id>/scanner-comparison.jsonl
docs/research/redaction-red-team-runs/<run_id>/projection-audit.jsonl
docs/research/redaction-red-team-runs/<run_id>/resource-read-audit.jsonl
docs/research/redaction-red-team-runs/<run_id>/client-retention-audit.jsonl
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

### Run Manifest

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
  "projections": ["bundle_json", "bundle_markdown", "cli_output", "http_api", "mcp_tool_result", "mcp_resource_read", "client_retention"],
  "client_retention_policy": {
    "codex_features_memories": false,
    "codex_memories_generate_memories": false,
    "codex_memories_use_memories": false,
    "codex_memories_disable_on_external_context": true,
    "claude_resource_attachment_checked": true,
    "oversized_output_persisted_artifacts_checked": true
  },
  "runtime_detector_version": "parallax-redact-rust-v0",
  "external_scanners": {
    "gitleaks": "8.30.1",
    "betterleaks": "1.3.1",
    "trufflehog": "3.95.3",
    "detect_secrets": "1.5.0",
    "presidio": "2.2.362",
    "otel_collector_contrib": "0.152.0",
    "otel_collector_distribution": "0.152.1",
    "github_pattern_snapshot": "2026-05-25"
  },
  "scanner_release_metadata": {
    "gitleaks": {
      "published_at": "2026-03-21T02:17:58Z",
      "development_posture": "feature_complete_security_patches_only",
      "replacement_watch": "betterleaks_experimental_active"
    },
    "betterleaks": {
      "published_at": "2026-05-22T16:18:18Z",
      "license": "MIT",
      "development_posture": "active_experimental_comparator",
      "verification_policy": "network_and_llm_validation_disabled_by_default"
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
    },
    "otel_collector_redaction": {
      "collector_core_latest": "0.153.0",
      "collector_core_published_at": "2026-05-25T08:09:17Z",
      "collector_releases_latest": "0.152.1",
      "collector_releases_published_at": "2026-05-20T15:47:17Z",
      "collector_contrib_latest": "0.152.0",
      "collector_contrib_published_at": "2026-05-11T13:38:04Z",
      "role": "upstream_otlp_minimizer_comparator_not_final_bundle_boundary"
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

### Surface Fixture Row

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
  "projection_targets": ["bundle_json", "bundle_markdown", "cli_output", "http_api", "mcp_tool_result", "mcp_resource_read", "client_retention"],
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
  "mcp_resource_read_leak_count": 0,
  "client_retention_leak_count": 0,
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

### Scanner Comparison Row

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
  "network_calls_allowed": false,
  "secret_validation_allowed": false,
  "llm_validation_allowed": false,
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

### Projection Audit Row

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
result, MCP resource read, client-retained artifact, and any model prompt
wrapper.

### Resource Read Audit Row

Resource rows catch leaks outside `tools/call`:

```json
{
  "schema_version": "a6-resource-read-audit-v1",
  "run_id": "a6-2026-05-25-phase1-canary",
  "resource_uri": "parallax://bundles/fixture_bundle_001",
  "client": "claude-code",
  "client_auto_fetch_path": "@-mention|autocomplete|direct_resources_read|none",
  "projection_derives_from_canonical": true,
  "canonical_bundle_hash": "sha256:...",
  "resource_output_hash": "sha256:...",
  "redaction_report_present": true,
  "source_field_policy_hash": "sha256:...",
  "scope_required": "evidence:read",
  "raw_ref_denied_without_sensitive_scope": true,
  "canary_leaks": 0,
  "raw_refs_expanded": 0,
  "audit_row_emitted": true,
  "verdict": "pass"
}
```

### Client Retention Audit Row

Client rows catch leaks after the immediate tool/resource response:

```json
{
  "schema_version": "a6-client-retention-audit-v1",
  "run_id": "a6-2026-05-25-phase1-canary",
  "client": "codex",
  "client_version": "0.133.0",
  "surface": "mcp",
  "features_memories": false,
  "memories_generate_memories": false,
  "memories_use_memories": false,
  "memories_disable_on_external_context": true,
  "tool_result_persisted_to_disk": false,
  "resource_attachment_persisted": false,
  "persisted_artifact_hash": null,
  "persisted_artifact_scanner_status": "not_applicable|pass|fail",
  "canary_leaks": 0,
  "raw_ref_material_persisted": false,
  "verdict": "pass"
}
```

### Source Field Policy Audit Row

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

### Usefulness Audit Row

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

### Repair Row

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

### Counting Rules

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
- MCP resource output counts only when `resources/list`, `resources/read`,
  resource templates, and client attachment paths preserve redaction,
  source-field policy, canonical hashes, output bounds, raw-ref denial, and
  audit rows.
- Client-retention output counts only when persisted files, attachments,
  memories, and oversized-output substitutions are either absent or contain only
  redacted bounded artifacts with scanner status recorded.
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
- Betterleaks validation, including HTTP, AWS, or LLM-assisted validation, must
  be disabled by default. If enabled, it requires an approved private run and a
  manifest entry naming every allowed endpoint/provider.
- A redaction report is complete only if it records policy version, rule IDs,
  action counts, detector versions, residual risk, and manual-review status.
- A pass requires both safety and usefulness: zero canary leaks and
  `minimum_debug_context_preserved = true` for required fixture classes.

### Required Fixture Classes

Cover each supported surface with at least these canary classes:

| Surface | Required canaries |
| --- | --- |
| Sentry event | auth headers, cookies, query tokens, user email, stack local value, breadcrumb URL, request/response headers, referrer URL. |
| OTLP span/log | custom token attribute, SQL text with password, URL query string, user/session ID, log body secret, typed `AnyValue` map/list/bytes secrets, resource/scope attribute secret. |
| CI log/artifact | env dump, transformed secret, private key block, database URL, test snapshot with PII. |
| CLI invocation | argv secret, env secret, cwd user path, stdout/stderr token, child-process command line. |
| Agent session | prompt secret, tool input secret, shell output secret, MCP response secret, generated Markdown leak. |
| MCP resource/client retention | resource URI/path canary, resource read body canary, attachment canary, persisted-output canary, memory-retention canary, elicitation-prompt canary. |
| Frontend | form-like value, DOM text, full request URL/query, referrer URL, request/response header, console log, baggage value, network body, replay/screenshot metadata, source-map/source-content ref. |
| Deploy/change provider payload | deployment status payload, deployment review comment, deploy log URL/body, release note, PR body, issue/comment text, environment URL, webhook delivery metadata. |
| Database evidence | row value PII, credential-like config row, SQL query text, query parameter, plan text, SQL error with connection string, export-like result. |
| A1/eval source fields | gold patch, test patch, fail/pass verifier IDs, generated hints, parser source, resolving commit URL, LLM metadata. |

Each class should appear in multiple encodings: plain text, JSON escaped,
URL-encoded, shell quoted, multiline, base64-like where realistic, Markdown
table, Markdown code block, YAML/TOML/env formats, and split-line log output.

### Claim Levels

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

### Freshness And Rerun Triggers

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

### Relationship To Other Research (Red-Team Ledger)

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

### Bottom Line (Red-Team Ledger)

A6 can pass only with reproducible red-team artifacts. The required claim is not
"we scrub secrets"; it is "for these canonical bundles, surfaces, and
projections, seeded canaries did not leak, forbidden source fields stayed out,
detector failures failed closed, raw refs stayed refs, MCP structured output
validated against the bundle schema, usefulness was preserved, and the claim
expires when the capture, source-field, schema, projection, or redaction surface
changes."
