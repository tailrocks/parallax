import { createFileRoute } from "@tanstack/react-router"
import { useEffect, useMemo, useState } from "react"
import { gqlString, graphql } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

export const Route = createFileRoute("/sql")({ component: SqlPage })

interface SqlResult {
  columns: string[]
  rows: string[]
  rowCount: number
}

/** Cross-signal starters — the point of this page: one SQL surface over
 * logs, traces, metrics, and error events together. */
const EXAMPLES: Array<{ label: string; sql: string }> = [
  {
    label: "Slow spans + their error logs (join traces ↔ logs)",
    sql: `SELECT s.ts, s.service, s.name, s.duration_ns / 1000000 AS ms,
       l.severity_text, l.body
FROM otel_spans s
JOIN otel_logs l ON l.trace_id = s.trace_id
WHERE s.duration_ns > 10000000 AND l.severity_num >= 17
ORDER BY s.ts DESC LIMIT 50`,
  },
  {
    label: "Error events per service (last hour)",
    sql: `SELECT service, error_type, count(*) AS events
FROM error_events
WHERE ts >= now() - INTERVAL '1 hour'
GROUP BY service, error_type
ORDER BY events DESC`,
  },
  {
    label: "Log volume by severity per service",
    sql: `SELECT service, severity_text, count(*) AS lines
FROM otel_logs
WHERE ts >= now() - INTERVAL '1 hour'
GROUP BY service, severity_text
ORDER BY lines DESC`,
  },
  {
    label: "Run cross-section: spans, logs, metric points for one run",
    sql: `SELECT 'span' AS signal, count(*) AS rows FROM otel_spans WHERE run_id = 'jk-run-…'
UNION ALL SELECT 'log', count(*) FROM otel_logs WHERE run_id = 'jk-run-…'
UNION ALL SELECT 'metric point', count(*) FROM otel_metrics_points WHERE run_id = 'jk-run-…'`,
  },
  {
    label: "Slowest root spans (p-worst by name)",
    sql: `SELECT name, service, count(*) AS calls,
       max(duration_ns) / 1000000 AS worst_ms,
       avg(duration_ns) / 1000000 AS avg_ms
FROM otel_spans
WHERE parent_span_id IS NULL OR parent_span_id = ''
GROUP BY name, service
ORDER BY worst_ms DESC LIMIT 25`,
  },
]

const HISTORY_KEY = "parallax.sql.history"

function loadHistory(): string[] {
  try {
    const raw = localStorage.getItem(HISTORY_KEY)
    const parsed: unknown = raw ? JSON.parse(raw) : []
    return Array.isArray(parsed) ? (parsed as string[]) : []
  } catch {
    return []
  }
}

interface SchemaColumn {
  name: string
  dataType: string
}

