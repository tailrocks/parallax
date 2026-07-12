# Plan 117: Enforce repository documentation link integrity

> **Executor instructions**: Two internal links drifted before this plan was
> created, activating the documented trigger. Build the required gate through
> the common xtask diagnostics/control plane. Parse Markdown links structurally;
> do not use a broad regex that rewrites code examples or external prose.

## Status

- **Priority**: P2
- **Effort**: S-M
- **Risk**: MEDIUM
- **Depends on**: 095
- **Category**: documentation / CI / developer experience
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: TODO

## Why

The consolidation audit found broken relative links in the four-way benchmark
matrix and alternatives research. They were repaired in the planning commit,
but no required gate prevents the same drift. Jackin's repository-link checker
is a useful pattern; Parallax needs a smaller plain-Markdown implementation.

## Scope

In scope:

- Internal Markdown file/directory/fragment links across tracked repository
  docs, plans, crate READMEs, and prompts.
- A parser-backed xtask command with common human/JSON/GitHub diagnostics.
- Path-aware required CI plus fixtures for add/delete/rename and generated or
  intentionally ignored paths.
- Optional external-link reporting only when deterministic and owner-backed.

Out of scope:

- A documentation site, Node-based tooling, automatic URL rewriting, or making
  flaky network availability a required merge gate.
- Treating paths inside fenced code or literal examples as links.

## Steps

1. Inventory Markdown dialects, inline/reference/autolinks, anchors, angle-bracket
   destinations, directories, encoded spaces, case sensitivity, and generated
   exclusions. Define exact tracked-file ownership and ignore syntax.
2. Implement `cargo xtask docs links` with a real Markdown parser. Resolve paths
   from the source file, validate repository-root containment and fragments
   against parsed heading IDs, aggregate all failures, and emit common
   diagnostics with source line, target, reason, and remediation.
3. Add fixtures for valid/invalid relative paths, fragments, reference links,
   images, directories, spaces, Unicode, fenced code, deleted/renamed targets,
   duplicate headings, malformed Markdown, and ignored/generated files.
4. Route every Markdown/path/config/parser input to a required CI partition and
   include the command in `cargo xtask ci --full`. An empty file selection or
   parser failure must not report success.
5. If external checks are added, keep them scheduled/advisory with caching,
   retry bounds, domain exceptions with owners/expiry, and separate reporting.

## Test Plan

- Parser/resolver/anchor positive and negative fixture matrix.
- Path-classifier add/delete/rename/mixed fixtures.
- Human/JSON/GitHub diagnostic equivalence.
- Clean-checkout full repository scan.
- Network-disabled proof that required internal checking remains deterministic.

## Done Criteria

- [ ] All tracked internal Markdown links and fragments resolve.
- [ ] Fenced/literal examples and approved generated paths are handled correctly.
- [ ] Broken, deleted, renamed, escaped-root, and malformed targets fail fixtures.
- [ ] `cargo xtask docs links` is parser/dispatch-tested and non-hollow.
- [ ] Path-aware `ci-required` and `cargo xtask ci --full` invoke the same gate.
- [ ] External network failures cannot fail the required internal-link gate.

## STOP Conditions

- The implementation relies on regex alone for Markdown structure.
- A required gate depends on live external-network availability.
- False positives require broad directory exclusions or ignoring parser errors.
- The tool needs Node or a foreign package manager/runtime.

## Remove When

Delete this plan and index row when parser-backed internal link integrity is a
required local/CI gate and the full tracked repository scans clean.
