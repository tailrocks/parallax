# CI Failure Context MVP

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-24

## Executive Summary

Parallax should start with a deterministic GitHub Actions failure-context
compiler, not a full flaky-test platform or production incident system.

The first useful product shape is:

> Given a failed GitHub Actions workflow run, collect CI metadata, job logs,
> test reports, check annotations, artifacts, git context, and bounded raw
> evidence into a portable bundle that a human or coding agent can inspect.

This keeps the MVP close to the current Parallax thesis:

- CLI-first.
- Local-first.
- Evidence-backed.
- Useful before any hosted storage layer exists.
- Designed for humans and agents, not only dashboards.

The key product boundary: the first version should explain what evidence exists,
normalize it, fingerprint the failure, and produce a clean bundle. It should not
promise automated root cause analysis until the deterministic context layer is
working.

## Why GitHub Actions First

GitHub Actions is a practical first CI target because its REST API exposes the
core surfaces Parallax needs:

| Surface | Useful data | Why it matters |
| --- | --- | --- |
| Workflow run | run ID, attempt, event, branch, head SHA, status, conclusion, run URL | Anchor for the bundle and reproducible references. |
| Workflow jobs | job IDs, steps, runner labels, start/end times, conclusions, check run URLs | Lets Parallax identify the failed job and failing step before reading raw logs. |
| Job logs | plain-text logs downloaded through a temporary redirect URL | Provides stack traces, command output, install failures, test output, and infrastructure errors. |
| Artifacts | names, sizes, digests, expiration, download URLs, workflow-run metadata | Lets users upload JUnit XML, screenshots, coverage, traces, and app-specific debug files. |
| Check runs and annotations | file paths, line references, levels, messages | Captures compiler, linter, test, and code-scanning feedback already attached to commits. |

Sources:

