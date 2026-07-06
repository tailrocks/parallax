# Plan 034: Design and build a minimal host-CLI → daemon → container → agent execution scenario for the playground

> **Executor instructions**: This is a **design-anchored** plan in a
> **different repository** (the telemetry playground, not the Parallax repo).
> Step 1 is a design spike; only build after it. If Step 1 concludes the
> minimal shape is larger than a single service + CLI mode, STOP and report
> the design so it can be split. Update the status row in `advisor-plans/README.md`
> (in the Parallax repo) when done.
>
> **Drift check (run first)**: in the playground repo,
> `git log --oneline -5` and confirm HEAD is at or after `ed1f975`; if the
> `libs/playground-telemetry`, `cli/`, or `deploy/` trees changed materially
> since, re-read them before designing.

## Status

- **Priority**: P3
- **Effort**: L
- **Risk**: MED
- **Depends on**: advisor-plans/029 (Story timeline) — the brief says build this
  "after UI can visualize run stories," so the run-story surface should exist
  first to consume the new telemetry
- **Category**: direction
- **Planned at**: commit `8bc3f13` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

The brief's sharpest product claim is a single graph across local CLI state,
container runtime, agent actions, and application telemetry — a category
Grafana/Kibana/Sentry do not cover as one product. The audit confirmed the
playground has **none** of it: no daemon/session/container topology, no
agent-session trace inside a container, and the CLI is a short-lived driver
only. Without this telemetry, the Parallax run-story/ecosystem execution-graph
surfaces have nothing real to render. This plan adds the minimum execution
stack that proves the boundary-propagation story (host CLI → daemon →
container → agent), which is the hard part (process/session boundaries, not
HTTP).

## Repository & current state

**Target repo:** `parallax-telemetry-playground` (sibling of the Parallax
repo). Not the Parallax product repo — its own `AGENTS.md`/conventions apply.

- `libs/playground-telemetry/src/lib.rs` — shared OTel setup: registers
  `TraceContextPropagator` only (`:59`); resource sets `service.name` +
  `service.version` (`:61-66`); `parallax.run.id` is expected via externally
  injected `OTEL_RESOURCE_ATTRIBUTES` (`:14` comment), not stamped in-lib;
  `shutdown()` flushes providers (`:46-50`).
- `cli/src/main.rs` — short-lived driver (`cron` subcommand exists at
  `:38-59`); flushes before `process::exit` (`:16`).
- `deploy/docker-compose.yml` — 11 static services; **nothing spawns
  containers or models a daemon→session→container hierarchy**.
- Boundary-propagation contract the brief specifies (the target design):
  CLI→daemon injects W3C `traceparent`/`tracestate`/`baggage` into local
  RPC/socket metadata; daemon→child sets `TRACEPARENT`/`TRACESTATE`/`BAGGAGE`
  + `OTEL_EXPORTER_OTLP_ENDPOINT` env vars on the child; child entrypoint
  extracts env context as the parent for its spans; agent process inherits
  `TRACEPARENT`, its `invoke_agent` span becomes a child of the container
  context.
- Deliberate failure mode to test: missing env injection creates an orphan
  container/agent trace — Parallax's story/gap surfaces (plans 029/032) should
  render it as broken-continuation, not hide it.
- OTLP endpoint config (how the playground points at Parallax): Rust reads
  `OTEL_EXPORTER_OTLP_ENDPOINT` (gRPC `:4317`); compose sets
  `host.docker.internal:4317`.

## Scope

**In scope** (after the spike approves the shape):
- `libs/playground-telemetry` — a composite propagator (trace context +
  baggage) and an env-context extraction helper for child processes
- a small `playground daemon` mode and a `playground enter <session>` child
  process (or the minimal equivalent the spike settles on)
- one agent-session simulation emitting `invoke_agent` → `execute_tool` /
  shell-command child spans inside the "container" context, sharing
  `parallax.run.id`
- a scenario script under `scenarios/` and a compose/env wiring
- the design note (Step 1)

**Out of scope**:
- Copying any specific external tool's architecture (brief rule: model the
  shape, name no external project).
- Real Docker container spawning if a simulated container span hierarchy
  proves the propagation just as well — prefer the simulation unless the spike
  shows real containers are needed.
- Adopting full GenAI/MCP semconv — use only the stable-enough core
  (`invoke_agent`, `execute_tool`, `gen_ai.operation.name`) the brief lists;
  content capture stays opt-in/redacted/off by default.
- Any change in the **Parallax** repo (the consuming UI already exists via
  plans 029/031/032).

## Steps

### Step 1 (SPIKE — output a design note first)

