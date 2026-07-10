# Plan 083: MCP read-only context adapter — SPIKE (prove projection equivalence, do not ship)

> **Executor instructions**: This is a SPIKE plan — the deliverable is a
> working local prototype + a findings report, NOT shipped product code.
> Follow it step by step, honor the STOP conditions, and when done update the
> status row in `advisor-plans/README.md` and write the findings file
> (Step 5) — that file IS the output.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- crates/parallax-api/src crates/parallax-core/src/bundle.rs docs/research/decisions/agent-access-surface.md`
> Verify the invariant/tool-catalog quotes below still match the decision doc.

## Status

- **Priority**: P3 (direction — gated bet, design-mature)
- **Effort**: M (spike slice; the full gate matrix is L and NOT this plan)
- **Risk**: MED (security-sensitive surface; the spike must not leak beyond
  the bundle projection)
- **Depends on**: 072 (redaction hardening — MCP must not ship the weaker
  redactor), 081 (CLI JSON output — the equivalence comparand), ideally 082
  (schema to validate `structuredContent`)
- **Category**: direction (spike)
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

The product-shape doc lists MCP as one of four first-class surfaces over one
canonical API; the go/no-go verdict calls MCP "required before any
agent-native product claim … table stakes, not a moat" (competitors already
ship it). The full design exists in
`docs/research/decisions/agent-access-surface.md`: a read-only tool catalog,
a rejected-tools list, and a 17-row ship-gate matrix. The guide currently
says "No MCP server yet (gated decision)" (`docs/guide/agent-howto.md:3`).
This spike answers the one question that unblocks the gated decision: **can a
thin stdio MCP server over the existing GraphQL API reproduce the canonical
bundle byte-for-byte (projection equivalence), inside the redaction
boundary?** Everything else in the gate matrix stays future work.

## Current state

- Design doc anchors (`docs/research/decisions/agent-access-surface.md`):
  - Required invariant (`:225-228`): "For the same principal, project,
    anchor, time window, redaction policy, and schema version, CLI, HTTP,
    and MCP must produce the same canonical JSON hash, including equivalent
    `redaction_report.source_field_policy` status."
  - First MCP tools (`:266-279`): `parallax_issue_context` (tool,
    `evidence:read`, "Canonical bundle JSON in `structuredContent`, bounded
    Markdown in text"), `parallax_trace_context`,
    `parallax_agent_session_show`, resource `parallax://bundles/{bundle_id}`,
    etc. Rejected tools (`:281-296`): `run_shell`, `run_sql`, deploy/rollback
    /delete, management CRUD — none may exist even in the spike.
  - Ship gates (`:346-367`): 17 fixtures (projection equivalence, client
    fixture, scope, redaction, audit, output budget, …). The spike covers
    ONLY projection equivalence + a client smoke; the report lists the rest
    as open.
- Existing building blocks:
  - GraphQL `bundle(fingerprint:|runId:)` returns
    `BundleOut { json, markdown, canonicalHash }`
    (`crates/parallax-api/src/lib.rs:1618-1637`).
  - `agentSession(runId:)` resolver (`lib.rs:2067`).
  - After Plan 081: `parallax issue context --format json` prints the
    canonical JSON (the CLI comparand).
  - Server: local, no auth, loopback bind, host-header guard on `/graphql`.
- Placement decision for the spike: a NEW workspace crate
  `crates/parallax-mcp-spike` (bin) so product crates stay untouched; it
  speaks stdio MCP and calls `http://127.0.0.1:4000/graphql` as a plain HTTP
  client (reqwest with the workspace's native-TLS feature set — plaintext
  local hop, TLS unused).
