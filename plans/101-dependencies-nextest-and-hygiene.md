# Plan 101: Enforce Cargo/Bun dependency policy and trustworthy test evidence

> **Executor instructions**: Resolve current stable tool versions at execution
> time, install them through mise, and add one enforced layer at a time. A tool
> invocation, empty report, or text grep is not evidence that its policy works.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 094, 095, 096
- **Category**: dependencies / tests / supply chain / CI
- **Planned at**: `a1d8bf82`, revised 2026-07-12
- **Status**: TODO

## Why

Parallax has strict ecosystem rules but no executable Cargo or Bun dependency
policy, repository nextest profiles, structured flaky/slow evidence, or
required unused-dependency and feature-matrix checks. At planning time a local
`cargo audit --no-fetch` reported `crossbeam-epoch 0.9.18`
(`RUSTSEC-2026-0204`, fixed in 0.9.20) and warned on `anyhow 1.0.102`
(`RUSTSEC-2026-0190`, fixed in 1.0.103). These are observations, not version
pins; re-resolve the live graph before changing anything.

A 2026-07-12 read-only `mise exec -- bun audit --json` also exited nonzero with
current transitive `undici` advisories. `bun pm untrusted` reports a blocked
lifecycle script, and direct TanStack dependencies appear unused or only
partially wired. These are live observations, not instructions to trust a script
or delete a package without reachability and behavior evidence.

## Scope

In scope:

- Required cargo-audit, cargo-deny, cargo-shear, Bun audit/trust/integrity, and
  supported feature checks.
- A real parser/dispatch-tested `cargo xtask dependencies` partition with
  equivalent human and machine-readable results.
- Apache-2.0-compatible Rust/Bun license/source policy, native-TLS feature
  policy, lifecycle trust, and unused direct dependencies.
- Exact compatible Oxc Rust/native-Oxlint binary sets and platform packages,
  plus TypeScript 7/Oxlint type-aware compatibility evidence for plan 131.
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
paths. Default/host development, check, test, and native release graphs use
`native-tls` and must not activate vendored OpenSSL. A dedicated release-only
feature/config partition enables `native-tls-vendored` exclusively for plan
102's Zig cross targets; keep it out of generic all-features/cargo-hack
powersets. Every graph rejects active rustls. Add positive/negative metadata/
tree fixtures for host-default, native release, supported feature, cross-release
vendored, renamed rustls, and accidental vendored activation.

Run cargo-shear with reviewed exceptions. Run cargo-hack only across supported
feature contracts. Exclude `embed-ui` from generic clean-checkout powersets and
test it through an xtask partition that runs the Bun build first. Add an MSRV
lane only if the project explicitly promises one.

Clippy every supported feature contract rather than only default features.
`embed-ui` runs after a Bun build; `conformance` runs through its owned test/
engine preparation. A bidirectional inventory fixture fails when a feature is
declared without a check or a check names no feature. Enforce `publish = false`
for every internal workspace package.

Implement the Rust partition of
`cargo xtask dependencies --rust|--ui|--all` in the same change as these gates.
It orchestrates the pinned audit/deny/shear/supported-feature commands,
aggregates all failures, emits the common human/JSON/GitHub diagnostic schema,
and fails on an empty or skipped partition. Parser, dispatch, command-inventory,
and intentional-failure fixtures are required; a placeholder success is
forbidden.

### Step 3: Encode Bun dependency and lifecycle policy

Use Bun's lockfile and structured commands under mise. Set
`trustedDependencies: []` before reviewing any script so Bun's built-in trusted
package list is not active. Verify plan 094's `bunfig.toml` effective
`[run] bun = true` and `[install] auto = "disable"` contract. Require:

- `bun audit --json` with fresh advisory data and the same reason/owner/expiry/
  reachability standard as Rust exceptions;
- registry/source/integrity and Apache-2.0-compatible license evidence for the
  resolved production graph;
- explicit package-and-locked-version/integrity/script review for every trusted
  lifecycle script, with empty-list/default-list fixtures and no blanket trust;
- AST/resolver-backed unused direct dependency detection across source, Vite,
  Oxlint/Oxfmt, generated code, dynamic imports, and scripts; and
