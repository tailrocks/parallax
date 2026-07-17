# UI defect ledger (plan 160)

Audit date: 2026-07-17. Corpus: playground `docs/corner-case-matrix.md`
(plan 161), seeded fresh against the live 12-container compose stack into a
live `parallax serve` (http://127.0.0.1:4000, managed GreptimeDB). Browser
tooling: Chrome DevTools MCP (sanctioned fallback; agent-browser CDP
screenshot path broken on this host — see plan 157 evidence README).
Checklist per cell: (1) data correctness vs matrix expectation, (2) links
navigate, (3) empty/loading/error states, (4) layout 1440px + 375px,
(5) live behavior, (6) clean console.

Screenshots: `ui/audit/*.png` in this directory.

## Defect records

Format: `D-NNN: surface, corpus id, expected, observed, severity
(broken|wrong|degraded|cosmetic), status`.

(populated during the walk)

## Audit grid

Legend: `pass` / `D-NNN` / `pending`.

### Traces × t-* / p-*

| Cell | Corpus id | Verdict | Evidence |
|---|---|---|---|
| Waterfall deep nesting | t-deep | pending | |
| Waterfall wide + minimap | t-wide | pending | |
| Waterfall multi-root | t-multiroot | pending | |
| Waterfall orphan | t-orphan | pending | |
| Waterfall skew | t-skew | pending | |
| Waterfall zero-duration | t-zero | pending | |
| Links panel | t-links | pending | |
| Long names inspector | t-longnames | pending | |
| Span events panel | t-events | pending | |
| RPC status codes | p-grpc-err | pending | |
| RPC stream panel | p-grpc-stream | pending | |
| GraphQL ops panel | p-graphql-err | pending | |
| Kafka lag + jobs | p-kafka-lag | pending | |

### Logs × l-*

| Cell | Corpus id | Verdict | Evidence |
|---|---|---|---|
| Live tail + histogram burst | l-burst | pending | |
| Bodies: JSON/32KiB/ANSI/blank/equal-ts | l-bodies | pending | |

### Metrics × m-*

| Cell | Corpus id | Verdict | Evidence |
|---|---|---|---|
| Counter reset / gauge gap / exemplar | m-shapes | pending | |

### Issues × e-*

| Cell | Corpus id | Verdict | Evidence |
|---|---|---|---|
| Grouped burst + type breakdown | e-burst | pending | |
| Multi-language fingerprints | e-multi-lang | pending | |

### Invocations / journey × j-*

| Cell | Corpus id | Verdict | Evidence |
|---|---|---|---|
| Journey happy narrative | j-happy | pending | |
| Journey error attribution | j-error | pending | |
| Journey outside bucket | j-outside | pending | |
| Sessions chain | j-reattach | pending | |
| Parallel isolation | j-parallel | pending | |

### Ecosystem / overview

| Cell | Corpus id | Verdict | Evidence |
|---|---|---|---|
| Ecosystem graph kinds + edges | eco-full | pending | |
| Overview charts | sweep | pending | |

## Generic-attributes conformance sweep (step 4)

(pending)

## Closure summary (step 5)

(pending)
