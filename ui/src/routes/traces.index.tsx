import { Link, createFileRoute, useNavigate } from "@tanstack/react-router"
import { useCallback, useEffect, useState } from "react"
import { gqlString, graphql } from "@/lib/api"
import type { TraceSummary } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

/** One finished span from the live feed (`/v1/traces/stream`). */
interface SpanDoc {
  tsNanos: string
  service: string
  traceId: string
  spanId: string
  name: string
  kind: string
  statusCode: string
  durationNs: string
}

const RANGES = [
  { label: "Latest", minutes: 0 },
  { label: "Last 1 minute", minutes: 1 },
  { label: "Last 15 minutes", minutes: 15 },
  { label: "Last 30 minutes", minutes: 30 },
  { label: "Last 1 hour", minutes: 60 },
  { label: "Last 24 hours", minutes: 1440 },
  { label: "Last 7 days", minutes: 10080 },
  { label: "Last 30 days", minutes: 43200 },
] as const

const REFRESH = [
  { label: "Refresh off", seconds: 0 },
  { label: "Live (stream)", seconds: -1 },
  { label: "Every 5s", seconds: 5 },
  { label: "Every 15s", seconds: 15 },
  { label: "Every 60s", seconds: 60 },
] as const

export const Route = createFileRoute("/traces/")({ component: TracesPage })

function formatTime(tsNanos: string): string {
  return new Date(Number(BigInt(tsNanos) / 1_000_000n)).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  })
}

function formatMillis(durationNs: string): string {
  return `${(Number(durationNs) / 1e6).toFixed(1)}ms`
}

/** "500ms" | "2s" | "1m" | bare millis → milliseconds (NaN = no filter). */
function parseDurationMs(value: string): number {
  const v = value.trim().toLowerCase()
  if (!v) return NaN
  if (v.endsWith("ms")) return Number(v.slice(0, -2))
  if (v.endsWith("s")) return Number(v.slice(0, -1)) * 1000
  if (v.endsWith("m")) return Number(v.slice(0, -1)) * 60_000
  return Number(v)
}

