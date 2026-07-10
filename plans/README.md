# Plans Audit Backlog

This folder holds only unfinished feature-backlog plan items. Completed plans
were removed by the 2026-07-09 and 2026-07-10 audits.

Rule: a plan file stays here only while it has an actionable incomplete,
contradicted, or weak-evidence item. When its "Remove When" section is true,
delete that plan file and update this README.

## Active Items

None.

Latest retirement evidence:

- [Plan 038 Route Range Continuity Evidence](../docs/research/validation/2026-07-10-plan-038-route-range-continuity.md)
- [Active Plans Live Evidence](../docs/research/validation/2026-07-10-active-plans-live-evidence.md)

## Previously Retired

Removed as complete after audit and patch verification:

- 035 UI correctness and feedback sweep
- 036 Playground trace spine
- 037 One-command demo
- 038 Route range continuity
- 039 Everything-clickable pivots
- 040 UI performance at scale
- 041 Releases/deploys lane
- 042 Playground release and flags reality
- 043 Service catalog
- 044 Runtime dashboards and metric discovery
- 045 Playground runtime scenarios
- 046 Field explorer phase 1
- 047 Playground GraphQL family
- 048 Playground Postgres reality
- 049 Playground messaging and gRPC semantics
- 050 Playground frontend RUM journey
- 051 traceCriticalPath and traceCompare
- 052 Investigations
- 053 Design-system and a11y consolidation
- 054 Playground quality scenarios and TOUR
- 056 Typed events and structured logs
- 057 Logs context and saved views
- 058 Trace-events backbone
- 059 GraphQL operation explorer
- 060 gRPC stream and messaging lanes
- 061 Trace view modes
- 062 SQL workbench affordances
- 063 Playground trace-shape scenarios
- 064 Command center v1
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
