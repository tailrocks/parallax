import {
  Link,
  createFileRoute,
  useNavigate,
  useRouterState,
} from "@tanstack/react-router"
import { IconBug, IconTerminal2 } from "@tabler/icons-react"
import { z } from "zod"

import { CardSparkline } from "@/components/console/stat-card"
import {
  ClearFiltersButton,
  FilterSelect,
  SearchInput,
  SortableHead,
  Toolbar,
} from "@/components/console/data-table"
import { CopyButton } from "@/components/console/copy-button"
import { EmptyState } from "@/components/console/empty-state"
import { useDelayedLoading } from "@/components/console/hooks"
import { RangePicker } from "@/components/console/range-picker"
import { RelativeTime } from "@/components/console/relative-time"
import { TableSkeleton } from "@/components/console/skeletons"
import { PageHeader } from "@/components/page-header"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { gqlString, graphql } from "@/lib/api"
import type { Issue } from "@/lib/api"
import { formatCount } from "@/lib/format"
import {
  rangeLinkSearch,
  rangeSearchSchema,
  resolveRangeSearch,
  updateRangeSearch,
} from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"

interface TrendPoint {
  tsNanos: string
  count: number
}

export type IssueRow = Issue & { tags: string; trend: TrendPoint[] }

export interface IssuesData {
  issues: { total: number; items: IssueRow[] }
  services: string[]
}

type IssueSort = "LAST_SEEN" | "FIRST_SEEN" | "EVENTS" | "TREND"

export interface IssuesSearch {
  q?: string
  service?: string
  status?: "open" | "resolved"
  sort?: IssueSort
  range?: string
  from?: string
  to?: string
}

type IssuesSearchPatch = {
  [K in keyof IssuesSearch]?: IssuesSearch[K] | undefined
}

const SORTS = ["LAST_SEEN", "FIRST_SEEN", "EVENTS", "TREND"] as const

const issuesSearchSchema = rangeSearchSchema.extend({
  q: z.unknown().optional(),
  service: z.unknown().optional(),
  status: z.unknown().optional(),
  sort: z.unknown().optional(),
})

export function validateIssuesSearch(
  search: Record<string, unknown>
): IssuesSearch {
  const parsed = issuesSearchSchema.parse(search)
  const result: IssuesSearch = {}
  if (typeof parsed.q === "string" && parsed.q) result.q = parsed.q
  if (typeof parsed.service === "string" && parsed.service) {
    result.service = parsed.service
  }
  if (parsed.status === "open" || parsed.status === "resolved") {
    result.status = parsed.status
  }
  if (SORTS.includes(parsed.sort as IssueSort))
    result.sort = parsed.sort as IssueSort
  if (parsed.range) result.range = parsed.range
  if (parsed.from) result.from = parsed.from
  if (parsed.to) result.to = parsed.to
  return result
}

export function topTags(tags: string): Array<{ label: string; rest: number }> {
  try {
    const parsed = JSON.parse(tags) as Record<string, Record<string, number>>
    const labels = Object.entries(parsed).flatMap(([key, values]) => {
      const top = Object.entries(values).sort(([, a], [, b]) => b - a)[0]
      return top ? [`${key}:${top[0]}`] : [key]
    })
    return labels.slice(0, 2).map((label, index) => ({
      label,
      rest: index === 1 ? Math.max(0, labels.length - 2) : 0,
    }))
  } catch {
    return []
  }
}

function trendEvents(issue: IssueRow): number {
  return issue.trend.reduce((sum, point) => sum + point.count, 0)
}

function issueArgs(search: IssuesSearch, range: ResolvedRange): string {
  return [
    search.q ? `query: "${gqlString(search.q)}"` : null,
    search.service ? `service: "${gqlString(search.service)}"` : null,
    search.status ? `status: "${search.status}"` : null,
    `fromNanos: "${range.fromNanos}"`,
    `toNanos: "${range.toNanos}"`,
    `sort: ${search.sort ?? "LAST_SEEN"}`,
    "limit: 100",
  ]
    .filter(Boolean)
    .join(", ")
}

