# Plan 111: Build the production redaction pipeline and prove A6

> **Executor instructions**: Treat every agent-visible byte as hostile until a
> source-aware, default-deny pipeline and a committed red-team run prove it.
> Keep external scanners offline validation tools, never runtime dependencies.
> Do not commit live or provider-shaped secret values.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: CRITICAL
- **Depends on**: 099, 101, 104
- **Category**: security / redaction / agent evidence
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED — Plan 104 canonical evidence-contract approval is absent

## Why

Product code still labels its regex-based policy `redaction-lite-v3`.
Research defines a source-aware, typed, fail-closed Rust pipeline and an A6
red-team ledger, but explicitly records that no qualifying run exists and the
claim remains `not_measured`. Projection-only fixes also leave raw issue titles
in Turso metadata, so secrets can persist at rest even when bundles mask them.

## Scope

In scope:

- A versioned source-field policy, runtime detector policy, redaction report,
  and final projection scan tied to plan 104's canonical bundle version.
- Typed OTLP/JSON traversal before string rendering, source minimization,
  secret/credential/PII/path rules, and fail-closed detector behavior.
- Stored issue-title/culprit safety and a migration/compatibility decision for
  existing Turso metadata.
- A public-safe generated canary corpus and offline comparator runs using the
  pinned tools from plan 101.
- Canonical JSON, Markdown, CLI, HTTP/GraphQL bundle, resource, and MCP-spike
  projection equivalence/redaction evidence.

Out of scope:

- Shelling out to Gitleaks/Betterleaks/TruffleHog/Presidio at runtime.
- Network credential validation by default or committing realistic secrets.
- Claiming all PII/secrets are detectable or exposing raw refs to agents.
- Shipping product MCP, owned by plan 112.

## Steps

### Step 1: Freeze the safety contract

After plan 104 resolves the bundle model, define source-field allow/drop/hash
policy, typed traversal, detector rule IDs/versions, failure behavior,
`redaction_report`, residual/manual-review states, raw-ref isolation, canonical
post-redaction hashing, and minimum evidence-usefulness requirements. Unknown
sources/fields default to exclusion, not pass-through.

### Step 2: Characterize every current leak surface

Inventory ingest normalization, issue title/culprit creation, Turso rows,
bundle assembly, logs, commands, attributes, hypotheses, JSON/Markdown/CLI,
GraphQL, and MCP spike metadata. Add canaries for transformed/encoded secrets,
typed arrays/maps/bytes, URLs, database strings, auth headers, paths, prompt/tool
content, frontend payloads, and source-field violations before changing code.

### Step 3: Implement a small Rust policy engine

Traverse typed values before formatting. Apply source minimization, maintained
key/value detectors, deterministic masking/HMAC where approved, then a final
scan over the canonical bundle and each renderer. A detector error strips the
affected field or blocks output with a typed safe error. Record rule/action
counts without storing matched secret material.

### Step 4: Protect metadata at rest

Decide and document whether issue titles/culprits are sanitized before Turso
storage, stored as safe structural summaries, or separated into protected raw
refs. Implement the approved model and an idempotent existing-row migration or
explicit expiry path. Preserve fingerprint/grouping identity and debugging
usefulness; never double-redact stable placeholders.

### Step 5: Build the public-safe corpus

Commit generator recipes, manifests, hashes, expected rule IDs, and redacted
outputs. Keep provider-shaped inputs private or generator-only. Run Gitleaks
and other approved comparators offline with network/validation/LLM features
disabled, pin versions/configuration, and fail on residual findings unless a
reviewed, narrow false-positive record exists.

### Step 6: Execute and record A6

Run every supported source through canonical JSON and all projections. Prove
zero seeded-canary leaks, fail-closed detector faults, raw-ref denial,
source-field isolation, deterministic hashes/reports, and retained minimum
diagnostic utility. Store the schema-versioned run ledger/artifacts under the
research validation layout without raw secrets.

## Test Plan

- Typed-tree/source-policy/detector/failure unit and property tests.
- Metadata migration, restart, grouping/fingerprint, and double-redaction tests.
- Cross-projection canonical hash and no-secret fixtures.
- Adversarial encoding, split-token, URL/shell/JSON/Markdown, maximal-size, and
  detector crash/timeout cases.
- Offline multi-scanner comparison and hosted-secret-scan-safe corpus review.
- Full A6 ledger validation and usefulness review.

## Done Criteria

- [ ] One versioned default-deny source/detector/report contract is canonical.
- [ ] Typed values are minimized/redacted before any string projection.
- [ ] Detector failure strips or blocks; it never passes the field through.
- [ ] Turso issue metadata no longer persists unsafe raw title/culprit values.
- [ ] Public fixtures contain no live/provider-shaped secret material.
- [ ] Every agent-visible projection is derived from the same redacted bundle.
- [ ] A committed A6 run proves zero seeded leaks and records residual risk.
- [ ] Debugging-usefulness and deterministic hash/report gates pass.

## STOP Conditions

- Plan 104's canonical bundle/redaction report is unresolved.
- Runtime safety requires a network service, foreign runtime, or external
  scanner process.
- A migration would destroy grouping identity or existing metadata without a
  reviewed compatibility path.
- Tests need real credentials/customer data or emit matched values to logs.
- Any agent projection bypasses the canonical redacted representation.

## Remove When

Delete this plan and index row when the production pipeline, metadata-at-rest
model, public-safe corpus, and source-backed A6 red-team run are enforced and
green.
