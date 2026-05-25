# Redaction Detector Toolchain

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

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

## Current Primary-Source Checks

| Source | What it provides | Parallax implication |
| --- | --- | --- |
| [OpenTelemetry Collector redaction processor](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/processor/redactionprocessor/README.md) | Checked 2026-05-25. Current README lists stability as alpha for logs/metrics and beta for traces. It supports `allowed_keys`, `blocked_values`, `allowed_values`, masking, HMAC hash functions, redaction audit attributes, URL sanitization, and database-query sanitization. `allowed_values` takes precedence over `blocked_values`. | Good model for OTLP attribute policy and audit fields, but broad allowlists can intentionally bypass blocked-value masking. Not enough for bundle safety because it only covers data that flowed through that Collector processor. |
| [OpenTelemetry Collector processor catalog](https://opentelemetry.io/docs/collector/components/processor/) | Checked 2026-05-25. The catalog was last modified 2026-03-16 and links Redaction Processor at contrib `v0.152.0`, included in contrib/K8s with traces beta and metrics/logs alpha. | Parallax should support Collector-side redaction, but still repeat policy in ingest and bundle building. Treat upstream Collector redaction as a useful pre-filter, not an agent-safety boundary. |
| [OpenTelemetry common `AnyValue`](https://opentelemetry.io/docs/specs/otel/common/#anyvalue) | OTLP values can be scalars, byte arrays, arrays, and key/value lists. | Redaction must traverse typed values directly; string rendering of maps/lists is only a projection and cannot be the detector input. |
| [GitHub secret scanning supported patterns](https://docs.github.com/en/code-security/reference/secret-security/supported-secret-scanning-patterns) | Current pattern taxonomy includes generic, AI-detected, and provider patterns, with 500+ provider entries and notes on precision, validity checks, partner alerts, and token-version churn. | Use as a maintained external reference for canary coverage and provider-pattern watchlists. Do not assume Parallax can copy GitHub's proprietary AI/validity checks. |
| [Gitleaks v8.30.1](https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1) | Latest release checked 2026-05-25; GitHub API shows `v8.30.1` published 2026-03-21. Mature open-source scanner for git history, directories, files, and stdin; supports default/custom config, pre-commit, GitHub Action, JSON/CSV/JUnit/SARIF-style workflows, archive/decode scanning, redacted output, and baseline/ignore behavior. Its README now says Gitleaks is feature-complete and future releases are security patches only while focus shifts to Betterleaks. | Best stable comparator for Phase 0 fixture scanning, but no longer a strong signal for new provider-pattern coverage. Its Go binary can validate redaction fixtures in CI, but a blocking runtime shell-out would hurt tiny-tier latency and failure modes. |
| [Betterleaks v1.3.1](https://github.com/betterleaks/betterleaks/releases/tag/v1.3.1) | Latest release checked 2026-05-25; GitHub API shows `v1.3.1` published 2026-05-22, public MIT repo, current push activity, JSON/SARIF reports, `dir`/`git`/`github`/`s3`/`stdin` sources, checksums, Sigstore metadata, CEL filters, and live validation hooks. Config docs include HTTP, AWS, and LLM-assisted validation examples. | Move from unvetted future candidate to experimental active comparator. Useful for red-team fixture comparison once binary/checksum/report schema are pinned. Validation/network/LLM features must be disabled by default and never run in the Parallax runtime path. |
| [TruffleHog v3.95.3](https://github.com/trufflesecurity/trufflehog/releases/tag/v3.95.3) | Latest release checked 2026-05-25; GitHub API shows `v3.95.3` published 2026-05-11. AGPL-3.0 scanner focused on finding, verifying, and analyzing leaked credentials across git, GitHub, S3/GCS, Docker, Hugging Face, stdin, and multi-source configs. Verified findings are confirmed by testing against provider APIs, and JSON output is available. | Useful red-team comparator for live/verified secrets and historical/source scans. Verification and credential analysis can create network, privacy, and rate-limit side effects, so A6 must record whether verification was disabled or approved for a private fixture run and must not run it in the default runtime path. |
| [Yelp detect-secrets v1.5.0](https://github.com/Yelp/detect-secrets/releases/tag/v1.5.0) | Latest release checked 2026-05-25; GitHub API shows `v1.5.0` published 2024-05-06. Baseline-driven secret detection with configurable plugins, allowlists, entropy detectors, filters, verification settings, pre-commit hooks, and audit workflow. | Useful for repository baselines, "new secret" regression checks, and human review workflows. Its older release cadence and Python/plugin model make it a secondary comparator, not a provider-churn source and not a fit for the Parallax hot path. |
| [Microsoft Presidio 2.2.362](https://github.com/microsoft/presidio/releases/tag/2.2.362) | Latest release checked 2026-05-25; GitHub API shows `2.2.362` published 2026-03-18, and PyPI shows `presidio-analyzer`/`presidio-anonymizer` `2.2.362` uploaded 2026-03-15. Open-source PII detection/anonymization framework with NER, regex, rule logic, image redaction, custom recognizers, and an explicit warning that automated detection cannot guarantee all sensitive information is found. | Best reference for PII/anonymization and optional offline processing. Too heavy and probabilistic to be the only runtime guarantee for agent-visible bundles. |
| [IssueGuard paper](https://arxiv.org/abs/2602.08072) | Current research on real-time secret leak prevention in issue reports; targets unstructured collaborative text and separates real secrets from false positives. | Confirms Parallax's risk surface: secrets leak in issue-like/debug text, not only git repos. Good design reference for pre-submit warning UX, not yet a runtime dependency. |

## What Existing Tools Are Good At

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

## 2026-05-25 Freshness Findings

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

## Runtime Architecture

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

## Recommended Toolchain By Phase

| Phase | Runtime dependency | CI / red-team dependency | Rationale |
| --- | --- | --- | --- |
| Phase 0 bundle eval | Internal Rust sanitizer plus a small curated canary corpus. | Run Gitleaks and/or Betterleaks plus detect-secrets over generated fixtures; manually inspect Presidio-style PII cases. | Keep the eval lightweight, but prevent obvious secret leaks in arms B/C. Betterleaks validation stays disabled. |
| Phase 1 tiny tier | Internal Rust redaction library is mandatory. No Python/Go sidecar in the request path. | Gitleaks/Betterleaks on fixture dirs and generated bundles; TruffleHog on private local red-team corpus when network verification is safe. | Tiny tier cannot depend on multi-runtime scanner services to claim simplicity. |
| Phase 2 red-team | Internal library plus optional offline Presidio adapter for PII-heavy text/image fixtures. | Gitleaks, Betterleaks, TruffleHog, detect-secrets, Presidio, GitHub pattern watchlist, and custom canary corpus. | This is where A6 can earn broader source coverage and decide whether Betterleaks replaces Gitleaks as the primary static comparator. |
| Phase 3 enterprise/pilots | Same runtime library; optional policy pack imports from customer secret-scanning tools. | Customer scanners can validate exported bundle fixtures. | Enterprise integration should validate Parallax, not replace its default-deny core. |

## Detector Catalog

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

## Test Corpus

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

## Pass / Fail Gate

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

## Product Decision

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

## Relationship To Other Research

- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  defines the A6 policy and red-team gate this toolchain implements.
- [Redaction toolchain Betterleaks recheck](redaction-toolchain-betterleaks-recheck.md)
  moves Betterleaks into the tracked comparator set while keeping validation
  disabled by default.
- [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) defines the
  run artifacts that prove the toolchain does not leak seeded canaries and still
  preserves minimum debug usefulness.
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

## Bottom Line

Use external secret and PII scanners to keep Parallax honest, but do not put
them on the critical request path. The critical request path needs a Rust,
source-aware, default-deny redaction engine that fails closed and proves its work
through `redaction_report`, fixture snapshots, and cross-tool red-team checks.
