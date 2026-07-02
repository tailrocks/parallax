import { Link, createFileRoute } from "@tanstack/react-router"
import { useCallback, useEffect, useMemo, useState } from "react"
import {
  ActivityIcon,
  BarChart3Icon,
  RadioIcon,
  ScrollTextIcon,
} from "lucide-react"
import { Bar, BarChart, CartesianGrid, XAxis, YAxis } from "recharts"
import { gqlString, graphql } from "@/lib/api"
import { LogsTable, formatTime } from "@/components/logs-table"
import type { LogDoc } from "@/components/logs-table"
import { Button } from "@/components/ui/button"
import { KpiCard } from "@/components/kpi-card"
import { LiveEventStack, LiveStreamPanel } from "@/components/live-stream-panel"
import { PageHeading } from "@/components/page-heading"
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

interface SeriesPoint {
  tsNanos: string
  value: number
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

const SEVERITIES = [
  { label: "All severities", min: 0 },
  { label: "Debug+", min: 5 },
  { label: "Info+", min: 9 },
  { label: "Warn+", min: 13 },
  { label: "Error+", min: 17 },
] as const

const REFRESH = [
  { label: "Refresh off", seconds: 0 },
  { label: "Live (stream)", seconds: -1 },
  { label: "Every 5s", seconds: 5 },
  { label: "Every 15s", seconds: 15 },
  { label: "Every 60s", seconds: 60 },
] as const

export const Route = createFileRoute("/logs")({ component: LogsPage })

const histogramConfig = {
  value: { label: "logs", color: "var(--chart-1)" },
} satisfies ChartConfig

function LogsPage() {
  const [services, setServices] = useState<string[]>([])
  const [service, setService] = useState<string>("all")
  const [severityMin, setSeverityMin] = useState<number>(0)
  const [query, setQuery] = useState("")
  const [pendingQuery, setPendingQuery] = useState("")
  // 0 = "Latest": newest rows regardless of window (kubectl --tail shape).
  const [rangeMinutes, setRangeMinutes] = useState<number>(0)
  const [refreshSeconds, setRefreshSeconds] = useState<number>(0)
  const [logs, setLogs] = useState<LogDoc[]>([])
  const [series, setSeries] = useState<SeriesPoint[]>([])
  const [loading, setLoading] = useState(false)
  const [olderLoading, setOlderLoading] = useState(false)
  const [exhausted, setExhausted] = useState(false)
  // Live is explicit, never the default — a tail costs a subscription, and
  // it narrows the surface: per-row filters only, no ranges, no aggregates.
  const live = refreshSeconds === -1

  const load = useCallback(async () => {
    setLoading(true)
    try {
      // "Latest" (0): newest rows with no lower bound — kubectl --tail.
      // The histogram still needs a window; 24h backs it in that mode.
      const nowNanos = BigInt(Date.now()) * 1_000_000n
      const windowMinutes = rangeMinutes === 0 ? 1440 : rangeMinutes
      const fromNanos = nowNanos - BigInt(windowMinutes) * 60_000_000_000n
      const shared = [
        service !== "all" ? `service: "${gqlString(service)}"` : "",
        severityMin > 0 ? `severityMin: ${severityMin}` : "",
        query.trim() ? `query: "${gqlString(query.trim())}"` : "",
      ].filter(Boolean)
      const logArgs = [
        ...(rangeMinutes === 0
          ? []
          : [`fromNanos: "${fromNanos}"`, `toNanos: "${nowNanos}"`]),
        ...shared,
        "limit: 500",
      ].join(", ")
      const seriesArgs = [
        `fromNanos: "${fromNanos}"`,
        `toNanos: "${nowNanos}"`,
        ...shared,
      ].join(", ")
      const stepSeconds = Math.max(1, Math.round((windowMinutes * 60) / 60))
      const data = await graphql<{
        services: string[]
        logs: LogDoc[]
        logCountSeries: SeriesPoint[]
      }>(
        `{
          services
          logs(${logArgs}) {
            tsNanos service severityNum severityText body
            traceId spanId runId scopeName attributes resource
          }
          logCountSeries(${seriesArgs}, stepSeconds: ${stepSeconds}) {
            tsNanos value
          }
        }`
      )
      setServices(data.services)
      setLogs(data.logs)
      setSeries(data.logCountSeries)
      setExhausted(data.logs.length < 500)
    } finally {
      setLoading(false)
    }
  }, [service, severityMin, query, rangeMinutes])

  // Cursor pagination: everything strictly older than the oldest row shown,
  // same filters, appended below (kubectl --tail … then scroll back).
  const loadOlder = useCallback(async () => {
    const oldest = logs[logs.length - 1]
    if (!oldest) return
    setOlderLoading(true)
    try {
      const args = [
        `toNanos: "${(BigInt(oldest.tsNanos) - 1n).toString()}"`,
        rangeMinutes > 0
          ? `fromNanos: "${(BigInt(Date.now()) - BigInt(rangeMinutes) * 60_000n) * 1_000_000n}"`
          : "",
        service !== "all" ? `service: "${gqlString(service)}"` : "",
        severityMin > 0 ? `severityMin: ${severityMin}` : "",
        query.trim() ? `query: "${gqlString(query.trim())}"` : "",
        "limit: 500",
      ]
        .filter(Boolean)
        .join(", ")
      const data = await graphql<{ logs: LogDoc[] }>(
        `{ logs(${args}) {
             tsNanos service severityNum severityText body
             traceId spanId runId scopeName attributes resource
           } }`
      )
      setLogs((current) => [...current, ...data.logs])
      if (data.logs.length < 500) setExhausted(true)
    } finally {
      setOlderLoading(false)
    }
  }, [logs, rangeMinutes, service, severityMin, query])

  useEffect(() => {
    void load()
  }, [load])

  // Refresh-every mode: poll on the chosen interval.
  useEffect(() => {
    if (refreshSeconds <= 0) return
    const timer = setInterval(() => void load(), refreshSeconds * 1000)
    return () => clearInterval(timer)
  }, [refreshSeconds, load])

  // Live mode (-1): tail over Server-Sent Events. Incoming batches buffer
  // and flush every 250ms (render batching), newest first, capped at 500.
  useEffect(() => {
    if (!live) return
    const params = new URLSearchParams()
    if (service !== "all") params.set("service", service)
    if (severityMin > 0) params.set("severity_min", String(severityMin))
    if (query.trim()) params.set("q", query.trim())
    const source = new EventSource(`/v1/logs/stream?${params}`)
    let buffer: LogDoc[] = []
    source.onmessage = (event) => {
      try {
        const batch: unknown = JSON.parse(event.data as string)
        if (Array.isArray(batch)) buffer.push(...(batch as LogDoc[]))
      } catch {
        // skip malformed frames
      }
    }
    const flush = setInterval(() => {
      if (buffer.length === 0) return
      const incoming = buffer
      buffer = []
      setLogs((current) => [...incoming.reverse(), ...current].slice(0, 500))
    }, 250)
    return () => {
      source.close()
      clearInterval(flush)
    }
  }, [live, service, severityMin, query])

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
  const errorCount = logs.filter((log) => log.severityNum >= 17).length
  return (
    <div className="grid gap-5">
      <PageHeading
        eyebrow="Event stream"
        title="Logs"
        description="Newest log records, live tails, and histogram context across services."
        action={
          <Link
            to="/sql"
            className="parallax-pill inline-flex h-8 items-center px-3 text-sm font-medium text-foreground transition-colors hover:bg-muted"
          >
            SQL workbench
          </Link>
        }
      />
      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <KpiCard
          icon={ScrollTextIcon}
          label="Shown logs"
          value={logs.length.toLocaleString()}
          detail={service === "all" ? "all services" : service}
          tone="blue"
          bars={chartData.map((point) => point.value)}
        />
        <KpiCard
          icon={ActivityIcon}
          label="Errors loaded"
          value={errorCount.toLocaleString()}
          detail="severity >= error"
          tone="rose"
        />
        <KpiCard
          icon={BarChart3Icon}
          label="Range total"
          value={live ? "-" : total.toLocaleString()}
          detail={live ? "streaming mode" : "histogram window"}
          tone="orange"
        />
        <KpiCard
          icon={RadioIcon}
          label="Mode"
          value={live ? "Live" : "Refresh"}
          detail={
            refreshSeconds > 0 ? `every ${refreshSeconds}s` : "manual refresh"
          }
          tone={live ? "green" : "violet"}
        />
      </section>

      <div className="parallax-panel flex flex-wrap items-center gap-2 p-3">
        <Select value={service} onValueChange={(v) => setService(v ?? "all")}>
          <SelectTrigger className="w-48 rounded-full bg-background/70">
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
          value={String(severityMin)}
          onValueChange={(v) => setSeverityMin(Number(v ?? 0))}
        >
          <SelectTrigger className="w-36 rounded-full bg-background/70">
            <SelectValue>
              {SEVERITIES.find((s) => s.min === severityMin)?.label}
            </SelectValue>
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
            className="rounded-full bg-background/70"
          />
        </form>
        <Select
          value={String(rangeMinutes)}
          onValueChange={(v) => setRangeMinutes(Number(v ?? 15))}
          disabled={live}
        >
          <SelectTrigger className="w-44 rounded-full bg-background/70">
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
          <SelectTrigger className="w-36 rounded-full bg-background/70">
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
            void load()
          }}
          disabled={loading || live}
          className="rounded-full"
        >
          Refresh
        </Button>
      </div>

