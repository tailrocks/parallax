# Test Reporting Ecosystem — Allure Deep-Dive, Landscape, and the Parallax Test-Observability Concept

Research date: 2026-07-14

Operator direction (2026-07-14): Parallax should be usable not only as a
telemetry backend but as a **test reporting system** — every test visible,
every failed test linked to the trace behind it, tests on their own surface
with filters, history, and version-under-test attribution. This document is
the evidence base: a deep teardown of Allure, a survey of the wider
test-reporting/test-observability ecosystem, the identity and flakiness models
in production use, and the mapping onto Parallax. The executable work packet
derived from this research lives in
[`plans/155-test-reporting-surface.md`](../../../plans/155-test-reporting-surface.md)
(product surface) and
[`plans/154-playground-capability-and-test-observability.md`](../../../plans/154-playground-capability-and-test-observability.md)
(playground payload). Capture mechanics and CI failure-bundle research remain
in [`capture/ci-and-flaky-tests.md`](../capture/ci-and-flaky-tests.md) (owned
by plan 124 for CI-provider collection).

Two premise corrections found during research: **Codecov is no longer
Sentry-owned** (Harness acquired it, announced 2026-06-02), and **Tracetest is
effectively dormant** (cloud EOL Oct 2024, last OSS push June 2025). Both
matter strategically: Sentry entered and then exited pre-release test
analytics without ever fusing test failures with its issue model, and the
trace-based-testing pattern is validated but currently ownerless.

## 1. Allure Report — the reference design for test reporting

Allure 2 (Java, v2.44.0 Jul 2026) is legacy-but-alive; **Allure 3**
(TypeScript rewrite, GA May 2025, v3.14.3 Jul 2026) adds a plugin UI system,
watch mode, first-class environments, known-issue lists, quality gates,
`history.jsonl`, and an optional hosted history service. Same
`allure-results` JSON format and adapters throughout.
Sources: allurereport.org/docs (how-it-works, test-identifiers,
history-and-retries, test-statuses, visual-analytics), github.com/allure-framework.

### 1.1 Data model

One JSON file per test **execution** (retries = multiple files), plus
container files for fixtures, attachment blobs, `categories.json`,
`environment.properties`, `executor.json`, and a history directory carried
between runs.

The **identity system** is the heart of the design — four strictly layered
identifiers:

| Identifier | Identifies | Derivation | Breaks on |
|---|---|---|---|
| `uuid` | one execution | random | never reused |
| `historyId` | test case **+ parameter values** | hash(testCaseId + ordered non-excluded params) | param value change, rename |
| `testCaseId` | test case (param-invariant) | usually md5(fullName) | rename/move |
| `fullName` | code location | framework-specific (pytest: module path; **Playwright: file + line/column!**) | rename, move, even line shift |

`historyId` powers everything cross-run: history tabs, trends, retry grouping,
flakiness. Retries: multiple results with the same historyId in one launch;
**latest timestamp wins** as canonical, earlier attempts shown under a Retries
tab but excluded from history — a documented weakness (masks flakiness in
pass-rate). The `ALLURE_ID` label is the manual override that survives all
renames — the officially recommended mitigation for identity fragility.
Parameters support `excluded` (out of historyId), `masked`, `hidden`.

**Status model** — Allure's single most-praised triage feature:
`passed / failed / broken / skipped / unknown`, where **failed** = assertion
error (product defect) and **broken** = any other exception (test/infra
defect). Cheap heuristic (exception class ∈ assertion family), outsized triage
value: failed routes to feature owner, broken routes to harness owner.

**`categories.json`** = user-authored, repo-versioned failure classification:
ordered first-match rules over `matchedStatuses` + `messageRegex` +
`traceRegex`. Allure 3 adds label matchers, `groupByMessage` (built-in error
fingerprinting), and **`transitions: [regressed, malfunctioned]`** —
status-change-aware rules distinguishing passed→failed (regressed) from
passed→broken (malfunctioned). This is declarative Sentry-style grouping,
authored by the user.

`statusDetails` flags: `flaky`, `muted` (excluded from statistics), `known`
(known-bug failure). Containers attach shared setup/teardown step trees to N
child results. `environment.properties` ≈ resource attributes.
`executor.json` (`buildOrder`, `buildUrl`, `reportUrl`) orders trend charts
and deep-links them to CI builds.

### 1.2 Report UI surfaces