export async function loadIssues(search: IssuesSearch, range: ResolvedRange) {
  return graphql<IssuesData>(`
    {
      issues(${issueArgs(search, range)}) {
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
}

export const Route = createFileRoute("/issues/")({
  validateSearch: validateIssuesSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => loadIssues(deps, resolveRangeSearch(deps)),
  component: IssuesPage,
})

function patchSearch(current: IssuesSearch, patch: IssuesSearchPatch) {
  const raw = { ...current, ...patch }
  const next: IssuesSearch = {}
  for (const key of Object.keys(raw) as Array<keyof IssuesSearch>) {
    const value = raw[key]
    if (value != null && value !== "") Object.assign(next, { [key]: value })
  }
  return next
}

function IssuesPage() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const range = resolveRangeSearch(search)
  const routerLoading = useRouterState({
    select: (state) => state.status === "pending",
  })
  const loading = useDelayedLoading(routerLoading)

  const setSearch = (patch: IssuesSearchPatch) =>
    navigate({ search: patchSearch(search, patch) })

  return (
    <IssuesContent
      data={data}
      search={search}
      range={range}
      loading={loading}
      onSearch={setSearch}
      onIssue={(fingerprint) =>
        navigate({
          to: "/issues/$fingerprint",
          params: { fingerprint },
          search: rangeLinkSearch(range),
        })
      }
    />
  )
}

export function IssuesContent({
  data,
  search,
  range,
  loading,
  onSearch,
  onIssue,
}: {
  data: IssuesData
  search: IssuesSearch
  range: ResolvedRange
  loading?: boolean
  onSearch: (patch: IssuesSearchPatch) => void
  onIssue: (fingerprint: string) => void
}) {
  const hasFilters = Boolean(search.q || search.service || search.status)
  const sort = search.sort ?? "LAST_SEEN"

  return (
    <div className="space-y-4">
      <PageHeader
        icon={IconBug}
        iconClassName="text-rose-500"
        title="Issues"
        description="Grouped errors by service, message, and culprit."
        actions={
          <RangePicker
            value={range}
            onChange={(next) => onSearch(updateRangeSearch(next))}
          />
        }
      />

      <Toolbar className="justify-between">
        <div className="flex flex-wrap items-center gap-2">
          <SearchInput
            value={search.q ?? ""}
            onChange={(q) => onSearch({ q })}
            placeholder="Search message/type"
          />
          <FilterSelect
            {...(search.service ? { value: search.service } : {})}
            onChange={(service) => onSearch({ service })}
            placeholder="All services"
            options={data.services.map((service) => ({
              value: service,
              label: service,
            }))}
          />
          <FilterSelect
            {...(search.status ? { value: search.status } : {})}
            onChange={(status) =>
              onSearch({
                status:
                  status === "open" || status === "resolved"
                    ? status
                    : undefined,
              })
            }
            placeholder="Any status"
            options={[
              { value: "open", label: "Open" },
              { value: "resolved", label: "Resolved" },
            ]}
          />
          <FilterSelect
            value={sort}
            onChange={(next) => onSearch({ sort: next as IssueSort })}
            placeholder="Sort"
            options={[
              { value: "LAST_SEEN", label: "Last seen" },
              { value: "FIRST_SEEN", label: "First seen" },
              { value: "EVENTS", label: "Events" },
              { value: "TREND", label: "Trend" },
            ]}
          />
          {hasFilters ? (
            <ClearFiltersButton
              onClick={() =>
                onSearch({
                  q: undefined,
                  service: undefined,
                  status: undefined,
                })
              }
            />
          ) : null}
        </div>
        <span className="text-xs text-muted-foreground">
          {formatCount(data.issues.items.length)} of{" "}
          {formatCount(data.issues.total)}
        </span>
      </Toolbar>

      {loading ? (
        <TableSkeleton rows={8} />
      ) : data.issues.items.length === 0 ? (
        <EmptyState
          icon={IconTerminal2}
          title={
            hasFilters ? "No issues match filters" : "No issues ingested yet"
          }
          description={
            hasFilters ? (
              "Loosen query, service, status, or range."
            ) : (
              <span className="inline-flex items-center gap-2">
                <code>OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317</code>
                <CopyButton value="OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317" />
              </span>
            )
          }
        />
      ) : (
        <div className="overflow-hidden rounded-lg border bg-card">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Issue</TableHead>
                <TableHead className="w-24">Trend</TableHead>
                <TableHead className="w-24 text-right">
                  <SortableHead
                    {...(sort === "EVENTS" ? { sort: "events:desc" } : {})}
                    sortKey="events"
                    onSort={() => onSearch({ sort: "EVENTS" })}
                  >
                    Events
                  </SortableHead>
                </TableHead>
                <TableHead className="w-28 text-right">Age</TableHead>
                <TableHead className="w-32 text-right">
                  <SortableHead
                    {...(sort === "LAST_SEEN" ? { sort: "lastSeen:desc" } : {})}
                    sortKey="lastSeen"
                    onSort={() => onSearch({ sort: "LAST_SEEN" })}
                  >
                    Last seen
                  </SortableHead>
                </TableHead>
                <TableHead className="w-56">Tags</TableHead>
                <TableHead className="w-24">Status</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.issues.items.map((issue) => {
                const recentOpen =
                  issue.status === "open" && trendEvents(issue) > 0
                const tags = topTags(issue.tags)
                return (
                  <TableRow
                    key={issue.fingerprint}
                    className={cn(
                      "cursor-pointer",
                      recentOpen &&
                        "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]"
                    )}
                    onClick={() => onIssue(issue.fingerprint)}
                  >
                    <TableCell className="max-w-xl">
                      <div className="min-w-0 space-y-1">
                        <Link
                          to="/issues/$fingerprint"
                          params={{ fingerprint: issue.fingerprint }}
                          search={rangeLinkSearch(range)}
                          className="block truncate font-medium hover:underline"
                          onClick={(event) => event.stopPropagation()}
                        >
                          {issue.errorType || issue.title}
                        </Link>
                        <div className="truncate text-sm text-muted-foreground">
                          {issue.title}
                        </div>
                        {issue.culprit ? (
                          <div className="truncate font-mono text-xs text-muted-foreground/70">
                            {issue.culprit}
                          </div>
                        ) : null}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Link
                        to="/issues/$fingerprint"
                        params={{ fingerprint: issue.fingerprint }}
                        search={rangeLinkSearch(range)}
                        className="block text-rose-500"
                        onClick={(event) => event.stopPropagation()}
                      >
                        <CardSparkline
                          data={issue.trend.map((point) => ({
                            value: point.count,
                          }))}
                          className="h-9"
                        />
                      </Link>
                    </TableCell>
                    <TableCell className="text-right tabular-nums">
                      <Link
                        to="/issues/$fingerprint"
                        params={{ fingerprint: issue.fingerprint }}
                        search={rangeLinkSearch(range)}
                        className="hover:underline"
                        onClick={(event) => event.stopPropagation()}
                      >
                        {formatCount(issue.eventCount)}
                      </Link>
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      <RelativeTime nanos={issue.firstSeenNanos} />
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      <RelativeTime nanos={issue.lastSeenNanos} />
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-1">
                        {tags.length === 0 ? (
                          <span className="text-muted-foreground">-</span>
                        ) : (
                          tags.map((tag) => (
                            <Badge
                              key={tag.label}
                              variant="secondary"
                              className="max-w-36 truncate font-mono"
                            >
                              {tag.label}
                              {tag.rest > 0 ? (
                                <span className="ml-1 text-muted-foreground">
                                  +{tag.rest}
                                </span>
                              ) : null}
                            </Badge>
                          ))
                        )}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant={issue.status === "open" ? "rose" : "emerald"}
                      >
                        {issue.status}
                      </Badge>
                    </TableCell>
                  </TableRow>
                )
              })}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  )
}
