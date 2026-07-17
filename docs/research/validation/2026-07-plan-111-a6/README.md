# Plan 111 — Redaction pipeline and A6 gate

**Status:** DONE (2026-07-17)
**Claim level:** measured (public-safe canaries; offline multi-scanner optional)

## Contract

| Layer | Version |
| --- | --- |
| Source-field policy | `evidence-source-v1` |
| Detector set | `detectors-v1` (owned Rust regex engine in `bundle::redaction`) |
| Bundle schema | immutable `bundle-v1` (`redaction-lite-v3` serialized label) / envelope `bundle-v2` (plan 104) |

Default-deny: unknown fields drop; hostile free-text fields run detectors;
structural identifiers are validated (detectors still applied so secrets cannot
smuggle in IDs); detector panic strips the field (`detector_failure`).

## Metadata at rest

Issue `title` / `culprit` are sanitized in `TursoMetadataStore::upsert_issue_occurrences`
before INSERT/UPDATE. Legacy rows: `sanitize_existing_issue_text()` rewrites
through the same sanitizer (idempotent; already-clean markers are stable).
Grouping identity (fingerprint) is unchanged.

## A6 canary ledger (public-safe only)

| Canary class | Seed shape (public) | Expected rule | Leak? |
| --- | --- | --- | --- |
| GitHub token | `ghp_` + public filler | `github_token` / `bearer_token` | no |
| Stripe live key | `sk_live_XXXXXXXX…` | `stripe_live_key` | no |
| DSN userinfo | `postgres://user:p@ssw0rd@…` | `dsn_userinfo` | no |
| Private key block | `BEGIN PRIVATE KEY` fixture | `private_key_block` | no |
| Basic auth | `Basic` + base64 filler | `basic_auth` | no |
| Generic assignment | `api_key=supersecretvalue` | `generic_secret_assignment` | no |
| Password assignment | `password=hunter2` | `password_assignment` | no |

No live provider credentials and no customer data appear in fixtures or this ledger.

## Projections checked

All derive from the same assembled `Bundle`. Runtime decisions use
`evidence-source-v1`; the immutable bundle-v1 report retains its shipped
`redaction-lite-v3` serialized label and canonical hash:

1. Canonical JSON (`serde_json` of `Bundle`)
2. Markdown (`bundle::to_markdown`)
3. GraphQL / CLI consume the same assemble path (plan 104)

## Verify

```sh
cargo nextest run --locked -p parallax-evidence -p parallax-metadata \
  -E 'test(/redact|a6_|sanitize|hostile|unknown_field|structural|issue_title_and_culprit|assembled_bundle|golden/)'
```

## Residual risk

- Regex detectors are not complete against every encoding split; residual risk
  is accepted for V1 with default-deny unknown fields and projection-only paths.
- Offline Gitleaks / Betterleaks / TruffleHog comparators remain optional
  operators' tools — not runtime dependencies.
- Retroactive purge of already-ingested raw Greptime signal bodies is out of
  A6 pre-exposure scope (see capture/redaction.md Tempo note).
