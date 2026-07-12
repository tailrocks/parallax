# MCP spike — projection equivalence

Timestamp: 2026-07-11 (plan 083)

## Sources

- Design: [`docs/research/decisions/agent-access-surface.md`](../decisions/agent-access-surface.md)
  (invariant at lines 225–228; first tools 266–279; ship gates 346–367)
- CLI comparand: Plan 081 (`parallax issue context` / `run bundle` `--format json`)
- Hash definition: `crates/parallax-core/src/bundle.rs` `canonical_hash` (sorted-key
  compact SHA-256 over evidence fields; excludes `generator`, `bounded`,
  `canonical_hash`)
- Spike crate: `crates/parallax-mcp-spike/` (**not** a product surface)

## Historical drift check

```text
git diff --stat dbaba3c..HEAD -- crates/parallax-api/src \
  crates/parallax-core/src/bundle.rs \
  docs/research/decisions/agent-access-surface.md
```

API + bundle assembly moved since `dbaba3c` (substantial lib.rs / bundle.rs
diff). The agent-access invariant quote, tool catalog, and 17-row gate matrix
in `agent-access-surface.md` still match the plan. GraphQL still exposes
`bundle { json markdown canonicalHash }` and `agentSession(runId:)`.

## What was built

New workspace bin crate `parallax-mcp-spike`:

| Mode | Behavior |
| --- | --- |
| `serve` (default) | stdio MCP server, two tools only |
| `check` | CLI ≡ HTTP ≡ MCP raw-JSON byte identity + hash recompute |

Tools (read-only catalog subset):

| Tool | Args | GraphQL |
| --- | --- | --- |
| `parallax_issue_context` | `fingerprint` | `bundle(fingerprint:)` |
| `parallax_agent_session_show` | `run_id` | `agentSession(runId:)` |

`tools/call` for issue context returns:

- `content[0].text` — bounded Markdown projection
- `structuredContent` — parsed canonical JSON object (parse once)
- `_meta.canonicalHash` — GraphQL `canonicalHash`
- `_meta.rawJson` — raw `json` field string (spike-only; for byte compare)

Rejected tools (`run_shell`, `run_sql`, `deploy`, `rollback`, …) are **absent**
(`grep` over `crates/parallax-mcp-spike/src` → 0 matches).

## SDK decision + TLS

| Choice | Detail |
| --- | --- |
| SDK | **`rmcp` 2.2.0** (official modelcontextprotocol/rust-sdk) |
| Features | `default-features = false`, enable only `server`, `transport-io`, `macros` |
| TLS | SDK HTTP/reqwest features **not** enabled. Stdio needs no TLS. GraphQL
  client uses workspace `reqwest` with `native-tls-vendored` over plaintext
  `http://127.0.0.1:4000` (TLS unused on the local hop). |
| rustls lockfile | `grep -c rustls Cargo.lock` = **2** at `dbaba3c` and **2** after
  this change (existing `rustls-pki-types` only; **no new rustls crate**). |

Hand-rolled JSON-RPC was **not** required; the stdio path stays under the
repo TLS rule without friction.

## What was run

Environment:

- Historical test setup used the since-removed `[storage] mode = "none"`
  (in-memory) on
  `127.0.0.1:4000` / OTLP HTTP `4318`
- Seeded via OTLP/HTTP protobuf: two distinct exception spans → two issues;
  one run-registered span with `parallax.run.id` → run-anchored bundle
- Binaries: `target/debug/parallax`, `target/debug/parallax-mcp-spike`

### tools/list smoke

Piped `initialize` + `notifications/initialized` + `tools/list` over stdio.
Returned exactly:

```text
parallax_agent_session_show
parallax_issue_context
```

### Projection equivalence (`check`)

