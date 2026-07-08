import {
  Link,
  createFileRoute,
  useNavigate,
  useRouterState,
} from "@tanstack/react-router"
import { IconServer, IconTerminal2 } from "@tabler/icons-react"
import { z } from "zod"

import { EmptyState } from "@/components/console/empty-state"
import { HeatCell } from "@/components/console/heat-cell"
import { useDelayedLoading } from "@/components/console/hooks"
import { RangePicker } from "@/components/console/range-picker"
import { RelativeTime } from "@/components/console/relative-time"
import { TableSkeleton } from "@/components/console/skeletons"
import {
  SearchInput,
  SortableHead,
  Toolbar,
  sortRows,
} from "@/components/console/data-table"
import { PageHeader } from "@/components/page-header"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { graphql } from "@/lib/api"
import { formatCount, formatDurationNs, formatPercent } from "@/lib/format"
import {
  rangeLinkSearch,
  rangeSearchSchema,
  resolveRangeSearch,
  updateRangeSearch,
} from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"

export interface ServiceSummary {
  name: string
  lastSeenNanos: string
  spanCount: string
  errorCount: string
  p95Ms: number | null
}

export interface ServicesData {
  serviceList: ServiceSummary[]
}

type ServiceSort =
  | "name:asc"
  | "spans:desc"
  | "spans:asc"
  | "errors:desc"
  | "errors:asc"
  | "errorRate:desc"
  | "errorRate:asc"
  | "p95:desc"
  | "p95:asc"
  | "lastSeen:desc"
  | "lastSeen:asc"

export interface ServicesSearch {
  q?: string
  range?: string
  from?: string
  to?: string
  sort?: ServiceSort
}

type ServicesSearchPatch = {
  [K in keyof ServicesSearch]?: ServicesSearch[K] | undefined
}

const serviceSorts = new Set<ServiceSort>([
  "name:asc",
  "spans:desc",
  "spans:asc",
  "errors:desc",
  "errors:asc",
  "errorRate:desc",
  "errorRate:asc",
  "p95:desc",
  "p95:asc",
  "lastSeen:desc",
  "lastSeen:asc",
])

const servicesSearchSchema = rangeSearchSchema.extend({
  q: z.unknown().optional(),
  sort: z.unknown().optional(),
})

export function validateServicesSearch(
  search: Record<string, unknown>
): ServicesSearch {
  const parsed = servicesSearchSchema.parse(search)
  const result: ServicesSearch = {}
  if (typeof parsed.q === "string" && parsed.q) result.q = parsed.q
  if (parsed.range) result.range = parsed.range
  if (parsed.from) result.from = parsed.from
  if (parsed.to) result.to = parsed.to
  if (serviceSorts.has(parsed.sort as ServiceSort)) {
    result.sort = parsed.sort as ServiceSort
  }
  return result
}

export function serviceErrorRate(row: ServiceSummary): number {
  const spans = Number(row.spanCount)
  if (!Number.isFinite(spans) || spans <= 0) return 0
  return Number(row.errorCount) / spans
}

export function serviceHref(service: string) {
  return `/services/${encodeURIComponent(service)}`
}

export function sortedServices(rows: ServiceSummary[], sort?: string) {
  return sortRows(rows, sort ?? "lastSeen:desc", {
    name: (row) => row.name.toLowerCase(),
    spans: (row) => Number(row.spanCount),
    errors: (row) => Number(row.errorCount),
    errorRate: serviceErrorRate,
    p95: (row) => row.p95Ms,
    lastSeen: (row) => Number(row.lastSeenNanos),
  })
}

export async function loadServices(range: ResolvedRange) {
  return graphql<ServicesData>(`
    {
      serviceList(fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}") {
        name
        lastSeenNanos
        spanCount
        errorCount
        p95Ms
      }
    }
  `)
}

export const Route = createFileRoute("/services")({
  validateSearch: validateServicesSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => loadServices(resolveRangeSearch(deps)),
  component: ServicesPage,
})

function patchSearch(current: ServicesSearch, patch: ServicesSearchPatch) {
  const raw = { ...current, ...patch }
  const next: ServicesSearch = {}
  for (const key of Object.keys(raw) as Array<keyof ServicesSearch>) {
    const value = raw[key]
    if (value != null && value !== "") {
      Object.assign(next, { [key]: value })
    }
  }
  return next
}

function ServiceDot({ errored }: { errored: boolean }) {
  return (
    <span
      className={cn(
        "size-2 rounded-full",
        errored ? "bg-rose-500" : "bg-emerald-500"
      )}
    />
  )
}

function ServicesPage() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const range = resolveRangeSearch(search)
  const routerLoading = useRouterState({
    select: (state) => state.status === "pending",
  })
  const pending = useDelayedLoading(routerLoading)

  const setSearch = (patch: ServicesSearchPatch) =>
    navigate({ search: patchSearch(search, patch) })

  return (
    <ServicesIndexContent
      data={data}
      search={search}
      range={range}
      loading={pending}
      onSearch={setSearch}
    />
  )
}