- MCP SDK: use the official Rust MCP SDK (`rmcp`, the
  modelcontextprotocol/rust-sdk crate) at its latest stable. VERIFY its TLS
  posture before adding: if any transitive default feature enables rustls,
  disable default features / pick feature flags so no rustls feature is
  enabled (repo TLS rule: never rustls; stdio transport needs no TLS at
  all). If rustls proves unavoidable in the SDK's stdio path, STOP — record
  it; a hand-rolled minimal stdio JSON-RPC loop is the fallback and is
  acceptable for a spike (the protocol surface needed here is small:
  `initialize`, `tools/list`, `tools/call`).

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Build | `rtk cargo build -p parallax-mcp-spike` | exit 0 |
| Serve (terminal A) | `rtk cargo run -p parallax-cli -- serve` | ready banner with GraphQL :4000 |
| Seed data | run any local telemetry source, or the playground repo if present; else use `parallax run start -- <failing cmd>` | at least one issue exists |
| CLI comparand | `parallax issue context <fp> --format json` | canonical JSON |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |

## Scope

**In scope** (the only files you should create/modify):
- `crates/parallax-mcp-spike/**` (new crate; add to workspace members)
- Root `Cargo.toml` (workspace member + deps entries)
- `docs/research/validation/2026-XX-XX-mcp-spike-projection-equivalence.md`
  (the findings report — follow the naming style of existing files in
  `docs/research/validation/`)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- Any change to `parallax-api`, `parallax-core`, `parallax-server`, the CLI.
- The 15 unaddressed gate fixtures (auth, audit, output budget, …) — listed
  as open in the report, not built.
- Shipping: no docs/guide changes, no packaging, no default-on anything. The
  spike crate may be deleted after the decision; say so in its README.
- Any tool outside the read-only catalog; especially NOTHING from the
  rejected list (`run_shell`, `run_sql`, …).

## Git workflow

- Work directly on `main` (repo rule — `BRANCHING.md`).
- Conventional Commits, DCO signoff (`git commit -s`), trailer
  `Co-authored-by: Claude <noreply@anthropic.com>`. E.g.
  `spike(mcp): stdio context adapter proving projection equivalence`.

## Steps

### Step 1: Crate skeleton + SDK decision

Create `crates/parallax-mcp-spike` (bin). Decide SDK vs hand-rolled per the
TLS check in Current state; record the decision + crate version in the
findings file as you go. Wire a stdio server that answers `initialize` and
`tools/list` with exactly two tools:
`parallax_issue_context { fingerprint: string }` and
`parallax_agent_session_show { run_id: string }` — names and shapes from the
design doc's catalog.

**Verify**: `rtk cargo build -p parallax-mcp-spike` → exit 0; piping a
hand-written `initialize` + `tools/list` JSON-RPC request pair into the
binary's stdin returns the two tools and nothing else.

### Step 2: Implement the two tools over GraphQL

`parallax_issue_context`: POST `{ bundle(fingerprint: $fp) { json markdown canonicalHash } }`
to `http://127.0.0.1:4000/graphql`; return MCP content with (a)
`structuredContent` = the PARSED canonical JSON (parse exactly once; do not
re-serialize for the hash comparison — keep the raw string too), (b) a text
block with the bounded markdown, (c) the `canonicalHash` in the result
metadata. `parallax_agent_session_show`: same pattern over
`agentSession(runId:)`.

Escape tool arguments into the GraphQL query the same way the CLI does
(single shared approach: copy `gql_str`'s semantics — backslash, quote,
newline, tab; cite `crates/parallax-cli/src/client.rs:118`).

**Verify**: with `parallax serve` running and one issue seeded, a scripted
`tools/call` for `parallax_issue_context` returns non-empty
`structuredContent` and markdown.

### Step 3: Projection-equivalence proof

Script (in the spike crate as a `#[ignore]`d integration test or a small
`--check` subcommand of the bin — prefer the subcommand for CI-independence):

