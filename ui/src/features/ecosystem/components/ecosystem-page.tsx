import { useMemo } from "react"
import { useNavigate, useRouterState } from "@tanstack/react-router"
import { IconSitemap } from "@tabler/icons-react"

import { EcosystemControls } from "@/features/ecosystem/components/ecosystem-controls"
import { EcosystemGraph } from "@/features/ecosystem/components/ecosystem-graph"
import type { EcosystemSearch } from "@/features/ecosystem/model/ecosystem-search"
import { projectServiceMap } from "@/features/ecosystem/model/project-service-map"
import type { ServiceMap } from "@/features/ecosystem/model/service-map"
import { useDelayedLoading } from "@/shared/console/hooks"
import { TableSkeleton } from "@/shared/console/skeletons"
import { PageHeader } from "@/shared/components/page-header"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { RangePicker } from "@/features/time-range"
import { resolveRangeSearch, updateRangeSearch } from "@/lib/range"

export function EcosystemPage({
  serviceMap,
  search,
}: {
  serviceMap: ServiceMap
  search: EcosystemSearch
}) {
  const navigate = useNavigate({ from: "/ecosystem" })
  const pending = useRouterState({
    select: (state) => state.status === "pending",
  })
  const showSkeleton = useDelayedLoading(pending)
  const range = useMemo(() => resolveRangeSearch(search), [search])
  const graph = useMemo(
    () => projectServiceMap(serviceMap, search),
    [search, serviceMap]
  )
  const update = (patch: Partial<EcosystemSearch>) => {
    void navigate({
      search: (current) => ({ ...current, ...patch }),
      replace: true,
    })
  }

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
              void navigate({
                search: (current) => ({
                  ...current,
                  ...updateRangeSearch(next),
                }),
                replace: true,
              })
            }
          />
        }
      />

      <Card>
        <CardHeader className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <CardTitle className="text-sm">Service map</CardTitle>
          <EcosystemControls
            services={serviceMap.nodes.map((node) => node.name).sort()}
            search={search}
            update={update}
          />
        </CardHeader>
        <CardContent>
          {showSkeleton ? (
            <TableSkeleton rows={6} />
          ) : (
            <EcosystemGraph
              nodes={graph.nodes}
              edges={graph.edges}
              range={range}
              dimmedNodeIds={graph.dimmedNodeIds}
              hiddenNodeCount={graph.hiddenNodeCount}
              hiddenEdgeCount={graph.hiddenEdgeCount}
            />
          )}
        </CardContent>
      </Card>
    </div>
  )
}
