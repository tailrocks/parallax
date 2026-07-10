# Plan 082: Publish `bundle-v1` as a versioned JSON Schema with a conformance test

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- crates/parallax-core/src/bundle.rs`
> Plan 072 legitimately lands first and CHANGES the bundle (new redaction
> policy string, possibly new report keys). Author the schema against the
> post-072 shape; if 072 has not landed yet, STOP and sequence after it.

## Status

- **Priority**: P3 (direction)
- **Effort**: M (design-first: the schema is a compatibility commitment)
- **Risk**: LOW-MED (publishing invites external dependence; versioning
  policy must be stated at birth)
- **Depends on**: 072 (bundle shape settles), 081 (the CLI JSON output is the
  byte surface the schema governs)
- **Category**: direction
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

The go/no-go decision (`docs/research/decisions/go-no-go.md`) names "the open
evidence schema and portable bundle format as a standard others build on" as
one of Parallax's four compounding moats. The code ships
`SCHEMA_VERSION = "bundle-v1"` (`crates/parallax-core/src/bundle.rs:13`), but
the only machine-readable schemas in the repo are the FROZEN PoC artifacts
(`poc/evidence-loop/schema/evidence-bundle.v0-poc.schema.json`,
`fix-candidate.v0.schema.json`); the v1 contract exists only as prose
(`docs/research/architecture/evidence-bundle-schema.md`). A moat with no
shippable artifact is a claim. This plan derives the real `bundle-v1` JSON
Schema from the code, validates production output against it in CI, and
states the versioning policy.

## Current state

- `crates/parallax-core/src/bundle.rs`:
  - `:13` `pub const SCHEMA_VERSION: &str = "bundle-v1";`
  - The serialized shape = `#[derive(Serialize)] pub struct Bundle` (`:15+`)
    and its nested structs (`Anchor`, `IssueSummary`, `RunSection`,
    `EventDetail`, `TraceSection`, `SpanLine`, `MetricWindow`, `Hypothesis`,
    `RedactionReport` `:301-305`, `BoundReport` `:308-313`, missing-evidence
    strings). Enumerate EVERY `#[derive(Serialize)]` type reachable from
    `Bundle` — the schema must cover the whole closure.
  - Canonical JSON + `canonical_hash` produced in this file (find the
    serializer fn — grep `canonical`); the schema describes THOSE bytes.
- Exposure paths of the JSON: GraphQL `BundleOut.json`
  (`crates/parallax-api/src/lib.rs:1618-1637`) and, after Plan 081,
  `parallax issue context --format json`.
- PoC precedent to follow for file conventions:
  `poc/evidence-loop/schema/evidence-bundle.v0-poc.schema.json` (draft
  2020-12 JSON Schema; look at its `$id`/`$schema` header style). The PoC
  files stay frozen — do not edit them.
- Prose spec: `docs/research/architecture/evidence-bundle-schema.md` (33.6K)
  — read it BEFORE authoring; where the prose and the code disagree, the CODE
  is the shipped truth: record each divergence in the plan-execution report
  (do not silently edit the research doc; flag contradictions).
- No JSON Schema validation crate is currently in the workspace. For the
  conformance test add `jsonschema` (dev-dependency of `parallax-core` only,
  latest stable, via `[workspace.dependencies]` per repo convention). It is
  test-only, so the native-TLS rule is untouched (no network at test time —
  ensure `$ref` resolution is offline; the schema must be self-contained,
  no remote refs).

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Core tests | `rtk cargo nextest run -p parallax-core` | all pass |
| Full suite | `rtk cargo nextest run --workspace` | all pass |
| Schema is valid JSON | `python3 -m json.tool schema/evidence-bundle.v1.schema.json` | parses |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |

## Scope

**In scope** (the only files you should modify):
- `schema/evidence-bundle.v1.schema.json` (create — new top-level `schema/`
  dir)
- `schema/README.md` (create — versioning policy)
- `crates/parallax-core/src/bundle.rs` (conformance test only, in the test
  module)
- `crates/parallax-core/Cargo.toml` + root `Cargo.toml` (dev-dep `jsonschema`)
- `PROJECT_STRUCTURE.md` (add the `schema/` row — same-change rule)
- `README.md` (one link line under Start Here or Using It)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- `poc/evidence-loop/schema/*` — frozen concept reference.
- `docs/research/architecture/evidence-bundle-schema.md` — research record;
  divergences get REPORTED, not edited in.
- The bundle's runtime shape — this plan DESCRIBES, never changes, the bytes.
- A `parallax schema` CLI verb — nice later; not now.

## Git workflow

- Work directly on `main` (repo rule — `BRANCHING.md`).
- Conventional Commits, DCO signoff (`git commit -s`), trailer
  `Co-authored-by: Claude <noreply@anthropic.com>`. E.g.
  `feat(schema): publish evidence-bundle v1 json schema`.

## Steps

### Step 1: Derive the schema from the code

