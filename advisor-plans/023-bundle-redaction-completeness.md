# Plan 023: Route issue title, culprit, and run command through redaction before they enter an evidence bundle

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 8bc3f13..HEAD -- crates/parallax-core/src/bundle.rs crates/parallax-server/tests/m2_bundle.rs`
> If either file changed, compare the "Current state" excerpts against the
> live code before proceeding; on a mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: S
- **Risk**: MED
- **Depends on**: none
- **Category**: security
- **Planned at**: commit `8bc3f13`, 2026-07-07

## Why this matters

The evidence bundle is Parallax's agent-facing artifact and it advertises a
redaction policy (`redaction-lite-v1`). But redaction is applied only to the
error message, stacktrace, `db.query.text`, and correlated log bodies. The
issue **title** and **culprit**, and the run **command**, are projected raw.
The title is built from the raw error-message first line at ingest, so a
secret in an error message — the single highest-signal place secrets appear
(auth failures echoing tokens, DSNs, `password=…`) — is redacted in the
`latest_event.message` field yet leaked verbatim in the bundle title and in
the markdown `# {title}` heading. The run command routinely carries
`--token=…` / `PGPASSWORD=… psql`. Every bundle is exported over GraphQL, to
the CLI, and to the clipboard, so this defeats the redaction guarantee for
exactly the fields most likely to contain secrets.

## Current state

- `crates/parallax-core/src/bundle.rs:220-232` — the redactor:

  ```rust
  fn redact(text: &str, report: &mut RedactionReport) -> String { … }
  ```

- `crates/parallax-core/src/bundle.rs:265-276` — `issue_summary` copies
  `title` and `culprit` **verbatim** (no `redact`):

  ```rust
  fn issue_summary(issue: &Issue) -> IssueSummary {
      IssueSummary {
          title: issue.title.clone(),
          error_type: issue.error_type.clone(),
          culprit: issue.culprit.clone(),
          ...
  ```

- `crates/parallax-core/src/bundle.rs:307-315` — the run section copies
  `command` verbatim:

  ```rust
  Some(RunSection {
      run_id: run.run_id.clone(),
      command: run.command.clone(),
      ...
  ```

- Contrast: `bundle.rs:337-343` **does** redact the message and stacktrace
  via `redact(&event.message, &mut redaction)`.
- `crates/parallax-core/src/bundle.rs:603-710` — `to_markdown` prints the
  title as `# {title}` and the command as `` - command: `{command}` `` from
  the already-projected (currently un-redacted) section fields, so fixing the
  projection fixes the markdown too.
- Redaction rules today (`bundle.rs:195-218`): AKIA key, `Bearer …`,
  `password=…`, email only.
- Existing test `crates/parallax-server/tests/m2_bundle.rs:69-72` places its
  canary in an **INFO log body** (the already-redacted path), so it does not
  catch this class.
- Repo conventions: zero clippy warnings; cargo-nextest; DCO signoff.

## Commands you will need

| Purpose | Command (repo root)                                                  | Expected            |
|---------|----------------------------------------------------------------------|---------------------|
| Format  | `rtk cargo fmt --all`                                                | exit 0              |
| Lint    | `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0, no warnings |
| Tests   | `rtk cargo nextest run --workspace`                                  | all pass            |

## Scope

**In scope**:
- `crates/parallax-core/src/bundle.rs`
- `crates/parallax-server/tests/m2_bundle.rs` (add cases)

**Out of scope**:
- The redaction **rule set** itself (new regex shapes) — that is plan 025.
  This plan only routes existing fields through the existing redactor.
- The stored issue title in Turso metadata (`crates/parallax-storage/src/
  metadata.rs`, `crates/parallax-server/src/worker.rs`) — see Maintenance
  notes; changing stored data is a separate, higher-risk decision.
- Any change to `redact()`'s signature beyond what Step 1 needs.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: Redact title and culprit in `issue_summary`

`issue_summary` (bundle.rs:265) currently takes only `&Issue`. It needs the
`RedactionReport` to record hits. Change its signature to
`fn issue_summary(issue: &Issue, report: &mut RedactionReport) -> IssueSummary`
and wrap `title` and `culprit`:

```rust
title: redact(&issue.title, report),
culprit: redact(&issue.culprit, report),
```

Update **every** call site of `issue_summary` to thread the report. Call
sites (verify with `grep -n issue_summary crates/parallax-core/src/bundle.rs`):
the `RunSection.issues` map (bundle.rs:314) and the `Bundle.issue` projection
(bundle.rs:425). Both are inside `assemble`, where the local `redaction`
report already exists — pass `&mut redaction`.

Note the borrow ordering: `bundle.rs:314` builds `RunSection` inside the
`match &inputs.anchor` that also produces `redaction`. Confirm `redaction` is
declared before the match (`bundle.rs:279`) — it is — so `&mut redaction` is
available. If the borrow checker complains about the `.map(issue_summary)`
closure, replace it with an explicit loop that calls
`issue_summary(i, &mut redaction)`.

**Verify**: `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0.