Write `docs/execution-stack-design.md` (in the playground repo) answering:
1. **Minimal topology.** The smallest set of processes that proves host CLI →
   daemon → container → agent propagation. Decide simulated-container-spans vs
   real Docker (default: simulated, justify).
2. **Propagation mechanics.** Exactly how context crosses each boundary
   (socket metadata for CLI→daemon; `TRACEPARENT`/`BAGGAGE` env for
   daemon→child; env extraction at child start). Confirm the composite
   propagator change in `libs/playground-telemetry` is small.
3. **run.id stitching.** How `parallax.run.id` is set once and inherited so
   the daemon, container, and agent spans share it (the brief's cross-trace
   spine). Confirm it flows via `OTEL_RESOURCE_ATTRIBUTES` to each child.
4. **The failure scenario.** How to deliberately omit env injection to produce
   an orphan agent trace for the evidence-gap demo.
5. **Acceptance.** Which Parallax questions this lets a reviewer answer
   (brief's playground acceptance list, e.g. "did a CLI run fail because of a
   command exit, service error, container issue, or agent action?").

If the minimal shape exceeds one daemon + one child + one agent sim, STOP and
propose splitting into sub-plans.

### Step 2: Composite propagator + env-context helper

In `libs/playground-telemetry`, register a composite text-map propagator
(trace context + baggage) replacing the trace-context-only registration
(`:59`), and add a helper that extracts `TRACEPARENT`/`TRACESTATE`/`BAGGAGE`
from the environment at process start and returns a parent `Context`.

**Verify (playground repo)**: `rtk cargo build` → exit 0;
`rtk cargo clippy --all-targets -- -D warnings` → exit 0 (match the
playground's own gate).

### Step 3: Daemon + enter child + agent sim

Add the `playground daemon` mode (receives a session command, opens spans,
injects context into the spawned child's env) and `playground enter` (child
that extracts context, opens a container-session span, then runs an
`invoke_agent` sim emitting a couple of `execute_tool`/shell-command child
spans, one of them failing to exercise error derivation). Share
`parallax.run.id` across all of them. Flush on exit (match the existing CLI
discipline).

**Verify**: `rtk cargo build` → exit 0; the binary runs the scenario without
panicking.

### Step 4: Scenario script + wiring

Add `scenarios/a27-execution-stack.sh` (id per the brief's backlog) that runs
the daemon + enter + agent sim against a running Parallax OTLP endpoint, plus
a variant with env injection omitted (the orphan-trace failure case). Update
`scenarios/README.md` (which the audit found stale) to index it.

**Verify**: running the script against a local `parallax serve` produces a run
whose Story timeline (Parallax plan 029) shows the CLI→daemon→container→agent
beats, and whose failure variant surfaces an evidence gap (plan 032).

### Step 5: Document acceptance

In the design note, check off which brief acceptance questions now have an
intuitive answer in the Parallax UI. Note explicitly what is simulated vs
real.

## Done criteria

- [ ] Design note committed (Step 1)
- [ ] Composite propagator + env helper in `libs/playground-telemetry`;
      playground `cargo build` + `clippy -D warnings` clean
- [ ] `playground daemon` + `playground enter` + agent sim run and emit one
      run stitched by `parallax.run.id` across daemon/container/agent spans
- [ ] `scenarios/a27-execution-stack.sh` (+ orphan variant) added and indexed
      in `scenarios/README.md`
- [ ] Against a local Parallax, the run's Story shows the execution beats and
      the failure variant shows an evidence gap (manual check, recorded)
- [ ] No change made in the Parallax product repo
- [ ] `advisor-plans/README.md` (Parallax repo) status row updated

## STOP conditions

- Spike concludes the minimal shape needs more than one daemon + one child +
  one agent sim → STOP, propose sub-plans.
- The composite propagator change ripples into every service's telemetry setup
  in a way that breaks existing scenarios → STOP and report.
- Real container spawning turns out to be required and pulls in Docker-in-the-
  loop test infra → STOP; that is a bigger commitment than this plan.

## Maintenance notes

- This is the producer side of Parallax's execution-graph/run-story surfaces;
  keep attribute names aligned with the brief's semconv table (`parallax.*`,
  `cli.*`, `tui.*`, `gen_ai.*`) so the Parallax UI can rely on them.
- **Deferred:** real Docker/mux integration, multi-agent-in-one-container,
  TUI screen/panel spans, asciicast recording — all additive on this base.
- Reviewer (playground repo conventions apply): confirm flush discipline on
  every short-lived process and that the orphan-trace failure case is
  reproducible.