Overview (status donut, severity distribution, top defect categories, history
trend ordered by buildOrder, executor + environment widgets) · Categories
(defect groups) · Suites (parentSuite/suite/subSuite tree) · Graphs (status
pie, severity, log-scale duration histogram, duration/retries/categories
trends) · **Timeline** (Gantt of the launch by host → thread — real
parallelism, stragglers) · Behaviors (epic/feature/story rollups) · Packages ·
Test detail (steps tree with per-step attachments, parameters, Set
up/Tear down blocks, Retries tab, History tab, links with issue/tms icons,
flaky bomb icon). Allure 3 adds: status-transition flows, test-base growth,
problems-by-environment heat map, status-age pyramid (how long red), Testing
Pyramid by `layer` label, multi-environment comparison of one test.

### 1.3 Allure TestOps (commercial)

The hosted backend the OSS report lacks: real-time launch upload, unified
manual+automated test-case repository (AllureID ledger), test plans +
selective CI reruns, analytics dashboards, server-side cross-launch **Defects**
(failure-grouping rules aggregating many results under one root cause, Jira
lifecycle sync), **muting/quarantine** with reasons, RBAC. The business model
is exactly the gap between static per-launch reports and a persistent
test-intelligence database.

### 1.4 Limits worth exploiting

Static per-launch artifact; history = files manually shuttled between CI runs
(routinely lost); no queryable store, no cross-branch/cross-env aggregation in
OSS; identity fragility (Playwright line/column!); RAM-bound generation
(hangs at ~70k results); stale renamed tests accumulate; **no distributed-trace
linkage at all** — no trace/span ID anywhere in the format; attachments are
opaque blobs, not correlated telemetry.

## 2. Landscape beyond Allure

### 2.1 Per-tool findings (condensed; full agent notes preserved in this doc's history)

- **ReportPortal** (Apache-2.0 + paid SaaS): launch→suite→test→step→log model.
  AI triage is **not an LLM**: OpenSearch `more_like_this` retrieval over
  previously-triaged ERROR logs + XGBoost classifier (~30-40 features),
  applies defect type at ≥50% probability; per-project model retraining with
  keep-only-if-beats-global validation. Defect taxonomy: To Investigate /
  Product Bug / Automation Bug / System Issue / No Defect (+custom subtypes).
  Unique Error Analysis clusters a launch's error logs for bulk triage.
  History keyed on `testCaseHash` with an explicit fallback chain (explicit
  id → code ref → name+parents, each + params); the deprecated `uniqueId`
  included launch name and broke history — a canonical identity mistake.
  No quarantine, no OTel. 2026: MCP server (15 tools), `isAgentic` badge.
- **Datadog Test Optimization** ($20/committer/mo): the reference for
  test-as-telemetry. Four span levels (session/module/suite/test); **each
  test run is a trace**; APM spans nest under the test span; RUM/Session
  Replay correlation for browser tests. Identity = FQN + parameters +
  `test.configuration.*` (configuration is a separate queryable axis).
  Flaky = pass+fail on the same commit; 4-state machine Active → Quarantined
  (failures suppressed, enforced by the tracer fetching state pre-session) →
  Disabled → Fixed (auto after 30 flake-free days); remediation via test key
  in commit message + 20 retries + grace period; automated
  quarantine policies; AI root-cause into 13 categories; Early Flake
  Detection (new tests retried 10× before polluting history); Failed Test
  Replay (local variables); Test Impact Analysis (per-test coverage × git
  diff → skip untouched, 40-90% claimed CI savings); PR comments + PR Gates.
- **Currents.dev** (closed SaaS, $49/mo+): Playwright/Cypress depth — hosted
  Playwright Trace Viewer for trace.zip, live step streaming, History tab.
  **Errors Explorer** dedups failures into Category/Action/Target (one bad
  selector across many tests). Impact-ranked flakiness (rate × executions).
  Quarantine via **Currents Actions** rules enforced by reporter/fixtures
  (skip / quarantine-report-as-skipped / tag) with git/error conditions.
  Identity = hash(projectId + specFilePath + testTitle path).
- **Codecov Test Analytics** (FSL, now Harness): JUnit XML only, 60-day
  retention. Identity = murmur3(name, classname, testsuite). Flake state
  created by any default-branch failure, **expires after 30 consecutive
  passes**. PR comment distinguishes "your code broke this" vs "known flaky
  on main". Sentry built then abandoned this space — never fused test
  failures with its issue model.
- **Buildkite Test Engine**: reliability = passed/(passed+failed) with
  skipped excluded; identity = suite + explicit **scope** + name; 5 monitor
  types (passed-on-retry same-SHA, transition count, Bayesian probabilistic
  flakiness, new test, duration); state machine enabled → muted (soft-fail) /
  skipped; OSS `bktec` client enforces quarantine at the runner; span
  timeline on test execution pages (Ruby); 120-day retention.
