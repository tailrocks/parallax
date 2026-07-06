# Plan 019: Fingerprint normalization v2 — stop one recurring failure from becoming N issues

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat f7f2c17..HEAD -- crates/parallax-core/src/fingerprint.rs crates/parallax-core/src/derive.rs crates/parallax-storage/src/metadata.rs`
> On excerpt mismatch below, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: none (coordinates with plan 016 issues-redesign UI; backend-only here)
- **Category**: bug
- **Planned at**: commit `f7f2c17`, 2026-07-03

## Why this matters

Live data showed one recurring jackin capsule-attach failure rendered as **14 separate open issues**, each with `EVENTS=1` — grouping is defeated, the issues list reads as 14 unique incidents, and recurrence counting (the whole point of issues) is lost. Two code-level causes in the fingerprint:

1. `top_frame` is hashed **raw** — any line number, path, or dynamic token in the first stacktrace line splits the group.
2. `normalize_message` only collapses digits, ≥16-char hex, and UUIDs — non-numeric dynamic tokens survive: container names like `jk-qfrehkbv-holla-thearchitect`, short hex ids, branch names, user ids like `501:20`, and free-form command text each mint a distinct fingerprint.

jackin is separately being fixed to send stable bodies plus structured fields (`error.type`, `jackin.operation`, container name as an attribute) — but Parallax must group correctly for *any* OTel producer, and prefer structured fields when present.

## Current state

`crates/parallax-core/src/fingerprint.rs` (verified firsthand at `f7f2c17`):

```rust
// :13-33 normalizers(): [uuid → <uuid>, [0-9a-fA-F]{16,} → <hex>, \d+ → <n>, \s+ → " "]
// :36-42
pub fn normalize_message(message: &str) -> String { /* applies normalizers */ }
// :45-51
pub fn top_frame(stacktrace: Option<&str>) -> String {
    stacktrace.and_then(|s| s.lines().next()).unwrap_or("").trim().to_string()
}
// :54-63
pub fn fingerprint(error_type: &str, message: &str, stacktrace: Option<&str>) -> String {
    // sha256(error_type \0 normalize_message(message) \0 top_frame(stacktrace)) → 16 hex
}
```

- Derivation feeding it: `crates/parallax-core/src/derive.rs:60` and `:116` (error events derived from spans/logs; check what it passes as `error_type`/`message`/`stacktrace` and which log/span attributes are available at that point — you need this for Step 3).
- Issue storage keyed by fingerprint: `crates/parallax-storage/src/metadata.rs:136` `upsert_issue_occurrence`; per-issue tag cache `merge_tags` `metadata.rs:85` (drops `exception.*`, caps 16 keys × 8 values × 64 chars).
- Tests: inline `#[cfg(test)]` in `fingerprint.rs` (`:65+`, e.g. `volatile_tokens_group_together` — `cache-7` vs `cache-9` collapse because `-7`→`<n>`; alpha-suffixed hosts would NOT).
- Existing rows: `issues` (Turso) and `error_events` (GreptimeDB) store historical fingerprints; changing the algorithm forks history.
- Conventions (repo AGENTS.md, verified by prior recon): work on `main`; Conventional Commits + `-s` + exactly one `Co-authored-by: Claude <noreply@anthropic.com>` trailer; `cargo nextest run --workspace`; clippy `-D warnings`; any new `TelemetryStore` method must be implemented in both `greptime.rs` and `memory.rs`; contract changes go to `docs/research/architecture/v1-implementation-spec.md` first.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| fmt/clippy | `rtk cargo fmt --all -- --check` ; `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| Tests | `rtk cargo nextest run --workspace --all-targets` | pass |

## Scope

**In scope**: `crates/parallax-core/src/fingerprint.rs`, `crates/parallax-core/src/derive.rs` (structured-field preference), `docs/research/architecture/v1-implementation-spec.md` (fingerprint contract note), tests in both files.

**Out of scope**: regrouping/migrating existing stored fingerprints (forward-only; see Step 4 decision), issues UI (plan 016 owns it), `metadata.rs` schema, ingest pipeline performance.

## Git workflow

- Work on `main` (repo convention — no PR flow). One commit per step, `git commit -s`, subject e.g. `fix(core): normalize top stack frame in fingerprints`, single Claude co-author trailer.

## Steps

### Step 1: Normalize the top frame

Add `fn normalize_frame(frame: &str) -> String`: strip trailing `:<line>[:<col>]`, collapse absolute-path segments to the last two components (`/very/long/path/src/payment.rs` → `src/payment.rs`), then run the message normalizers. Use it in `fingerprint()`:

```rust
hasher.update(normalize_frame(&top_frame(stacktrace)).as_bytes());
```

**Verify**: new tests — same function at `src/payment.rs:184` vs `:200` → equal fingerprints; different functions → different.

### Step 2: Broaden token normalization (surgical, not greedy)

Extend `normalizers()` with, in order BEFORE the digit rule:

- hex runs ≥ 6 (not 16) when delimiter-bounded: `\b[0-9a-f]{6,15}\b` → `<hex>` (lowercase-only to reduce false hits on words; UUIDs/long-hex already covered). Guard: must contain at least one digit (`(?=[0-9a-f]*[0-9])`) so words like `deadbe` in prose don't match — if look-ahead is unsupported by the `regex` crate (it is unsupported), implement as a replace callback checking `s.chars().any(|c| c.is_ascii_digit())`.
- container/slug tokens: `\bjk-[a-z0-9-]+\b` → `<container>` — jackin's documented container-name prefix; cheap and high-yield for the observed split. Keep it: Parallax's primary producer is jackin (repo docs `docs/guide/jackin.md`), and the pattern is harmless for others.
- `uid:gid` pairs `\b\d+:\d+\b` → `<uid>` (place before the bare-digit rule so it reads as one token).

Do NOT attempt generic hostname/branch normalization (over-merge risk). Property to preserve (add as tests): distinct `error_type`s never merge; distinct exception messages with no volatile tokens never merge.

**Verify**: new tests — messages differing only by `jk-…` container name, `501:0` vs `501:20`, or 8-hex id → equal; `redis::ConnectionTimeout` vs `redis::AuthError` → different. Existing tests still pass.

### Step 3: Prefer structured fields when the producer sends them

In `derive.rs` (both derivation sites `:60`, `:116`): when the source record carries an `error.type` attribute, use it as the fingerprint's `error_type` input (today's value — inspect what it currently uses; likely span status/exception type or a derived label). When a `jackin.operation` attribute is present, include it in the hash as a fourth component (extend `fingerprint` with `pub fn fingerprint_with_operation(error_type, message, stacktrace, operation: Option<&str>)` and keep the 3-arg fn delegating with `None` — additive, no call-site churn outside derive.rs). Structured fields are producer-stated identity; free-text normalization is the fallback.

**Verify**: derive-level test (follow the existing test pattern in `derive.rs` or `memory.rs` integration tests — locate with `rg -n "fn.*derive" crates/parallax-core/src/derive.rs` and mirror): two synthetic log records, same `error.type`+`jackin.operation`, different container names in body → one fingerprint.

### Step 4: Forward-only cutover note

Changing the hash regroups only NEW events; existing issues keep old fingerprints (history fork accepted — local single-tenant tool, 7–30 d TTLs age old rows out naturally). Record the decision + date in `docs/research/architecture/v1-implementation-spec.md`'s fingerprint section (spec-first convention), and in the commit body.

**Verify**: `rtk cargo nextest run --workspace --all-targets` → all green; fmt/clippy exit 0.

## Test plan

Named per step; all in `fingerprint.rs` inline tests + one derive-level test. Include the anti-merge properties (Step 2) and a fixture pair modeled on the real observed split (attach-failure message with two container names + differing uid) — synthesize the strings from the pattern, do not paste real tokens.

## Done criteria

- [ ] Frame normalization: line-number variance no longer splits (test)
- [ ] Container-name / uid / short-hex variance no longer splits (tests)
- [ ] Distinct error types still split (test)
- [ ] `error.type`/`jackin.operation` preferred when present (derive test)
- [ ] Spec doc updated; fmt/clippy/nextest green
- [ ] `advisor-plans/README.md` row updated (add plan 019 to the table)

## STOP conditions

- `derive.rs` has no access to record attributes at the derivation point (would require plumbing through `normalize.rs`/`model.rs` — report the required plumbing instead of doing it ad hoc).
- Any existing fingerprint test asserts a hash VALUE (not grouping behavior) — hashes will change; if a stored-value contract exists (e.g. m-series integration test fixtures with baked fingerprints), list them and STOP for a decision.
- The regex crate's lack of look-around forces a normalizer shape that mass-matches prose in existing tests.

## Maintenance notes

- jackin's plan 006 (stable bodies + `error.type` + `jackin.operation` attributes) makes Step 3 the dominant path for jackin traffic; the text normalizers remain the safety net for other producers.
- Plan 016 (issues UI redesign) should surface the fingerprint inputs (`error.type`, operation) as facets — coordinate.
- Over-merge is the failure mode to watch post-land: if two genuinely different failures group, tighten by adding a discriminating structured field, not by weakening normalizers.
