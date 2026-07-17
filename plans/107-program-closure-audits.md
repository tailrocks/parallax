# Plan 107: Independently prove and close the active restructuring program

> **Executor instructions**: Plan status and file existence are not evidence.
> Run two independent audits from clean detached checkouts of the same pushed
> implementation commit. Resolve findings in source, produce both packets,
> then use a mechanically limited closure commit and independently verify its
> final pushed tree without creating a self-referential evidence commit.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: HIGH
- **Depends on**: Every other actionable indexed plan; all blockers freshly rechecked
- **Category**: validation / closure / plan lifecycle
- **Planned at**: `a1d8bf82`, revised 2026-07-12
- **Status**: IN PROGRESS (2026-07-17) — every other actionable plan retired;
  remaining indexed plans 089/114 are minimal BLOCKED files with fresh
  2026-07-17T16:40Z trigger rechecks; `closure-final --dry-run` fixtures pass;
  freezing C0 next (claimed by Claude Code executor)

## Current Evidence

- 2026-07-15: Step 0 now has a repository-owned `cargo xtask closure-final`
  verifier and a read-only aggregate CI lane. Before mechanical closure, the
  lane executes passing/tampered fixtures; after Plan 107 is removed it
  automatically verifies the real commit. The verifier binds two distinct
  auditor trailers to C0, C1, and the pushed tree, checks DCO and the single
  Codex co-author trailer, requires C1's exact four same-date evidence paths,
  restricts the closure diff to the mechanical allowlist, hashes and validates
  both JSON packets, proves program-file/authorization retirement and the
  remaining plan/index BLOCKED bijection, and accepts future closure dates.
  Packet validation is fail-closed over the exact top-level schema and requires
  a clean independent audit, non-empty tool versions, passing commands with
  output bytes bound to their hashes, repository artifact hashes verified
  against safe relative paths, structured finding/exception arrays, and fresh
  blocked-trigger records; hollow, extra-field, path traversal, or tampered
  packets fail. C1 must add rather than modify all four same-date packets, each
  JSON packet must hash its matching Markdown report, and both auditors must
  record the full aggregate command.
  The final lane runs the repository-owned `scripts/ci/closure-baseline.sh`,
  which mirrors the complete Shared Verification block, before structural
  verification with every required pinned tool. Workflow fixtures require
  every published baseline command and the executable bit. All 72 xtask tests,
  strict xtask Clippy, Actionlint, workflow/classifier fixtures, Rust
  formatting, structural policy, and the explicit dry-run command pass locally.
- 2026-07-15: the fresh direct/dependency blocker matrix is preserved in
  [`docs/research/validation/2026-07-15-active-plan-blocker-audit.md`](../docs/research/validation/2026-07-15-active-plan-blocker-audit.md).
  No other product plan is ready, so Criteria for Freezing C0 are not met and
  Steps 1–7 cannot honestly begin.

## Why

The program changes contracts, dependency direction, CI, releases, storage,
agent boundaries, and UI ownership. A green aggregate job alone cannot prove
that policy is non-hollow, documentation is truthful, or temporary exceptions
were removed. Closure requires separate source-based reviewers and durable,
machine-readable evidence.

## Scope

In scope:

- Two independent full source/artifact audits of one pushed implementation
  candidate and durable packets committed before lifecycle cleanup.
- Reconciliation of all findings, exceptions, suppressions, quarantines, and
  documentation claims.
- A mechanical final-plan cleanup tree inspected by both auditors, with their
  tree/diff attestations embedded durably in the closure commit and a
  repository-owned required check validating that exact pushed commit.

Out of scope:

- Waiving findings through plan status or broad exceptions.
- Product/config/workflow changes in the mechanical cleanup commit.
- Keeping terminal plan files as an archive.

## Criteria For Freezing C0

- Every other actionable plan passed its gates and was already deleted with its
  index row in its own completion commit. Completion evidence is read from
  source, tests, durable validation records, and Git history, never from a
  retained terminal plan file.
- Every other remaining indexed file is a minimal BLOCKED plan whose exact
  external/operator/phase condition was freshly reproduced. A blocker may not
  hide internally actionable steps.
- The candidate commit is pushed and the main working tree is clean.
- Required CI and release evidence is accessible at that exact commit.
- Step 0's repository-owned `closure-final` verifier/check exists on the
  candidate, has read-only permissions, validates closure commit trailers,
  parent/tree/diff allowlists and remaining plan state, is required by the
  ruleset, and has passed a dry-run fixture. It may not rely on expiring
  Actions artifacts or files that the closure commit deletes.

