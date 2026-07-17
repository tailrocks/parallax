# Test Reporting Ecosystem — Allure Deep-Dive, Landscape, and the Parallax Test-Observability Concept

> **Superseded as the canonical comparison by [`competitors/`](competitors/)** —
> see the [overview matrix](competitors/README.md) + [comparison set](competitors/comparison-set.md)
> + the per-product `competitors/parallax-vs-<product>.md` deep-dives (30 products,
> all verified 2026-07-17). This legacy note (a survey / watch / analysis **source**)
> is a **lead, not the destination**; re-verify specifics in the canonical folder
> before trusting, and migrate still-true content there.

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

## 4. Allure TestOps / Allure Service — run-preview UI inventory (second pass, 2026-07-14)

Deep pass on the commercial server (docs.qameta.io) and Allure 3's hosted
service, to derive what Parallax's UI needs to preview test-run results.

### 4.1 Launch (test run) model

- Launch list: card per launch with **Open/Closed state**, per-status result
  counts (clickable deep-links), metadata chips (tags, issues, env vars,
  release), assignee, defect count. **Open = uploads/triage in progress**
  (effectively "live"); **Closed** triggers post-processing (test-case
  upsert, dashboard statistics, cleanup). Configurable **auto-close policy**
  handles crashed producers. Launch contains 1..n job runs (one per CI
  execution / environment set).
- Launch detail — six tabs: **Overview** (unresolved failures, retries,
  muted, defects widgets) · **Tree** (grouping via project trees: Suites,
  Features, or custom label paths; per-node rollups; bulk actions) ·
  **Categories** (regex error categories) · **Errors** (message clustering) ·
  **Graphs** · **Timeline** (host × thread gantt).
- Live behavior: `allurectl watch` streams each result file as the test
  finishes; open launch fills incrementally. TestOps has no push transport
  documented — **a truly push-updating (SSE) launch view would out-execute
  TestOps**; Parallax already has the live infra (`LiveStreamPanel` on run
  detail).
- Cross-launch: **Compare** (N-launch status matrix, All/Intersect/Diff,
  "only status changes") and per-result transition badges
  **New / Regressed / Malfunctioned / Fixed** vs previous launch,
  environment-aware, filterable. **Rerun failed** creates a new job run in
  the same launch via a `testplan.json` subset contract.

### 4.2 Result detail and triage semantics