- [GitHub REST API - workflow runs](https://docs.github.com/en/rest/actions/workflow-runs)
- [GitHub REST API - workflow jobs](https://docs.github.com/en/rest/actions/workflow-jobs?apiVersion=2022-11-28)
- [GitHub REST API - artifacts](https://docs.github.com/en/rest/actions/artifacts?apiVersion=2022-11-28)
- [GitHub REST API - checks](https://docs.github.com/en/rest/checks?apiVersion=2026-03-10)
- [GitHub REST API - check runs](https://docs.github.com/en/rest/checks/runs?apiVersion=2026-03-10)

GitHub also explicitly positions workflow artifacts as a way to upload build and
test output for debugging failed tests or crashes. That is enough to make
artifact collection part of the MVP instead of a later enterprise feature.

Source:

- [GitHub Actions - store and share data with workflow artifacts](https://docs.github.com/en/actions/tutorials/store-and-share-data)

## Test Report Reality

GitHub job logs are necessary but insufficient. A useful Parallax bundle needs
stable test identities, durations, statuses, and failure bodies. Those usually
come from test reports, not CI logs.

JUnit XML is the best first interchange format because it is broadly supported
even though it is not a clean formal standard:

- Gradle says its `Test` task writes XML test results in a "JUnit XML" pseudo
  standard and calls out CI servers and tooling as common consumers.
- Maven Surefire generates XML files by default under
  `${basedir}/target/surefire-reports/TEST-*.xml`.
- pytest supports `--junitxml` / `--junit-xml` output.
- GitLab CI requires JUnit XML for unit test reports.
- Buildkite Test Engine can import JUnit XML and documents required
  `testcase` attributes such as `classname` and `name`.

Sources:

- [Gradle - communicating test results via XML files](https://docs.gradle.org/current/userguide/java_testing.html#sec:java_test_reporting)
- [Maven Surefire Plugin](https://maven.apache.org/surefire/maven-surefire-plugin/index.html)
- [pytest reference](https://docs.pytest.org/en/stable/reference.html)
- [GitLab unit test reports](https://docs.gitlab.com/ci/testing/unit_test_reports/)
- [Buildkite importing JUnit XML](https://buildkite.com/docs/test-engine/test-collection/importing-junit-xml)

Go should be a first-class non-JUnit path because `go test -json` produces
structured test events directly.

Source:

- [Go test2json documentation](https://go.dev/cmd/test2json/)

## OpenTelemetry Naming, Not Storage

OpenTelemetry now has development-stage CI/CD, VCS, and test semantic
conventions. Parallax should use those names where they fit, but it should not
make OpenTelemetry ingestion or an observability database mandatory for the
first MVP.

Useful early fields include:

| Parallax concept | OpenTelemetry-aligned field |
| --- | --- |
| Pipeline run ID | `cicd.pipeline.run.id` |
| Pipeline run URL | `cicd.pipeline.run.url.full` |
| Repository URL | `vcs.repository.url.full` |
| Repository name | `vcs.repository.name` |
| Head branch | `vcs.ref.head.name` |
| Test case name | `test.case.name` |
| Test case result | `test.case.result.status` |
| Test suite name | `test.suite.name` |
| Test suite run status | `test.suite.run.status` |

The caveat is important: OpenTelemetry marks these conventions as
development-stage. Use the naming as a compatibility influence, not as an
external contract that blocks local bundle design.

Sources:

- [OpenTelemetry semantic conventions for CI/CD](https://opentelemetry.io/docs/specs/semconv/cicd/)
- [OpenTelemetry CI/CD resource conventions](https://opentelemetry.io/docs/specs/semconv/resource/cicd/)
- [OpenTelemetry test attributes](https://opentelemetry.io/docs/specs/semconv/registry/attributes/test/)

## MVP User Flow

The first workflow should be post-run collection:

```bash
parallax gha collect donbeave/parallax --run-id 123456789 --out parallax-bundle.zip
parallax bundle summarize parallax-bundle.zip
```

This can work outside the failing workflow and can fetch the finalized run,
jobs, logs, artifacts, and annotations through the GitHub API.

A GitHub Action wrapper should come next:

```yaml
- name: Upload Parallax failure bundle
  if: failure() || cancelled()
  uses: parallax/parallax-action@v0
  with:
    junit: "**/test-results/**/*.xml"
    include: |
      playwright-report/**
      coverage/**
```

The action mode is valuable because it can collect workspace-local files before
the runner disappears. The CLI mode is valuable because it can reconstruct a
bundle after the run is complete.

## Bundle Shape

The bundle should be a directory or ZIP with stable, inspectable files:

```text
parallax-bundle/
  manifest.json
  summary.md
  normalized/
    run.json
    jobs.json
    steps.json
    test_cases.jsonl
    failures.jsonl
    annotations.jsonl
    artifacts.json
  evidence/
    logs/
      <job_id>.excerpt.txt
    artifacts/
      index.json
    git/
      head.diff
      changed_files.txt
  agent/
    prompt.md
    evidence_index.json
```

The manifest should include:

| Field | Purpose |
| --- | --- |
| `schema_version` | Allows bundle evolution. |
| `created_at` | Research and debugging timestamp. |
| `collector` | CLI/action version and platform. |
| `provider` | `github_actions` for the first MVP. |
| `repository` | Owner, name, URL, default branch. |
| `run` | Run ID, attempt, workflow, event, actor, branch, SHA, URL. |
| `inputs` | Which APIs, report globs, and artifact patterns were collected. |
| `redaction` | Redaction policy, findings, and skipped files. |

## Normalized Evidence Model

Parallax should normalize evidence into a small set of records:

| Record | Minimum fields |
| --- | --- |
| `job` | `id`, `name`, `status`, `conclusion`, `started_at`, `completed_at`, `runner`, `url` |
| `step` | `job_id`, `number`, `name`, `status`, `conclusion`, `started_at`, `completed_at` |
| `test_case` | `suite`, `name`, `classname`, `file`, `status`, `duration_ms`, `failure_message`, `failure_body`, `source_report` |
| `failure` | `kind`, `signature`, `confidence`, `summary`, `evidence_refs`, `first_observed_in_bundle` |
| `annotation` | `check_run_id`, `path`, `start_line`, `end_line`, `level`, `message`, `raw_url` |
| `artifact` | `id`, `name`, `size_bytes`, `digest`, `expires_at`, `downloaded`, `local_path` |

The first bundle does not need a database. JSON and Markdown are enough until
the schema proves itself against real failing runs.

## Failure Fingerprinting

The MVP should compute deterministic failure signatures before asking an LLM for
anything.

Useful initial signatures:

| Failure source | Signature material |
| --- | --- |
| JUnit XML failure | normalized `classname`, `name`, failure element type, first useful stack frame, normalized message |
| Go test JSON | package, test name, fail action, normalized output window |
| Step failure | job name, step name, command line if available, exit code pattern, last relevant log block |
| Compiler/linter annotation | tool/check name, path, rule/code if available, normalized message |
| Infrastructure failure | runner label, setup step, known transient log phrases, timeout/cancel status |

Each signature should keep references to raw evidence, not just the hash. The
goal is to let a human or agent say: "this was grouped because these exact lines
match."

## Redaction and Data Safety

CI logs and artifacts can contain secrets. Parallax should assume GitHub's own
masking is helpful but incomplete:

- GitHub tells users to mask non-secret sensitive values with `::add-mask::`.
- GitHub warns that large-secret workarounds may not be redacted if printed.
- Artifacts can contain arbitrary build output and should be treated as
  sensitive by default.

MVP behavior should be conservative:

1. Default to bounded log excerpts, not full logs.
2. Record skipped files and reasons.
3. Redact common token, key, password, and credential patterns.
4. Allow `--include-raw-logs` only as an explicit option.
5. Keep bundles local unless the user uploads them.

Source:

- [GitHub Actions - using secrets](https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/use-secrets)

## What Not To Build Yet

Avoid these in the first implementation:

- Hosted backend.
- Dashboard UI.
- GreptimeDB or ClickHouse dependency.
- Cross-CI abstraction beyond naming the provider interface.
- Automatic PR generation.
- Test quarantine.
- Full flaky-test classifier.
- Production logs, traces, and metrics ingestion.
- Long-term historical analytics.

These are plausible later, but they will slow the first validation loop. The
first proof is whether a bundle makes a failed run easier to debug.

## Validation Questions

The next research and prototype work should answer:

1. Can Parallax reconstruct a useful bundle from a public failed GitHub Actions
   run using only API access?
2. How often do real projects upload machine-readable test reports as
   artifacts?
3. Which artifact names and paths are common enough to auto-detect?
4. Does a bounded log excerpt preserve enough evidence for an agent to act?
5. Which redaction false positives make bundles less useful?
6. Does the normalized schema fit Python, Java/JVM, Go, and JavaScript projects
   without becoming too generic?
7. Is the GitHub Action wrapper more valuable than post-run CLI collection for
   early users?

## Recommended Next Step

Build a tiny schema-and-sample fixture before writing a full CLI:

1. Pick one failed GitHub Actions run from an open-source project with uploaded
   JUnit XML or Go JSON output.
2. Manually assemble `manifest.json`, `summary.md`, `test_cases.jsonl`, and one
   log excerpt.
3. Use that bundle as the acceptance fixture for the first collector.
4. Only then implement `parallax gha collect`.

This sequence keeps the product honest: the bundle format should be judged by
whether it improves debugging, not by how much infrastructure it can ingest.
