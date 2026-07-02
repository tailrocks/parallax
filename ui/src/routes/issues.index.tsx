import { Link, createFileRoute, useNavigate } from "@tanstack/react-router"
import {
  ActivityIcon,
  AlertCircleIcon,
  BoxesIcon,
  Clock3Icon,
} from "lucide-react"
import { graphql, gqlString, relativeTime } from "@/lib/api"
import type { Issue } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { KpiCard } from "@/components/kpi-card"
import { PageHeading } from "@/components/page-heading"
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

interface TrendPoint {
  tsNanos: string
  count: number
}

type IssueRow = Issue & { tags: string; trend: TrendPoint[] }

interface IssuesSearch {
  q?: string | undefined
  service?: string | undefined
  status?: string | undefined
  sort?: string | undefined
}

const SORTS = ["LAST_SEEN", "FIRST_SEEN", "EVENTS", "TREND"] as const

export const Route = createFileRoute("/issues/")({
  validateSearch: (search: Record<string, unknown>): IssuesSearch => ({
    q: typeof search.q === "string" && search.q ? search.q : undefined,
    service:
      typeof search.service === "string" && search.service
        ? search.service
        : undefined,
    status:
      search.status === "open" || search.status === "resolved"
        ? search.status
        : undefined,
    sort: SORTS.includes(search.sort as (typeof SORTS)[number])
      ? (search.sort as string)
      : undefined,
  }),
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => {
    const args = [
      deps.q ? `query: "${gqlString(deps.q)}"` : null,
      deps.service ? `service: "${gqlString(deps.service)}"` : null,
      deps.status ? `status: "${deps.status}"` : null,
      `sort: ${deps.sort ?? "LAST_SEEN"}`,
      "limit: 100",
    ]
      .filter(Boolean)
      .join(", ")
    return graphql<{
      issues: { total: number; items: IssueRow[] }
      services: string[]
    }>(`
      {
        issues(${args}) {
          total
          items {
            fingerprint title errorType culprit service status
            firstSeenNanos lastSeenNanos eventCount lastTraceId tags
            trend { tsNanos count }
          }
        }
        services
      }
    `)
  },
  component: IssuesPage,
})

/** Inline 24h sparkline — hourly buckets as tiny bars. */
function Sparkline({ trend }: { trend: TrendPoint[] }) {
  if (trend.length === 0) {
    return <span className="text-xs text-muted-foreground">—</span>
  }
  const max = Math.max(1, ...trend.map((p) => p.count))
  const width = 72
  const bar = width / 24
  return (
    <svg width={width} height={18} role="img" aria-label="24h occurrence trend">
      {trend.slice(-24).map((point, index) => {
        const h = Math.max(1.5, (point.count / max) * 16)
        return (
          <rect
            key={point.tsNanos}
            x={index * bar}
            y={18 - h}
            width={Math.max(1, bar - 1)}
            height={h}
            rx={0.5}
            className="fill-(--chart-1)"
          />
        )
      })}
    </svg>
  )
}

function topTags(tags: string): string[] {
  try {
    const parsed = JSON.parse(tags) as Record<string, Record<string, number>>
    return Object.entries(parsed)
      .slice(0, 2)
      .map(([key, values]) => {
        const top = Object.entries(values).sort(([, a], [, b]) => b - a)[0]
        return top ? `${key}:${top[0]}` : key
      })
  } catch {
    return []
  }
}

function aggregateTrend(items: IssueRow[]): number[] {
  const buckets = new Map<string, number>()
  for (const issue of items) {
    for (const point of issue.trend.slice(-24)) {
      buckets.set(
        point.tsNanos,
        (buckets.get(point.tsNanos) ?? 0) + point.count
      )
    }
  }
  return [...buckets.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([, count]) => count)
}

