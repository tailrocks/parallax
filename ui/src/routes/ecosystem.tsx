import {
  createFileRoute,
  useNavigate,
  useRouterState,
} from "@tanstack/react-router"
import { IconSitemap } from "@tabler/icons-react"
import { useMemo } from "react"

import { EcosystemGraph } from "@/components/console/ecosystem-graph"
import { useDelayedLoading } from "@/components/console/hooks"
import { RangePicker } from "@/components/console/range-picker"
import { TableSkeleton } from "@/components/console/skeletons"
import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { graphqlCached } from "@/lib/api"
import type { ServiceMap } from "@/lib/api"
import {
  rangeSearchSchema,
  resolveRangeSearch,
  updateRangeSearch,
} from "@/lib/range"

type EcosystemSearch = {
  range?: string | undefined
  from?: string | undefined
  to?: string | undefined
}

export const Route = createFileRoute("/ecosystem")({
  validateSearch: (search: Record<string, unknown>): EcosystemSearch => {
    const parsed = rangeSearchSchema.parse(search)
    return {
      range: typeof parsed.range === "string" ? parsed.range : undefined,
      from: typeof parsed.from === "string" ? parsed.from : undefined,
      to: typeof parsed.to === "string" ? parsed.to : undefined,
    }
  },
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => {
    const range = resolveRangeSearch(deps)
    return graphqlCached<{ serviceMap: ServiceMap }>(`
      {
        serviceMap(
          fromNanos: "${range.fromNanos}"
          toNanos: "${range.toNanos}"
          maxTraces: 100
        ) {
          nodes { name lastSeenNanos spanCount errorCount p95Ms }
          edges { source target callCount errorCount p50Ms p95Ms }
        }
      }
    `)
  },
  component: EcosystemPage,
})

function EcosystemPage() {
  const { serviceMap } = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const pending = useRouterState({
    select: (state) => state.status === "pending",
  })
  const showSkeleton = useDelayedLoading(pending)
  const range = useMemo(
    () => resolveRangeSearch(search),
    [search]
  )

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        icon={IconSitemap}
        title="Ecosystem"
        description="Trace-path service dependencies across the selected window."
        actions={
          <RangePicker
            value={range}
            onChange={(next) =>
              void navigate({ search: updateRangeSearch(next), replace: true })
            }
          />
        }
      />

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">Service map</CardTitle>
        </CardHeader>
        <CardContent>
          {showSkeleton ? (
            <TableSkeleton rows={6} />
          ) : (
            <EcosystemGraph
              nodes={serviceMap.nodes}
              edges={serviceMap.edges}
              range={range}
            />
          )}
        </CardContent>
      </Card>
    </div>
  )
}
