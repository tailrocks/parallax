# Flaky Test Investigation and Replay

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note answers the flaky-test section of the Parallax research prompt:
Uber Testopedia, Google flaky-test research, CI observability, test failure
clustering, retry analysis, deterministic replay, root-cause analysis, flaky
detection algorithms, whether this is a standalone product, and whether agents
can fix flaky tests.

Version freshness rule: this recommendation is based on current public docs and
source material checked on 2026-05-25. Every future benchmark or comparison must
use the latest reasonably available stable/public version of each candidate as
of the benchmark date, and must label older benchmark posts or architecture docs
as historical evidence.

## Short Answer

Flaky-test investigation is a real pain and a useful Parallax wedge, but not as
a generic flaky-test dashboard.

The stronger path is:

> Start with deterministic CI failure bundles, then add historical flaky-test
> memory, failure clustering, reproducer hints, and agent-ready repair context.

A standalone "flaky test management" product is already crowded. Datadog,
Trunk, BuildPulse, CloudBees, and CI-autofix startups already cover detection,
history, quarantine, prioritization, ownership, and some AI grouping. Parallax
can still win if it is:

- open-source and local-first;
- built around portable evidence bundles;
- useful before a hosted backend exists;
- designed for coding agents, not only QA dashboards;
- excellent for Rust and GitHub Actions;
- honest about confidence and reproducibility.

## Evidence That The Problem Is Real

Google and Uber both show that flaky tests are not a toy problem:

- Google reported a continual rate of about 1.5% of test runs producing flaky
  results, nearly 16% of tests having some flakiness, and about 84% of observed
  pass-to-fail CI transitions involving a flaky test.
- Google later reported about 4.2 million tests in CI, with roughly 63,000 tests
  showing a flaky run over one week. Large tests were much more likely to be
  flaky: 0.5% of small tests, 1.6% of medium tests, and 14% of large tests in
  the sampled week.
- Google's DeFlake research studied flaky tests across 428 projects and reported
  82% accuracy for identifying the code-level location of flakiness root causes.
- Uber's monorepo migration exposed flakiness at scale and led to Test Analyzer,
  dynamic reproducer tools, static checkers, and then Testopedia as a
  language/repo-agnostic service for test reliability and performance state.

Sources:

- [Google Testing Blog: Flaky Tests at Google and How We Mitigate Them](https://testing.googleblog.com/2016/05/flaky-tests-at-google-and-how-we.html)
- [Google Testing Blog: Where do our flaky tests come from?](https://testing.googleblog.com/2017/04/where-do-our-flaky-tests-come-from.html)
- [Google Research: De-Flake Your Tests](https://research.google/pubs/de-flake-your-tests-automatically-locating-root-causes-of-flaky-tests-in-code-at-google/)
- [Uber Engineering: Handling Flaky Unit Tests in Java](https://www.uber.com/us/en/blog/handling-flaky-tests-java/)
- [Uber Engineering: Flaky Tests Overhaul at Uber](https://www.uber.com/us/en/blog/flaky-tests-overhaul/)

## What Existing Systems Teach

### Google

Google's public posts make two points that matter for Parallax:

1. A flaky result is not just "a failed run." It is a relationship between test
   identity, code version, and inconsistent outcomes.
2. Mitigation has product consequences. Reruns and quarantine reduce immediate
   CI pain, but they can also mask real bugs and delay legitimate failures.

Google's DeFlake paper is especially relevant because it says adoption depends
on developer workflow integration, simple debugging aids, and automated fixes.
That is exactly where Parallax should aim: not just detecting flakiness, but
handing an agent or developer enough evidence to act.

### Uber Test Analyzer and Testopedia

Uber's 2021 Test Analyzer work is a practical blueprint:

- collect test name, suite, target/build rule, result, duration, consecutive
  successes, stack traces, and current state;
- classify main-branch tests from historical runs;
- treat tests with 100 consecutive successful runs as stable;
- route flaky tests out of critical CI, while acknowledging this reduces
  reliability coverage;
- build dynamic reproducers to reproduce observed failure modes locally;
- add static checks to prevent known flakiness patterns such as fragile timed
  waits.

Uber's 2024 Testopedia work generalizes the concept:

- a language/repo-agnostic service;
- a test entity identified by a fully qualified name;
- realms owned by platform teams;
- read/write/notify domains;
- historical data sources tagged by source;
- configurable analyzers;
- state transitions such as new, stable, unstable, disabled, and deleted;
- ticketing when a test becomes unhealthy.

Parallax should borrow the entity model and analyzer model, not the
dashboard-first surface.

## Competitive Reality

| Product | Current position | Implication |
| --- | --- | --- |
| Datadog Test Optimization | Tracks flaky tests as pass/fail across multiple runs for the same commit; uses tags such as `is_flaky`, `is_new_flaky`, and `is_known_flaky`; tracks first/last flaked, commits flaked, failure rate, total time, and history over recent commits. | Datadog owns the integrated enterprise CI/test visibility version. |
| Trunk Flaky Tests | AI grouping of related failures, auto-quarantine, ticketing, flaky analytics, status history, environment segmentation, PR comments, webhooks, failure fingerprints. | The dashboard/quarantine product is already strong and focused. |
| BuildPulse | Flaky detection/history, impact metrics, reports, PR bots, quarantine, API access, enterprise features; starts from JUnit XML plus CI integration/test reporter upload. | JUnit-based hosted flaky management is already a product category. |
| CloudBees Smart Tests | AI test intelligence for relevant-test selection, flaky patterns, CI waste, classification, ownership, and triage. | Larger CI platforms are absorbing flaky-test management into test selection and CI optimization. |
| Playwright Test | Built-in retries and result categories: passed, flaky, failed. | Test frameworks are making flakiness visible at source. |
| cargo-nextest | Rust test runner with retries; tests that pass on retry are marked flaky, and `--flaky-result fail` can fail the run on flaky tests. | Rust already has a good local signal source for Parallax to consume. |

Sources:

- [Datadog: Working with Flaky Tests](https://docs.datadoghq.com/tests/flaky_tests/)
- [Trunk Flaky Tests](https://trunk.io/flaky-tests)
- [BuildPulse flaky tests overview](https://docs.buildpulse.io/flaky-tests/overview)
- [CloudBees Smart Tests](https://www.cloudbees.com/capabilities/cloudbees-smart-tests)
- [Playwright retries](https://playwright.dev/docs/test-retries)
- [cargo-nextest retries and flaky tests](https://nexte.st/docs/features/retries/)

## Detection Model

The core definition should be:

> A test is flaky when it both passes and fails for the same effective code and
> configuration.

The phrase "effective code and configuration" matters. Same commit is the easy
case; same dependency lockfile, test target, feature flags, runner image, test
environment, seed, shard, and external service state are harder.

Parallax should store:

| Field | Why |
| --- | --- |
| Test identity | Stable key for history and ownership. |
| Commit SHA and branch | Defines same-code comparison. |
| Attempt/retry number | Distinguishes first-fail/pass-on-retry from persistent failure. |
| Runner image and labels | Captures environment-dependent flakes. |
| Shard, seed, order, parallelism | Captures schedule/order-dependent flakes. |
| Duration and timeout | Captures slow, timing-sensitive, and resource-contention flakes. |
| Failure signature | Groups similar failure modes. |
| Stacktrace / assertion / log excerpt | Root-cause evidence. |
| Changed files and owners | Routing and repair context. |
| Previous pass/fail history | Separates new regression from known flaky behavior. |

## Algorithms To Use First

Parallax should avoid opaque ML as the first classifier. Deterministic evidence
is enough for a useful first version.

### 1. Same-Commit Outcome Classifier

Classify as `observed_flaky` when the same test identity has both pass and fail
outcomes for the same commit, lockfile, runner class, and test configuration.

This is simple, explainable, and aligns with Google, Datadog, and BuildPulse.

### 2. Retry Outcome Classifier

Classify as `retry_flaky_candidate` when a test fails on attempt 0 and passes on
a retry. This is weaker than same-commit multi-run history because retries can
change state or environment, but it is immediately available from Playwright,
cargo-nextest, and CI logs.

### 3. Failure Signature Clustering

Group failures by normalized:

- test identity;
- assertion/error type;
- first useful stack frame;
- normalized message;
- timeout/exit code;
- environment and runner labels.

This should be deterministic first, with LLM-based grouping only as a secondary
hint.

### 4. Change-Point Detection

Detect when a test moves from stable to unstable around a commit, dependency
update, runner image change, or infrastructure change. Uber Testopedia's state
machine is the right model: stable, unstable, disabled/quarantined, recovered.

### 5. Flake Cause Labels

Classify root-cause hypotheses into practical buckets:

| Bucket | Evidence |
| --- | --- |
| Test order/state leakage | Fails only after particular tests, passes alone, shared temp/db/cache state. |
| Concurrency/race | Thread/task timing, lock/wait patterns, rare interleavings, Loom/Shuttle evidence. |
| Fixed timeouts | Failures around waits, sleeps, bounded awaits, high CPU load. |
| External dependency | Network, sandbox, API, database, service availability, rate limits. |
| Resource contention | CPU, memory, disk, file descriptors, port collisions. |
| Selector/UI brittleness | DOM/locator/video/trace evidence, UI test framework reports. |
| Infrastructure/runner | Runner image, OS, container, hardware, region, clock, filesystem. |
| Real regression masked as flake | First failure after code change, deterministic repro, same failure across retries. |

## Deterministic Replay and Reproduction

Replay is the difference between "known flaky" and "fixable flaky."

Useful levels:

| Level | Tooling | Product value |
| --- | --- | --- |
| Retry replay | Re-run failed test with same command, seed, env, shard, and attempt context. | Cheap and should be first. |
| Stress replay | Re-run under CPU/memory load, altered parallelism, repeated loop, randomized order. | Matches Uber's dynamic reproducer strategy. |
| Framework trace replay | Playwright trace/video/screenshot; JUnit/log excerpts; nextest JUnit and retry metadata. | Strong evidence for UI and test-runner flakes. |
| Deterministic schedule replay | Shuttle, Loom, Antithesis-style deterministic simulation. | Powerful for Rust concurrency/distributed-system failures, but requires test design or external platform support. |

Rust-specific tools:

- Loom `0.7.2` explores possible valid concurrent executions under the memory
  model and can expose rare interleavings deterministically.
- Shuttle `0.9.1` controls scheduling for Rust concurrent tests, supports
  deterministic replay from a failing schedule string, and scales better than
  exhaustive checking at the cost of soundness.
- Antithesis-style deterministic simulation is extremely valuable for
  distributed systems, but it is heavyweight and not the first Parallax feature.

Sources:

- [Loom docs](https://docs.rs/loom/latest/loom/)
- [Shuttle docs](https://docs.rs/shuttle/latest/shuttle/)
- [Antithesis deterministic simulation testing](https://antithesis.com/docs/resources/deterministic_simulation_testing/)

## Agent Fixability

Agents can fix some flaky tests, but only with the right evidence.

| Failure class | Agent fix likelihood | Why |
| --- | --- | --- |
| Fixed sleeps/timeouts in tests | High | Replace with event-driven waits, readiness checks, or framework primitives. |
| Shared temp files, fixed ports, leaked DB state | High | Evidence points to isolation bug and fix is local. |
| UI locator brittleness | Medium-high | Needs trace/screenshot/DOM context and product intent. |
| Missing mock or external dependency | Medium | Agent can add mocks if test design is clear. |
| Concurrency race in product code | Medium | Requires reproducer or strong schedule evidence. |
| Infrastructure-only transient | Low-medium | Agent can quarantine or adjust infra, but may not fix root cause. |
| Unknown rare flake with only one log line | Low | Too little evidence. |

The best agent workflow is:

1. produce failure bundle;
2. classify likely flaky versus likely regression;
3. generate reproducer command and evidence summary;
4. ask agent for a fix only when evidence is strong;
5. require local or CI validation;
6. open a PR with failure signature, evidence, reproduction, and validation log.

## Product Decision

Flaky-test investigation can become a product, but Parallax should not start as
"Trunk, but open source." That would push the project toward dashboards,
quarantine workflow, enterprise QA reporting, and CI-provider integrations
before the evidence layer is proven.

The better Parallax sequence:

1. **CI failure bundle MVP.** Collect run/job/step/log/artifact/JUnit/go-test
   context and produce a portable evidence bundle.
2. **Historical test memory.** Store test identity, outcome, retries,
   signatures, duration, environment, and commit history in Turso for tiny mode.
3. **Flaky classification.** Add deterministic same-commit and retry-based
   classifiers.
4. **Reproducer hints.** Generate command lines for retry loops, stress, serial
   mode, seed replay, and framework-specific trace viewing.
5. **Agent PR workflow.** Only after reproducible classes are detected, ask an
   agent to propose a fix.

This keeps Parallax aligned with the bigger thesis: a runtime/context engine for
agents, not a standalone CI analytics dashboard.

## Data Model Extension

Add these records after the CI bundle schema is proven:

```text
test_entities(
  project_id,
  test_id,
  realm,
  fully_qualified_name,
  suite,
  package,
  file,
  owner,
  first_seen_at,
  last_seen_at,
  state
)

test_runs(
  project_id,
  test_id,
  run_id,
  attempt,
  commit_sha,
  branch,
  runner,
  shard,
  seed,
  status,
  duration_ms,
  failure_signature,
  evidence_refs
)

test_state_transitions(
  project_id,
  test_id,
  from_state,
  to_state,
  reason,
  confidence,
  evidence_refs,
  created_at
)

test_reproducers(
  project_id,
  test_id,
  failure_signature,
  command,
  environment,
  confidence,
  last_verified_at
)
```

Use Turso for this metadata in local/tiny deployments. This is product state and
test history, not high-volume telemetry.

## Bottom Line

Flaky-test investigation is validated, but crowded. The naive version is a
dashboard that lists flaky tests. The useful Parallax version is an evidence
compiler that turns a failed run into stable test identity, history, failure
signature, reproducer hints, and an agent-ready repair context.

Build bundles first. Add history second. Add autonomous fixes only for
well-evidenced, reproducible classes of flaky failure.