function IssuesPage() {
  const { issues, services } = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const update = (patch: Partial<IssuesSearch>) =>
    navigate({ search: { ...search, ...patch } })
  const openCount = issues.items.filter(
    (issue) => issue.status === "open"
  ).length
  const eventCount = issues.items.reduce(
    (total, issue) => total + issue.eventCount,
    0
  )
  const newestIssue = issues.items[0]
  const trendBars = aggregateTrend(issues.items)

  return (
    <div className="grid gap-5">
      <PageHeading
        eyebrow="Failure groups"
        title="Issues"
        description="Grouped exceptions and error logs across services, ready for human review or agent context."
        action={
          <span className="parallax-pill inline-flex h-8 items-center gap-2 px-3 text-xs text-muted-foreground">
            <span className="size-1.5 rounded-full bg-(--brand-green)" />
            Latest first
          </span>
        }
      />

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <KpiCard
          icon={AlertCircleIcon}
          label="Matching issues"
          value={String(issues.total)}
          detail={`${issues.items.length} loaded`}
          tone="rose"
          bars={trendBars}
        />
        <KpiCard
          icon={ActivityIcon}
          label="Open loaded"
          value={String(openCount)}
          detail="visible result window"
          tone="orange"
          bars={issues.items.map((issue) => (issue.status === "open" ? 1 : 0))}
        />
        <KpiCard
          icon={BoxesIcon}
          label="Events loaded"
          value={eventCount.toLocaleString()}
          detail="across visible groups"
          tone="blue"
          bars={issues.items.map((issue) => issue.eventCount)}
        />
        <KpiCard
          icon={Clock3Icon}
          label="Freshness"
          value={newestIssue ? relativeTime(newestIssue.lastSeenNanos) : "-"}
          detail={newestIssue?.service ?? "no service yet"}
          tone="green"
        />
      </section>

      <div className="parallax-panel flex flex-wrap items-center gap-2 p-3">
        <Input
          className="h-9 w-64 rounded-full bg-background/70 font-mono text-xs"
          placeholder="Search title, type, fingerprint…"
          defaultValue={search.q ?? ""}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              update({ q: event.currentTarget.value || undefined })
            }
          }}
        />
        <Select
          value={search.service ?? "all"}
          onValueChange={(value) => {
            const service = value ?? "all"
            update({ service: service === "all" ? undefined : service })
          }}
        >
          <SelectTrigger
            className="h-9 w-44 rounded-full bg-background/70"
            size="sm"
          >
            <SelectValue placeholder="All services" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All services</SelectItem>
            {services.map((service) => (
              <SelectItem key={service} value={service}>
                {service}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select
          value={search.status ?? "all"}
          onValueChange={(value) => {
            const status = value ?? "all"
            update({ status: status === "all" ? undefined : status })
          }}
        >
          <SelectTrigger
            className="h-9 w-32 rounded-full bg-background/70"
            size="sm"
          >
            <SelectValue placeholder="Any status" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">Any status</SelectItem>
            <SelectItem value="open">open</SelectItem>
            <SelectItem value="resolved">resolved</SelectItem>
          </SelectContent>
        </Select>
        <Select
          value={search.sort ?? "LAST_SEEN"}
          onValueChange={(value) => {
            const sort = value ?? "LAST_SEEN"
            update({ sort: sort === "LAST_SEEN" ? undefined : sort })
          }}
        >
          <SelectTrigger
            className="h-9 w-36 rounded-full bg-background/70"
            size="sm"
          >
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="LAST_SEEN">Last seen</SelectItem>
            <SelectItem value="FIRST_SEEN">First seen</SelectItem>
            <SelectItem value="EVENTS">Events</SelectItem>
            <SelectItem value="TREND">Trend (24h)</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {issues.items.length === 0 ? (
        <div className="parallax-panel p-6">
          <p className="text-sm font-medium">
            {issues.total === 0 &&
            !search.q &&
            !search.service &&
            !search.status
              ? "No issues ingested yet"
              : "No issues match these filters"}
          </p>
          <p className="mt-2 text-sm text-muted-foreground">
            {issues.total === 0 &&
            !search.q &&
            !search.service &&
            !search.status
              ? "Connect an app to the local OTLP endpoint and Parallax will group failures here."
              : "Loosen the query, service, status, or sort filter."}
          </p>
          {issues.total === 0 &&
          !search.q &&
          !search.service &&
          !search.status ? (
            <code className="mt-4 block w-fit rounded-xl border border-border/70 bg-background/80 px-3 py-2 font-mono text-xs text-foreground">
              OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317
            </code>
          ) : null}
        </div>
      ) : (
        <div className="parallax-panel overflow-hidden">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Issue</TableHead>
                <TableHead className="w-24">Trend</TableHead>
                <TableHead className="w-20 text-right">Events</TableHead>
                <TableHead className="w-28">Last seen</TableHead>
                <TableHead className="w-28">Age</TableHead>
                <TableHead className="w-32">Service</TableHead>
                <TableHead className="w-24">Status</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {issues.items.map((issue) => (
                <TableRow key={issue.fingerprint}>
                  <TableCell className="max-w-xl">
                    <div className="flex items-start gap-3">
                      <span className="mt-1 size-2 rounded-full bg-(--brand-rose)" />
                      <div className="min-w-0">
                        <Link
                          to="/issues/$fingerprint"
                          params={{ fingerprint: issue.fingerprint }}
                          className="block truncate font-medium text-foreground hover:underline"
                        >
                          {issue.title}
                        </Link>
                        <span className="mt-1 flex flex-wrap items-center gap-1">
                          {issue.culprit ? (
                            <span className="truncate text-xs text-muted-foreground">
                              {issue.culprit}
                            </span>
                          ) : null}
                          {topTags(issue.tags).map((tag) => (
                            <Badge
                              key={tag}
                              variant="secondary"
                              className="max-w-40 truncate rounded-full font-mono text-[10px]"
                            >
                              {tag}
                            </Badge>
                          ))}
                        </span>
                      </div>
                    </div>
                  </TableCell>
                  <TableCell>
                    <Sparkline trend={issue.trend} />
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {issue.eventCount}
                  </TableCell>
                  <TableCell>{relativeTime(issue.lastSeenNanos)}</TableCell>
                  <TableCell>{relativeTime(issue.firstSeenNanos)}</TableCell>
                  <TableCell>
                    <Badge variant="outline" className="rounded-full">
                      {issue.service}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    <Badge
                      variant={
                        issue.status === "open" ? "destructive" : "secondary"
                      }
                      className="rounded-full"
                    >
                      {issue.status}
                    </Badge>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  )
}
