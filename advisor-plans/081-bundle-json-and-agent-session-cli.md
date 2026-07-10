# Plan 081: Give agents the structured surface — `--format json` on bundle commands + an agent-session CLI path

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- crates/parallax-cli/src crates/parallax-api/src`
> Plan 078 may have moved the `bundle`/`agentSession` resolvers into
> `resolvers/` modules — the GraphQL SCHEMA is unchanged by 078, so this plan
> is unaffected; verify only that the CLI excerpts below still match.

## Status

- **Priority**: P2 (direction — highest-leverage agent-surface win)
- **Effort**: M
- **Risk**: LOW (additive flags; markdown stays default)
- **Depends on**: none (composes with 072's improved redaction)
- **Category**: direction
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

The agent is Parallax's primary consumer, and the design doc
(`docs/research/decisions/agent-access-surface.md`, "The CLI remains the
first usable interface" table) explicitly specifies
`parallax issue context <id> --format json` → "Canonical bundle JSON" and
`parallax agent session show <id> --format json`. Today:

- The GraphQL `BundleOut` already exposes `json` (canonical bundle JSON)
  alongside `markdown`/`canonicalHash` — but the CLI never requests it; both
  bundle commands print Markdown only. An agent that wants structure must
  hand-roll a GraphQL POST.
- The `agentSession(runId:)` resolver and the UI card exist, but the CLI —
  the only agent surface until MCP lands — has no command that reaches the
  agent-session projection (tool steps, commands, token totals).

Both are thin verbs over existing resolvers: the cheapest, lowest-risk step
toward the designed agent contract, and a prerequisite shape for MCP
projection-equivalence (Plan 083).

## Current state

- `crates/parallax-api/src/lib.rs:1618-1637` — `BundleOut { json, markdown,
  canonical_hash }`, all exposed as GraphQL fields (`json` documented as
  "The bundle as canonical JSON").
- `crates/parallax-cli/src/commands.rs:444-459` — `issue_context`:

  ```rust
  pub async fn issue_context(client: &Client, fingerprint: &str) -> anyhow::Result<()> {
      let response = client
          .graphql(&format!(
              r#"{{ bundle(fingerprint: "{}") {{ markdown canonicalHash }} }}"#,
              gql_str(fingerprint)
          ))
          .await?;
      let Some(bundle) = response.pointer("/data/bundle").filter(|v| !v.is_null()) else {
          anyhow::bail!("issue {fingerprint} not found");
      };
      println!("{}", bundle["markdown"].as_str().unwrap_or(""));
      if let Some(hash) = bundle["canonicalHash"].as_str() {
          println!("\n---\nbundle: {hash}");
      }
      Ok(())
  }
  ```

  `run_bundle` (`:365-380`) is the same shape with `bundle(runId:)`.
- `crates/parallax-cli/src/main.rs:173-187` — `enum IssueCommand` with
  `Context { fingerprint: String }` (no flags); the `RunCommand` enum nearby
  has the `Bundle { run_id }` variant (locate it in the same file).
- `agentSession` resolver at `crates/parallax-api/src/lib.rs:2067-2079`:
  takes `run_id: String`, returns `Option<AgentSessionOut>`
  (`AgentSessionOut` at `:767+`, wrapping
  `parallax_core::agent_session::AgentSession` + `truncated: bool`). Read
  `AgentSessionOut`'s `#[graphql_object]` impl (`:784+`) to enumerate its
  field names for the CLI query — do not guess them.
- CLI conventions: subcommands in `main.rs` enums dispatch to
  `commands.rs` free fns taking `&Client`; GraphQL via
  `client.graphql(&format!(...))` with `gql_str` escaping
  (`client.rs:118`); human output via `println!`.
- Docs to update in the same change: `docs/guide/cli.md` (command table) and
  `docs/guide/agent-howto.md` (the agent handoff doc — it currently
  describes Markdown output; add the JSON mode).

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Build | `rtk cargo build -p parallax-cli` | exit 0 |
| Full suite | `rtk cargo nextest run --workspace` | all pass |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| Manual (needs local server + data) | `parallax issue context <fp> --format json \| python3 -m json.tool` | valid JSON |

## Scope

**In scope** (the only files you should modify):
- `crates/parallax-cli/src/main.rs` (flags + subcommand)
- `crates/parallax-cli/src/commands.rs`
- `docs/guide/cli.md`, `docs/guide/agent-howto.md`
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- The GraphQL schema — everything needed already exists.
- `--format json` on OTHER commands (`trace context` doesn't exist as a CLI
  verb yet; `cli invocation show` has no backing projection) — do not invent
  them; they're listed in the design doc for later.
- MCP (Plan 083), bundle schema publication (Plan 082).
- Raw/unredacted output flags — requires the read-sensitive permission design.

## Git workflow

- Work directly on `main` (repo rule — `BRANCHING.md`).
- Conventional Commits, DCO signoff (`git commit -s`), trailer
  `Co-authored-by: Claude <noreply@anthropic.com>`. E.g.
  `feat(cli): add --format json to bundle commands`.

## Steps

### Step 1: `--format` flag on both bundle commands

In `main.rs`:

```rust
#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat { Markdown, Json }
```

(check clap derive imports; `clap::ValueEnum` with the existing `derive`
feature). Add `#[arg(long = "format", value_enum, default_value = "markdown")]
format: OutputFormat` to `IssueCommand::Context` and `RunCommand::Bundle`;
thread it into the dispatch calls.

