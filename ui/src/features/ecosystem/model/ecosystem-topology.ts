/** Service-map topology pure helpers (plan 166).
 *
 * Focus neighborhood, low-traffic edge filter, and external-dependency
 * identity resolution from generic span attributes. Layout (ELK) and graph
 * rendering stay separate so this file can land without touching the peer's
 * ecosystem-graph.tsx WIP.
 *
 * Preliminary — peer must wire into ecosystem-graph/ecosystem route, add ELK
 * layout, backend external-node derivation, browser evidence.
 */

export type TopologyNodeKind = "service" | "cli" | "browser" | "database" | "queue" | "external"

export interface TopologyNode {
  id: string
  name: string
  kind?: TopologyNodeKind
  system?: string
}

export interface TopologyEdge {
  id?: string
  source: string
  target: string
  callCount: number
  errorCount?: number
  p50Ms?: number
  p95Ms?: number
}

export type FocusMode = "dim" | "hide"

export interface FocusOptions {
  /** Focused service/node id (usually service name). */
  focus: string | null
  /** Hop radius from the focus node (1 or 2 typical). */
  hops: number
  /** Dim keeps outsiders; hide removes them. */
  mode: FocusMode
}

export interface FocusResult<N extends TopologyNode, E extends TopologyEdge> {
  nodes: N[]
  edges: E[]
  /** Node ids inside the focus neighborhood (includes focus). */
  inFocus: ReadonlySet<string>
  /** Node ids outside the neighborhood (for dim rendering). */
  outside: ReadonlySet<string>
}

/** Build undirected adjacency for hop expansion. */
export function buildAdjacency(edges: readonly TopologyEdge[]): Map<string, Set<string>> {
  const adj = new Map<string, Set<string>>()
  const add = (a: string, b: string) => {
    let set = adj.get(a)
    if (!set) {
      set = new Set()
      adj.set(a, set)
    }
    set.add(b)
  }
  for (const e of edges) {
    add(e.source, e.target)
    add(e.target, e.source)
  }
  return adj
}

/** Nodes within `hops` undirected hops of `focus` (includes focus). */
export function neighborhoodIds(
  focus: string,
  hops: number,
  edges: readonly TopologyEdge[]
): Set<string> {
  const adj = buildAdjacency(edges)
  const result = new Set<string>([focus])
  if (hops <= 0) return result
  let frontier = new Set<string>([focus])
  for (let h = 0; h < hops; h++) {
    const next = new Set<string>()
    for (const id of frontier) {
      for (const n of adj.get(id) ?? []) {
        if (!result.has(n)) {
          result.add(n)
          next.add(n)
        }
      }
    }
    frontier = next
    if (frontier.size === 0) break
  }
  return result
}

/** Apply focus mode: hide filters the graph; dim keeps all nodes but tags sets. */
export function applyFocus<N extends TopologyNode, E extends TopologyEdge>(
  nodes: readonly N[],
  edges: readonly E[],
  options: FocusOptions
): FocusResult<N, E> {
  const knownIds = new Set(nodes.map((node) => node.id))
  // A bookmarked focus can outlive a renamed/removed service. Treat that as
  // no focus instead of returning an empty graph in hide mode.
  if (!options.focus || !knownIds.has(options.focus)) {
    const all = new Set(nodes.map((n) => n.id))
    return {
      nodes: [...nodes],
      edges: [...edges],
      inFocus: all,
      outside: new Set(),
    }
  }
  const inFocus = neighborhoodIds(options.focus, Math.max(0, options.hops), edges)
  const outside = new Set(nodes.map((n) => n.id).filter((id) => !inFocus.has(id)))
  if (options.mode === "dim") {
    return {
      nodes: [...nodes],
      edges: [...edges],
      inFocus,
      outside,
    }
  }
  // hide
  return {
    nodes: nodes.filter((n) => inFocus.has(n.id)),
    edges: edges.filter((e) => inFocus.has(e.source) && inFocus.has(e.target)),
    inFocus,
    outside,
  }
}

/** Traffic filter presets as fraction of max edge callCount (0 = show all). */
export const TRAFFIC_PRESETS = {
  all: 0,
  "0.1%": 0.001,
  "1%": 0.01,
  "5%": 0.05,
} as const

export type TrafficPreset = keyof typeof TRAFFIC_PRESETS

export interface TrafficFilterResult<E extends TopologyEdge> {
  edges: E[]
  hiddenCount: number
  maxCallCount: number
  minCallCount: number
}

/**
 * Hide edges whose callCount is strictly below `minFraction * max(callCount)`.
 * `minFraction <= 0` keeps every edge.
 */
export function filterLowTraffic<E extends TopologyEdge>(
  edges: readonly E[],
  minFraction: number
): TrafficFilterResult<E> {
  if (edges.length === 0) {
    return { edges: [], hiddenCount: 0, maxCallCount: 0, minCallCount: 0 }
  }
  const maxCallCount = edges.reduce((m, e) => Math.max(m, e.callCount), 0)
  if (minFraction <= 0 || maxCallCount <= 0) {
    return {
      edges: [...edges],
      hiddenCount: 0,
      maxCallCount,
      minCallCount: 0,
    }
  }
  const minCallCount = maxCallCount * minFraction
  const kept: E[] = []
  let hiddenCount = 0
  for (const e of edges) {
    if (e.callCount < minCallCount) {
      hiddenCount++
    } else {
      kept.push(e)
    }
  }
  return { edges: kept, hiddenCount, maxCallCount, minCallCount }
}

/** Generic span attributes used for external-node identity (plan 166 ladder). */
export interface ExternalSpanAttrs {
  "db.system.name"?: string
  "db.system"?: string
  "db.namespace"?: string
  "db.name"?: string
  "server.address"?: string
  "messaging.system"?: string
  "messaging.destination.name"?: string
  [key: string]: string | undefined
}

export interface ExternalNodeIdentity {
  kind: "database" | "queue" | "external"
  /** Display / graph node name. */
  name: string
  /** System label (postgresql, kafka, host, …). */
  system: string
}

/**
 * Resolve an external dependency identity from CLIENT/PRODUCER span attrs.
 * Returns null when no external-node attributes are present.
 * Ladder: db.* → messaging.* → server.address.
 */
export function resolveExternalNode(attrs: ExternalSpanAttrs): ExternalNodeIdentity | null {
  const dbSystem = attrs["db.system.name"] ?? attrs["db.system"]
  if (dbSystem) {
    const name = attrs["db.namespace"] ?? attrs["db.name"] ?? attrs["server.address"] ?? dbSystem
    return { kind: "database", name, system: dbSystem }
  }
  const messaging = attrs["messaging.system"]
  if (messaging) {
    const name = attrs["messaging.destination.name"] ?? messaging
    return { kind: "queue", name, system: messaging }
  }
  const host = attrs["server.address"]
  if (host) {
    return { kind: "external", name: host, system: host }
  }
  return null
}

/** Edge error rate in [0, 1]; 0 when no calls. */
export function edgeErrorRate(edge: Pick<TopologyEdge, "callCount" | "errorCount">): number {
  if (edge.callCount <= 0) return 0
  return (edge.errorCount ?? 0) / edge.callCount
}

/** log2-scaled edge width in [minW, maxW] from callCount. */
export function edgeWidthFromCalls(callCount: number, minW = 1, maxW = 8): number {
  if (callCount <= 0) return minW
  const t = Math.log2(callCount + 1) / Math.log2(1_000 + 1)
  return minW + (maxW - minW) * Math.min(1, Math.max(0, t))
}
