import { createFileRoute } from "@tanstack/react-router"
import { useEffect, useMemo, useRef, useState } from "react"
import {
  IconDatabase,
  IconHistory,
  IconPlayerPlay,
  IconTable,
} from "@tabler/icons-react"

import { PageHeader } from "@/components/page-header"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Kbd, KbdGroup } from "@/components/ui/kbd"
import { ScrollArea } from "@/components/ui/scroll-area"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { gqlString, graphql } from "@/lib/api"
import { formatCount } from "@/lib/format"

export const Route = createFileRoute("/sql")({ component: SqlPage })

interface SqlResult {
  columns: string[]
  rows: string[]
  rowCount: number
}

export const EXAMPLES: Array<{ label: string; sql: string }> = [
  {
    label: "Slow spans + error logs",
    sql: `SELECT s."timestamp", s.service_name, s.span_name,
       s.duration_nano / 1000000 AS ms,
       l.severity_text, l.body
FROM opentelemetry_traces s
JOIN opentelemetry_logs l ON l.trace_id = s.trace_id
WHERE s.duration_nano > 10000000 AND l.severity_number >= 17
ORDER BY s."timestamp" DESC LIMIT 50`,
  },
  {
    label: "Error events per service",
    sql: `SELECT service, error_type, count(*) AS events
FROM error_events
WHERE ts >= now() - INTERVAL '1 hour'
GROUP BY service, error_type
ORDER BY events DESC`,
  },
  {
    label: "Log volume by severity",
    sql: `SELECT severity_text, count(*) AS lines
FROM opentelemetry_logs
WHERE "timestamp" >= now() - INTERVAL '1 hour'
GROUP BY severity_text
ORDER BY lines DESC`,
  },
  {
    label: "Run cross-section",
    sql: `SELECT 'span linked to run log' AS signal, count(DISTINCT s.span_id) AS rows
FROM opentelemetry_traces s
JOIN opentelemetry_logs l ON l.trace_id = s.trace_id
WHERE l."parallax.run.id" = '<run-id>'
UNION ALL
SELECT 'log', count(*)
FROM opentelemetry_logs
WHERE "parallax.run.id" = '<run-id>'
UNION ALL
SELECT 'metric point', count(*)
FROM run_metric_points
WHERE run_id = '<run-id>'`,
  },
  {
    label: "Slowest root spans",
    sql: `SELECT span_name, service_name, count(*) AS calls,
       max(duration_nano) / 1000000 AS worst_ms,
       avg(duration_nano) / 1000000 AS avg_ms
FROM opentelemetry_traces
WHERE parent_span_id IS NULL OR parent_span_id = ''
GROUP BY span_name, service_name
ORDER BY max(duration_nano) DESC LIMIT 25`,
  },
]

const HISTORY_KEY = "parallax.sql.history"