- every script/generator/codegen resolves an exact lock entry, with no
  `bunx ...@latest`, implicit install, or undeclared executable; and
- `bun outdated` as a scheduled report only, never a branch or automatic major
  upgrade.

Reconcile every lock entry to advisory/source/license/integrity coverage. Bun's
audit may omit non-default registries; any uncovered entry fails or receives a
separate equivalent scanner/evidence path.

Treat current Oxc adoption as reviewed compatibility units. The operator's
2026-07-12 Oxc-only lint/format direction authorizes exactly two narrow
pre-stable exceptions: the latest official `oxlint-tsgolint` release used with
the selected stable Oxlint, and the latest official Oxfmt Beta release. Record
those exceptions in durable repository policy and the executable dependency
predicate before either package becomes live. Each is exact-pinned, grouped with
its platform packages, manually reviewed, fixture-gated, and rechecked on every
upgrade; the exception expires when that component becomes stable. It does not
authorize Oxlint JS plugins, integrated type-check authority, React compiler
rules, or direct Oxc transformer/minifier packages.

Review one exact compatible
Rust `oxc_parser`/`oxc_ast`/`oxc_semantic` family plus resolver version, and the
exact stable native Oxlint package/platform set used by plan 131. Record upstream
status, integrity, license/source, native package and lifecycle behavior on
supported macOS/Linux architectures, and fail missing/unsupported bindings
without runtime download. TypeScript 7.0 is GA; inventory the exact TypeScript/
Oxlint/tsgolint package, platform, peer, API-consumer, and integrity
graph for plan 131 without changing the live compiler in this policy plan.
Explicitly identify direct `eslint` and `@tanstack/eslint-config` ownership plus
the transitive typescript-eslint `<6.1` peer/API path as cutover edges, not a
reason for a TypeScript 6 alias. Fixture `bun why` and lock-reachability output
so plan 131 must distinguish direct removal from transitive disappearance and
name any unrelated surviving owner. Plan 131 pins/group-upgrades the live
type-aware unit and removes those edges atomically. Record the exact TypeScript
Go revision embedded by tsgolint;
the current 0.24.0 package points to a 2026-06-25 pre-GA revision, so plan 131
requires a GA-or-newer revision or checked-in project/diagnostic parity proof.
Oxc package wrappers have Node shebangs, so verify exact lock-local invocations
force Bun with `--bun` and disable installation using synthetic fixture configs/
package metadata plus the stable native Oxlint executable only. Plan 131 owns
the live `.oxlintrc.jsonc`; plan 130 alone installs/configures live Oxfmt. This
policy plan records both narrow exceptions but does not require either live
config to retire.

Define and fixture the dependency-policy predicate plan 132 must apply to
Playwright. The final direct browser runner is exact `@playwright/test`; direct
`playwright`, direct `playwright-core`, Node runtime, and an additional browser
framework are forbidden. Require the transitive core/browser revision to match
the runner, Apache-2.0-compatible licensing, registry integrity/source review,
explicit browser provisioning after an ignore-scripts install, no implicit
install/lifecycle download, and macOS/Linux Bun-only process ancestry. Evaluate
`@types/bun` only when plan 132 proves it is needed, and exact stable
`@axe-core/playwright` only when plan 146 first uses it. Failure of plan 132's
exact matrix blocks browser adoption; Node is not a fallback.

Record latest compatible `@testing-library/user-event` for plan 129 and fixture
its Bun runtime, peer graph, license, integrity, and absence of unreviewed
lifecycle behavior. Do not add ESLint, `eslint-plugin-playwright`, or an alpha
Oxlint JavaScript plugin. Playwright-only invariants use stable config, runtime
self-tests, and plan 095's Rust/Oxc AST policy provider.

Reconcile the current TanStack suite as one mutually compatible set. Remove a
current devtools/bridge package only after resolver/build evidence proves it is
unused; do not keep it merely to imply a future Query architecture. Record the
compatible TypeScript 7/Query/Router/Start versions and prove the current
TanStack/Vite/Vitest/React graph has no TypeScript compiler peer/API dependency
apart from the ESLint path assigned to plan 131. Plan 133 adds every new Query
dependency in the same slice that first imports it. Add a report-only
`skipLibCheck=false` lane that records exact current third-party declaration
failures. Plan 128 consumes that inventory and owns the narrow compatible
upgrade/removal work before making the lane required.