## Steps

### Step 0: Build the final-closure verifier

Before freezing an implementation candidate, implement and fixture-test the
persistent `closure-final` command/workflow. It records auditor identities and
attestation digests in dedicated commit trailers, validates the staged-tree
hash and audited parent, rejects any non-mechanical path/content change, and
runs the required final baseline. The check needs `contents: read` only, no
secrets or write token. Add the stable check to the live ruleset, push it, and
prove both passing and tampered dry-run fixtures. Only then apply the entry
criteria above and freeze C0.

### Step 1: Prepare a closure manifest

Record commit, repository cleanliness, tool/engine versions, active policies,
retired-plan Git history, plan-to-source/test evidence, required checks,
exceptions, suppressions, quarantines, generated artifacts, artifact hashes,
and open blocked triggers. Do not use historical plan checkboxes as proof. The
closure mechanism's durable evidence is the Git commit/tree/trailers plus the
source audit packets; the GitHub check is enforcement, not the sole retained
artifact.

### Step 2: Run auditor A

From a fresh clean detached checkout at the pushed commit, an auditor focused
on architecture/contracts/source behavior inspects Cargo metadata, dependency
tiers, public facades, product configuration, storage/native-table behavior,
zero-copy ownership, retry/idempotency, GraphQL/bundle compatibility, UI import
boundaries, tests, and source-linked documentation. The auditor runs relevant
commands independently and records findings, not just command names.

### Step 3: Run auditor B

A skeptical auditor who did not implement the final wave uses a separate fresh
detached checkout of the same commit. It owns CI path routing, required-check
topology, security/dependency/TLS policy, nextest evidence, cache behavior,
Oxc/Bun process ownership, release determinism, signatures/SBOM/attestations,
the exact two-entry Oxc pre-stable allowlist and expiry behavior, tamper failures,
blocked condition checks, and plan lifecycle. It must inspect
workflow/source logic and actual artifacts rather than trust green labels.

### Step 4: Reconcile and repeat

Resolve every disagreement or finding in source, policy, or tests. Push a new
candidate and rerun both auditors from separate clean detached checkouts at that
same commit. Reconcile every defect-ledger row, policy/tier exception, lint
allow, ignored/quarantined test, generated drift, and documentation claim.
Remove stale entries instead of grandfathering them. Repeat until both audits
agree on one pushed implementation candidate, C0.

### Step 5: Store durable evidence without commit recursion

Write matching
`docs/research/validation/<date>-active-plans-closure-{a,b}.md` and `.json`.
JSON includes schema version, audited commit C0, clean state, auditor
identity/independence, tool versions, commands, exit codes, artifact hashes,
findings, exceptions, and blocked-trigger evidence. Markdown explains source
review and dispositions.

Commit and push only those already-generated packets as evidence commit C1,
whose parent is C0. A repository-owned verifier must prove that C1 changes only
the expected packet paths, that both packets validate and name parent C0, and
that their recorded hashes agree. Do not regenerate packets to name C1: they
attest their parent implementation commit by design. If packet preparation
reveals a source, policy, test, or documentation defect, discard the candidate,
fix it, and return to Step 2 with a new C0 instead of mixing remediation into C1.

### Step 6: Retire the final program plan

Confirm that earlier completion commits already retired every other actionable
plan and index row. Delete this plan and its index row in the closure commit.
Delete `ENGINEERING-STANDARDS.md`, `OXC-IMPLEMENTATION.md`,
`IMPLEMENTATION.md`, and `GOAL.md` when no actionable
restructuring/alignment work remains and their rules have landed in durable
source/config/tests/conventions or in a remaining self-contained BLOCKED plan.
Remove the exact `GOAL.md` authorization/registry text from `AGENTS.md` and
`PROJECT_STRUCTURE.md` in that same mechanical commit.
Keep every other BLOCKED file only when a fresh exact condition still prevents
execution; shrink it to current evidence and trigger before staging the closure
tree. Plans 130 and 131 are actionable Oxc-only prerequisites and must already
have retired; closure cannot preserve Prettier, ESLint, or an unfinished Oxc
migration as BLOCKED work. Confirm all active plan material exists only in
`plans/`.

The cleanup commit may change only this plan, its index row, the four program
reference/contract/goal files, the `GOAL.md` registration in this index, and
the exact `GOAL.md` authorization/registry text in `AGENTS.md` and
`PROJECT_STRUCTURE.md`, plus already-generated closure packet references
required by lifecycle. It may not batch-delete plans that should have retired
earlier or change product source, manifests, policy, CI, release, or other
durable contract documents.

