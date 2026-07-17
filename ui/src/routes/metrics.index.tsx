import { createFileRoute } from "@tanstack/react-router"
import { useMemo, useState } from "react"
import { IconChartLine } from "@tabler/icons-react"

import { FilterSelect, SearchInput, Toolbar } from "@/shared/console/data-table"
import { EmptyState } from "@/shared/console/empty-state"
import { PageHeader } from "@/shared/components/page-header"
import { MetricsTable } from "./-metrics-table"
import { graphqlCached } from "@/platform/graphql/transport"
import { inferMetricKind, type MetricKind } from "@/features/runtime-metrics"

// Plan 168 metrics explorer browse surface — metricCatalog for kind-aware
// listing (falls back to metricNames + name inference if catalog empty).

interface CatalogRow {
  name: string
  kind: string
  unit?: string | null
  services?: string[]
  lastDatapointNanos?: string
  pointCount?: string
}

interface MetricsSearch {
  q?: string | undefined
  kind?: string | undefined
}

function searchString(value: unknown) {
  return typeof value === "string" && value ? value : undefined
}

const KIND_OPTIONS: Array<{ value: MetricKind; label: string }> = [
  { value: "sum", label: "Sum / counter" },
  { value: "gauge", label: "Gauge" },
  { value: "histogram", label: "Histogram" },
  { value: "summary", label: "Summary" },
  { value: "unknown", label: "Unknown" },
]

export const Route = createFileRoute("/metrics/")({
  validateSearch: (search: Record<string, unknown>): MetricsSearch => ({
    q: searchString(search["q"]),
    kind: searchString(search["kind"]),
  }),
  loader: async (): Promise<CatalogRow[]> => {
    const now = Date.now() * 1_000_000
    const from = now - 7 * 24 * 60 * 60 * 1_000_000_000
    try {
      const data = await graphqlCached<{
        metricCatalog: CatalogRow[]
      }>(
        `{ metricCatalog(fromNanos: "${from}", toNanos: "${now}", limit: 500) { name kind unit services lastDatapointNanos pointCount } }`
      )
      if (data.metricCatalog.length > 0) return data.metricCatalog
    } catch {
      // fall through
    }
    const names = await graphqlCached<{ metricNames: string[] }>(`{ metricNames }`).then(
      (d) => d.metricNames
    )
    return names.map((name): CatalogRow => ({ name, kind: inferMetricKind(name) }))
  },
  component: MetricsPage,
})

function MetricsPage() {
  const catalog = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = Route.useNavigate()
  const [text, setText] = useState(search.q ?? "")
  const rows = useMemo(() => {
    const needle = text.toLowerCase()
    return catalog
      .map((row) => ({
        ...row,
        kind: (row.kind as MetricKind) || inferMetricKind(row.name),
      }))
      .filter(
        (row) =>
          (!needle || row.name.toLowerCase().includes(needle)) &&
          (!search.kind || row.kind === search.kind)
      )
  }, [catalog, search.kind, text])
  return (
    <div className="space-y-4 p-4">
      <PageHeader title="Metrics" description="Browse metrics, then open one." />
      <Toolbar>
        <SearchInput
          value={text}
          onChange={(value) => {
            setText(value)
            void navigate({
              search: (prev) => ({ ...prev, q: value || undefined }),
              replace: true,
            })
          }}
          placeholder="Search"
        />
        <FilterSelect
          {...(search.kind ? { value: search.kind } : {})}
          onChange={(kind) => void navigate({ search: (prev) => ({ ...prev, kind }) })}
          options={KIND_OPTIONS}
          placeholder="Kind"
        />
        <span className="ml-auto text-xs text-muted-foreground tabular-nums">
          {rows.length} of {catalog.length} metrics
        </span>
      </Toolbar>
      {rows.length === 0 ? (
        <EmptyState
          icon={IconChartLine}
          title="No metrics match"
          description="Adjust filters or send metrics."
        />
      ) : (
        <MetricsTable rows={rows} />
      )}
    </div>
  )
}
