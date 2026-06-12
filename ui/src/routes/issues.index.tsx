import { Link, createFileRoute, useNavigate } from "@tanstack/react-router"
import { graphql, gqlString, relativeTime } from "@/lib/api"
import type { Issue } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
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

interface TrendPoint {
  tsNanos: string
  count: number
}

type IssueRow = Issue & { tags: string; trend: TrendPoint[] }

interface IssuesSearch {
  q?: string
  service?: string
  status?: string
  sort?: string
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
    <svg
      width={width}
      height={18}
      role="img"
      aria-label="24h occurrence trend"
    >
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

function IssuesPage() {
  const { issues, services } = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const update = (patch: Partial<IssuesSearch>) =>
    navigate({ search: { ...search, ...patch } })

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="text-lg font-semibold">Issues</h1>
        <span className="text-sm text-muted-foreground">
          {issues.total} matching
        </span>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <Input
          className="h-8 w-56"
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
          onValueChange={(value) =>
            update({ service: value === "all" ? undefined : value })
          }
        >
          <SelectTrigger className="h-8 w-44" size="sm">
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
          onValueChange={(value) =>
            update({ status: value === "all" ? undefined : value })
          }
        >
          <SelectTrigger className="h-8 w-32" size="sm">
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
          onValueChange={(value) =>
            update({ sort: value === "LAST_SEEN" ? undefined : value })
          }
        >
          <SelectTrigger className="h-8 w-36" size="sm">
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
        <p className="text-sm text-muted-foreground">
          {issues.total === 0 && !search.q && !search.service && !search.status
            ? "No issues yet — connect an app with "
            : "Nothing matches these filters. "}
          {issues.total === 0 &&
          !search.q &&
          !search.service &&
          !search.status ? (
            <code>OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317</code>
          ) : null}
        </p>
      ) : (
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
                  <Link
                    to="/issues/$fingerprint"
                    params={{ fingerprint: issue.fingerprint }}
                    className="block truncate font-medium hover:underline"
                  >
                    {issue.title}
                  </Link>
                  <span className="flex flex-wrap items-center gap-1">
                    {issue.culprit ? (
                      <span className="truncate text-xs text-muted-foreground">
                        {issue.culprit}
                      </span>
                    ) : null}
                    {topTags(issue.tags).map((tag) => (
                      <Badge
                        key={tag}
                        variant="secondary"
                        className="max-w-40 truncate font-mono text-[10px]"
                      >
                        {tag}
                      </Badge>
                    ))}
                  </span>
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
                  <Badge variant="outline">{issue.service}</Badge>
                </TableCell>
                <TableCell>
                  <Badge
                    variant={
                      issue.status === "open" ? "destructive" : "secondary"
                    }
                  >
                    {issue.status}
                  </Badge>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </div>
  )
}
