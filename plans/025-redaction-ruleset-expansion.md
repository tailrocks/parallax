# Plan 025: Expand the evidence-bundle redaction rule set to cover DSN userinfo, private keys, and common token shapes

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 8bc3f13..HEAD -- crates/parallax-core/src/bundle.rs`
> On excerpt mismatch, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW-MED
- **Depends on**: plans/023-bundle-redaction-completeness.md (land 023 first so
  the new fields are already routed through `redact`)
- **Category**: security
- **Planned at**: commit `8bc3f13`, 2026-07-07

## Why this matters

`redaction-lite-v1` covers only four secret shapes — AWS access-key id,
`Bearer …`, `password=…`, and email. It misses the highest-value shapes that
appear in telemetry that Parallax actually projects: connection-string
userinfo (`scheme://user:pass@host`, realistic in `db.query.text` which the
bundle already projects), PEM private-key blocks, and common token prefixes
(GitHub `ghp_`/`github_pat_`, Slack `xox[bap]-`, JWTs). The `password=…` rule
does not match `://user:pass@`, so DB connection strings leak today. The
policy string self-labels `pre-A6`, so partial coverage is acknowledged, but
the gap is live in every exported bundle.

## Current state

- `crates/parallax-core/src/bundle.rs:195-218` — `redaction_rules()` returns
  the four rules. Each is `("<name>", Regex)`; `redact` (bundle.rs:220-232)
  applies each in order, replacing with `[REDACTED:<name>]` and counting hits
  in `RedactionReport`.
- `crates/parallax-core/src/bundle.rs:280` — policy string
  `"redaction-lite-v1 (pre-A6)"`.
- The `regex` crate is already a dependency (used at bundle.rs:201). It does
  **not** support backreferences or lookaround — write rules accordingly.
- Repo conventions: zero clippy warnings; cargo-nextest; DCO signoff.

## Commands you will need

| Purpose | Command (repo root)                                                  | Expected |
|---------|----------------------------------------------------------------------|----------|
| Format  | `rtk cargo fmt --all`                                                | exit 0   |
| Lint    | `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0   |
| Tests   | `rtk cargo nextest run --workspace`                                  | all pass |

## Scope

**In scope**:
- `crates/parallax-core/src/bundle.rs` (rules + a `#[cfg(test)]` module)

**Out of scope**:
- Which fields get redacted (that is plan 023). This plan only widens the
  rule set that `redact` applies.
- Changing the redaction architecture (whole-bundle pass) — deferred, noted
  in 023.
- Bumping the policy label to a new milestone name unless Step 4 says so.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one agent trailer. Push when
  done.

## Steps

### Step 1: Add DSN userinfo redaction

Add a rule that masks the `user:pass@` segment of a URL without destroying the
scheme/host (useful context stays). Because `regex` has no lookaround, match
the whole userinfo and reconstruct is not possible via backrefs in
`replace_all`; instead match `://[^/\s:@]+:[^/\s@]+@` and replace with
`://[REDACTED:dsn_userinfo]@`:

```rust
(
    "dsn_userinfo",
    Regex::new(r"://[^/\s:@]+:[^/\s@]+@").expect("static regex"),
),
```

Note: the replacement string is fixed `[REDACTED:...]` today. To keep the
`://` and `@`, either (a) special-case this rule's replacement, or (b) change
`redact` to let each rule carry its own replacement template. Prefer (b): make
`redaction_rules()` return `(&str name, Regex, &str replacement)` and thread
the replacement through `redact`. That keeps future rules flexible.

### Step 2: Add PEM private-key block redaction

```rust
(
    "private_key_block",
    Regex::new(r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----")
        .expect("static regex"),
),
```

`(?s)` lets `.` match newlines; the non-greedy `.*?` bounds it to one block.

### Step 3: Add common token-prefix rules

```rust
("github_token", Regex::new(r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{20,}\b").expect("static regex")),
("github_pat",   Regex::new(r"\bgithub_pat_[A-Za-z0-9_]{20,}\b").expect("static regex")),
("slack_token",  Regex::new(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b").expect("static regex")),
("jwt",          Regex::new(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b").expect("static regex")),
```

Keep the existing four rules. Order the DSN and PEM rules **before** the
`password_assignment` rule so a DSN isn't half-masked by the narrower rule.

### Step 4: Bump the policy label

Change `bundle.rs:280` policy string from `"redaction-lite-v1 (pre-A6)"` to
`"redaction-lite-v2"` (the coverage materially changed, so consumers can tell
which policy produced a report). If a canonical-hash-stability plan (027) has
already landed and excludes the policy string from the hash, this is safe;
if not, note that this label change will shift bundle hashes for callers that
still include it — that is acceptable because the redaction output itself
changed.

### Step 5: Tests

Add a `#[cfg(test)]` module in `bundle.rs` (or extend an existing one) that
calls `redact` directly on strings and asserts:

- `postgres://admin:s3cr3t@db:5432/app` → contains `[REDACTED:dsn_userinfo]`,
  no `s3cr3t`, still contains `postgres://` and `@db:5432`.
- A PEM block → replaced wholesale, no `BEGIN`/key body remains.
- `ghp_` + 30 chars, a `xoxb-…`, and a 3-part `eyJ…` JWT → each redacted.
- A benign string with a colon-in-URL but no userinfo
  (`https://example.com/path`) is **unchanged** (no false positive).
- The `RedactionReport` counts reflect the hits.

Use only fake canaries. Do not reproduce any real secret.

**Verify**: `rtk cargo nextest run --workspace` → all pass with the new cases.

## Test plan

Covered in Step 5 (unit tests on `redact`). Add one integration assertion in
`crates/parallax-server/tests/m2_bundle.rs` that a bundle whose `db.query.text`
span attribute contains a DSN comes back with the userinfo redacted — this
proves the rule reaches the real projection path (23 routes `db.query.text`
through `redact` already at bundle.rs:375-379).

## Done criteria

- [ ] `rtk cargo fmt --all` no diff; clippy exits 0 with `-D warnings`
- [ ] `rtk cargo nextest run --workspace` exits 0 with new cases present
- [ ] `grep -c "Regex::new" crates/parallax-core/src/bundle.rs` increased by
      the number of rules added (≥ 6 rules total now)
- [ ] No false positive on `https://example.com/path` (asserted by test)
- [ ] No out-of-scope files modified (`git status`)
- [ ] `plans/README.md` status row updated

## STOP conditions

Stop and report if:

- The excerpts don't match live code (drift).
- A new regex causes `regex::Regex::new` to fail to compile (unsupported
  feature) — simplify, don't add a new crate.
- Broadening a rule redacts a benign existing test fixture — that means the
  rule is too greedy; tighten it, and report which fixture.

## Maintenance notes

- **Deferred root cause:** rule-by-rule regex redaction is inherently
  incomplete; the durable fix is structural (typed sensitive fields / a
  denylist-plus-entropy detector). Track against the A6 redaction milestone.
- Reviewer should scrutinize regex greediness (especially the PEM `(?s).*?`
  and DSN userinfo) for catastrophic-backtracking or over-masking.
- Any new field projected into the bundle in future work benefits from these
  rules automatically **only if** it is routed through `redact` (plan 023's
  lesson).
