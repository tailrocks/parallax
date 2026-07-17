import {
  createFileRoute,
  useNavigate,
  useRouterState,
} from "@tanstack/react-router"
import { IconSitemap } from "@tabler/icons-react"
import { useMemo } from "react"

import { EcosystemGraph } from "@/components/console/ecosystem-graph"
import { useDelayedLoading } from "@/components/console/hooks"
import { RangePicker } from "@/features/time-range"
import { TableSkeleton } from "@/components/console/skeletons"
import { PageHeader } from "@/shared/components/page-header"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { graphqlCached } from "@/lib/api"
import type { ServiceMap, ServiceMapEdge, ServiceMapNode } from "@/lib/api"
import {
  TRAFFIC_PRESETS,
  applyFocus,
  filterLowTraffic,
} from "@/lib/ecosystem-topology"
import type {
  FocusMode,
  TopologyEdge,
  TopologyNode,
  TrafficPreset,
} from "@/lib/ecosystem-topology"
import {
  rangeSearchSchema,
  resolveRangeSearch,
  updateRangeSearch,
} from "@/lib/range"

type EcosystemSearch = {
  range?: string | undefined
  from?: string | undefined
  to?: string | undefined
  focus?: string | undefined
  hops?: 1 | 2 | undefined
  focusMode?: FocusMode | undefined
  minTraffic?: TrafficPreset | undefined
}

const TRAFFIC_VALUES = new Set<TrafficPreset>(["all", "0.1%", "1%", "5%"])

export function validateEcosystemSearch(
  search: Record<string, unknown>
): EcosystemSearch {
  const parsed = rangeSearchSchema.parse(search)
  const hops = Number(search["hops"])
  const minTraffic = search["minTraffic"]
  return {
    range: typeof parsed.range === "string" ? parsed.range : undefined,
    from: typeof parsed.from === "string" ? parsed.from : undefined,
    to: typeof parsed.to === "string" ? parsed.to : undefined,
    focus:
      typeof search["focus"] === "string" && search["focus"].length > 0
        ? search["focus"]
        : undefined,
    hops: hops === 2 ? 2 : undefined,
    focusMode: search["focusMode"] === "hide" ? "hide" : undefined,
    minTraffic:
      typeof minTraffic === "string" &&
      TRAFFIC_VALUES.has(minTraffic as TrafficPreset) &&
      minTraffic !== "all"
        ? (minTraffic as TrafficPreset)
        : undefined,
  }
}

export const Route = createFileRoute("/ecosystem")({
  validateSearch: validateEcosystemSearch,
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
          nodes { name kind lastSeenNanos spanCount errorCount p95Ms }
          edges { source target callCount errorCount p50Ms p95Ms }
        }
      }
    `)
  },
  component: EcosystemPage,
})

interface GraphNode extends TopologyNode {
  data: ServiceMapNode
}

interface GraphEdge extends TopologyEdge {
  data: ServiceMapEdge
}

function projectServiceMap(serviceMap: ServiceMap, search: EcosystemSearch) {
  const nodes: GraphNode[] = serviceMap.nodes.map((node) => ({
    id: node.name,
    name: node.name,
    kind: node.kind,
    data: node,
  }))
  const edges: GraphEdge[] = serviceMap.edges.map((edge, index) => ({
    id: `${edge.source}->${edge.target}:${index}`,
    source: edge.source,
    target: edge.target,
    callCount: Number(edge.callCount) || 0,
    errorCount: Number(edge.errorCount) || 0,
    p50Ms: edge.p50Ms,
    p95Ms: edge.p95Ms,
    data: edge,
  }))
  const focused = applyFocus(nodes, edges, {
    focus: search.focus ?? null,
    hops: search.hops ?? 1,
    mode: search.focusMode ?? "dim",
  })
  const traffic = filterLowTraffic(
    focused.edges,
    TRAFFIC_PRESETS[search.minTraffic ?? "all"]
  )
  return {
    nodes: focused.nodes.map((node) => node.data),
    edges: traffic.edges.map((edge) => edge.data),
    dimmedNodeIds: focused.outside,
    hiddenNodeCount: nodes.length - focused.nodes.length,
    hiddenEdgeCount: edges.length - traffic.edges.length,
  }
}

function EcosystemControls({
  services,
  search,
  update,
}: {
  services: string[]
  search: EcosystemSearch
  update: (patch: Partial<EcosystemSearch>) => void
}) {
  return (
    <div className="flex flex-wrap items-center gap-2">
      <Select
        value={search.focus ?? "all"}
        onValueChange={(value) =>
          update({ focus: !value || value === "all" ? undefined : value })
        }
      >
        <SelectTrigger size="sm" aria-label="Focus service">
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
        value={String(search.hops ?? 1)}
        onValueChange={(value) =>
          update({ hops: value === "2" ? 2 : undefined })
        }
      >
        <SelectTrigger size="sm" aria-label="Focus hops">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="1">1 hop</SelectItem>
          <SelectItem value="2">2 hops</SelectItem>
        </SelectContent>
      </Select>
      <Select
        value={search.focusMode ?? "dim"}
        onValueChange={(value) =>
          update({ focusMode: value === "hide" ? "hide" : undefined })
        }
      >
        <SelectTrigger size="sm" aria-label="Outside focus behavior">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="dim">Dim outside</SelectItem>
          <SelectItem value="hide">Hide outside</SelectItem>
        </SelectContent>
      </Select>
      <Select
        value={search.minTraffic ?? "all"}
        onValueChange={(value) =>
          update({
            minTraffic:
              value && value !== "all" ? (value as TrafficPreset) : undefined,
          })
        }
      >
        <SelectTrigger size="sm" aria-label="Minimum traffic">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">All traffic</SelectItem>
          <SelectItem value="0.1%">&gt;0.1% traffic</SelectItem>
          <SelectItem value="1%">&gt;1% traffic</SelectItem>
          <SelectItem value="5%">&gt;5% traffic</SelectItem>
        </SelectContent>
      </Select>
    </div>
  )
}

function EcosystemPage() {
  const { serviceMap } = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
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
