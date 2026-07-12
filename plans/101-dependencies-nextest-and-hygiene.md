# Plan 101: Enforce dependency policy and produce trustworthy test evidence

> **Executor instructions**: Resolve current stable tool versions at execution
> time, install them through mise, and add one enforced layer at a time. A tool
> invocation, empty report, or text grep is not evidence that its policy works.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 094, 095
- **Category**: dependencies / tests / supply chain / CI
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: TODO

## Why

Parallax has strict ecosystem rules but no executable dependency policy,
repository nextest profiles, structured flaky/slow evidence, or required
unused-dependency and feature-matrix checks. At planning time a local
`cargo audit --no-fetch` reported `crossbeam-epoch 0.9.18`
(`RUSTSEC-2026-0204`, fixed in 0.9.20) and warned on `anyhow 1.0.102`
(`RUSTSEC-2026-0190`, fixed in 1.0.103). These are observations, not version
pins; re-resolve the live graph before changing anything.

## Scope

In scope:

- Required cargo-audit, cargo-deny, cargo-shear, and supported feature checks.
- A real parser/dispatch-tested `cargo xtask dependencies` partition with
  equivalent human and machine-readable results.
- Apache-2.0-compatible license/source policy and native-TLS feature policy.
- Local, CI, and real-engine nextest profiles with structured evidence.
- Doctests, native macOS smoke, cache telemetry, and staged gitleaks/zizmor.
- A scheduled dependency-discovery report that never creates branches.

Out of scope:

- Automatic dependency-update branches or pull requests.
- A fabricated MSRV lower than latest stable.
- Cargo-vet, cargo-auditable, Miri, mutants, Dylint, Hakari, chaos, or
  self-hosted runners without a separate measured decision.
- Replacing GreptimeDB, Turso, native TLS, or Bun to satisfy a generic policy.

## Steps

### Step 1: Clear the live advisory baseline

Run a fresh advisory scan and dependency tree. Upgrade to the latest mutually
compatible stable versions and verify behavior. An exception is allowed only
with reachability analysis, upstream reference, owner, expiry, and a scheduled
recheck; an expired or structurally invalid exception fails closed.

### Step 2: Encode dependency policy

Add mise-pinned tools and a reasoned `deny.toml` covering advisories, allowed
licenses, registries/git sources, wildcards, duplicates, and banned feature
paths. The release-target graph must allow operating-system native TLS and the
cross-compiled `native-tls-vendored` OpenSSL path while rejecting every active
rustls backend. Add positive/negative policy fixtures so required OpenSSL is
not accidentally banned and rustls cannot pass through a renamed feature.

Run cargo-shear with reviewed exceptions. Run cargo-hack only across supported
feature contracts. Exclude `embed-ui` from generic clean-checkout powersets and
test it through an xtask partition that runs the Bun build first. Add an MSRV
lane only if the project explicitly promises one.

Implement `cargo xtask dependencies` in the same change as these gates. It
orchestrates the pinned audit/deny/shear/supported-feature commands, aggregates
all failures, emits the common human/JSON/GitHub diagnostic schema, and fails
on an empty or skipped partition. Parser, dispatch, command-inventory, and
intentional-failure fixtures are required; a placeholder success is forbidden.

### Step 3: Make nextest evidence structural

Create `.config/nextest.toml` with local, CI, and real-engine profiles,
explicit no-test behavior, slow/global timeouts, bounded CI retries, engine
serialization, JUnit output, and a shrink-only quarantine ledger. Parse JUnit
as XML or use nextest-native metadata. Fixture tests must prove first-pass
fail/retry/pass, persistent fail, slow, timeout, zero-test, and malformed
report behavior. Never copy PR #759's ineffective `flaky=\"true\"` grep.

Upload JUnit plus slowest-test summaries as durable CI artifacts. A required
job fails on a real unapproved flake/quarantine, not on display text.

### Step 4: Add platform and hygiene signal

Add scheduled advisory refresh, doctests, and a native macOS build/test/CLI
smoke. Preserve the real-GreptimeDB workflow. Introduce gitleaks and zizmor in
advisory mode, classify/baseline real findings, then require deterministic
clean scans after the baseline is reviewed. Until plan 108 is resolved,
gitleaks is restricted to the checked-out current tree: it may not scan Git
history, refs, unreachable objects, or emit matched secret values. Store only
redacted path/rule/fingerprint summaries. Any history scan, exposure decision,
rotation, or rewrite remains exclusively owned by plan 108. Each new lane needs
an owner, runtime expectation, and failure disposition before becoming required.

### Step 5: Measure caches and dependency discovery

Upload Cargo timings and sccache statistics. Measure cold and immediate-warm
runs before extracting cache actions or changing backends; report cache use
and clean closed-PR entries. Add scheduled latest-stable dependency discovery
as a report/artifact only. The workflow must not create a branch or PR.

## Test Plan

- cargo-audit/deny/shear/hack positive and deliberately failing fixtures.
- Native-TLS/rustls feature-tree policy fixtures.
- Nextest-generated JUnit flake/slow/timeout/zero-test fixtures.
- Clean-checkout generic and embed-ui partitions.
- Native macOS CLI smoke and staged current-tree-only gitleaks/zizmor scans.
- Cold/warm cache evidence with sccache statistics.

## Done Criteria

- [ ] The live graph has no unhandled vulnerability or soundness advisory.
- [ ] License/source/duplicate/wildcard/feature policy is required and tested.
- [ ] Active rustls fails while required native-TLS/OpenSSL paths pass.
- [ ] cargo-shear and supported feature checks have reasoned, expiring exceptions only.
- [ ] `cargo xtask dependencies` is fully implemented, fixture-tested, and used by CI.
- [ ] Nextest profiles emit structurally validated JUnit/slow/flaky evidence.
- [ ] No empty test selection or hollow flake grep can pass a required job.
- [ ] Doctest, native macOS, advisory refresh, and reviewed hygiene gates pass.
- [ ] Cache changes are justified by recorded cold/warm evidence.
- [ ] Dependency discovery reports without creating branches or PRs.

## STOP Conditions

- Policy would require rustls, reject required native OpenSSL, or weaken the
  GreptimeDB/Turso/Bun constraints.
- A live advisory cannot be upgraded and lacks defensible reachability,
  upstream, owner, and expiry evidence.
- Feature testing assumes unsupported combinations or requires `ui/dist` in a
  generic clean checkout.
- Flake detection is a grep or is not tested against output from the pinned
  nextest version.
- A new hygiene lane is made required before its findings and stability are
  reviewed.
- Any workflow creates an update branch or pull request.

## Remove When

Delete this plan and index row when dependency policy, structured nextest
evidence, native smoke, and staged hygiene are enforced and green.
