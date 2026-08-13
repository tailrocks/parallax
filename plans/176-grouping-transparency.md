# Plan 176: Grouping transparency — every issue explains why its events grouped, and users can steer it

> **Executor instructions**: Follow this plan step by step. Spec-first for
> the contract-changing half; Step 4 is gated on an operator decision. Run
> every verification command. On any "STOP conditions" item, stop and
> report.
>
> **Drift check (run first)**: `git diff --stat 7418bc9..HEAD -- crates/parallax-analysis/src/fingerprint.rs crates/parallax-analysis/src/derive.rs crates/parallax-api/src/resolvers/issues.rs docs/research/architecture/v1-implementation-spec.md ui/graphql/schema.graphql docs/guide/conventions.md`
> — on mismatch with the excerpts below, STOP.
>
> **Ratchet gate**: exact-match `ratchet.toml` rows per touched Rust file
> updated in the same commit (`cargo xtask policy --only structural`); new
> UI tests need `ui/test-matrix.json` entries
> (`cargo xtask policy --only ui.tests`).

## Status

- **Priority**: P2
- **Effort**: M (Step 1–3); Step 4 sized separately after approval
- **Risk**: LOW for explanation (read-only derivation); MED for Step 4
  (grouping is issue identity — changing it churns history)
- **Depends on**: none
- **Category**: direction
- **Planned at**: parallax `7418bc9`, 2026-08-14
- **Evidence base**: `docs/research/market/competitor-pain-points.md` —
  Sentry grouping opacity is a high-recurrence complaint class:
  over/under-grouping across releases (sentry#64354, #71630, #68355,
  discussion 66319), `<uuid>` normalization defeating manual fingerprints
  (sentry-java#3246); users fight an opaque server algorithm they can
  neither inspect nor reliably override.

## Why this matters

Grouping is the error-tracking product decision: it defines what an
"issue" IS. Sentry's users' recurring pain is not that grouping exists but
that it is *opaque* (why did these merge?) and *unsteerable* (my
fingerprint hint got normalized away). Parallax's fingerprinting is
already deterministic — a pure function over error type, normalized
message, and stack frame (`crates/parallax-analysis/src/fingerprint.rs:122`)
— which is exactly the property an explanation needs. Not exposing the
explanation wastes the architecture's own honesty: the user sees the same
black box Sentry shows, while the box is in fact glass. Correctness
framing: the product claims deterministic evidence with provenance; issue
identity is evidence, so its provenance belongs in the surface.

## Current state (verified)

- `crates/parallax-analysis/src/fingerprint.rs:122`
  `pub fn fingerprint(error_type, message, stacktrace) -> String` and
  `:127` `fingerprint_with_operation(...)` — deterministic; tests cover
  grouping and anti-grouping (`fingerprint/tests.rs`, 8 tests).
- Error derivation from exception spans + ERROR/FATAL logs in
  `crates/parallax-analysis/src/derive.rs` (one identity across sources —
  `derive/tests.rs:140`).
- Issue surface: `crates/parallax-api/src/resolvers/issues.rs` (list,
  detail, trend); UI issue detail shows stacktrace/tags/occurrences
  (`ui/src/features/issues/components/issue-detail-page.tsx`).
- No "why grouped" data crosses the API: the fingerprint function's
  INPUTS (which normalized message template, which frame, which operation)
  are computed and discarded — only the hash is stored.
- Contract homes: `docs/research/architecture/v1-implementation-spec.md`
  §8 (GraphQL), `docs/guide/conventions.md` (what users send; exception
  encodings).
- Guides drift warning: `docs/guide/` still carries `parallax run`
  wording (plan 166 owns the rename cleanup) — do not copy stale wording
  into new doc text.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Gates | `cargo xtask ci --fast && cargo xtask lint && cargo xtask test && cargo xtask arch && cargo xtask policy --only structural` | green |
| SDL | `cargo xtask ui graphql export && cargo xtask ui graphql check` | new field family only |
| Suites | `cargo nextest run -p parallax-analysis -p parallax-api -E 'test(/fingerprint|grouping/)'` | pass |
| Docs links | `cargo xtask docs links` | pass |

## Scope

**In scope**: `crates/parallax-analysis/` (return grouping-explanation
data alongside the hash), the derivation → storage path for the
explanation fields (occurrence rows or derivation-time projection —
Step 2 decides against the storage constraints),
`crates/parallax-api/src/resolvers/issues.rs` + SDL (explanation field),
issue-detail UI card, `docs/guide/` grouping documentation,
v1-implementation-spec §8, and (Step 4, gated) the fingerprint-rules
contract.

**Out of scope**: changing the default grouping algorithm's behavior (the
explanation must describe what IS, not redesign it); Sentry-envelope
grouping parity questions (multi-SDK ledger is plan-164 c8 / inventory
territory); merge/split of EXISTING issues' history (Step 4 defines
forward-only semantics precisely to avoid history rewrites).

