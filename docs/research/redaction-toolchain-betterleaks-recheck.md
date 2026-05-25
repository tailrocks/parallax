# Redaction Toolchain Betterleaks Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The redaction detector note said to watch Betterleaks only after its license,
CLI, and output contracts were checked. That claim was too stale for an A6
toolchain decision because Gitleaks now states that it is feature-complete while
Betterleaks has active releases.

This pass rechecks that boundary:

> Betterleaks should move from "unvetted future candidate" to "tracked offline
> comparator candidate." It still must not become Parallax's runtime redaction
> boundary, and any network, credential-verification, or LLM validation features
> must be disabled by default for A6 fixture runs.

## Current Primary-Source Snapshot

| Source | Current facts | Parallax impact |
| --- | --- | --- |
| [Gitleaks `v8.30.1`](https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1) and [README](https://github.com/gitleaks/gitleaks) | Latest GitHub release remains `v8.30.1`, published `2026-03-21T02:17:58Z`. The README states Gitleaks is feature-complete and future releases are security patches only while focus shifts to Betterleaks. It still supports git, dir, stdin, JSON/CSV/JUnit/SARIF reports, redacted output, archive scanning, decode depth, and baselines. | Keep as the stable static comparator for now, but do not rely on it as the best current provider-pattern source. |
| [Betterleaks repo](https://github.com/betterleaks/betterleaks), [release `v1.3.1`](https://github.com/betterleaks/betterleaks/releases/tag/v1.3.1), and [scanning docs](https://github.com/betterleaks/betterleaks/blob/main/docs/scanning.md) | Repository is public, MIT-licensed, pushed on `2026-05-25T13:15:48Z`, and has latest release `v1.3.1` published `2026-05-22T16:18:18Z`. The scanning docs cover `dir`, `git`, `github`, `s3`, and `stdin`; reports include JSON and SARIF; releases ship checksums and Sigstore metadata. | Add to A6 as an experimental offline comparator candidate for generated fixtures. Require pinned binary/checksum and report-format snapshot before it can replace Gitleaks in a gate. |
| [Betterleaks config docs](https://github.com/betterleaks/betterleaks/blob/main/docs/config.md) | Betterleaks uses CEL filters and validation expressions. Validation can make HTTP requests, AWS validation calls, and even LLM-based validation examples that call OpenAI with obfuscated candidate/context. | Validation is powerful but unsafe as a default A6 comparator mode. A6 must record `network_calls_allowed=false` and `llm_validation_allowed=false` unless a private, approved red-team run explicitly opts in. |
| [TruffleHog `v3.95.3`](https://github.com/trufflesecurity/trufflehog/releases/tag/v3.95.3) | Latest release remains `v3.95.3`, published `2026-05-11T18:38:34Z`; repository license API reports AGPL-3.0. | Still useful for private verified-secret checks, not runtime. |
| [detect-secrets `v1.5.0`](https://github.com/Yelp/detect-secrets/releases/tag/v1.5.0) | Latest release remains `v1.5.0`, published `2024-05-06T18:05:06Z`; docs emphasize baseline/audit/plugin workflows. | Keep as legacy-baseline/new-secret comparator, not provider-churn source. |
| [Presidio `2.2.362`](https://github.com/microsoft/presidio/releases/tag/2.2.362), [PyPI analyzer](https://pypi.org/project/presidio-analyzer/), and [PyPI anonymizer](https://pypi.org/project/presidio-anonymizer/) | Latest release remains `2.2.362`, published `2026-03-18T05:32:57Z`; PyPI analyzer/anonymizer latest versions are `2.2.362`. README warns automated detection cannot guarantee all sensitive information is found. | Keep as PII reference and optional offline comparator; default-deny source policy still carries the safety guarantee. |
| [OpenTelemetry Collector redaction processor](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/processor/redactionprocessor/README.md) and [contrib `v0.152.0`](https://github.com/open-telemetry/opentelemetry-collector-contrib/releases/tag/v0.152.0) | Current README still lists alpha logs/metrics and beta traces. It supports fail-closed `allowed_keys`, `blocked_values`, `allowed_values`, HMAC hashes, audit attributes, URL sanitization, and database-query sanitization; `allowed_values` takes precedence over `blocked_values`. Latest contrib release remains `v0.152.0`. | Good upstream minimizer and policy model, but still not the final bundle-output boundary. |

## Decision Update

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

## Betterleaks Guardrails

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

## Impact On A6

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

## Falsification Triggers

Revisit this decision when:

- Betterleaks changes license, report schema, default validation behavior, or
  release signing/checksum contract;
- Gitleaks receives new feature releases rather than security patches only;
- Betterleaks red-team output proves less stable or less useful than Gitleaks on
  Parallax synthetic fixtures;
- Betterleaks validation or LLM-based examples become enabled by default;
- A6 cannot run Betterleaks in an offline, no-network, no-LLM mode.

## Sources

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

## Bottom Line

Betterleaks is now relevant enough to track in A6, but it strengthens the
offline comparator story, not the runtime safety boundary. The safest near-term
position is Gitleaks for stable fixture comparison, Betterleaks as an active
experimental comparator with validation disabled, and Parallax's Rust
source-aware policy engine as the only default agent-output gate.
