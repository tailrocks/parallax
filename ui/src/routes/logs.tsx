import { Link, createFileRoute } from "@tanstack/react-router"
import { useCallback, useEffect, useMemo, useState } from "react"
import { Bar, BarChart, CartesianGrid, XAxis, YAxis } from "recharts"
import { gqlString, graphql } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

interface LogDoc {
  tsNanos: string
  service: string
  severityNum: number
  severityText: string
  body: string
  traceId: string
  spanId: string
  runId: string | null
  scopeName: string
  attributes: string
  resource: string
}

interface SeriesPoint {
  tsNanos: string
  value: number
}

const RANGES = [
  { label: "Last 1 minute", minutes: 1 },
  { label: "Last 15 minutes", minutes: 15 },
  { label: "Last 30 minutes", minutes: 30 },
  { label: "Last 1 hour", minutes: 60 },
  { label: "Last 24 hours", minutes: 1440 },
  { label: "Last 7 days", minutes: 10080 },
  { label: "Last 30 days", minutes: 43200 },
] as const

const SEVERITIES = [
  { label: "All severities", min: 0 },
  { label: "Debug+", min: 5 },
  { label: "Info+", min: 9 },
  { label: "Warn+", min: 13 },
  { label: "Error+", min: 17 },
] as const

const REFRESH = [
  { label: "Refresh off", seconds: 0 },
  { label: "Every 5s", seconds: 5 },
  { label: "Every 15s", seconds: 15 },
  { label: "Every 60s", seconds: 60 },
] as const

export const Route = createFileRoute("/logs")({ component: LogsPage })

interface SqlResult {
  columns: string[]
  rows: string[]
  rowCount: number
}

/** Raw read-only SQL against the engine — the GreptimeDB power surface. */
function SqlPanel() {
  const [statement, setStatement] = useState(
    "SELECT ts, service, severity_text, body FROM otel_logs ORDER BY ts DESC LIMIT 50"
  )
  const [result, setResult] = useState<SqlResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [running, setRunning] = useState(false)

  async function run() {
    setRunning(true)
    setError(null)
    try {
      const data = await graphql<{ sql: SqlResult }>(
        `{ sql(query: "${gqlString(statement)}") { columns rows rowCount } }`
      )
      setResult(data.sql)
    } catch (e) {
      setResult(null)
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
    <div className="space-y-3">
      <p className="text-xs text-muted-foreground">
        Read-only SQL straight to GreptimeDB — tables: otel_logs, otel_spans,
        otel_metrics_points, otel_metrics_histograms, error_events.
      </p>
      <textarea
        value={statement}
        onChange={(event) => setStatement(event.target.value)}
        rows={4}
        spellCheck={false}
        className="w-full rounded-md border bg-transparent p-3 font-mono text-xs shadow-xs outline-none focus-visible:ring-2 focus-visible:ring-ring"
      />
      <Button onClick={() => void run()} disabled={running}>
        Run query
      </Button>
      {error ? <p className="text-sm text-destructive">{error}</p> : null}
      {result ? (
        <div className="space-y-1 overflow-x-auto">
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
          <p className="text-xs text-muted-foreground">
            {result.rowCount} row(s)
          </p>
        </div>
      ) : null}
    </div>
  )
}

function severityVariant(num: number): "destructive" | "secondary" | "outline" {
  if (num >= 17) return "destructive"
  if (num >= 13) return "secondary"
  return "outline"
}

function formatTime(tsNanos: string): string {
  return new Date(Number(BigInt(tsNanos) / 1_000_000n)).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  })
}

/** Flatten one log into ordered field/value rows for the doc viewer. */
function docFields(log: LogDoc): Array<[string, string]> {
  const rows: Array<[string, string]> = [
    [
      "@timestamp",
      new Date(Number(BigInt(log.tsNanos) / 1_000_000n)).toISOString(),
    ],
    ["severity", `${log.severityText} (${log.severityNum})`],
    ["service.name", log.service],
    ["body", log.body],
  ]
  if (log.traceId) rows.push(["trace_id", log.traceId])
  if (log.spanId) rows.push(["span_id", log.spanId])
  if (log.runId) rows.push(["run_id", log.runId])
  if (log.scopeName) rows.push(["scope.name", log.scopeName])
  for (const [prefix, json] of [
    ["", log.attributes],
    ["resource.", log.resource],
  ] as const) {
    try {
      const parsed: unknown = JSON.parse(json)
      if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
        for (const [key, value] of Object.entries(parsed)) {
          rows.push([
            `${prefix}${key}`,
            typeof value === "string" ? value : JSON.stringify(value),
          ])
        }
      }
    } catch {
      // non-object payloads stay out of the table
    }
  }
  return rows
}