- **Trunk Flaky Tests** (uploader MIT, backend closed, free ≤5 committers):
  JUnit XML/Bazel BEP/xcresult in; identity = **variant** + file path + name
  (variant = env/arch axis); health = Healthy/Flaky/**Broken** — broken =
  consistently failing, **never auto-quarantined** (only flaky is);
  quarantine = exit-code rewriting at the runner with backend lookup +
  fail-safe; PR comments, Jira/Linear mirroring, dashboard sorted by
  PRs-impacted; AI failure summaries; failure-reason grouping.
- **CTRF** (ctrf.io, pre-1.0, single maintainer, MIT): JSON schema successor
  candidate to JUnit XML — first-class `flaky` bool, `retryAttempts[]`
  (per-attempt status/trace/attachments), `ai` summary field, tags/labels/
  parameters/steps, environment (buildUrl/commit/branch), per-test
  `insights` (passRate/flakyRate/p95). MSTest committed native support; no
  vendor ingestion yet at Trunk/Currents/Datadog. Verdict: ingest both;
  normalize JUnit → CTRF-shaped internal model.
- **CloudBees Smart Tests** (ex-Launchable): predictive subset selection from
  diff↔test similarity + confidence curves; flakiness score 0-1.
- **TMS tier** (TestRail, Qase, Testmo, Tesults): do NOT rebuild authoring,
  plans, milestones, approvals, traceability. Worth absorbing: one-command
  bulk result submission (Testmo CLI wraps the runner and captures
  console+timing), per-test history, defect linking, Tesults'
  `/insights/flaky-tests` simplicity.
- **Trace-based testing**: Tracetest (dormant) = trigger + assertions over
  the resulting distributed trace, traceparent injected via meta tag so the
  browser SDK joins browser→backend into one trace. Pattern validated,
  ownerless. echoed (Jest/Playwright per-test trace capture, stale),
  Helios (dead, Snyk). BrowserStack Test Observability: auto-tags
  Flaky/Always-Failing/New-Failure/Performance-Anomaly, mute workflow.
- **CI-native table stakes**: GitLab MR widget with **new/resolved/existing
  failure diff vs target branch**; GitHub has no native test UI (ecosystem
  actions fill it); Jenkins JUnit trend + Claim plugin (failed-test triage
  assignment) + Flaky Test Handler.

### 2.2 Capability matrix (who has what)

| Capability | Allure OSS | TestOps | ReportPortal | Datadog | Currents | Codecov | Buildkite | Trunk |
|---|---|---|---|---|---|---|---|---|
| Queryable cross-run store | ❌ (files) | ✅ | ✅ | ✅ | ✅ | ⚠️ 60d | ✅ 120d | ✅ |
| Stable test identity | ⚠️ fragile hash + ALLURE_ID | ✅ ledger | ✅ fallback chain | ✅ FQN+params+config | ⚠️ path hash | ⚠️ name hash | ✅ +scope | ✅ +variant |
| Retry/attempt chain | ⚠️ latest-wins | ✅ | ⚠️ last-retry-wins | ✅ | ✅ attempts | ❌ | ✅ | ✅ |
| Flaky detection | ⚠️ flag only | ✅ | ⚠️ widget | ✅ multi-signal + states | ✅ intra-run | ✅ stateful+expiry | ✅ 5 monitors | ✅ 3 monitors + Broken split |
| Quarantine/mute enforced at runner | ❌ | ✅ | ❌ | ✅ tracer | ✅ Actions | ❌ | ✅ bktec | ✅ exit-code |
| Failure clustering | ⚠️ categories.json | ✅ Defects | ✅ ML + patterns | ⚠️ 13 AI categories | ✅ Cat/Action/Target | ❌ | ❌ | ✅ reason groups |
| Failed-vs-broken taxonomy | ✅ | ✅ | ✅ (5 types) | ❌ | ⚠️ | ❌ | ❌ | ✅ Flaky-vs-Broken |
| Distributed-trace linkage | ❌ | ❌ | ❌ | ✅ walled garden | ⚠️ PW trace.zip only | ❌ | ⚠️ Ruby spans | ❌ |
| Logs/metrics of SUT in test window | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Shared fingerprint with production issues | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| PR feedback w/ known-flaky diff | ❌ | ⚠️ | ⚠️ gates | ✅ | ✅ | ✅ | ⚠️ | ✅ |
| Impact analysis / selection | ❌ | ⚠️ plans | ❌ | ✅ TIA | ⚠️ timing | ❌ | ⚠️ splitting | ❌ |

The two bottom-left cells nobody fills — **SUT telemetry in the test window**
and **shared fingerprints between test failures and production issues** — are
exactly where an observability backend that ingests test telemetry natively
wins structurally.

### 2.3 Identity-model lessons (consolidated)

1. Nobody trusts the bare name. Winning shape: **logical id with fallback
   chain** (explicit override → code reference → name path), **plus parameter
   values as a sub-identity**, **plus an environment/configuration axis kept
   OUT of the hash** as a queryable dimension (Datadog `test.configuration.*`,
   Trunk `variant`, Buildkite `scope`) — that separation is what lets a tool
   say "flaky only on iOS".
2. Never include run/launch identity in test identity (ReportPortal's
   deprecated `uniqueId` mistake). A run id scopes one session; test history
   must survive across runs, branches, and CI jobs.
3. Keep hash components queryable (Currents/Codecov store opaque hashes and
   regret it in filtering).
4. Provide an explicit stable override (`ALLURE_ID` pattern) from day one;
   consider fuzzy re-linking on rename (nobody does this — differentiator).

### 2.4 Flakiness algorithms in the wild

Detection signals: (1) intra-run attempt mix (≥1 pass AND ≥1 fail among
retries); (2) **same-commit cross-run divergence** — the most defensible
primitive, code pinned by SHA; (3) windowed status-transition counting.
Persistent state: signal → per-test state (Healthy/Flaky/Broken, where
**Broken = consistently failing ≠ flaky and is never auto-quarantined**) →
expiry/recovery (30 consecutive passes / 30 days / 7 days + 100 executions).
Ranking: by impact (rate × executions, PRs-impacted, time-lost), not raw
rate. Proactive: retry new tests N× before they enter history (Datadog EFD).
Quarantine enforcement has exactly two proven mechanisms: backend-state fetch
by the runner/tracer (Datadog, Buildkite bktec, Currents Actions) or
exit-code rewriting at upload (Trunk).

## 3. Parallax mapping — what to steal, what falls out free, what must be built

### 3.1 Concepts with no OTel equivalent (must be modeled explicitly)

Stable test identity (historyId/testCaseId analog) · attempt chains ·
failed-vs-broken taxonomy · flaky/muted/known flags · persistent flaky state
machine · declarative category rules · TMS/issue typed links · launch
ordering (`buildOrder`) · parameter excluded/masked semantics · manual test
cases (out of scope). OTel semconv `test.*` exists but is 4 attributes at
Development stability; `cicd.pipeline.*` is Release Candidate; CI/CD SIG is
active (OTEP #223/#258) and nobody owns the standards-track position yet.

### 3.2 Falls out of existing Parallax machinery nearly free

- Test = root span, steps = child spans, assertion = span status + exception
  event → **trace waterfall is the test detail view**; Allure's Timeline tab
  is the trace lane view Parallax already renders.
- Trends/histograms/heat maps = queries over GreptimeDB — no history-file
  shuttling, Parallax's structural advantage over Allure OSS.
- `environment.properties` = resource attributes; multi-env comparison =
  GROUP BY.
- Failure clustering = existing `fingerprint_with_operation` normalization
  pipeline (strips volatile tokens — directly applicable to assertion
  messages like "expected 4, got 2").
- **Shared fingerprint between test failures and production issues** — the
  capability no vendor has, enabled because the same grouping engine sees
  both. "This test failure IS production issue #123"; "this prod issue first
  appeared in CI run X".
- SUT logs/metrics in the test window via time bounds + trace id; release
  attribution via the same `service.version`/`vcs.*` plumbing as prod.
- Evidence bundle anchored on a failed test = the test-mode bundle (bounded
  failure context for humans and agents) — Parallax's wedge applied to tests.

### 3.3 Must be built as product (owned by plan 155)

Test identity registry + variant model (Turso) · attempt-chain semantics ·
failed/broken derivation · flaky state machine with expiry · Tests UI surface
(separate page with filters; test session list; per-case history; failure
detail = trace + logs + issue link) · GraphQL `tests` namespace · declarative
category/override rules · (triggers, not V1) JUnit XML/CTRF ingestion,
runner-enforced quarantine protocol, PR feedback, impact analysis.

### 3.4 Strategic note

Sentry owned both halves (issues + Codecov test analytics) and never fused
them, then sold Codecov (June 2026) — the fusion Parallax can do is validated
as a gap by the incumbent's exit. Datadog proves the test-as-trace model
commercially but is closed and per-committer priced. Allure proves the report
UX and identity semantics but has no store and no trace linkage. Tracetest
proved trace-based testing and died. The open position: **OTel-native,
self-hosted, evidence-bundle-centric test observability fused with production
error tracking.**
