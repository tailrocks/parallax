import {
  Outlet,
  useLocation,
  useNavigate,
  useRouterState,
} from "@tanstack/react-router"
import { IconServer, IconTerminal2 } from "@tabler/icons-react"
import { useMemo } from "react"

import { EmptyState } from "@/components/console/empty-state"
import { buildHeatScale } from "@/components/console/heat-cell"
import { useDelayedLoading } from "@/components/console/hooks"
import { TableSkeleton } from "@/components/console/skeletons"
import { SearchInput, Toolbar } from "@/components/console/data-table"
import { ServicesTable } from "@/features/services/components/services-table"
import { RangePicker } from "@/features/time-range"
import {
  serviceErrorRate,
  servicesWithCatalog,
  sortedServices,
  type ServicesData,
} from "@/features/services/model/service-summary"
import {
  patchServicesSearch,
  type ServicesSearch,
  type ServicesSearchPatch,
} from "@/features/services/model/services-search"
import { formatCount } from "@/lib/format"
import {
  resolveRangeSearch,
  updateRangeSearch,
  type ResolvedRange,
} from "@/lib/range"
import { PageHeader } from "@/shared/components/page-header"

export function ServicesRouteShell({
  data,
  search,
}: {
  data: ServicesData
  search: ServicesSearch
}) {
  const { pathname } = useLocation()
  const normalized = pathname.replace(/\/+$/, "") || "/"
  if (normalized !== "/services") return <Outlet />
  return <ServicesPage data={data} search={search} />
}

export function ServicesPage({
  data,
  search,
}: {
  data: ServicesData
  search: ServicesSearch
}) {
  const navigate = useNavigate({ from: "/services" })
  const range = resolveRangeSearch(search)
  const routerLoading = useRouterState({
    select: (state) => state.status === "pending",
  })
  const pending = useDelayedLoading(routerLoading)

  const setSearch = (patch: ServicesSearchPatch) =>
    void navigate({ search: patchServicesSearch(search, patch) })

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
  const catalogRows = servicesWithCatalog(data)
  const filtered = catalogRows.filter((row) =>
    [
      row.name,
      row.serviceVersion,
      row.telemetrySdkLanguage,
      row.deploymentEnvironment,
    ]
      .filter((value): value is string => Boolean(value))
      .some((value) => value.toLowerCase().includes(query))
  )
  const rows = sortedServices(filtered, search.sort)
  const p95Values = rows
    .map((row) => row.p95Ms)
    .filter((value): value is number => Number.isFinite(value))
  const errorRates = rows.map(serviceErrorRate)
  const p95Scale = useMemo(() => buildHeatScale(p95Values), [p95Values])
  const errorRateScale = useMemo(() => buildHeatScale(errorRates), [errorRates])

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
          {formatCount(rows.length)} of {formatCount(catalogRows.length)}
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
        <ServicesTable
          rows={rows}
          search={search}
          range={range}
          p95Scale={p95Scale}
          errorRateScale={errorRateScale}
          onSearch={onSearch}
        />
      )}
    </div>
  )
}
