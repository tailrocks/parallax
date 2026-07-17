# Plan 103: Add focused Rust/UI property, fuzz, and performance gates

> **Executor instructions**: Add only targets tied to named defect classes.
> Compile broad checks on pull-request-equivalent CI, run expensive measurement
> on stable scheduled runners, and set budgets only after variance is known.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MEDIUM
- **Depends on**: 097, 099, 101, 104, 133, 147, 148
- **Category**: testing / fuzzing / performance
- **Planned at**: `a1d8bf82`, revised 2026-07-12
- **Status**: IN PROGRESS (Rust lanes) — claimed 2026-07-17 by Claude Code
  agent (session 5904). Rust property/golden work proceeds now; UI property
  work stays gated on plans 133/147/148 owners.

## Why

Parallax's highest-risk parsers and invariants have examples but no systematic
property/fuzz corpus, and its ingest/read performance promises lack durable
allocation and regression evidence. Expensive tools need named defect classes,
owners, reproducible environments, and measured thresholds.

## Scope

- Properties/goldens for normalization, redaction, bundle hashing, trace trees,
  SQL builders/validation, serialization, and retry invariants.
- Bounded UI properties for route-search round trips, GraphQL/SSE runtime
  decoders, query-key identity, live ordering/deduplication, and feature state
  machines after plans 128/151 establish their owners, plus query-key,
  live-data, and bundle properties after plans 133, 147, and 148 establish their
  final owners.
- Fuzz targets and minimized corpora for OTLP/protobuf, Arrow, spool, and
  redaction boundaries.
- Target-to-workflow drift validation.
- Benchmarks for normalization, Arrow decode, spool, worker issue derivation,
  adapter overhead, and measured Greptime row clone/`SELECT *` candidates.
- PR compile/smoke and scheduled stable-runner measurement.

Out of scope:

- Arbitrary benchmark thresholds copied from an unrelated project or machine.
- Four-engine database performance claims outside the mandated four-build
  benchmark protocol.
- Miri, mutants, Dylint, Hakari, chaos, or self-hosted runners without a
  separately owned, measured proposal.
- Optimizing fingerprint regex passes before measurement identifies them.

## Steps

### Step 1: Name invariants and failure oracles

For each target, document the defect class, input domain, oracle, corpus owner,
runtime class, and promotion/removal rule. Cover normalization determinism,
redaction idempotence and no-secret output, canonical bundle hash stability,
trace parent/child invariants, SQL escaping/validation, serialization
compatibility, and late-retry no-replay properties from plan 099. For the UI,
cover search encode/decode round trips, decoder accept/reject domains, Query key
stability, SSE ordering/deduplication, and exhaustive reducer/state-machine
transitions without browser-global generation.

### Step 2: Add bounded property suites

Use deterministic seeded generation, shrinkable cases, explicit size limits,
and committed minimal regressions. Ensure timestamps, Unicode/invalid byte
boundaries, high-cardinality attributes, duplicate/out-of-order telemetry,
redaction variants, and nanosecond string compatibility are represented.
Run TypeScript properties through Bun, reuse Plan-152/153 and feature-owned
runtime schemas, and
persist only minimal deterministic regression cases.

### Step 3: Add parser/recovery fuzz targets

Fuzz OTLP/protobuf decode, Arrow response decode, spool framing/recovery, and
redaction inputs. Assert no panic, unbounded allocation, infinite loop, invalid
ownership clone on ingest, or unsafe secret emission. Keep minimized crash
corpora. Add a machine test that every declared target has a build/run workflow
and every workflow target exists.

### Step 4: Establish performance baselines

Benchmark owned-row normalization, Arrow decoding, spool append/replay,
worker issue derivation, and adapter dispatch. Instrument allocations and
copies on the zero-copy ingest path. Characterize current Greptime span read
row clones and `SELECT *`, and the seven-pass fingerprint normalizer, before
changing them. Apply the four-build rule to any Greptime-vs-ClickHouse claim.

### Step 5: Promote evidence carefully

Compile benchmarks/fuzz targets and run short deterministic smoke corpora in
normal CI. Run longer fuzz and benchmarks on stable scheduled runners. Gather
enough samples to model variance, then adopt relative/allocation ratchets with
documented noise margins and owners. A regression should report the metric and
baseline, not silently refresh it.

### Step 6: Evaluate advanced tools independently

For Miri, mutation testing, Dylint, Hakari, chaos, or self-hosted runners,
require a named uncovered defect class, owner, runtime/cost baseline,
pass/fail decision threshold, and removal policy. Adopt only the tools whose
spike produces useful stable signal.

## Test Plan

- Seeded property suites replay identically and shrink intentional failures.
- UI search/schema/cache/state properties run through Bun without duplicating
  production decoders.
- Each fuzz target builds, runs its corpus, and rejects an intentional panic
  fixture in its harness tests.
- Target/workflow drift fixtures fail for missing names in both directions.
- PR benchmark compile/smoke and scheduled repeated measurement.
- Allocation/copy instrumentation on representative ingest batches.
- Four-build evidence update for any cross-engine performance result.

## Done Criteria

- [ ] Named invariants have bounded property/golden coverage and regressions.
- [ ] UI search, runtime decoder, Query identity, live ordering, and state
  invariants have bounded generated coverage.
- [ ] Four initial fuzz boundaries have maintained minimized corpora.
- [ ] Declared fuzz/benchmark targets and workflows cannot drift silently.
- [ ] Representative hot paths have reproducible time/allocation baselines.
- [ ] PR checks compile/smoke; scheduled runners perform stable measurement.
- [ ] Ratchets use measured variance and fail without auto-refreshing baselines.
- [ ] Deferred hot-path candidates are optimized only when evidence warrants it.
- [ ] Every adopted advanced tool has an owner, threshold, cost, and removal rule.

## STOP Conditions

- A target has no precise oracle or can consume unbounded CI time/memory.
- A performance threshold is copied or chosen before variance measurement.
- Measurement changes hot-path ownership or clones telemetry to observe it.
- A database comparison omits any required stable/nightly build.
- A tool is promoted with flaky signal, no owner, or no failure disposition.

## Remove When

Delete this plan and index row when the focused property/fuzz corpus and
measured performance/allocation gates are maintained, stable, and enforced.