1. Fetch the bundle via the MCP tool path (raw JSON string as received).
2. Fetch via the CLI: `parallax issue context <fp> --format json`.
3. Fetch via plain HTTP GraphQL directly.
4. Assert all three JSON strings are byte-identical AND that
   `sha256(json)` relates to `canonicalHash` the way the server defines it
   (read how `canonical_hash` is computed in `crates/parallax-core/src/bundle.rs`
   — grep `canonical` — and reproduce that exact computation; if the hash is
   computed over something other than the emitted JSON string, record the
   actual definition in the findings).

Run it against ≥2 distinct issues (different anchors if possible: issue +
run bundle).

**Verify**: the check subcommand prints `equivalence: OK` for every case, or
a byte-level diff on failure.

### Step 4: One-client smoke (Claude Code)

Register the spike binary as a local stdio MCP server in Claude Code
(project-scoped `.mcp.json` in a scratch directory OUTSIDE this repo — do not
commit client config), call `parallax_issue_context` once from a session, and
record in the findings: the config used, trust prompt behavior, whether
`structuredContent` surfaced, and output-budget behavior on a large bundle.
If Claude Code is unavailable in the environment, record "client smoke
skipped: no client available" — the equivalence proof (Step 3) stands alone.

**Verify**: findings section for the client smoke exists (with results or the
skip note).

### Step 5: Findings report

Write `docs/research/validation/2026-XX-XX-mcp-spike-projection-equivalence.md`
(use today's date; follow the existing validation-note style — dated title,
sources, what was run, results, open questions):

- Equivalence result (byte-identical? hash definition confirmed?).
- SDK decision + any TLS-rule friction.
- Redaction posture observed: confirm MCP output contains ONLY
  bundle-projection data (already redacted by Plan 072's pipeline) — list
  any field visible via MCP that the CLI markdown doesn't show.
- The 15 unaddressed ship-gates from `agent-access-surface.md:346-367`,
  each with one line on what building it would take.
- Recommendation: proceed to a product MCP crate or park; name the gating
  items (expected: scope model + audit events + output budget).

**Verify**: file exists, linked from `advisor-plans/README.md` status row.

### Step 6: Gates

**Verify**: `rtk cargo fmt --all`; `rtk cargo clippy --workspace --all-targets`
→ zero warnings (spike crate included);
`rtk cargo nextest run --workspace` → all pass (spike adds no default-run
tests beyond what compiles).

## Test plan

- The `--check` equivalence subcommand is the spike's test artifact (run
  manually against a live server; documented in the findings).
- No CI wiring for the spike (it needs a live server + seeded data); note
  that explicitly in the crate README.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `crates/parallax-mcp-spike` builds; `tools/list` returns exactly 2 tools
- [ ] `grep -rn "run_shell\|run_sql\|deploy\|rollback" crates/parallax-mcp-spike/src` → 0 matches
- [ ] Equivalence check ran against ≥2 anchors; results recorded
- [ ] Findings file exists under `docs/research/validation/` with the
      open-gates list and a recommendation
- [ ] No rustls feature enabled anywhere: `grep -rn "rustls" Cargo.lock` shows
      no NEW entries vs `dbaba3c` (compare: `git show dbaba3c:Cargo.lock | grep -c rustls` vs current)
- [ ] `rtk cargo clippy --workspace --all-targets` → zero warnings
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- The MCP SDK cannot be used without enabling a rustls feature AND the
  hand-rolled stdio loop exceeds ~300 lines — report both costs and let the
  operator pick.
- Byte-equivalence FAILS between CLI and HTTP (that's a Plan 081 bug —
  report it there, don't paper over it in the spike).
- Plan 072 has not landed — the spike would demonstrate the weaker redaction
  to an agent transport; sequencing violation.
- Anything requires touching product crates.

## Maintenance notes

- If the decision is "proceed", the product implementation starts from the
  gate matrix (`agent-access-surface.md:346-367`), reusing this spike's
  equivalence checker as a permanent fixture; the spike crate is otherwise
  deleted (it must not rot half-shipped).
- `docs/guide/agent-howto.md`'s "No MCP server yet (gated decision)" line is
  updated only when a product decision is made — not by this spike.