function TracesPage() {
  const [services, setServices] = useState<string[]>([])
  const [service, setService] = useState<string>("all")
  const [errorsOnly, setErrorsOnly] = useState(false)
  const [minDuration, setMinDuration] = useState("")
  const [pendingMinDuration, setPendingMinDuration] = useState("")
  const [query, setQuery] = useState("")
  const [pendingQuery, setPendingQuery] = useState("")
  // 0 = "Latest": newest traces regardless of window.
  const [rangeMinutes, setRangeMinutes] = useState<number>(0)
  // Live is explicit, never the default — a tail costs a subscription.
  const [refreshSeconds, setRefreshSeconds] = useState<number>(0)
  const [traces, setTraces] = useState<TraceSummary[]>([])
  const [spans, setSpans] = useState<SpanDoc[]>([])
  const [lookup, setLookup] = useState("")
  const [loading, setLoading] = useState(false)
  const navigate = useNavigate()
  const live = refreshSeconds === -1

  const load = useCallback(async () => {
    setLoading(true)
    try {
      const nowNanos = BigInt(Date.now()) * 1_000_000n
      const minDurationMs = parseDurationMs(minDuration)
      const args = [
        service !== "all" ? `service: "${gqlString(service)}"` : "",
        rangeMinutes > 0
          ? `fromNanos: "${nowNanos - BigInt(rangeMinutes) * 60_000_000_000n}"`
          : "",
        rangeMinutes > 0 ? `toNanos: "${nowNanos}"` : "",
        Number.isFinite(minDurationMs) && minDurationMs > 0
          ? `minDurationMs: ${minDurationMs}`
          : "",
        errorsOnly ? "errorOnly: true" : "",
        query.trim() ? `query: "${gqlString(query.trim())}"` : "",
        "limit: 100",
      ]
        .filter(Boolean)
        .join(", ")
      const data = await graphql<{
        services: string[]
        traces: TraceSummary[]
      }>(
        `{
          services
          traces(${args}) {
            traceId rootName service startNanos durationNs spanCount hasError
          }
        }`
      )
      setServices(data.services)
      setTraces(data.traces)
    } finally {
      setLoading(false)
    }
  }, [service, errorsOnly, minDuration, query, rangeMinutes])

  useEffect(() => {
    void load()
  }, [load])

  // Refresh-every mode: poll on the chosen interval.
  useEffect(() => {
    if (refreshSeconds <= 0) return
    const timer = setInterval(() => void load(), refreshSeconds * 1000)
    return () => clearInterval(timer)
  }, [refreshSeconds, load])

  // Live mode: a finished-span tail over SSE — per-row filters only
  // (service, duration floor, errors, name substring); aggregates and
  // time ranges belong to refresh mode.
  useEffect(() => {
    if (!live) return
    const params = new URLSearchParams()
    if (service !== "all") params.set("service", service)
    const minDurationMs = parseDurationMs(minDuration)
    if (Number.isFinite(minDurationMs) && minDurationMs > 0) {
      params.set("min_duration_ms", String(minDurationMs))
    }
    if (errorsOnly) params.set("errors_only", "true")
    if (query.trim()) params.set("q", query.trim())
    setSpans([])
    const source = new EventSource(`/v1/traces/stream?${params}`)
    let buffer: SpanDoc[] = []
    source.onmessage = (event) => {
      try {
        const batch: unknown = JSON.parse(event.data as string)
        if (Array.isArray(batch)) buffer.push(...(batch as SpanDoc[]))
      } catch {
        // skip malformed frames
      }
    }
    const flush = setInterval(() => {
      if (buffer.length === 0) return
      const incoming = buffer
      buffer = []
      setSpans((current) => [...incoming.reverse(), ...current].slice(0, 500))
    }, 250)
    return () => {
      source.close()
      clearInterval(flush)
    }
  }, [live, service, errorsOnly, minDuration, query])

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-3">
        <h1 className="text-lg font-semibold">Traces</h1>
        <form
          className="flex gap-2"
          onSubmit={(event) => {
            event.preventDefault()
            const id = lookup.trim()
            if (id) {
              void navigate({ to: "/traces/$traceId", params: { traceId: id } })
            }
          }}
        >
          <Input
            value={lookup}
            onChange={(event) => setLookup(event.target.value)}
            placeholder="Open a trace id…"
            className="w-72 font-mono text-xs"
          />
          <Button type="submit" variant="outline">
            Open
          </Button>
        </form>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <Select value={service} onValueChange={(v) => setService(v ?? "all")}>
          <SelectTrigger className="w-48">
            <SelectValue>
              {service === "all" ? "All services" : service}
            </SelectValue>
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
          value={errorsOnly ? "errors" : "all"}
          onValueChange={(v) => setErrorsOnly(v === "errors")}
        >
          <SelectTrigger className="w-36">
            <SelectValue>
              {errorsOnly ? "Errors only" : "All statuses"}
            </SelectValue>
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All statuses</SelectItem>
            <SelectItem value="errors">Errors only</SelectItem>
          </SelectContent>
        </Select>
        <form
          className="flex gap-2"
          onSubmit={(event) => {
            event.preventDefault()
            setMinDuration(pendingMinDuration)
          }}
        >
          <Input
            value={pendingMinDuration}
            onChange={(event) => setPendingMinDuration(event.target.value)}
            placeholder="Min duration (500ms, 2s)"
            className="w-44"
          />
        </form>
        <form
          className="flex min-w-56 flex-1 gap-2"
          onSubmit={(event) => {
            event.preventDefault()
            setQuery(pendingQuery)
          }}
        >
          <Input
            value={pendingQuery}
            onChange={(event) => setPendingQuery(event.target.value)}
            placeholder="Filter span names (substring)"
          />
        </form>
        <Select
          value={String(rangeMinutes)}
          onValueChange={(v) => setRangeMinutes(Number(v ?? 0))}
          disabled={live}
        >
          <SelectTrigger className="w-44">
            <SelectValue>
              {RANGES.find((r) => r.minutes === rangeMinutes)?.label}
            </SelectValue>
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
            <SelectValue>
              {REFRESH.find((o) => o.seconds === refreshSeconds)?.label}
            </SelectValue>
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
            setMinDuration(pendingMinDuration)
            void load()
          }}
          disabled={loading || live}
        >
          Refresh
        </Button>
      </div>

      {live ? (
        <>
          <p className="text-xs text-muted-foreground">
            live span tail · {spans.length} shown · per-row filters only
            (service, min duration, errors, name) — switch off Live for time
            ranges and trace aggregates
          </p>
          {spans.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              Waiting for spans — they appear here the moment a service exports
              them.
            </p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-28">Time</TableHead>
                  <TableHead className="w-36">Service</TableHead>
                  <TableHead>Span</TableHead>
                  <TableHead className="w-28 text-right">Duration</TableHead>
                  <TableHead className="w-24">Status</TableHead>
                  <TableHead className="w-44">Trace</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {spans.map((span, index) => (
                  <TableRow key={`${span.spanId}-${index}`}>
                    <TableCell className="font-mono text-xs">
                      {formatTime(span.tsNanos)}
                    </TableCell>
                    <TableCell className="truncate">{span.service}</TableCell>
                    <TableCell className="max-w-md truncate">
                      {span.name}
                    </TableCell>
                    <TableCell className="text-right font-mono text-xs tabular-nums">
                      {formatMillis(span.durationNs)}
                    </TableCell>
                    <TableCell>
                      {span.statusCode === "STATUS_CODE_ERROR" ? (
                        <Badge variant="destructive">error</Badge>
                      ) : (
                        <Badge variant="outline">ok</Badge>
                      )}
                    </TableCell>
                    <TableCell className="font-mono text-xs">
                      <Link
                        to="/traces/$traceId"
                        params={{ traceId: span.traceId }}
                        className="underline underline-offset-4"
                      >
                        {span.traceId.slice(0, 16)}…
                      </Link>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </>
      ) : (
        <>
          <p className="text-xs text-muted-foreground">
            {traces.length} trace(s) · newest first
            {refreshSeconds > 0 ? ` · refreshing every ${refreshSeconds}s` : ""}
          </p>
          {traces.length === 0 ? (
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">
                No traces in this window — widen the range or drop a filter.
              </p>
              {rangeMinutes !== 0 ? (
                <Button variant="outline" onClick={() => setRangeMinutes(0)}>
                  Show latest traces
                </Button>
              ) : null}
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-28">Started</TableHead>
                  <TableHead>Root span</TableHead>
                  <TableHead className="w-36">Service</TableHead>
                  <TableHead className="w-20 text-right">Spans</TableHead>
                  <TableHead className="w-28 text-right">Duration</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {traces.map((trace) => (
                  <TableRow key={trace.traceId}>
                    <TableCell className="font-mono text-xs">
                      {formatTime(trace.startNanos)}
                    </TableCell>
                    <TableCell className="max-w-md truncate">
                      <Link
                        to="/traces/$traceId"
                        params={{ traceId: trace.traceId }}
                        className="underline underline-offset-4"
                      >
                        {trace.rootName}
                      </Link>{" "}
                      {trace.hasError ? (
                        <Badge variant="destructive">error</Badge>
                      ) : null}
                    </TableCell>
                    <TableCell className="truncate">{trace.service}</TableCell>
                    <TableCell className="text-right tabular-nums">
                      {trace.spanCount}
                    </TableCell>
                    <TableCell className="text-right font-mono text-xs tabular-nums">
                      {formatMillis(trace.durationNs)}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </>
      )}
    </div>
  )
}
