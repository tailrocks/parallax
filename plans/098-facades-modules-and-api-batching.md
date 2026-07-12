# Plan 098: Seal facades, split modules, and batch nested API reads

> **Executor instructions**: Characterize each responsibility before moving it.
> Use compiler errors and checked facade manifests, not broad search/replace.
> File size is a ratchet input, not the decomposition algorithm. Preserve SDL,
> SQL, CLI output, and public behavior.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 126
- **Category**: architecture / API performance / docs
- **Planned at**: `a1d8bf82`, revised 2026-07-12
- **Status**: TODO

## Why

Current core, storage, and server expose broad module trees. Large files mix unrelated
responsibilities: Greptime queries/bootstrap/ingest, metadata domains, bundle
assembly/redaction/rendering, CLI commands, and GraphQL types/resolvers/tests.
Nested `Issue::latestEvent`/`events` can also become a store N+1 if selected on
a list, even though current UI queries do not activate it.

## Scope

In scope:

- Intentional facades for every product crate created/retained by plan 126.
- Responsibility splits for named Rust hotspots and tests.
- API resolver/module cleanup and nested issue batching.
- Facade manifests/compile contracts and semantic per-crate docs.
- Post-split file/public-surface ratchets.
- Self-named Rust modules, thin crate roots, and the exact structural budgets in
  `ENGINEERING-STANDARDS.md`.

Out of scope:

- GraphQL schema/product feature changes.
- Typed errors, plan 099.
- UI feature moves, plan 100.
- Universal flat APIs or arbitrary crate creation.

## Target Responsibilities

| Current hotspot | Target ownership |
|-----------------|------------------|
| `storage/adapter.rs` | `parallax-storage` capability modules and query-neutral values |
| `storage/greptime.rs` | `parallax-greptime` transport, bootstrap/reconcile, ingest/Arrow, trace/log/metric/analytics modules |
| `storage/metadata.rs` | `parallax-metadata` connection/migration, issue/run/dashboard/investigation, and row-mapping modules |
| `storage/spool.rs` | `parallax-spool` framing, append, recovery, replay, limits, and compatibility modules |
| `core/normalize.rs` | `parallax-ingest` signal-specific decode/normalize modules plus `parallax-analysis` error derivation |
| `core/bundle.rs` | `parallax-evidence` model/assembly/story/gaps, bounding, redaction, ranking, hashing, Markdown, tests/properties |
| trace/fingerprint/span-event modules | `parallax-analysis` domain-specific modules |
| `cli/commands.rs` | run, issue, logs, traces, SQL, live/follow, bundle output, forwarding/render helpers |
| API traces/services | GraphQL types, query orchestration, analysis/events, catalog/map/RED/runtime, tests |
| `api/resolvers.rs` facade created by plan 127 | responsibility-focused resolver children without reopening the facade contract |

## Steps

### Step 1: Pilot the server facade

Run `cargo xtask facade refresh -p parallax-server`, then make implementation
modules private and re-export only `Config`, `start`, `ServerHandle`, and
intentional lifecycle/error types. Enable `unreachable_pub`, fix consumers
compiler-first, refresh the manifest intentionally, and review its diff.

Add compile-contract tests for supported root paths and compile-fail docs for
representative private implementation paths.

### Step 2: Apply facades to the final workspace

After plan 126 ownership settles, repeat the pilot for model, proto, ingest,
analysis, storage ports, evidence, concrete adapters, API, and CLI as
applicable. Preserve meaningful public domain namespaces; do not flatten merely
to reduce `pub mod` counts. Each intentional export has an owner and consumer.

### Step 3: Split production responsibilities

Before each hotspot split, land characterization/golden tests and a write
allowlist. Move one concern at a time, then its coherent external test modules
from plan 127. Use the already-established `foo.rs` plus `foo/` children; plan
127 has already eliminated `mod.rs`. Multiple
focused test files are required when scenarios have independent concerns;
reject an exactly-one-`tests.rs` rule.

Decompose behind the existing `resolvers.rs` facade without reopening Juniper's
single-root impl or changing SDL. Enforce the new/restructured targets: 200 lines for crate roots,
400 for Rust production files, 600 for test scenarios, 100 for functions,
cognitive complexity 25, nesting 4, and 6 arguments. Existing larger items are
exact shrink-only rows; file size does not replace responsibility analysis.

### Step 4: Batch nested issue fields

Add a storage/API batch contract for latest/events by fingerprint and a
request-local batching/memoization layer consistent with the existing resolver
context pattern. A query of the maximum issue page with both nested fields must
use a bounded constant number of storage calls, not calls per issue.

Do not add an external loader library unless current docs and the existing
Juniper version prove it integrates cleanly. Test storage-call counts directly.

### Step 5: Make crate orientation semantic

Add concise crate READMEs with purpose, owned concerns, source map, public
surface, and narrow verification. Add crate-local AGENTS/linkers only for
non-derivable rules. Validate the README's actual Cargo tier/dependencies,
module links, and root exports.

Generate a plain-Markdown workspace map and slim `PROJECT_STRUCTURE.md`; do not
build a docs site.

## Test Plan

- Facade manifest and compile-contract tests.
- Existing API SDL/hash and resolver behavior snapshots.
- Storage SQL/conformance and bundle/CLI output oracles.
- Nested issue query at 1 and maximum page size with storage-call counters.
- Semantic crate-doc fixtures.
- Size/public-surface ratchet fixtures.

## Done Criteria

- [ ] Every final product crate exposes only a reviewed root facade.
- [ ] `unreachable_pub` and `cargo xtask facade check --workspace` pass.
- [ ] Named hotspots are split by responsibility with coherent tests.
- [ ] No crate root contains business logic, no `mod.rs` remains, and every new
  or restructured item meets the structural targets or an exact expiring
  exception.
- [ ] GraphQL SDL, SQL, bundles, and CLI output are unchanged.
- [ ] Nested issue fields remain bounded to constant storage calls.
- [ ] Crate docs match Cargo/source semantically.
- [ ] New/post-split size and public-surface baselines cannot grow.

## STOP Conditions

- A facade change requires a public contract break without an approved spec
  migration.
- A split changes query semantics, ingest ownership, or allocation behavior.
- Batching requires per-item fallback at maximum page size.
- Semantic docs cannot be derived without duplicating mutable implementation
  prose.

## Remove When

Delete this plan and row when facades, module ownership, nested batching, and
semantic crate orientation are machine-enforced and green.