```text
  hash=sha256:c4edcfdd177637af7034fd23ae3289dd76f1a3a60f9f7801a755dc6a15c2999a  json_bytes=1965  (CLI≡HTTP≡MCP)
equivalence: OK  (issue fingerprint=0dcb3ddbb90d3f2e)
  hash=sha256:6566c0d467d4e9353691086850fff59e6dc78471bf9cf65fa963306693a106fb  json_bytes=2434  (CLI≡HTTP≡MCP)
equivalence: OK  (run bundle run_id=18c11ddf8f8c104c)
equivalence: OK for all 2 case(s)

  hash=sha256:66a4d8fe252d3130f629c405ac5f8213df3c6097d56f9042684a6c8ebba58c0e  json_bytes=2011  (CLI≡HTTP≡MCP)
equivalence: OK  (issue fingerprint=d7ed88124f6c4a1a)
```

Anchors exercised: **2 issue fingerprints** + **1 run bundle** (≥2 required).

Byte-identical JSON strings across:

1. MCP tool path (shared `fetch_bundle` used by tools — raw GraphQL `json`)
2. Plain HTTP GraphQL
3. CLI (`parallax issue context` / `run bundle` `--format json`, trailing newline stripped)

### Hash definition confirmed

Server embeds `canonical_hash` inside the JSON. Spike recompute:

1. Parse emitted JSON
2. Drop `canonical_hash`, `generator`, `bounded`
3. Sorted-key compact form (same algorithm as `parallax-core::bundle::canonical_hash`)
4. `sha256:` + lowercase hex

Recomputed hash **equals** both the embedded field and GraphQL `canonicalHash`
on every case above.

**Note:** `structuredContent` is a *parsed* object; re-serializing it on the
MCP wire is **not** required to match the raw string. Byte-equivalence claims
are on the raw `json` string (and `_meta.rawJson` in the spike). Product MCP
should keep the raw string available for audit/hash fixtures or hash only
evidence fields, never the pretty wire form of `structuredContent`.

## Redaction posture

MCP returns only the GraphQL bundle projection (already through Plan 072's
redaction pipeline on the server). Observed on seed data:

```json
"redaction": {
  "policy": "redaction-lite-v3",
  "redacted_counts": {}
}
```

Seed payloads had no synthetic secrets; `redacted_counts` empty is expected.
Fields visible via MCP `structuredContent` that Markdown also summarizes:
full `issue`, `trace`, `hypotheses`, `missing_evidence`, `redaction`,
`bounded`, `canonical_hash`, etc. — the same object the CLI JSON path
emits. No extra privileged fields appear on the MCP path; it is a pure
projection of `bundle { json markdown canonicalHash }`.

Markdown is a bounded human/agent projection; JSON is the claimable
canonical object. MCP text content uses Markdown; agents should treat
`structuredContent` as authoritative (per design doc).

## Client smoke (Claude Code)

**client smoke skipped: no client available** in this agent environment
(no Claude Code / Codex MCP registration). Equivalence proof (Step 3)
stands alone. Example project-scoped config for a future local smoke
(do **not** commit; keep outside this repo):

```json
{
  "mcpServers": {
    "parallax-spike": {
      "command": "/path/to/parallax-mcp-spike",
      "args": ["serve"],
      "env": { "PARALLAX_URL": "http://127.0.0.1:4000" }
    }
  }
}
```

When a client is available, record: trust prompt behavior,
`structuredContent` surfacing, and output-budget behavior on a large bundle.

## Product ship work migrated

This packet proves projection equivalence only. It does not authorize or plan a
product MCP surface. All unfinished client, resource, scope/auth, redaction,
budget, audit, protocol/capability, retention, and spike-disposition work now
lives exclusively in
[`plans/112-product-mcp-ship-gates.md`](../../../plans/112-product-mcp-ship-gates.md),
which remains blocked on an explicit operator ship/no-ship decision. This
validation file is durable evidence, not an active checklist.

## Reproduce

```bash
# terminal A
parallax serve   # managed GreptimeDB + Turso

# terminal B — seed any failing telemetry, then:
cargo run -p parallax-mcp-spike -- check \
  --fingerprint <fp> \
  --run-id <run_id> \
  --parallax-bin target/debug/parallax
```

No CI wiring for the spike (needs live server + seeded data).