Implement the UI partition of the same xtask command and prove `--all` runs both
partitions, aggregates failures, and cannot report green when Bun is missing,
the lockfile is absent, or the UI selection is empty.

### Step 4: Make nextest evidence structural

Create `.config/nextest.toml` with local, CI, and real-engine profiles,
explicit no-test behavior, slow/global timeouts, bounded CI retries, engine
serialization, JUnit output, and a shrink-only quarantine ledger. Parse JUnit
as XML or use nextest-native metadata. Fixture tests must prove first-pass
fail/retry/pass, persistent fail, slow, timeout, zero-test, and malformed
report behavior. Never use a raw `flaky=\"true\"` grep as the retry-pass gate;
it does not parse the report contract reliably.

Set CI `flaky-result = "fail"`: a retry-pass remains a failing required result
and retains structured JUnit evidence. Quarantined tests continue running in a
separately visible selection; every row has test expression, owner, reason,
expiry, failure link, and removal condition. Command-line retry overrides may
not erase per-test policy or convert quarantine into a skip.

Upload JUnit plus slowest-test summaries as durable CI artifacts. A required
job fails on a real unapproved flake/quarantine, not on display text.

### Step 5: Add platform and hygiene signal

Add scheduled advisory refresh, doctests, and a native macOS build/test/CLI
smoke. Preserve the real-GreptimeDB workflow. Introduce gitleaks and zizmor in
advisory mode, classify/baseline real findings, then require deterministic
clean scans after the baseline is reviewed. Until plan 108 is resolved,
gitleaks is restricted to the checked-out current tree: it may not scan Git
history, refs, unreachable objects, or emit matched secret values. Store only
redacted path/rule/fingerprint summaries. Any history scan, exposure decision,
rotation, or rewrite remains exclusively owned by plan 108. Each new lane needs
an owner, runtime expectation, and failure disposition before becoming required.

### Step 6: Measure caches and dependency discovery

Upload Cargo timings and sccache statistics. Measure cold and immediate-warm
runs before extracting cache actions or changing backends; report cache use
and clean closed-PR entries. Add scheduled latest-stable dependency discovery
as a report/artifact only. The workflow must not create a branch or PR.

## Test Plan

- cargo-audit/deny/shear/hack and supported-feature positive/deliberately
  failing fixtures.
- Bun audit, integrity/source/license, untrusted lifecycle, unused-direct, and
  outdated-report fixtures.
- Explicit-empty `trustedDependencies`, built-in-list suppression, reviewed
  locked-script, bunfig run/auto-install, Vite/Vitest/tsc/lint/format/codegen/
  shadcn process ancestry, alternate-registry audit coverage, and Oxc native-
  binding/no-runtime-download/no-Node fixtures.
- Host/default/native-release/cross-release native-TLS-vendored and rustls
  feature-tree policy fixtures.
- Compatible Oxc Rust family/native-Oxlint fixtures, synthetic grouped
  Oxlint/tsgolint/TypeScript policy fixtures and report-only maturity evidence,
  plus synthetic Playwright/Bun runner/core/browser-version, no-Node,
  no-lifecycle-download, user-event, and axe dependency-policy proof.
- Nextest-generated JUnit flake/slow/timeout/zero-test fixtures.
- Clean-checkout generic and embed-ui partitions.
- Native macOS CLI smoke and staged current-tree-only gitleaks/zizmor scans.
- Cold/warm cache evidence with sccache statistics.

## Incoming Handoff From 127

Plan 127 replaces this placeholder before retirement. The handoff is
schema-validated; stable IDs are never reused and no row may remain pending or
unowned when plan 101 begins nextest configuration.