      {live ? (
        <LiveStreamPanel
          title="Log tail"
          description="Streaming newest log records over Server-Sent Events. Per-row filters stay active while range analytics pause."
          count={logs.length}
          endpoint="/v1/logs/stream"
          active
        >
          <LiveEventStack
            items={logs.slice(0, 6).map((log) => ({
              id: `${log.tsNanos}-${log.traceId}-${log.spanId}`,
              title: log.body,
              meta: `${formatTime(log.tsNanos)} · ${log.service} · ${log.severityText}`,
              status: log.severityNum >= 17 ? "error" : "ok",
              detail: log.runId ? `run ${log.runId}` : log.traceId,
            }))}
          />
        </LiveStreamPanel>
      ) : (
        <div className="parallax-panel space-y-2 p-4">
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
            {refreshSeconds > 0 ? ` · refreshing every ${refreshSeconds}s` : ""}
          </p>
        </div>
      )}

      {logs.length === 0 ? (
        <div className="parallax-panel space-y-3 p-6">
          <p className="text-sm text-muted-foreground">
            No logs in this window — widen the range or drop a filter.
          </p>
          {rangeMinutes < 1440 ? (
            <Button variant="outline" onClick={() => setRangeMinutes(1440)}>
              Show last 24 hours
            </Button>
          ) : null}
        </div>
      ) : (
        <div className="parallax-panel overflow-hidden">
          <LogsTable logs={logs} />
        </div>
      )}

      {!live && logs.length > 0 && !exhausted ? (
        <Button
          variant="outline"
          onClick={() => void loadOlder()}
          disabled={olderLoading}
        >
          {olderLoading ? "Loading…" : "Load older"}
        </Button>
      ) : null}
    </div>
  )
}