### Step 2: Redact the run command in the run section

At `bundle.rs:307-315`, wrap the command:

```rust
command: run.command.as_deref().map(|c| redact(c, &mut redaction)),
```

(`RunRecord.command` is `Option<String>` — `crates/parallax-storage/src/
model.rs:141`.)

**Verify**: `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0.

### Step 3: Prove it with tests that put the canary where the leak was

In `crates/parallax-server/tests/m2_bundle.rs`, add cases (model them on the
existing bundle test in the same file):

1. An issue whose `title` contains `Bearer abcdef123456` → assert the bundle
   JSON and `markdown` contain `[REDACTED:bearer_token]` and **not** the raw
   token. Use the public assembly/GraphQL path the existing test uses.
2. An issue whose message (and thus title) contains `password=hunter2` →
   assert both the `latest_event.message` **and** the title are redacted
   (the whole point: no field leaks what another redacts).
3. A run-anchored bundle whose `command` contains `--token=SECRETVALUE123`
   → assert `command` in the bundle is redacted.

Do NOT hardcode a real credential; use obviously-fake canaries as above (the
existing test uses the public AWS doc example key — follow that convention).

**Verify**: `rtk cargo nextest run --workspace` → all pass, including the 3
new cases.

## Test plan

- New tests: title-with-token, message+title parity, command-with-token
  (Step 3), in `crates/parallax-server/tests/m2_bundle.rs`.
- Structural pattern: the existing bundle integration test already in that
  file (fixtures → assemble/query → assert on JSON + markdown).
- Verification: `rtk cargo nextest run --workspace` → all pass.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `rtk cargo fmt --all` no diff; clippy exits 0 with `-D warnings`
- [ ] `rtk cargo nextest run --workspace` exits 0 with 3 new cases present
- [ ] `grep -n "issue.title.clone()" crates/parallax-core/src/bundle.rs`
      returns nothing (title no longer copied raw)
- [ ] `grep -n "run.command.clone()" crates/parallax-core/src/bundle.rs`
      returns nothing
- [ ] No files outside the in-scope list modified (`git status`)
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report if:

- The excerpts don't match live code (drift).
- Threading `&mut redaction` into `issue_summary` creates a borrow conflict
  you cannot resolve with the explicit-loop workaround in Step 1.
- You discover the title is *also* consumed by a non-bundle path that now
  double-redacts in a way a test asserts against — report it.

## Maintenance notes

- **Deferred root cause (named):** the structural enabling condition is that
  redaction happens at *projection* time per-field, so any newly projected
  field is un-redacted by default. The robust fix is a redaction pass over
  the whole serialized bundle (or a typed "sensitive string" wrapper that
  cannot be serialized without redaction). That is a larger design change;
  this plan patches the three known-leaking fields and adds regression tests
  so the class is at least covered where it bites hardest.
- The **stored** issue title in Turso is still raw (only the bundle
  projection is cleaned). A follow-up may redact before
  `upsert_issue_occurrence` (`worker.rs`) so metadata-at-rest is clean too —
  deferred here because it changes stored data and needs its own migration
  decision. Note it in the README "considered" section.
- Operationally: recommend rotating any credential known to have transited an
  error message or a wrapped run command before this landed.
- Plan 025 broadens the rule set (DSN userinfo, PEM, token prefixes); once it
  lands, these same fields benefit automatically.
