import type { EcosystemSearch } from "@/features/ecosystem/model/ecosystem-search"
import type {
  ServiceMap,
  ServiceMapEdge,
  ServiceMapNode,
} from "@/features/ecosystem/model/service-map"
import {
  TRAFFIC_PRESETS,
  applyFocus,
  filterLowTraffic,
  type TopologyEdge,
  type TopologyNode,
} from "@/features/ecosystem/model/ecosystem-topology"

type GraphNode = TopologyNode & { data: ServiceMapNode }
type GraphEdge = TopologyEdge & { data: ServiceMapEdge }

export type ProjectedServiceMap = {
  readonly nodes: ServiceMapNode[]
  readonly edges: ServiceMapEdge[]
  readonly dimmedNodeIds: ReadonlySet<string>
  readonly hiddenNodeCount: number
  readonly hiddenEdgeCount: number
}

export function projectServiceMap(
  serviceMap: ServiceMap,
  search: EcosystemSearch
): ProjectedServiceMap {
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