## Git workflow

PR-only `main`; Steps 1–3 = one PR; Step 4 spec proposal = its own PR
awaiting operator approval; `git commit -s`; Conventional Commits; agent
trailer per `COMMITS.md`.

## Steps

### Step 1: Spec the explanation

v1-implementation-spec §8: `issue` gains `groupingExplanation` —
`{ algorithmVersion, errorType, messageTemplate (the normalized form
actually hashed, redaction-safe), anchorFrame (module/function, no source
lines beyond what issues already show), operation (when the
with-operation variant applied), inputsPresent (which of the three inputs
existed) }`. Read-only, derived deterministically; documented as "this is
WHY these events share an issue".

**Verify**: spec diff consistent with the fingerprint function's actual
signature; `cargo xtask docs links` pass.

### Step 2: Surface the explanation

Make the fingerprint path return the explanation struct alongside the
hash (pure refactor of `fingerprint.rs` return type or a sibling
`fingerprint_explained` used by derivation — pick the one that keeps every
existing call site compiling and the hash byte-identical: golden test
first). Persist per-issue (latest-event explanation on the occurrence
path, wherever the issue title/culprit already come from — follow that
exact flow in `derive.rs` → metadata writes). Resolver + SDL + UI card
("Grouped by: `TypeError` · template `connection to <host> refused` ·
frame `pool::acquire`") on issue detail near the fingerprint copy button.

**Verify**: golden test proves hashes unchanged for the existing corpus;
new tests: explanation matches inputs for span-derived, log-derived, and
Sentry-envelope-derived events (same identity ⇒ same explanation);
`cargo xtask ui graphql export && check`; UI unit test (matrix entry).

### Step 3: Document the algorithm

`docs/guide/` (extend `conventions.md` or a new `grouping.md`): the
deterministic algorithm in words — inputs, normalization rules (what gets
templated), the with-operation variant, `algorithmVersion` semantics, and
what users can do TODAY to influence grouping (shape `error.type` /
message; the conventions doc's exception-encoding rules). No overclaim:
document current behavior only.

**Verify**: every documented rule maps to a named test in
`fingerprint/tests.rs` (add the mapping as doc comments or a doc table);
`cargo xtask docs links` pass.

### Step 4 (operator-gated): User-steerable fingerprint rules

Spec PROPOSAL (own PR, no code until approved): per-service, declarative
grouping overrides — ordered rules (`match` on error type/message
template/frame → `group-by` extra keys or `split-by` a named attribute),
stored in Turso, applied FORWARD-ONLY at derivation time (existing issues
keep their identity; a rule change never rewrites history — the exact
failure Sentry's opaque re-grouping causes), rules visible in
`groupingExplanation` (`matchedRule: <name>`) so steering stays
transparent. Include the abuse case (rule explosion) via a bounded rule
count. STOP here until the operator approves the spec.

**Verify (post-approval, separate execution)**: rules round-trip via
GraphQL; forward-only proven by test (pre-rule events unmoved); explanation
names the matched rule.

## Test plan

Golden hash-stability corpus (Step 2, the load-bearing one); explanation
correctness per derivation source; UI card render; doc-rule → test
mapping. Step 4 adds forward-only + bounded-rules tests after approval.

## Done criteria

- [ ] Spec §8 carries `groupingExplanation`; SDL updated, drift-gated.
- [ ] Hashes byte-identical to pre-change (golden test in-tree).
- [ ] Issue detail shows the Grouped-by card; all three derivation sources
      tested for explanation fidelity.
- [ ] Grouping algorithm documented with rule→test mapping.
- [ ] Step-4 spec proposal PR open (or approved+scheduled) — plan not DONE
      while it's unwritten; plan IS done with it open and awaiting the
      operator.
- [ ] All gates green (`ci --fast`, lint, test, arch, structural, graphql,
      ui.tests).
- [ ] `plans/README.md` row updated.

## STOP conditions

1. Drift check fails.
2. The explanation cannot be persisted without a schema change AND plan
   169's versioned migrations have not landed — coordinate ordering,
   don't ad-hoc a PRAGMA sniff (the pattern 169 retires).
3. Any approach would change an existing fingerprint hash — identity churn
   is the Sentry failure this plan exists to avoid; STOP and redesign.
4. Redaction: if any explanation field could carry un-redacted message
   content beyond what issue titles already expose, route it through the
   same sanitize path first; if that's structurally unclear, STOP.
5. Step 4 without operator approval.

## Maintenance notes

- `algorithmVersion` is now load-bearing: any future normalization change
  must bump it and keep old explanations labeled with their version.
- The golden hash corpus is the regression net for ALL fingerprint work —
  grow it with every grouping bug report.
- Root-cause note: this plan removes the opacity condition (inputs
  discarded after hashing). The steering half (Step 4) removes the
  unsteerability condition but changes issue-identity contract space —
  hence the operator gate, mirroring plan 171's MCP-catalog gate.
