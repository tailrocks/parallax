# Phase 0 Telemetry Overlay Contract

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The A1 bundle-value gate depends on telemetry-augmented coding tasks. Existing
notes define the task sources and the paired agent-eval arms, but "add a
telemetry overlay" is still too loose. A loose overlay can accidentally leak the
gold patch, create unrealistically perfect traces, or let the Parallax bundle
win because a human curated the evidence after seeing the task.

This note defines the contract:

> The Phase 0 overlay is a frozen, provenance-labeled eval artifact generated
> before any agent run. The raw dump and Parallax bundle must be derived from
> the same normalized overlay bytes, with no gold patch, no LLM-authored
> evidence, and explicit missing-evidence labels for anything not observed.

The goal is not to simulate production perfectly. The goal is to test whether a
bounded, structured bundle beats a raw dump under honest conditions.
The eval result must then be published through the
[A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md)
so the run has model snapshots, contamination tiers, result rows, and an expiry
date. The current public dataset SHA and feature snapshots that feed this
contract are recorded in
[A1 task source freeze check](a1-task-source-freeze-check.md). The companion
[A1 source drift and leakage recheck](a1-source-drift-and-leakage-recheck.md)
adds a stricter rule: datasets-server `first-rows` previews are not row-hash
evidence when they report `truncated=true`. The concrete pinned-row hash method
is defined in
[A1 Hugging Face row hash procedure](a1-huggingface-row-hash-procedure.md).

## Current Primary-Source Checks

