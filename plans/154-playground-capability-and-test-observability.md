# Plan 154: Validate playground observability on the live fan-out

> **Executor instructions**: Cross-repository implementation is complete on
> `codex/active-plan-closure-7f3c`. Do not redesign or duplicate W1–W5. Run the
> remaining acceptance sweep only on a host that can start the full Docker
> topology and reach all five configured backends.

## Status

- **Priority**: P1
- **Effort**: M remaining
- **Risk**: MEDIUM
- **Depends on**: a Docker-capable host, backend credentials/configuration,
  and the linked playground branch
- **Category**: cross-repository playground / live validation
- **Planned at**: `cb7c514`, revised 2026-07-15
- **Status**: BLOCKED — this arm64 host has no Docker; collector-backed
  Parallax/Maple/SigNoz/OpenObserve/Sentry evidence cannot be produced locally

## Completed Contract

The companion repository implements the complete source program:

- real W3C baggage, Rust HTTP semantic conventions, Java Sentry SDK wiring,
  flag-driven chaos, messaging conventions and explicit Kafka causal links;
- a Juniper Rust storefront with GraphQL HTTP/subscriptions and Java GraphQL /
  gRPC gateway paths, Java streaming gRPC, and a Java gRPC client hop;
- tests for all ten app services, CLI, and web across nextest, JUnit 5,
  Vitest, and Playwright, including deterministic assertion/harness retries;
- generated cross-language test conventions, run-session parenting, Rust
  JUnit reconciliation, Java JUnit telemetry, a Bun-compatible Playwright
  reporter, browser traceparent injection, and fail-closed GraphQL acceptance;
- real catalog Postgres, exponential-histogram and cross-language error probes,
  and exact dispatch coverage for all 45 scenarios;
- one pinned, least-privilege CI workflow aggregating scenario, Rust, Java,
  Bun, and Chromium lanes while retaining native test artifacts.

The durable implementation and operator commands live in the playground's
`README.md` and `VERIFICATION.md`. Plan 119 owns the completed shared registry;
Plan 155 owns the Parallax test-reporting product surface.

## Local Evidence

At companion commit `6488bf3`:

- the Rust nextest CI profile previously passed 57 tests across 11 binaries,
  and its JUnit report reconciled successfully;
- clean catalog, payment, and fulfillment Gradle suites passed with their OTel
  agents and generated JUnit extension;
- five normal Chromium journeys passed under the host's temporary user-owned
  runtime, and both W4 retry fixtures failed once then passed;
- the final workflow passes Actionlint; the scenario checker proves all 45
  IDs; all 15 CLI tests, nine Vitest tests, production build/typecheck, and
  seven-test Playwright discovery pass locally.

No remote workflow or live backend-rendering result is claimed.

## Remaining Acceptance

1. Start the full playground topology and all five fan-out backends on a
   Docker-capable host using the documented pinned configuration.
2. Run each Rust, Java, and web observable session through
   `parallax run start -- scripts/observable-test-session.sh <stack>
   --acceptance`, then run `playground test-verify` against the finished run.
3. Prove baggage and the storefront gateway chains remain single distributed
   traces, the Kafka producer/consumer causal link renders, and each scenario
   driver produces its claimed signal.
4. Inspect the failed Playwright and Rust attempts in Parallax and record the
   test telemetry, exponential-histogram, database-semconv, and
   cross-language `PaymentError` disposition for Maple, SigNoz, OpenObserve,
   Sentry, and Parallax in playground `VERIFICATION.md`.
5. Run the pushed playground workflow at the same branch head. Preserve its
   exact SHA and native test artifacts as final cross-platform evidence.
6. Reconcile Plan 122's disposition table, preserve durable validation evidence
   in root `docs/research/validation/`, then delete this plan and index row.

## STOP Conditions

- Do not replace the real fan-out with mocks or screenshots.
- Do not claim backend behavior from configuration or source inspection.
- Do not introduce a backend, broker, product fallback, or wire-contract drift
  to make the validation pass.

## Remove When

Delete this plan and index row when the collector-backed acceptance verifier,
five-backend matrix, scenario sweep, and exact-head playground workflow pass.