### Step 7: Attest the staged tree and verify the pushed commit

Stage the mechanical cleanup on evidence commit C1 and compute its Git tree hash
without committing. Both auditors independently inspect that exact tree, its
diff from C1, and the complete C0-to-staged-tree diff. They prove that C1 added
only the two audit packets, the final-plan cleanup matches the complete Step 6
allowlist, and every byte outside the union of all four exact evidence artifact
paths and that allowlist remains byte-identical to the fully audited C0. They
also verify the remaining `plans/`
bijection/statuses and plans-only policy. Record auditor ID, implementation
candidate C0, evidence commit C1, staged tree hash, result, and full-attestation
digest in `Closure-Audit-A` / `Closure-Audit-B` commit trailers alongside
exactly one normal agent co-author trailer. Commit and push once.

The required `closure-final` check validates C0, parent C1, C1's evidence-only
diff, the pushed closure tree/trailers, the mechanical cleanup diff, remaining
plans, byte identity outside the union of all four exact evidence artifact paths
and all Step 6 allowlisted removals, and the full baseline. Because the source
packets and exact tree/attestation hashes live in Git, no follow-up evidence
commit or expiring artifact is needed. If either auditor changes the staged
tree, or the diff contains non-mechanical work, recompute/re-audit the tree; for
any change beyond that union, return to two full source audits on a new C0.

## Test Plan

- Two clean detached checkout manifests at one pushed commit.
- Full shared verification plus plan-specific real-engine/UI/release gates.
- Independent source/config/workflow/artifact reviews.
- JSON schema/required-field validation, artifact hash verification, and an
  evidence-only C1 fixture whose packets attest parent C0 without self-reference.
- Negative tests for stale exceptions, skipped checks, hollow reports, and any
  previously completed numbered plan left in `plans/`.
- Oxc-only proof: no invoked ESLint/Prettier path and exactly the expiring
  Oxfmt/`oxlint-tsgolint` policy entries, with no broader pre-stable surface.
- Repository search proving no active plan/prompt/index exists outside `plans/`.
- Mechanical removal of `GOAL.md` and its index/agent/structure registration
  text with the completed program contracts.
- Final cleanup-diff/trailer/tree fixtures and required `closure-final` check.

## Done Criteria

- [ ] Both auditors used separate clean detached checkouts at one pushed
  implementation candidate.
- [ ] Auditor B did not implement the final wave.
- [ ] Both packets contain exact commands, results, hashes, source review, and findings.
- [ ] Every difference/finding was resolved before C0 and both packets attest C0.
- [ ] C1 changes only the expected packets, whose schemas, commit fields, and
  hashes validate against parent C0 without self-reference.
- [ ] Required checks are non-hollow and actual artifact/sidecar tampering fails.
- [ ] Exceptions, suppressions, quarantines, and docs match live source/policy.
- [ ] ESLint and Prettier are not invoked or directly owned; the only pre-stable
  Oxc exceptions are exact Oxfmt and `oxlint-tsgolint` entries with stable-release
  expiry and negative broadening fixtures.
- [ ] Git history proves every earlier actionable plan retired with its index
  row in its own completion commit; this final plan and the program
  reference/execution/goal files and exact goal authorization/registry text are
  deleted by the closure commit.
- [ ] Only genuinely unfinished BLOCKED plans remain, with fresh trigger evidence.
- [ ] No active plan material exists outside `plans/`.
- [ ] The closure commit durably embeds both independent auditor IDs, C0 and C1
  SHAs, exact tree hash, results, and attestation digests.
- [ ] The repository-owned required `closure-final` check passes at the exact
  final pushed commit with read-only permissions and no expiring evidence dependency.

## STOP Conditions

- Checkouts differ by commit, contain uncommitted changes, or reuse implementation
  state that invalidates independence.
- An audit relies on plan prose, status, grep alone, or a green label without
  source/artifact inspection.
- Any finding is waived by raising a ratchet, weakening a required check, or
  broadening an exception.
- A blocked condition is asserted without a fresh reproducible check.
- Durable evidence omits failures, hashes, versions, or auditor independence.
- The closure commit changes anything outside its mechanical allowlist.
- The required verifier/check is absent, untested, not required, or depends on
  expiring artifacts, secrets, or deleted plan files.

## Remove When

Delete this plan and row in the mechanical closure commit after both full
audits agree and all actionable work is closed. Closure is valid only after
both auditors attest the exact staged tree in commit metadata and the required
final pushed check passes as described above.
