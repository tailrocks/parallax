import type { ServiceMapEdge, ServiceMapNode } from "@/lib/api"

export type FocusHops = 1 | 2
export type FocusMode = "dim" | "hide"

export interface EcosystemFocusOptions {
  focus: string | null
  hops: FocusHops
  mode: FocusMode
  /** Fraction of the busiest edge, in [0, 1]. Zero keeps every edge. */
  minTraffic: number
}

export interface FocusedNode extends ServiceMapNode {
  dimmed: boolean
}

export interface EcosystemFocusResult {
  nodes: FocusedNode[]
  edges: ServiceMapEdge[]
  hiddenNodeCount: number
  hiddenEdgeCount: number
  neighborhood: ReadonlySet<string>
}

function calls(edge: ServiceMapEdge): number {
  const value = Number(edge.callCount)
  return Number.isFinite(value) && value > 0 ? value : 0
}

/** Undirected hop neighborhood: callers and callees both matter while
 * investigating a service. Unknown focus names produce no focus filter so a
 * stale permalink never blanks the graph. */
export function focusNeighborhood(
  nodes: readonly ServiceMapNode[],
  edges: readonly ServiceMapEdge[],
  focus: string | null,
  hops: FocusHops
): ReadonlySet<string> {
  const known = new Set(nodes.map((node) => node.name))
  if (!focus || !known.has(focus)) return known

  const neighborhood = new Set([focus])
  let frontier = new Set([focus])
  for (let distance = 0; distance < hops; distance += 1) {
    const next = new Set<string>()
    for (const edge of edges) {
      if (frontier.has(edge.source) && known.has(edge.target)) {
        next.add(edge.target)
      }
      if (frontier.has(edge.target) && known.has(edge.source)) {
        next.add(edge.source)
      }
    }
    for (const name of next) neighborhood.add(name)
    frontier = next
  }
  return neighborhood
}

/** Pure focus/declutter projection shared by the graph and URL-state tests.
 * Traffic threshold is relative to the busiest edge. Focus-hide removes
 * outside nodes and their incident edges; focus-dim retains topology and
 * annotates outside nodes. */
export function applyEcosystemFocus(
  nodes: readonly ServiceMapNode[],
  edges: readonly ServiceMapEdge[],
  options: EcosystemFocusOptions
): EcosystemFocusResult {
  const neighborhood = focusNeighborhood(
    nodes,
    edges,
    options.focus,
    options.hops
  )
  const focusActive = options.focus !== null && neighborhood.size < nodes.length
  const thresholdFraction = Math.min(Math.max(options.minTraffic, 0), 1)
  const maxCalls = Math.max(0, ...edges.map(calls))
  const trafficThreshold = maxCalls * thresholdFraction
  const trafficEdges = edges.filter(
    (edge) => thresholdFraction === 0 || calls(edge) > trafficThreshold
  )

  const visibleNodes =
    focusActive && options.mode === "hide"
      ? nodes.filter((node) => neighborhood.has(node.name))
      : [...nodes]
  const visibleNames = new Set(visibleNodes.map((node) => node.name))
  const visibleEdges = trafficEdges.filter(
    (edge) => visibleNames.has(edge.source) && visibleNames.has(edge.target)
  )

  return {
    nodes: visibleNodes.map((node) => ({
      ...node,
      dimmed:
        focusActive && options.mode === "dim" && !neighborhood.has(node.name),
    })),
    edges: visibleEdges,
    hiddenNodeCount: nodes.length - visibleNodes.length,
    hiddenEdgeCount: edges.length - visibleEdges.length,
    neighborhood,
  }
}