Author `schema/evidence-bundle.v1.schema.json` (JSON Schema draft 2020-12,
self-contained, `$id` like
`https://github.com/tailrocks/parallax/schema/evidence-bundle.v1.schema.json`)
by walking `Bundle`'s Serialize closure in `bundle.rs`. Rules:

- Field names exactly as serde emits them (check for `#[serde(rename)]` /
  `rename_all` attributes; absent those, Rust field names verbatim).
- `schema_version` is `"const": "bundle-v1"`.
- Nanosecond timestamps are STRINGS (they're `.to_string()`ed — e.g.
  `first_seen_nanos`); document `pattern: "^[0-9]+$"`.
- Optional fields (`Option<T>`): serde emits `null` unless
  `skip_serializing_if` says otherwise — check each attribute and encode
  nullability accurately.
- `redaction_report.policy`: enumerate known values
  (`redaction-lite-v2`, `redaction-lite-v3`) as examples, but type it as a
  plain string (policies will evolve).
- Mark `additionalProperties: false` ONLY where the prose spec promises a
  closed shape; otherwise leave open (schema evolution room) — state the
  choice per object in `schema/README.md`.

**Verify**: `python3 -m json.tool schema/evidence-bundle.v1.schema.json` →
parses.

### Step 2: Conformance test

In `bundle.rs`'s test module: build a representative bundle via `assemble`
(reuse the fixture style of the existing hash tests near `:965` — cover an
issue anchor WITH run section, trace section, metric windows, hypotheses,
missing evidence, redaction counts), serialize it exactly the way the API
does (the same canonical-JSON fn), and validate against the schema file with
the `jsonschema` crate (`include_str!("../../../schema/evidence-bundle.v1.schema.json")`
— compute the right relative path from the crate). Assert zero validation
errors, printing them all on failure.

Add a second, adversarial case: a bundle with every `Option` field `None` —
validates too.

**Verify**: `rtk cargo nextest run -p parallax-core` → both conformance tests
pass. Break-glass check: temporarily add a bogus required field to the schema
→ test fails with a validation error naming it → revert. (Proves the test
actually validates.)

### Step 3: Versioning policy + wiring

`schema/README.md` (short): what the file governs (the canonical JSON bytes
from `BundleOut.json` / `parallax issue context --format json`); versioning
policy — additive-only within `bundle-v1` (new optional fields allowed;
renames/removals/type changes require `bundle-v2` + a new schema file, old
file stays); the conformance test is the enforcement.

Add the `schema/` row to `PROJECT_STRUCTURE.md`'s table and one line in
`README.md` linking the schema as the portable-bundle contract.

**Verify**: `grep -n "schema/" PROJECT_STRUCTURE.md` → ≥1;
`grep -n "evidence-bundle.v1" README.md` → ≥1; links resolve (`ls` targets).

### Step 4: Record prose-vs-code divergences

While walking the closure in Step 1 you compared against
`docs/research/architecture/evidence-bundle-schema.md`. List every divergence
(field in prose absent from code, or vice versa; type mismatches) in your
final execution report AND as a short note appended to the
`advisor-plans/README.md` open-questions section. Do not resolve them.

**Verify**: the divergence list exists (possibly "none").

### Step 5: Full gates

**Verify**: `rtk cargo fmt --all`; clippy zero warnings;
`rtk cargo nextest run --workspace` → all pass.

## Test plan

- 2 conformance tests (representative + all-None), plus the temporary
  break-glass negative check (not committed).
- Existing hash-determinism tests remain untouched and green.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `schema/evidence-bundle.v1.schema.json` exists, parses, `$id` set,
      `schema_version` const = `bundle-v1`
- [ ] `grep -n "jsonschema" crates/parallax-core/Cargo.toml` → dev-dependency present
- [ ] 2 new conformance tests pass; `rtk cargo nextest run --workspace` exits 0
- [ ] `schema/README.md` states the additive-only policy
- [ ] `PROJECT_STRUCTURE.md` + `README.md` reference the schema
- [ ] Divergence notes recorded (or "none")
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- Plan 072 has not landed (schema would freeze the pre-hardening shape).
- The canonical-JSON serializer's bytes differ from plain
  `serde_json::to_string(&bundle)` in ways a schema can't capture (key
  ordering is fine — schemas don't constrain order — but any custom value
  encoding needs documenting first).
- The `jsonschema` crate's latest stable requires network access for
  metaschema resolution in tests (it shouldn't — but if it does, report
  rather than vendoring).
- You find `#[serde(flatten)]` or enums-with-data in the closure whose JSON
  representation is ambiguous — enumerate them and confirm the intended
  encoding before schematizing.

## Maintenance notes

- Every future bundle-shape PR must update the schema + conformance fixture
  in the same commit; the conformance test makes forgetting loud.
- When `bundle-v2` happens, copy don't edit: `evidence-bundle.v2.schema.json`
  beside v1.
- Plan 083 (MCP) should validate its `structuredContent` against this same
  file — one contract, three transports.
- Publishing the schema beyond the repo (docs site, schema registry) is a
  later operator decision; the `$id` URL should stay stable either way.
