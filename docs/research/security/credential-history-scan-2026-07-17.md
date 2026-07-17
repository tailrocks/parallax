# Credential-history scan — Plan 108 evidence (2026-07-17)

Operator directive (unblock, 2026-07-17): **no history rewrite**; run the
credential-history scan, rotate anything found, record secret-free evidence.
This record contains **no secret values** — fingerprints are
`commit:path:rule:line` only.

## Scan scope and tooling

| Item | Value |
|---|---|
| Tool | gitleaks 8.30.1 (default ruleset) |
| History scan | `gitleaks git --redact` — 1,329 commits, ~17.09 MB scanned |
| Current-tree scan | `gitleaks dir --redact` at `main` `b680fd8` — ~537.95 MB, **no leaks found** |
| Date | 2026-07-17 |

## Findings (full history) and classification

Five raw findings; four are intentional test canaries or documentation text
(false positives), one is the Plan-108 subject.

| # | Fingerprint (commit:path:rule:line) | Classification |
|---|---|---|
| 1 | `a1d8bf8:REPOSITORY_PROTECTION.md:generic-api-key:26` | False positive — repository-protection policy prose, no token present. |
| 2 | `533113b:crates/parallax-core/src/bundle.rs:generic-api-key:1265` | Intentional redaction-test fixture (`ghp_…` canary in a test issue culprit). |
| 3 | `3f2ef7e:crates/parallax-server/tests/m2_bundle.rs:stripe-access-token:98` | Intentional canary `sk_live_XXXXXXXXCANARYKEY` planted to prove span-name redaction. |
| 4 | `3f2ef7e:crates/parallax-server/tests/m2_bundle.rs:stripe-access-token:227` | Same canary asserted **absent** from the projection (the redaction test itself). |
| 5 | `8dde008:bench/otlp-fanout/rotel.env:generic-api-key:44` | The Plan-108 subject — classified below. |

## Finding 5: the historical `rotel.env` Sentry values

`bench/otlp-fanout/rotel.env` was tracked from `9ecff8c` (2026-06-23 era
fan-out lab) until `78e84db` replaced it with the safe
`rotel.env.example`. The flagged line is `ROTEL_EXPORTER_SENTRY_CUSTOM_HEADERS`
carrying an `X-Sentry-Auth` header with a 32-hex `sentry_key` (a Sentry DSN
ingest key), paired with `ROTEL_EXPORTER_SENTRY_ENDPOINT` on port 9000.

Secret-safe classification evidence:

- The endpoint hostname uses a **reserved, non-routable lab TLD**
  (`.test`/`.local`/`.lan`/`.internal` class) — it does not resolve on the
  public internet; a probe from this host returned no connection
  (curl exit, HTTP 000).
- The instance is the **self-hosted lab Sentry** from the OTLP fan-out bench
  (`rotel.env.example` documents "Verified end-to-end on Sentry v26.6.0",
  self-hosted per Plan 154's no-external-credentials rule).
- A Sentry DSN key is an **ingest-only** credential: it can submit events to
  that project, nothing else. Combined with a non-routable host, the
  historical value grants no access to any external service and no read or
  admin capability anywhere.

**Disposition: no real exposure.** The value is a lab-internal, ingest-only
key for an unreachable self-hosted instance. Per the operator's no-rewrite
directive, history stays as is. Rotation is defense-in-depth only: if the lab
Sentry compose stack is ever brought back up, regenerate the project DSN
(Sentry → Project Settings → Client Keys) before reuse — no provider or
external rotation exists to perform.

## Conclusion

- Current tree at `b680fd8`: **clean** (no leaks found).
- Full history: no real external credential ever entered Git history; the
  single non-fixture finding is a lab-internal ingest key for a non-routable
  self-hosted instance.
- No history rewrite required or authorized; none performed.
- Remaining operator-optional step: regenerate the lab Sentry DSN if that
  compose stack is revived.