- Tabs: Overview (error + trace, category, defect links, **"similar
  failures"**, parameters, scenario = nested steps + fixtures, comments) ·
  History (same logical test across launches) · Retries (attempts within
  launch; same test id + same environment) · Attachments (inline image/
  video/log/HTML preview; fixture vs test-body attachments distinguished).
- **Execution status vs resolution** — the load-bearing triage concept:
  status = how the run ended; resolution = whether the team triaged it
  (defect-linked or muted). Unresolved-failure count is the launch's
  headline number.
- **Defects**: curated known-failure records with per-defect regex matchers
  (message pattern + stacktrace pattern), created inline from a failing
  result; while a defect is Open, matching failures in new launches
  auto-link and show as resolved; Closed disables matchers; issue-tracker
  sync can auto-close the defect. "Apply defect matchers" re-scans an open
  launch.
- **Mutes**: per test case (not per result), required reason, excluded from
  stats but visible, bulk mute, Mutes tab for review. No env scoping, no
  expiry — a beatable gap (env-scoped mutes + TTL/review queue).
- **Flaky rule**: ≥3 status transitions within last 10 executions →
  automatic bomb icon + filter (non-configurable).
- Test-case repository: AllureID ledger, workflow statuses, custom trees by
  label paths, **AQL** query language (`cf["Epic"] = "Auth"`,
  `ev["browser"] = "chrome"`, `not tag in [..]`) powering filters, saved
  views, widgets, dynamic test plans.
- Dashboards: 10 widget types (launch trend, automation trend,
  **low-performing tests** ranked by success rate/duration, tree map, pies).

### 4.3 Upload/session mechanics (defines a live test-ingest API)

Adapters write files; `allurectl` is the transport: create launch → open
numbered **upload session** per job run → batched file upload (results /
containers / attachments as separate categories) → close session → close
launch (or auto-close). CI-triggered runs get `ALLURE_JOB_RUN_ID` injected so
results land in the pre-created job run; `allurectl job-run env` exports
launch context for fan-out CI jobs. Raw upload endpoints are declared
proprietary/unstable — only the CLI is supported. Lessons for Parallax:
session lifecycle with server-side auto-close; env-var context injection
(`PARALLAX_RUN_ID` already exists); batch endpoints separating results from
attachments; stable test identity so streaming results attach to known
entities; environment as part of retry identity. Allure Service (OSS-adjacent)
is just remote history + static report hosting keyed by repo+branch — no
server UI; Parallax being the persistent backend makes per-run permalinks
sufficient.

### 4.4 Ranked UI requirements for a Parallax test-run preview

Tier 1 (not credible without): run list with status counts + open/closed
state + metadata chips + filters · **live-filling run detail** (results
appear as each test finishes — maps to existing SSE; differentiator in
execution since TestOps lacks push) · test tree with grouping + rollups ·
result detail with error/stack + nested step tree (maps to trace waterfall)
+ parameters + attachments viewer · five-status model + retries grouped
under one logical result.
Tier 2 (expected by Allure migrants): per-test history + flaky badge ·
failure grouping ("similar failures" — maps directly to Parallax
fingerprinting) · defect records with regex matchers + status-vs-resolution
split · mute with reason (env-scoped + expiry beats TestOps) · transition
badges (new/regressed/malfunctioned/fixed) · run comparison matrix.
Tier 3 (defer/scope down): dashboards, timeline tab (≈ trace waterfall with
host/thread lanes — near-free), AQL-style query grammar (share one grammar
with existing pages), test-case repository + CI trigger/test plans (only the
ingest-side hooks first), comments/assignees, PDF/CSV export.
Bottom line: Parallax already owns the primitives Allure lacks (push
updates, trace waterfall, issue grouping, metrics). Net-new UI concentrates
in four components: test tree with rollups, step tree + attachments viewer,
retry/history identity layer, and matcher/mute triage flows.

## 5. Runner integration — Gradle, cargo test, cargo-nextest → Parallax (2026-07-14)

Parenting chain for all runners: `parallax run start` root span →
`TRACEPARENT` env (bare uppercase; OTel env-carrier spec is RC) → runner →
per-test span `setParent(extract(TRACEPARENT))`; `parallax.run.id` stamped on
every span. Universal pitfall: batch span processors (5 s default delay) lose
spans in short-lived processes — every path below needs explicit
flush/shutdown or a simple/sync processor.

### 5.1 Gradle / JUnit 5 (ranked)

1. **Custom JUnit Platform listener jar (recommended; live + best identity).**
   `testRuntimeOnly` jar registering `TestExecutionListener` +
   `LauncherSessionListener` via ServiceLoader; Gradle auto-registers it in
   each forked worker JVM. Identity = **JUnit `UniqueId`**
   (`[engine:junit-jupiter]/[class:FQCN]/[method:name(paramSig)]`,
   `[test-template-invocation:#N]` for parameterized) — structured,
   deterministic, param-signature-aware; full `recordException` stacks.
   Listener callbacks are multi-threaded → keep
   `ConcurrentHashMap<UniqueId, Span>` + explicit `setParent`, never
   `Span.current()`. Flush in `launcherSessionClosed` per fork JVM.
   Spring Boot integration tests: OTel javaagent on the test JVM +
   agent-bridged `GlobalOpenTelemetry` + a Jupiter `InvocationInterceptor`
   that wraps the invocation on the test thread with `makeCurrent()` → agent
   HTTP/JDBC spans nest under the per-test span. Gradle daemon env drift:
   forward `TRACEPARENT`/`PARALLAX_RUN_ID` explicitly in the `Test` task DSL.
   Retries: Gradle test-retry rounds = fresh worker JVM, same UniqueId —
   count attempts listener-side; JUnit Pioneer `@RetryingTest` attempts
   appear as template invocations natively. Prior art all dormant/archived
   (ryandens/junit-platform-otel, Dynatrace extension) — build, don't adopt.
2. **`com.atkinsondev.opentelemetry-build` 4.6.2 (zero-code quick win).**
   Live per-test spans (100 ms batch), OTLP gRPC/HTTP, build→task→worker→
   class→method nesting. Accept: displayName-only identity (param
   collisions), stacks truncated to 5 frames, no attempt counter, custom
   attribute names (`test.result`, `test.failure.*` — not semconv),
   TRACE_ID/SPAN_ID-style parenting (not TRACEPARENT), config-cache mode
   degrades (retroactive spans, retries deduped by name).
3. **Post-run reconciliation from Gradle JUnit XML** with `mergeReruns=true`
   → Surefire-style `<flakyFailure>`/`<rerunFailure>` classification;
   XML written only at task end (never live); no per-testcase wall-clock
   timestamp. Best as authority layer over 1 or 2 for crashed/killed JVMs.

### 5.2 cargo test / libtest (structurally worst — nextest preferred)

- **No hooks, no listener API.** JSON output still unstable mid-2026
  (rust-lang/rust#49359 open; eRFC 3558 stalled, 2026 project goal unowned);
  works on stable only via
  `RUSTC_BOOTSTRAP=1 cargo test -- -Z unstable-options --format json
  --report-time`; events carry no wall-clock timestamps (reader-side
  stamping), `exec_time` for duration.
- **Fatal flush class**: libtest exits via `process::exit(101)` on failure —
  destructors never run; Drop-based exporters lose exactly the failing runs.
  Mitigations: `SimpleSpanProcessor` (sync export), libtest-mimic custom
  harness (own `main`, flush before `Conclusion::exit()`), nightly-only exit
  callback.
- In-process subscriber: one process, tests parallel on threads —
  `#[tokio::test(flavor="multi_thread")]` futures hop threads → context
  bleed unless every body is instrumented. No established crate does
  per-test OTel for stock cargo test — niche open.
- Recommended if forced: wrapper-side converter streaming the unstable JSON
  into synthesized spans.

### 5.3 cargo-nextest (best Rust story)

1. **In-test subscriber lib + env identity (recommended; live + richest).**
   Process-per-test → per-process subscriber is contention-free. Identity
   from env (all verified, 0.9.116+): `NEXTEST_RUN_ID`, `NEXTEST_BINARY_ID`,
   `NEXTEST_TEST_NAME`, `NEXTEST_ATTEMPT` (1-indexed),
   `NEXTEST_TOTAL_ATTEMPTS`, **`NEXTEST_ATTEMPT_ID`**
   (`run-id:binary-id$test-name#attempt` — globally unique per attempt).
   Parent from passed-through `TRACEPARENT`. Export with
   `SimpleSpanProcessor` or explicit `SdkTracerProvider::shutdown()` (never
   Drop-at-exit; `global::shutdown_tracer_provider()` was removed in OTel
   Rust 0.28). Retry attempts land automatically (fresh process + fresh
   attempt env).
2. **Wrapper consumes `libtest-json-plus` stream (live, zero test-code).**
   `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 cargo nextest run --message-format
   libtest-json-plus --message-format-version 0.1` — per-event flush since
   0.9.103; caveats: experimental, only final aggregate per test (no
   per-attempt events), test names carry `#total-runs` suffix.
3. **Post-run JUnit XML (authoritative retries).** `[profile.ci.junit]
   path`; testsuite per binary id, real start timestamps, full error chain,
   `<flakyFailure>`/`<rerunFailure>` per attempt with output;
   `flaky-fail-status` configurable (0.9.131+). Written only on run finish.
   Required as reconciliation for SIGKILL/timeout-killed tests that never
   flushed (nextest SIGTERM grace default 10 s; Windows timeout = hard
   kill).
4. Exotic: USDT probes (per-attempt start/done events, needs
   bpftrace/DTrace, unstable); experimental `run-wrapper` scripts can
   interpose a span-emitting wrapper around every test invocation.

### 5.4 Stable test_case_key inputs per runner

| Runner | Key input | Caveat |
|---|---|---|
| Gradle/JUnit 5 | UniqueId segments (class + method + param signature) | `#N` template invocations positional; avoid displayName with argument toString |
| cargo test | binary target + `module::path::fn_name` | line-free; module rename breaks history (no lineage) |
| cargo-nextest | `NEXTEST_BINARY_ID` + `NEXTEST_TEST_NAME`; attempt = `NEXTEST_ATTEMPT`; per-attempt key = `NEXTEST_ATTEMPT_ID` | binary-id forms: `crate`, `crate::bin-name`, `crate::kind/bin-name` |
| rstest | each `#[case]` = own test fn `parent::case_N[_desc]` | positional numbering shifts on insert/reorder — prefer `#[case::name]` |

### 5.5 Recommended hybrid (feeds plans 154/155)

Live spans from Gradle listener jar + nextest in-test subscriber, parented on
the `parallax run` wrapper's TRACEPARENT; wrapper additionally parses
post-run JUnit XML (Gradle `mergeReruns`, nextest junit profile) to reconcile
retry/flaky classification and gap-fill tests that died without flushing.
Policy note: `opentelemetry-otlp` 0.32 gRPC TLS is rustls-only — irrelevant
for plaintext local `:4317`/`:4318`, but any future remote-TLS test export
must use OTLP/HTTP with a native-TLS client per repository TLS policy.