In `commands.rs`, extend `issue_context`/`run_bundle` signatures with
`format: OutputFormat`:
- `Markdown` (default): EXACT current behavior — same query, same prints
  (byte-identical output; scripts may depend on it).
- `Json`: query `{{ bundle(...) {{ json canonicalHash }} }}`; print the
  `json` field verbatim (it is already a canonical-JSON string — do NOT
  re-serialize or pretty-print it; canonical bytes are the contract), then
  NOTHING else on stdout (the `\n---\nbundle: <hash>` trailer would corrupt
  JSON consumers — the hash is inside the canonical JSON already; verify by
  reading `parallax_core::bundle::Bundle`'s serialized fields, and if the
  hash is NOT inside the JSON, print it to stderr instead).

**Verify**: `rtk cargo build -p parallax-cli` → exit 0;
`rtk cargo clippy -p parallax-cli --all-targets` → zero warnings.

### Step 2: Agent-session CLI verb

Add to `RunCommand` (keeps the anchor consistent — the projection is
run-scoped): `Agent { run_id: String, #[arg(long = "format", value_enum, default_value = "markdown")] format: OutputFormat }`
dispatching to a new `commands.rs` fn `run_agent_session`:

- Query `agentSession(runId: "...")` selecting every field
  `AgentSessionOut`'s graphql impl exposes (enumerate from
  `lib.rs:784+` — include `truncated`).
- `Json`: print the `data.agentSession` object as returned (serde_json
  to_string of that subtree — this projection has no canonical-bytes
  contract yet, so re-serialization is acceptable here).
- `Markdown`: a compact human rendering (session summary line, then one line
  per step/tool-call following whatever the projection's fields offer; match
  the style of the existing `run inspect` output in `commands.rs`).
- `None`/null → `anyhow::bail!("no agent session detected for run {run_id}")`.

Update the docs: `docs/guide/cli.md` table rows for
`parallax issue context <fp> [--format json|markdown]`,
`parallax run bundle <run_id> [--format ...]`,
`parallax run agent <run_id> [--format ...]`; `agent-howto.md` gains a
"Structured output" paragraph showing the `--format json` call as the
machine path.

**Verify**: `rtk cargo build -p parallax-cli` → 0; grep gates below.

### Step 3: Tests + manual proof

- Unit-level: the CLI's GraphQL calls go through `Client` — check
  `commands.rs`/`client.rs` for an existing test seam (mock server or
  fixture). The repo's integration tests boot a real server
  (`crates/parallax-server/tests/`); `m2_bundle.rs` proves the `json` field
  exists server-side. If no CLI test seam exists, add a minimal unit test
  for the markdown-vs-json print decision if it can be factored as a pure fn
  (e.g. `fn render_bundle(format, bundle_value) -> (stdout_string, stderr_string)`)
  — factor it so, and test both formats incl. the no-trailer-in-json rule.
- Manual (only if a local `parallax serve` with data is available):
  `parallax issue context <fp> --format json | python3 -m json.tool` → parses;
  `parallax run agent <run_id>` → renders or bails cleanly. Otherwise note
  "manual check skipped: no live server" in the commit message.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 4: Full gates

**Verify**: `rtk cargo fmt --all`; clippy zero warnings; full nextest green.

## Test plan

- Pure-fn tests for the render decision (Step 3): json mode emits exactly the
  canonical string + newline, markdown mode byte-identical to today's output,
  json mode never mixes the hash trailer into stdout.
- Existing `m2_bundle.rs` remains the server-side guarantee of `json`.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `grep -n "ValueEnum" crates/parallax-cli/src/main.rs` → ≥1; `--format` on Context, Bundle, Agent variants
- [ ] `grep -n "bundle(fingerprint:.*json canonicalHash" crates/parallax-cli/src/commands.rs` → ≥1 (json-mode query)
- [ ] `grep -n "run agent\|Agent {" crates/parallax-cli/src/main.rs` → the new subcommand exists
- [ ] `grep -n "format json" docs/guide/cli.md` → ≥1; `grep -n "format json" docs/guide/agent-howto.md` → ≥1
- [ ] `rtk cargo nextest run --workspace` exits 0
- [ ] `rtk cargo clippy --workspace --all-targets` → zero warnings
- [ ] Markdown-mode output unchanged (compare a captured before/after if a live server is available; else assert via the pure-fn test)
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- `BundleOut.json` turns out NOT to be the canonical-hash input bytes (i.e.
  hashing `json` ≠ `canonicalHash`) — the "print verbatim" contract needs the
  operator to define which bytes are canonical.
- `AgentSessionOut`'s field set is unstable/experimental per code comments —
  report before freezing it into a CLI contract.
- Adding `ValueEnum` requires enabling a new clap feature beyond the
  workspace's current `["derive"]`.

## Maintenance notes

- Plan 082 (bundle-v1 JSON Schema) validates exactly the bytes this flag
  prints — land 082's conformance test against this output path.
- Plan 083 (MCP) must return the SAME canonical JSON (projection
  equivalence, `agent-access-surface.md:227`); this plan's verbatim-print
  rule is what makes byte-equality provable.
- When `trace context` / `cli invocation show` verbs land, reuse
  `OutputFormat` — it becomes the CLI-wide convention.