| Source | What it shows | Parallax implication |
| --- | --- | --- |
| [SWE-bench dataset docs](https://www.swebench.com/SWE-bench/guides/datasets/) | Task instances carry repo, issue URL, PR URL, base commit, gold patch, test patch, fail-to-pass tests, and pass-to-pass tests. | The overlay harness must treat `patch` and `test_patch` as private grader inputs, not context-builder inputs. |
| [SWE-bench evaluation guide](https://www.swebench.com/SWE-bench/guides/evaluation/) | Evaluation applies generated patches in a containerized Docker environment and reports per-instance resolved results and logs. | Phase 0 should reuse the containerized pre-fix workspace model, but add telemetry capture around the failing command and agent run. |
| [SWE-bench Docker setup](https://www.swebench.com/SWE-bench/guides/docker_setup/) | SWE-bench isolation has real resource cost: large disk needs, cache levels, and worker-count tradeoffs. | The overlay runbook must record container image/cache/resource settings because telemetry differences can come from harness resource pressure. |
| [SWE-bench-Live MultiLang](https://huggingface.co/datasets/SWE-bench-Live/MultiLang) | Current rows expose fields beyond the issue/fix/test minimum: `patch`, `test_patch`, `problem_statement`, `hints_text`, `all_hints_text`, `commit_urls`, `commit_url`, rebuild/test/print commands, `log_parser`, fail-to-pass/pass-to-pass tests, and `docker_image`. | The overlay must use a source-field policy. Runner fields can drive the harness, but hints, resolving commit URLs, patches, test patches, and verifier IDs cannot silently become agent context. |
| [SWE-bench-Live OS-bench](https://huggingface.co/datasets/SWE-bench-Live/OS-bench) and [Windows](https://huggingface.co/datasets/SWE-bench-Live/Windows) | Current public platform slices expose patch/test fields, Docker images, rebuild/test/print commands, log parsers, platform/language metadata, hints, commit URLs, and verifier lists. The Windows viewer currently shows 61 rows across eight language values. | OS/CLI/platform tasks are useful, but generated statements, parser bodies, platform labels, hints, commit URLs, and patch/test metadata are contamination-sensitive. Freeze dataset revision and use them for harness/grading only unless an allowlist explicitly marks a field agent-visible. |
| [SWE-rebench V2](https://huggingface.co/datasets/nebius/SWE-rebench-V2) and [SWE-rebench V2 PRs](https://huggingface.co/datasets/nebius/SWE-rebench-V2-PRs) | Current rows include scale-friendly fields such as `install_config`, `interface`, `pr_description`, `problem_statement`, `meta`, `llm_metadata`, patches, test patches, and fail/pass test IDs. The PR-scale set exposes long `hints_text` and generated task metadata. The Nebius org also exposes trajectory and leaderboard/result datasets. | Large automatically collected corpora require stricter field-level quarantine than small curated tasks. LLM metadata, generated interfaces, hints, and PR descriptions must be audited before use and should not feed hypotheses by default. Trajectory and leaderboard/result datasets are excluded from task-source use unless the experiment is explicitly about contamination. |
| [Terminal-Bench](https://www.tbench.ai/) | Terminal-Bench measures agents in terminal environments and publishes task/verifier style examples; it also includes a public canary warning against training contamination. | Parallax Phase 0 should borrow the task/verifier discipline and include contamination/leakage canaries in committed artifacts. |
| [OpenTelemetry semantic conventions 1.41.0](https://opentelemetry.io/docs/specs/semconv/) | OTel provides common semantic attributes across traces, metrics, logs, events, CLI, CI/CD, process, exception, and VCS domains. | The overlay should use OTel names where possible, while marking development-status conventions as provisional. |
| [OpenTelemetry CLI spans](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) | CLI span conventions require executable name, exit code, PID, and `error.type` for non-zero exits; command args are recommended but should not be collected by default without sanitization. | Phase 0 command telemetry must capture exit/error status and sanitized args, never raw argv by default. |
| [OpenTelemetry CI/CD spans](https://opentelemetry.io/docs/specs/semconv/cicd/cicd-spans/) | CI/CD spans model pipeline runs and task runs with results such as success, failure, timeout, skipped, cancellation, and error. | The failing test or command wrapper should produce CI-like task spans even when the source task is not from a real CI run. |
| [OpenTelemetry logs data model](https://opentelemetry.io/docs/specs/otel/logs/data-model/) | Logs have timestamps, observed timestamps, trace/span fields, severity, body, resource, scope, attributes, and event names; exception attributes should follow exception semantic conventions. | Raw stdout/stderr should be normalized as log records with line refs, severity guesses, and trace/span IDs when available. |
| [OpenTelemetry exception logs](https://opentelemetry.io/docs/specs/semconv/exceptions/exceptions-logs/) and [exception spans](https://opentelemetry.io/docs/specs/semconv/exceptions/exceptions-spans/) | Exceptions can be represented as events/logs associated with span context, with standardized exception attributes. | Reconstructed exceptions must be labeled as reconstructed and must not be upgraded to SDK-observed production events. |
| [Sentry Python envelope source docs](https://getsentry.github.io/sentry-python/_modules/sentry_sdk/envelope.html) | The official SDK envelope implementation maps item types like event, transaction, span, attachment, log, and profile, and notes documented envelope constraints. | For Phase 0, emit a minimal Sentry-style error event object and point exact envelope compatibility to the Sentry fixture gate rather than rebuilding Relay semantics in the eval harness. |
| [MCP tools specification `2025-11-25`](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) and [RFC 8785 JCS](https://www.rfc-editor.org/rfc/rfc8785.html) | MCP tools can return JSON `structuredContent` validated by `outputSchema`; JCS gives a deterministic JSON form for hashing. | Arm C/D must be generated as canonical bundle JSON first, then rendered to Markdown/CLI/HTTP/MCP projections with recorded hashes. Text-only context cannot prove bundle value. |

## Decision

The overlay is an **eval fixture**, not a product pipeline:

| Question | Decision |
| --- | --- |
| Is it allowed to be semi-synthetic? | Yes, but every record carries provenance. |
| Can it use the gold patch or test patch? | No. Gold artifacts are private grader inputs only. |
| Can an LLM summarize logs before the bundle is built? | No. Overlay generation and truncation are deterministic. |
| Can raw dump and bundle use different evidence? | No. They must derive from the same normalized overlay artifact. |
| Can Arm C be only a hand-written Markdown summary? | No. The bundle JSON is canonical; Markdown and MCP text are projections with hashes. |
| Can perfect trace IDs be invented? | Only as harness span IDs with `observed_from_harness`; never claim production-grade trace linkage. |
| Can missing production telemetry be hidden? | No. Missing spans, SDK events, deploys, frontend breadcrumbs, or metrics appear in `missing_evidence`. |
| Can Hugging Face `first-rows` preview rows be used as source row hashes? | No. Checked previews returned `truncated=true`; selected rows must be fetched in full from pinned dataset revisions before hashing and field separation. |

## Source Field Policy

Modern SWE task rows mix fields with very different roles: issue text, runner
commands, hidden verifier IDs, gold patches, generated hints, and metadata
produced by LLM-based filters can all live in one dataset record. Phase 0 must
therefore produce an explicit source-field policy before any arm artifact is
generated.

Use these zones:

| Zone | Examples | Agent-visible? | Rule |
| --- | --- | --- | --- |
| `agent_visible_seed` | Reviewed issue title/body or `problem_statement`, base repo identity, public package version, failure anchor captured from the pre-fix run. | Yes | Only after a leakage review confirms the field does not contain gold patch text, resolving commit text, post-fix comments, or implementation-specific grader knowledge. |
| `runner_private` | `docker_image`, `rebuild_cmds`, `test_cmds`, `print_cmds`, `install_config`, `log_parser`, cache/resource settings. | No, except sanitized command identity in telemetry rows. | Allowed to run and parse the harness. The parser body and exact hidden test command internals are not agent context. |
| `grader_private` | `patch`, `test_patch`, `FAIL_TO_PASS`, `PASS_TO_PASS`, hidden verifier output, fixed commit hash, resolving PR/commit URLs. | No | Hash for audit, but keep content and direct resolving links out of Arm A/B/C. |
| `triage_private` | `hints_text`, `all_hints_text`, PR discussion text, `interface`, `meta`, `llm_metadata`, difficulty labels, model/filter quality flags. | No by default | May help select or reject tasks. If promoted to context, pre-register why and downgrade the contamination tier. |
| `public_audit` | Dataset name, source version/check date, task ID, repo, base commit, language, license, artifact hashes. | Yes in manifests; not necessarily in prompts. | Safe for committed audit artifacts, but agent prompts should receive only what the selected arm needs. |

For moving public datasets, `public_audit` also includes dataset revision,
source role, row-count snapshot, split-count snapshot, source checked timestamp,
`first_rows_truncated_observed`, selected-row fetch method, full selected-row
hash, agent-visible row hash, and source field policy hash. A task selected from
a mutable dataset cannot count toward A1 unless these values are frozen before
the agent run.

Every task directory must include `source-field-policy.json` or an equivalent
section in `provenance.md` that records, for each source field:

```json
{
  "field": "hints_text",
  "zone": "triage_private",
  "agent_visible": false,
  "reason": "May contain implementation hints or post-issue discussion.",
  "hash_recorded": true
}
```

This is now a no-cheat dependency. If the source-field policy is missing, the
task can debug the harness but cannot count toward A1.

The Phase 0 claim should read:

```text
The bundle beat/tied/lost against raw dump on provenance-labeled eval overlays.
```

It should not read:

```text
The bundle improves production incident fixes.
```

until real telemetry or fault-injected reference-app runs confirm the same
direction.

## Overlay Artifact Set

Each task gets a frozen overlay directory:

```text
docs/research/bundle-value-eval/tasks/<task_id>/
  source.json
  telemetry/
    raw.ndjson
    normalized.jsonl
    redaction-report.json
    refs/
      stdout.txt
      stderr.txt
      harness.log
  arm-a-context.md
  arm-b-raw-dump.md
  arm-c-bundle.json
  arm-c-bundle.md
  arm-c-projection-manifest.json
  arm-c-mcp-structured-content.json
  source-field-policy.json
  provenance.md
  hashes.sha256
  grader-private.sha256
```

Artifact meanings:

| Artifact | Public? | Meaning |
| --- | --- | --- |
| `source.json` | Yes | Dataset/task metadata, excluding gold patch/test patch contents. |
| `raw.ndjson` | Yes if redacted | Original captured records with minimal parsing and stable refs. |
| `normalized.jsonl` | Yes if redacted | Deterministic normalized rows used to build both Arm B and Arm C. |
| `redaction-report.json` | Yes | Policy version, seeded canaries, findings, and denied fields. |
| `refs/*.txt` | Usually yes if redacted | Bounded stdout/stderr/harness excerpts with line numbers. |
| `arm-b-raw-dump.md` | Yes | Token-budgeted raw dump generated from `normalized.jsonl`. |
| `arm-c-bundle.*` | Yes | Product-arm canonical bundle JSON and Markdown projection generated from the same `normalized.jsonl`. |
| `arm-c-projection-manifest.json` | Yes | Hashes proving Markdown, CLI, HTTP, and MCP projections derive from the same canonical bundle. |
| `arm-c-mcp-structured-content.json` | Yes when MCP is tested | Exact MCP `structuredContent` object validated against the bundle `outputSchema`. |
| `source-field-policy.json` | Yes | Field-level allow/deny policy separating agent-visible, runner-private, grader-private, triage-private, and audit-only fields. |
| `grader-private.sha256` | Yes | Hashes of gold patch/test patch/verifier-only files, not the files. |

Private raw artifacts can exist outside the repo, but committed public artifacts
must be enough to audit the evidence parity between Arm B and Arm C.

## Event Families

The overlay should produce these families when available:

| Family | Required fields | Notes |
| --- | --- | --- |
| `task_manifest` | `task_id`, `source`, `source_version`, `repo`, `base_commit`, `issue_url?`, `language`, `license_review`, `source_field_policy_ref` | No gold patch text or resolving PR/commit URL in agent-visible projections. |
| `test_run` | `command_ref`, `started_at`, `finished_at`, `exit_code`, `result`, `verifier_ref_hashes[]` | The harness can hash verifier IDs, not leak test patch content or exact hidden verifier names into agent context. |
| `process_span` | `span_id`, `parent_span_id?`, `executable`, `args_sanitized`, `cwd_ref`, `pid?`, `exit_code`, `error_type?`, `duration_ms` | Follow OTel CLI/process semantics; raw args are denied by default. |
| `ci_task_span` | `pipeline_name`, `task_name`, `task_run_id`, `result`, `error_type?`, `url_ref?` | Even a local failing command can be represented as one CI-like task. |
| `exception_event` | `error_type`, `message`, `stack_frames[]`, `handled?`, `trace_id?`, `span_id?`, `provenance` | `reconstructed_from_test_output` is weaker than SDK-observed. |
| `log_record` | `timestamp?`, `observed_timestamp`, `severity`, `body_ref`, `line_range`, `trace_id?`, `span_id?`, `attributes` | Preserve line refs instead of copying long output into bundles. |
| `metric_sample` | `name`, `value`, `unit`, `time_range`, `attributes` | Keep to timings, retry counts, memory/time limits unless task data supports more. |
| `release_marker` | `version`, `repo`, `base_commit`, `environment`, `source` | For public tasks, this is usually harness release context. |
| `change_marker` | `repo`, `base_commit`, `issue_url?`, `dataset_instance_id`, `fixed_patch_hash?`, `resolving_ref_hash?` | Patch hash and resolving PR/commit refs stay private-facing or grader-only unless pre-registered as public audit metadata. |
| `redaction_finding` | `policy_version`, `field`, `finding_type`, `action`, `canary?` | Seeded canary findings are mandatory. |

## Provenance Labels

Every record and every derived node carries one or more provenance labels:

| Label | Meaning | Claim strength |
| --- | --- | --- |
| `observed_from_sdk` | Produced by a real SDK or app instrumentation while the failure ran. | Strongest for runtime linkage. |
| `observed_from_harness` | Produced by the wrapper around the failing command. | Valid for eval behavior; weaker than production telemetry. |
| `reconstructed_from_test_output` | Parsed from stderr/stdout/test logs after the fact. | Useful, but do not call it observed runtime telemetry. |
| `synthetic_fault_injection` | Produced by a controlled reference app fault. | Good for known-cause cases; not wild-bug evidence. |
| `operator_private_real` | Comes from private real incidents. | Separate from public claims unless independently auditable. |
| `provider_backfilled` | Pulled from GitHub/Sentry/CI APIs after the run. | Valid metadata, but record observed/backfill time. |

Bundle hypotheses must cite provenance. A conclusion supported only by
`reconstructed_from_test_output` should be phrased as a harness-derived
hypothesis, not as production causality.

## No-Cheat Rules

These rules are part of the evaluation, not optional hygiene:

1. Write and freeze the source-field policy before reading source fields into
   any arm artifact.
2. Build `raw.ndjson` and `normalized.jsonl` before generating any agent arm.
3. Hash and freeze `normalized.jsonl` before creating Arm B or Arm C.
4. Generate Arm B first using a fixed truncation rule.
5. Generate Arm C canonical JSON from the same normalized rows and refs; no
   extra evidence.
6. Keep gold patch, test patch, resolving PR/commit URLs, hints, LLM metadata,
   parser source, hidden verifier IDs, and hidden verifier output out of all agent
   contexts.
7. Do not use an LLM to parse, summarize, rank, or redact overlay evidence.
8. Include seeded secrets/canaries in at least one safe field and prove they are
   absent from agent-visible files. Use the
   [A6 synthetic canary fixture corpus](../../capture/redaction.md)
   for fixture classes and public/private commit boundaries.
9. Preserve missing evidence instead of filling it with plausible defaults.
10. Commit artifact hashes and provenance labels for every public task.
11. Compute `schema_ref`, post-redaction `canonical_hash`, and
    `projection_manifest` before any agent run, and prove Markdown/CLI/HTTP/MCP
    projections match the canonical bundle.
12. If a human manually repairs a malformed record, record the edit in
    `provenance.md` before any agent run.

If any no-cheat rule fails, the task can be used for harness debugging but not
for the A1 decision gate.

## Normalized Row Example

```json
{
  "row_id": "ovl_01J...",
  "task_id": "swe-live-rust-example",
  "family": "process_span",
  "timestamp": "2026-05-25T16:42:12Z",
  "trace_id": "4c3a...",
  "span_id": "8b1d...",
  "parent_span_id": "root",
  "data": {
    "process.executable.name": "cargo",
    "process.command_args.redacted": ["cargo", "test", "<test-filter>"],
    "process.exit.code": 101,
    "error.type": "test_failure",
    "duration_ms": 18422
  },
  "refs": {
    "stderr": "refs/stderr.txt#L120-L184",
    "stdout": "refs/stdout.txt#L1-L40"
  },
  "provenance": ["observed_from_harness"],
  "redaction": {
    "policy": "phase0-overlay-v1",
    "raw_args_available": false
  }
}
```

## Arm Generation Rules

Arm A:

- issue title/body;
- top failure anchor;
- repo checkout instructions;
- no overlay except the minimal stack or failing-test output already allowed in
  the repo-only baseline.

Arm B:

- same `normalized.jsonl` as Arm C;
- raw-ish chronological dump;
- fixed token ceiling;
- deterministic truncation order:
  1. task/failure summary;
  2. Sentry-style event or exception event;
  3. same-trace process/CI spans;
  4. bounded stderr/stdout refs;
  5. logs by time;
  6. timings/metrics;
  7. release/change metadata;
  8. missing evidence.

Arm C:

- canonical evidence bundle from the same normalized rows;
- `schema_ref`, post-redaction `canonical_hash`, `projection_manifest`, and
  `access` fields;
- typed nodes/edges;
- edge strengths;
- missing evidence;
- redaction report;
- hypotheses only when the cited evidence supports them.
- Markdown is a projection of the canonical JSON, never the source of truth.
- MCP delivery, if tested, must expose the canonical object as
  `structuredContent` validated against the bundle `outputSchema`.

Arm D:

- Arm C without the ranked hypothesis block.

## Pass/Fail Gates

The overlay corpus is acceptable for Phase 0 only if:

| Gate | Pass condition |
| --- | --- |
| Evidence parity | 100 percent of Arm C evidence rows are present in Arm B's source overlay, even if Arm B truncates presentation. |
| Hash freeze | `normalized.jsonl`, Arm B, and Arm C hashes are recorded before agent runs. |
| Gold isolation | Zero gold patch/test patch lines appear in Arm A/B/C artifacts. |
| Source-field isolation | 100 percent of source fields are classified, and zero `runner_private`, `grader_private`, or default `triage_private` fields appear in agent-visible artifacts. |
| Redaction | Zero seeded secrets or PII canaries appear in agent-visible artifacts. |
| Canonical bundle | Arm C/D include `schema_ref`, post-redaction `canonical_hash`, `projection_manifest`, `access`, `redaction_report`, and `source_field_policy` in canonical JSON. |
| Projection equivalence | Markdown, CLI, HTTP, and MCP projections derive from the canonical bundle and carry matching hashes; MCP `structuredContent` validates against the bundle `outputSchema` when MCP is in the run. |
| Provenance coverage | 100 percent of records carry provenance labels. |
| Missing evidence | 100 percent of absent production-grade signals are represented as `missing_evidence`. |
| Reproducibility | Re-running capture on the same task produces equivalent normalized rows except timestamps, durations, and nondeterministic line ordering documented in `provenance.md`. |
| Token fairness | Arm B and Arm C stay within the pre-registered token ceiling. |

Failure consequences:

- If evidence parity fails, discard the task for A1.
- If gold isolation fails, discard the task and rotate the canary.
- If source-field isolation fails, discard the task for A1 and repair the field
  policy before reusing that source.
- If canonical bundle or projection equivalence fails, discard Arm C/D rows until
  bundle generation is repaired; the failure can debug the renderer but cannot
  count toward bundle value.
- If reproducibility fails, keep the task only as a harness-debugging case.
- If provenance is mostly reconstructed, report the task under
  `reconstructed_overlay`, not `runtime_overlay`.

## Implementation Order

1. Add a tiny Rust or shell-free Rust-wrapper capture tool later, but do not make
   it depend on the Parallax backend.
2. Start with process/CI spans, stdout/stderr logs, exception parsing, release
   metadata, and redaction reports.
3. Add real SDK/OTLP capture only for tasks where it can be done without
   changing the target fix.
4. Generate Arm B and Arm C from the same normalized rows in one command.
5. Only after Phase 0 has signal, move the same contract into automated
   Parallax bundle generation.

## Relationship To Other Research

- [Bundle-value evaluation](bundle-value-evaluation.md) defines A1 and the
  paired arms this overlay feeds.
- [Bundle-value seed corpus](bundle-value-seed-corpus.md) defines which tasks
  are eligible before overlay generation.
- [Bundle-value Phase 0 runbook](bundle-value-phase0-runbook.md) defines the
  first run matrix, scoring, and decision rules.
- [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md)
  defines how overlay hashes, no-cheat gates, and contamination labels are
  carried into the public A1 result artifact.
- [Evidence bundle and open schema](../../architecture/evidence-bundle-schema.md) defines Arm C
  once the overlay rows are transformed into nodes and edges.
- [Redaction pipeline and secret safety](../../capture/redaction.md)
  owns the policy used before artifacts become agent-visible.
- [Deploy, change, and issue-tracker context](../../capture/deploy-change-context.md)
  defines how release/change/work-item records should be labeled and downgraded
  when they are only harness metadata.

## Bottom Line

The Phase 0 overlay should be boring, deterministic, and hard to cheat. If a
Parallax bundle wins only because the overlay invented perfect links, hid
missing evidence, quietly curated better context than the raw dump, or exposed a
Markdown-only summary with no canonical bundle/projection proof, the eval is
worse than useless. Freeze one normalized, provenance-labeled overlay first;
derive raw and canonical bundled arms from it; then let the agent comparison
mean something.