export function ServicesIndexContent({
  data,
  search,
  range,
  loading = false,
  onSearch,
}: {
  data: ServicesData
  search: ServicesSearch
  range: ResolvedRange
  loading?: boolean
  onSearch: (patch: ServicesSearchPatch) => void
}) {
  const query = search.q?.toLowerCase() ?? ""
  const filtered = data.serviceList.filter((row) =>
    row.name.toLowerCase().includes(query)
  )
  const rows = sortedServices(filtered, search.sort)
  const p95Values = rows
    .map((row) => row.p95Ms)
    .filter((value): value is number => Number.isFinite(value))
  const errorRates = rows.map(serviceErrorRate)

  return (
    <div className="space-y-4">
      <PageHeader
        icon={IconServer}
        iconClassName="text-emerald-500"
        title="Services"
        description="Health index across services emitting telemetry."
        actions={
          <RangePicker
            value={range}
            onChange={(next) => onSearch(updateRangeSearch(next))}
          />
        }
      />

      <Toolbar className="justify-between">
        <SearchInput
          value={search.q ?? ""}
          onChange={(q) => onSearch({ q })}
          placeholder="Search services"
        />
        <span className="text-xs text-muted-foreground">
          {formatCount(rows.length)} of {formatCount(data.serviceList.length)}
        </span>
      </Toolbar>

      {loading ? (
        <TableSkeleton rows={8} />
      ) : rows.length === 0 ? (
        <EmptyState
          icon={IconTerminal2}
          title="No services yet"
          description={
            <span>
              Point OTLP at <code>http://127.0.0.1:4317</code>; services appear
              after spans, logs, or metrics arrive.
            </span>
          }
        />
      ) : (
        <div className="overflow-hidden rounded-lg border bg-card">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>
                  <SortableHead
                    {...(search.sort ? { sort: search.sort } : {})}
                    sortKey="name"
                    onSort={(sort) => onSearch({ sort: sort as ServiceSort })}
                  >
                    Service
                  </SortableHead>
                </TableHead>
                <TableHead className="w-28 text-right">
                  <SortableHead
                    {...(search.sort ? { sort: search.sort } : {})}
                    sortKey="spans"
                    onSort={(sort) => onSearch({ sort: sort as ServiceSort })}
                  >
                    Spans
                  </SortableHead>
                </TableHead>
                <TableHead className="w-28 text-right">
                  <SortableHead
                    {...(search.sort ? { sort: search.sort } : {})}
                    sortKey="errors"
                    onSort={(sort) => onSearch({ sort: sort as ServiceSort })}
                  >
                    Errors
                  </SortableHead>
                </TableHead>
                <TableHead className="w-28 text-right">Error rate</TableHead>
                <TableHead className="w-28 text-right">
                  <SortableHead
                    {...(search.sort ? { sort: search.sort } : {})}
                    sortKey="p95"
                    onSort={(sort) => onSearch({ sort: sort as ServiceSort })}
                  >
                    p95
                  </SortableHead>
                </TableHead>
                <TableHead className="w-32 text-right">
                  <SortableHead
                    {...(search.sort ? { sort: search.sort } : {})}
                    sortKey="lastSeen"
                    onSort={(sort) => onSearch({ sort: sort as ServiceSort })}
                  >
                    Last seen
                  </SortableHead>
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {rows.map((row) => {
                const errors = Number(row.errorCount)
                const rate = serviceErrorRate(row)
                return (
                  <TableRow
                    key={row.name}
                    className={cn(
                      "cursor-pointer",
                      errors > 0 &&
                        "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]"
                    )}
                  >
                    <TableCell>
                      <Link
                        to="/services/$service"
                        params={{ service: row.name }}
                        search={rangeLinkSearch(range)}
                        className="flex min-w-0 items-center gap-2 font-medium"
                      >
                        <ServiceDot errored={errors > 0} />
                        <span className="truncate">{row.name}</span>
                      </Link>
                    </TableCell>
                    <TableCell className="text-right tabular-nums">
                      <Link
                        to="/traces"
                        search={{ service: row.name, ...rangeLinkSearch(range) }}
                        className="hover:underline"
                      >
                        {formatCount(Number(row.spanCount))}
                      </Link>
                    </TableCell>
                    <TableCell
                      className={cn(
                        "text-right tabular-nums",
                        errors > 0
                          ? "text-rose-600 dark:text-rose-400"
                          : "text-muted-foreground/40"
                      )}
                    >
                      <Link
                        to="/traces"
                        search={{
                          service: row.name,
                          errors: true,
                          ...rangeLinkSearch(range),
                        }}
                        className="hover:underline"
                      >
                        {formatCount(errors)}
                      </Link>
                    </TableCell>
                    <TableCell className="text-right">
                      <HeatCell value={rate} values={errorRates}>
                        {formatPercent(rate)}
                      </HeatCell>
                    </TableCell>
                    <TableCell className="text-right">
                      {row.p95Ms == null ? (
                        <span className="text-muted-foreground">-</span>
                      ) : (
                        <HeatCell value={row.p95Ms} values={p95Values}>
                          {formatDurationNs(row.p95Ms * 1_000_000)}
                        </HeatCell>
                      )}
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      <RelativeTime nanos={row.lastSeenNanos} />
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