function loadHistory(): string[] {
  try {
    const parsed: unknown = JSON.parse(
      localStorage.getItem(HISTORY_KEY) ?? "[]"
    )
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
  const editorRef = useRef<HTMLTextAreaElement>(null)
  const [statement, setStatement] = useState(EXAMPLES[0]?.sql ?? "")
  const [result, setResult] = useState<SqlResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [running, setRunning] = useState(false)
  const [elapsedMs, setElapsedMs] = useState<number | null>(null)
  const [history, setHistory] = useState<string[]>(loadHistory)
  const [schema, setSchema] = useState<Map<string, SchemaColumn[]>>(new Map())
  const [openTable, setOpenTable] = useState<string | null>(null)

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

  function insertIdentifier(identifier: string) {
    const textarea = editorRef.current
    if (!textarea) {
      setStatement((current) => `${current} ${identifier}`)
      return
    }
    const start = textarea.selectionStart
    const end = textarea.selectionEnd
    setStatement(
      (current) =>
        `${current.slice(0, start)}${identifier}${current.slice(end)}`
    )
    requestAnimationFrame(() => {
      textarea.focus()
      textarea.setSelectionRange(
        start + identifier.length,
        start + identifier.length
      )
    })
  }

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
        localStorage.setItem(HISTORY_KEY, JSON.stringify(next))
        return next
      })
    } catch (err) {
      setResult(null)
      setElapsedMs(null)
      setError(err instanceof Error ? err.message : String(err))
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
    <div className="flex flex-col gap-4">
      <PageHeader
        icon={IconDatabase}
        iconClassName="text-yellow-500"
        title="SQL"
        description="Read-only queries over telemetry tables."
      />

      <div className="grid gap-4 lg:grid-cols-[16rem_1fr]">
        <Card className="h-fit">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-sm">
              <IconTable />
              Tables
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ScrollArea className="h-[520px]">
              <ul className="flex flex-col gap-1 text-xs">
                {[...schema.keys()].map((table) => (
                  <li key={table}>
                    <button
                      type="button"
                      className="font-mono hover:underline"
                      onClick={() =>
                        setOpenTable((current) =>
                          current === table ? null : table
                        )
                      }
                    >
                      {table}
                    </button>
                    {openTable === table ? (
                      <ul className="mt-1 ml-3 flex flex-col gap-0.5">
                        {(schema.get(table) ?? []).map((column) => (
                          <li key={column.name}>
                            <button
                              type="button"
                              className="font-mono text-muted-foreground hover:text-foreground"
                              onClick={() => insertIdentifier(column.name)}
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
            </ScrollArea>
          </CardContent>
        </Card>

        <div className="flex flex-col gap-3">
          <Card>
            <CardHeader className="flex-row items-center justify-between">
              <CardTitle className="text-sm">Editor</CardTitle>
              <KbdGroup>
                <Kbd>⌘</Kbd>
                <Kbd>Enter</Kbd>
              </KbdGroup>
            </CardHeader>
            <CardContent className="flex flex-col gap-3">
              <textarea
                ref={editorRef}
                name="sql-statement"
                value={statement}
                onChange={(event) => setStatement(event.target.value)}
                onKeyDown={(event) => {
                  if (
                    (event.metaKey || event.ctrlKey) &&
                    event.key === "Enter"
                  ) {
                    event.preventDefault()
                    void run(statement)
                  }
                }}
                rows={9}
                spellCheck={false}
                className="min-h-56 w-full rounded-md border bg-background p-3 font-mono text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring"
              />
              <div className="flex flex-wrap items-center gap-2">
                <Button onClick={() => void run(statement)} disabled={running}>
                  <IconPlayerPlay />
                  Run query
                </Button>
                <DropdownMenu>
                  <DropdownMenuTrigger render={<Button variant="outline" />}>
                    Examples
                  </DropdownMenuTrigger>
                  <DropdownMenuContent>
                    <DropdownMenuGroup>
                      {EXAMPLES.map((example) => (
                        <DropdownMenuItem
                          key={example.label}
                          onClick={() => setStatement(example.sql)}
                        >
                          {example.label}
                        </DropdownMenuItem>
                      ))}
                    </DropdownMenuGroup>
                  </DropdownMenuContent>
                </DropdownMenu>
                {history.length > 0 ? (
                  <DropdownMenu>
                    <DropdownMenuTrigger render={<Button variant="outline" />}>
                      <IconHistory />
                      History
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      <DropdownMenuGroup>
                        {history.map((entry, index) => (
                          <DropdownMenuItem
                            key={`${index}-${entry.slice(0, 20)}`}
                            onClick={() => setStatement(entry)}
                          >
                            {entry.replace(/\s+/g, " ").slice(0, 72)}
                          </DropdownMenuItem>
                        ))}
                      </DropdownMenuGroup>
                    </DropdownMenuContent>
                  </DropdownMenu>
                ) : null}
                {elapsedMs != null ? (
                  <span className="text-xs text-muted-foreground">
                    {elapsedMs.toFixed(0)} ms
                  </span>
                ) : null}
              </div>
              {error ? (
                <p className="text-sm text-destructive">{error}</p>
              ) : null}
            </CardContent>
          </Card>

          {result ? (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-sm">
                  Query result
                  <Badge variant="outline">
                    {formatCount(result.rowCount)} rows
                  </Badge>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <ScrollArea className="max-h-[520px] overflow-auto">
                  <Table>
                    <TableHeader className="sticky top-0 bg-card">
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
                </ScrollArea>
              </CardContent>
            </Card>
          ) : null}
        </div>
      </div>
    </div>
  )
}