| Stable ID | Resource/current selector | Proposed group | Timeout/owner | Status |
|-----------|---------------------------|----------------|---------------|--------|
| `127-nextest-greptime-roundtrip` | Cached GreptimeDB process; `parallax-server::m1_greptime::managed_engine_roundtrip` | `greptime-engine` | 10 minutes; Plan 101 | OWNED |
| `127-nextest-native-table-inventory` | Cached GreptimeDB process; `parallax-server::m1_table_inventory_greptime::only_extension_tables_are_custom` | `greptime-engine` | 10 minutes; Plan 101 | OWNED |
| `127-nextest-native-metrics` | Cached GreptimeDB process; `parallax-server::m2_metrics_greptime::managed_engine_metrics_roundtrip` | `greptime-engine` | 10 minutes; Plan 101 | OWNED |
| `127-nextest-storage-conformance` | Cached GreptimeDB process; `parallax-server::m6_conformance_greptime::greptime_conformance_scenarios` | `greptime-engine` | 10 minutes; Plan 101 | OWNED |
| `127-nextest-exemplar-migration` | Cached GreptimeDB process; `parallax-server::m7_metric_exemplar_migration_greptime::migrates_legacy_metric_exemplars_without_mutation` | `greptime-engine` | 10 minutes; Plan 101 | OWNED |
| `127-nextest-performance-gates` | Cached GreptimeDB process; `parallax-server::m5_gates::measure_m5_gates` | `greptime-engine` | 15 minutes; Plan 101 | OWNED |

## Done Criteria

- [ ] The live graph has no unhandled vulnerability or soundness advisory.
- [ ] License/source/duplicate/wildcard/feature policy is required and tested.
- [ ] Bun advisories, sources/integrity/licenses, lifecycle trust, and unused
  direct dependencies are required and tested.
- [ ] Bun's built-in lifecycle trust list is disabled; every trusted script maps
  to an exact locked version/integrity/review and every lock entry has advisory
  coverage.
- [ ] Every JS CLI/generator uses Bun and an exact lock entry; global auto-install,
  Node ancestry, mutable `@latest`, and undeclared executable fixtures fail.
- [ ] Active rustls and host/default vendored OpenSSL fail; native TLS passes on
  hosts and vendored OpenSSL passes only in the explicit Zig cross-release graph.
- [ ] Oxc core/resolver and stable native Oxlint units are exact,
  platform-complete, Bun-only, and have no runtime downloads; stable TypeScript
  7, the operator-excepted exact type-aware candidate, every direct/transitive
  TS6-API consumer, and lock-reachability fixtures are handed to plan 131 without
  changing the live compiler in this policy plan.
- [ ] Durable and executable pre-stable policy contains exactly Oxfmt and
  `oxlint-tsgolint`, with pins, owners, evidence, upgrade rules, and stable-release
  expiry; broader alpha/Beta Oxc adoption fails.
- [ ] cargo-shear and supported feature checks have reasoned, expiring exceptions only.
- [ ] `cargo xtask dependencies` is fully implemented, fixture-tested, and used by CI.
- [ ] Nextest profiles emit structurally validated JUnit/slow/flaky evidence.
- [ ] No empty test selection or hollow flake grep can pass a required job.
- [ ] Retry-pass flakes fail CI and quarantined tests keep running visibly with
  owner/reason/expiry.
- [ ] Default, `embed-ui`, `conformance`, and every supported feature contract
  have a drift-checked Clippy/test owner.
- [ ] Doctest, native macOS, advisory refresh, and reviewed hygiene gates pass.
- [ ] Cache changes are justified by recorded cold/warm evidence.
- [ ] Dependency discovery reports without creating branches or PRs.

## STOP Conditions

- Policy would require rustls, reject required native OpenSSL, or weaken the
  GreptimeDB/Turso/Bun constraints.
- A Bun lifecycle script would be trusted globally or without reviewing the
  exact package/version/behavior.
- A live advisory cannot be upgraded and lacks defensible reachability,
  upstream, owner, and expiry evidence.
- Feature testing assumes unsupported combinations or requires `ui/dist` in a
  generic clean checkout.
- An unused-dependency result ignores dynamic/config/generated imports or a
  dependency upgrade breaks the mutually compatible TanStack suite.
- Flake detection is a grep or is not tested against output from the pinned
  nextest version.
- A new hygiene lane is made required before its findings and stability are
  reviewed.
- Any workflow creates an update branch or pull request.

## Remove When

Delete this plan and index row when dependency policy, structured nextest
evidence, native smoke, and staged hygiene are enforced and green.