function SqlPage() {
  const [statement, setStatement] = useState(EXAMPLES[0]?.sql ?? "")
  const [result, setResult] = useState<SqlResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [running, setRunning] = useState(false)
  const [elapsedMs, setElapsedMs] = useState<number | null>(null)
  const [history, setHistory] = useState<string[]>(loadHistory)
  const [schema, setSchema] = useState<Map<string, SchemaColumn[]>>(new Map())
  const [openTable, setOpenTable] = useState<string | null>(null)

  // Schema browser data: one information_schema read, grouped client-side.
  useEffect(() => {
    void graphql<{ sql: SqlResult }>(`
      {
        sql(
          query: "SELECT table_name, column_name, data_type FROM information_schema.columns WHERE table_schema = 'public' ORDER BY table_name, column_name"
        ) {
          columns
          rows
          rowCount
        }
      }
    `).then((data) => {
      const grouped = new Map<string, SchemaColumn[]>()
      for (const row of data.sql.rows) {
        try {
          const cells: unknown = JSON.parse(row)
          if (!Array.isArray(cells)) continue
          const [table, column, dataType] = cells as Array<string | undefined>
          if (!table || !column || !dataType) continue
          if (!grouped.has(table)) grouped.set(table, [])
          grouped.get(table)?.push({ name: column, dataType })
        } catch {
          // skip malformed rows
        }
      }
      setSchema(grouped)
    })
  }, [])

  async function run(sql: string) {
    setRunning(true)
    setError(null)
    const startedAt = performance.now()
    try {
      const data = await graphql<{ sql: SqlResult }>(
        `{ sql(query: "${gqlString(sql)}") { columns rows rowCount } }`
      )
      setResult(data.sql)
      setElapsedMs(performance.now() - startedAt)
      setHistory((current) => {
        const next = [sql, ...current.filter((q) => q !== sql)].slice(0, 20)
        try {
          localStorage.setItem(HISTORY_KEY, JSON.stringify(next))
        } catch {
          // storage full/blocked: history just doesn't persist
        }
        return next
      })
    } catch (e) {
      setResult(null)
      setElapsedMs(null)
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setRunning(false)
    }
  }

  const parsedRows = useMemo(
    () =>
      (result?.rows ?? []).map((row) => {
        try {
          const cells: unknown = JSON.parse(row)
          return Array.isArray(cells)
            ? cells.map((cell) =>
                typeof cell === "string" ? cell : JSON.stringify(cell)
              )
            : []
        } catch {
          return []
        }
      }),
    [result]
  )

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-3">
        <h1 className="text-lg font-semibold">SQL</h1>
        <p className="text-xs text-muted-foreground">
          Read-only SQL straight to the telemetry engine (GreptimeDB) — cross
          logs, traces, metrics, and error events in one statement. Single
          SELECT-shaped queries; same surface as <code>parallax sql</code>.
        </p>
      </div>

      <div className="grid gap-4 lg:grid-cols-[14rem_1fr]">
        <div className="space-y-2">
          <p className="text-xs font-medium text-muted-foreground">Tables</p>
          <ul className="space-y-1 text-xs">
            {[...schema.keys()].map((table) => (
              <li key={table}>
                <button
                  type="button"
                  className="font-mono underline-offset-4 hover:underline"
                  onClick={() =>
                    setOpenTable((current) =>
                      current === table ? null : table
                    )
                  }
                >
                  {table}
                </button>
                {openTable === table ? (
                  <ul className="mt-1 ml-3 space-y-0.5">
                    {(schema.get(table) ?? []).map((column) => (
                      <li key={column.name}>
                        <button
                          type="button"
                          className="font-mono text-muted-foreground hover:text-foreground"
                          title={column.dataType}
                          onClick={() =>
                            setStatement(
                              (current) => `${current} ${column.name}`
                            )
                          }
                        >
                          {column.name}{" "}
                          <span className="opacity-60">
                            {column.dataType.toLowerCase()}
                          </span>
                        </button>
                      </li>
                    ))}
                  </ul>
                ) : null}
              </li>
            ))}
          </ul>
        </div>

        <div className="space-y-3">
          <textarea
            name="sql-statement"
            value={statement}
            onChange={(event) => setStatement(event.target.value)}
            onKeyDown={(event) => {
              if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
                event.preventDefault()
                void run(statement)
              }
            }}
            rows={8}
            spellCheck={false}
            className="w-full rounded-md border bg-transparent p-3 font-mono text-xs shadow-xs outline-none focus-visible:ring-2 focus-visible:ring-ring"
          />
          <div className="flex flex-wrap items-center gap-2">
            <Button onClick={() => void run(statement)} disabled={running}>
              Run query
            </Button>
            <span className="text-xs text-muted-foreground">⌘⏎ runs</span>
            <select
              name="sql-examples"
              className="rounded-md border bg-transparent px-2 py-1.5 text-xs"
              value=""
              onChange={(event) => {
                const example = EXAMPLES[Number(event.target.value)]
                if (example) setStatement(example.sql)
              }}
            >
              <option value="" disabled>
                Examples (cross-signal)…
              </option>
              {EXAMPLES.map((example, index) => (
                <option key={example.label} value={index}>
                  {example.label}
                </option>
              ))}
            </select>
            {history.length > 0 ? (
              <select
                name="sql-history"
                className="rounded-md border bg-transparent px-2 py-1.5 text-xs"
                value=""
                onChange={(event) => {
                  const entry = history[Number(event.target.value)]
                  if (entry) setStatement(entry)
                }}
              >
                <option value="" disabled>
                  History…
                </option>
                {history.map((entry, index) => (
                  <option key={`${index}-${entry.slice(0, 20)}`} value={index}>
                    {entry.replace(/\s+/g, " ").slice(0, 72)}
                  </option>
                ))}
              </select>
            ) : null}
          </div>

          {error ? <p className="text-sm text-destructive">{error}</p> : null}

          {result ? (
            <div className="space-y-1 overflow-x-auto">
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <Badge variant="outline">{result.rowCount} row(s)</Badge>
                {elapsedMs != null ? (
                  <span>{elapsedMs.toFixed(0)} ms round-trip</span>
                ) : null}
              </div>
              <Table>
                <TableHeader>
                  <TableRow>
                    {result.columns.map((column) => (
                      <TableHead key={column}>{column}</TableHead>
                    ))}
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {parsedRows.map((cells, rowIndex) => (
                    <TableRow key={rowIndex}>
                      {cells.map((cell, cellIndex) => (
                        <TableCell
                          key={cellIndex}
                          className="max-w-md truncate font-mono text-xs"
                          title={cell}
                        >
                          {cell}
                        </TableCell>
                      ))}
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          ) : null}
        </div>
      </div>
    </div>
  )
}
