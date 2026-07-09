# Plans Audit Backlog

This folder now holds only plan items that were not fully retired by the
2026-07-09 sub-agent audit. Completed plans were removed. Remaining plan
files are reduced to the exact missing implementation, verification, or
evidence needed before deletion.

Rule: a plan file stays here only while it has an actionable incomplete,
contradicted, or weak-evidence item. When its "Remove When" section is true,
delete that plan file and update this README.

## Active Items

| Plan | Status | Title | Repo | Remaining reason |
|------|--------|-------|------|------------------|
| 036 | TODO | Playground trace spine | playground | live trace-spine proof and dependency note |
| 038 | TODO | Time-window continuity | parallax | route-level URL range proof |
| 040 | TODO | UI performance at scale | parallax | seeded performance/manual proof |
| 041 | TODO | Releases/deploys lane | parallax | seeded release UI proof |
| 042 | TODO | Playground release + flags reality | playground | live release/flag/catalog proof |
| 043 | TODO | Service catalog | parallax | seeded catalog UI proof |
| 044 | TODO | Runtime dashboards + metric discovery | parallax | seeded metric discovery/UI proof |
| 045 | TODO | Playground runtime scenarios | playground | recorded runtime scenario evidence |
| 046 | TODO | Field explorer phase 1 | parallax | seeded field-explorer UI proof |
| 048 | TODO | Playground Postgres reality | playground | live db span/pool metric proof |
| 049 | TODO | Playground messaging + gRPC semantics | playground | live messaging/grpc proof |
| 050 | TODO | Playground frontend RUM journey | playground | live browser RUM proof |
| 051 | TODO | traceCriticalPath + traceCompare | parallax | real trace compare proof |
| 052 | TODO | Investigations | parallax | manual save/restore proof |
| 053 | TODO | Design-system + a11y consolidation | parallax | visual/a11y sweep proof |
| 054 | TODO | Playground quality scenarios + TOUR | playground | recorded scenario tour evidence |
| 056 | TODO | Typed events + structured logs | playground | live native-log event proof |
| 057 | TODO | Logs context + saved views | parallax | route integration tests/proof |
| 061 | TODO | Trace view modes | parallax | route/deep-link/skew proof |
| 063 | TODO | Playground trace-shape scenarios | playground | live backdated skew proof |
| 064 | TODO | Command center v1 | parallax | manual Overview brush proof |

## Retired In This Audit

Removed as complete after audit and patch verification:

- 035 UI correctness and feedback sweep
- 037 One-command demo
- 039 Everything-clickable pivots
- 047 Playground GraphQL family
- 058 Trace-events backbone
- 059 GraphQL operation explorer
- 060 gRPC stream and messaging lanes
- 062 SQL workbench affordances
- 065 Agent-session surface
- 066 Semconv registry
- 067 Playground cache reality
- 068 Command palette

## Verification Discipline

- Use `rtk` for all commands.
- Prefer native GreptimeDB tables for evidence: `opentelemetry_logs`,
  native metric tables, and native trace/span tables. Do not invent custom
  telemetry tables for logs, metrics, or traces.
- If native tables cannot satisfy a requirement, record the failed query,
  code evidence, and proposed upstream question before asking the GreptimeDB
  team.