const histogramConfig = {
  value: { label: "logs", color: "var(--chart-1)" },
} satisfies ChartConfig

function LogsPage() {
  const [services, setServices] = useState<string[]>([])
  const [service, setService] = useState<string>("all")
  const [severityMin, setSeverityMin] = useState<number>(0)
  const [query, setQuery] = useState("")
  const [pendingQuery, setPendingQuery] = useState("")
  const [rangeMinutes, setRangeMinutes] = useState<number>(15)
  const [refreshSeconds, setRefreshSeconds] = useState<number>(0)
  const [logs, setLogs] = useState<LogDoc[]>([])
  const [series, setSeries] = useState<SeriesPoint[]>([])
  const [selected, setSelected] = useState<LogDoc | null>(null)
  const [fieldSearch, setFieldSearch] = useState("")
  const [loading, setLoading] = useState(false)

  const load = useCallback(async () => {
    setLoading(true)
    try {
      const nowNanos = BigInt(Date.now()) * 1_000_000n
      const fromNanos = nowNanos - BigInt(rangeMinutes) * 60_000_000_000n
      const args = [
        `fromNanos: "${fromNanos}"`,
        `toNanos: "${nowNanos}"`,
        service !== "all" ? `service: "${gqlString(service)}"` : "",
        severityMin > 0 ? `severityMin: ${severityMin}` : "",
        query.trim() ? `query: "${gqlString(query.trim())}"` : "",
      ]
        .filter(Boolean)
        .join(", ")
      const stepSeconds = Math.max(1, Math.round((rangeMinutes * 60) / 60))
      const data = await graphql<{
        services: string[]
        logs: LogDoc[]
        logCountSeries: SeriesPoint[]
      }>(
        `{
          services
          logs(${args}, limit: 500) {
            tsNanos service severityNum severityText body
            traceId spanId runId scopeName attributes resource
          }
          logCountSeries(${args}, stepSeconds: ${stepSeconds}) {
            tsNanos value
          }
        }`
      )
      setServices(data.services)
      setLogs(data.logs)
      setSeries(data.logCountSeries)
    } finally {
      setLoading(false)
    }
  }, [service, severityMin, query, rangeMinutes])

  useEffect(() => {
    void load()
  }, [load])

  // Live mode: poll on the chosen interval (Kibana-style refresh-every).
  useEffect(() => {
    if (refreshSeconds === 0) return
    const timer = setInterval(() => void load(), refreshSeconds * 1000)
    return () => clearInterval(timer)
  }, [refreshSeconds, load])

  const chartData = useMemo(
    () =>
      series.map((point) => ({
        time: formatTime(point.tsNanos),
        value: point.value,
      })),
    [series]
  )
  const total = useMemo(
    () => series.reduce((acc, point) => acc + point.value, 0),
    [series]
  )
  const selectedFields = useMemo(() => {
    if (!selected) return []
    const all = docFields(selected)
    const needle = fieldSearch.trim().toLowerCase()
    if (!needle) return all
    return all.filter(
      ([key, value]) =>
        key.toLowerCase().includes(needle) ||
        value.toLowerCase().includes(needle)
    )
  }, [selected, fieldSearch])

  const [mode, setMode] = useState<"filters" | "sql">("filters")

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <h1 className="text-lg font-semibold">Logs</h1>
        <Select
          value={mode}
          onValueChange={(v) => setMode(v === "sql" ? "sql" : "filters")}
        >
          <SelectTrigger className="w-32">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="filters">Filters</SelectItem>
            <SelectItem value="sql">SQL</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {mode === "sql" ? <SqlPanel /> : null}

      <div
        className={
          mode === "sql" ? "hidden" : "flex flex-wrap items-center gap-2"
        }
      >
        <Select value={service} onValueChange={(v) => setService(v ?? "all")}>
          <SelectTrigger className="w-48">
            <SelectValue placeholder="All services" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All services</SelectItem>
            {services.map((s) => (
              <SelectItem key={s} value={s}>
                {s}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select
          value={String(severityMin)}
          onValueChange={(v) => setSeverityMin(Number(v ?? 0))}
        >
          <SelectTrigger className="w-36">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {SEVERITIES.map((s) => (
              <SelectItem key={s.min} value={String(s.min)}>
                {s.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <form
          className="flex min-w-64 flex-1 gap-2"
          onSubmit={(event) => {
            event.preventDefault()
            setQuery(pendingQuery)
          }}
        >
          <Input
            value={pendingQuery}
            onChange={(event) => setPendingQuery(event.target.value)}
            placeholder="Filter log bodies (substring)"
          />
        </form>
        <Select
          value={String(rangeMinutes)}
          onValueChange={(v) => setRangeMinutes(Number(v ?? 15))}
        >
          <SelectTrigger className="w-44">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {RANGES.map((range) => (
              <SelectItem key={range.minutes} value={String(range.minutes)}>
                {range.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select
          value={String(refreshSeconds)}
          onValueChange={(v) => setRefreshSeconds(Number(v ?? 0))}
        >
          <SelectTrigger className="w-36">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {REFRESH.map((option) => (
              <SelectItem key={option.seconds} value={String(option.seconds)}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button
          onClick={() => {
            setQuery(pendingQuery)
            void load()
          }}
          disabled={loading}
        >
          Refresh
        </Button>
      </div>

      <div className={mode === "sql" ? "hidden" : "space-y-1"}>
        <ChartContainer config={histogramConfig} className="h-32 w-full">
          <BarChart data={chartData} margin={{ left: 8, right: 8, top: 4 }}>
            <CartesianGrid vertical={false} />
            <XAxis
              dataKey="time"
              tickLine={false}
              axisLine={false}
              minTickGap={48}
            />
            <YAxis tickLine={false} axisLine={false} width={48} />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Bar dataKey="value" fill="var(--color-value)" radius={2} />
          </BarChart>
        </ChartContainer>
        <p className="text-xs text-muted-foreground">
          {total.toLocaleString()} log(s) in range · showing newest{" "}
          {logs.length}
          {refreshSeconds > 0 ? ` · live (every ${refreshSeconds}s)` : ""}
        </p>
      </div>

      {mode === "sql" ? null : logs.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          No logs match — widen the range or drop a filter.
        </p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-28">Time</TableHead>
              <TableHead className="w-24">Severity</TableHead>
              <TableHead className="w-36">Service</TableHead>
              <TableHead>Body</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {logs.map((log, index) => (
              <TableRow
                key={`${log.tsNanos}-${index}`}
                className="cursor-pointer"
                onClick={() => {
                  setSelected(log)
                  setFieldSearch("")
                }}
              >
                <TableCell className="font-mono text-xs">
                  {formatTime(log.tsNanos)}
                </TableCell>
                <TableCell>
                  <Badge variant={severityVariant(log.severityNum)}>
                    {log.severityText || "—"}
                  </Badge>
                </TableCell>
                <TableCell className="truncate">{log.service}</TableCell>
                <TableCell className="max-w-xl truncate font-mono text-xs">
                  {log.body}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}

      <Sheet
        open={selected !== null}
        onOpenChange={(open) => {
          if (!open) setSelected(null)
        }}
      >
        <SheetContent className="w-full overflow-y-auto sm:max-w-xl">
          <SheetHeader>
            <SheetTitle>Log document</SheetTitle>
            <SheetDescription>
              {selected
                ? `${selected.service} · ${formatTime(selected.tsNanos)}`
                : ""}
            </SheetDescription>
          </SheetHeader>
          {selected ? (
            <div className="space-y-3 px-4 pb-6">
              <Input
                value={fieldSearch}
                onChange={(event) => setFieldSearch(event.target.value)}
                placeholder="Search field names or values"
              />
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-44">Field</TableHead>
                    <TableHead>Value</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {selectedFields.map(([key, value]) => (
                    <TableRow key={key}>
                      <TableCell className="align-top font-medium">
                        {key}
                      </TableCell>
                      <TableCell className="font-mono text-xs break-all whitespace-pre-wrap">
                        {key === "trace_id" ? (
                          <Link
                            to="/traces/$traceId"
                            params={{ traceId: value }}
                            className="underline underline-offset-4"
                          >
                            {value}
                          </Link>
                        ) : key === "run_id" ? (
                          <Link
                            to="/runs/$runId"
                            params={{ runId: value }}
                            className="underline underline-offset-4"
                          >
                            {value}
                          </Link>
                        ) : (
                          value
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          ) : null}
        </SheetContent>
      </Sheet>
    </div>
  )
}
