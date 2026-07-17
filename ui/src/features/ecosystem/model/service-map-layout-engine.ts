import type { ElkNode } from "elkjs/lib/elk-api"

import type {
  ServiceMapEdge,
  ServiceMapNode,
} from "@/features/ecosystem/model/service-map"

export const ECOSYSTEM_NODE_WIDTH = 150
export const ECOSYSTEM_NODE_HEIGHT = 58

export interface EcosystemPosition {
  id: string
  x: number
  y: number
}

export interface EcosystemLayout {
  positions: EcosystemPosition[]
  width: number
  height: number
}

export interface EcosystemLayoutRequest {
  nodes: ServiceMapNode[]
  edges: ServiceMapEdge[]
}

export type EcosystemLayoutResponse =
  | { ok: true; layout: EcosystemLayout }
  | { ok: false; error: string }


function sortedRequest(
  request: EcosystemLayoutRequest
): EcosystemLayoutRequest {
  return {
    nodes: [...request.nodes].sort((a, b) => a.name.localeCompare(b.name)),
    edges: [...request.edges].sort((a, b) => {
      const source = a.source.localeCompare(b.source)
      return source !== 0 ? source : a.target.localeCompare(b.target)
    }),
  }
}

/** Stable topology-only cache key: metric changes do not trigger relayout. */
export function ecosystemTopologyKey(request: EcosystemLayoutRequest): string {
  const sorted = sortedRequest(request)
  return JSON.stringify({
    nodes: sorted.nodes.map((node) => node.name),
    edges: sorted.edges.map((edge) => [edge.source, edge.target]),
  })
}

function elkGraph(request: EcosystemLayoutRequest): ElkNode {
  const sorted = sortedRequest(request)
  return {
    id: "ecosystem",
    layoutOptions: {
      "elk.algorithm": "layered",
      "elk.direction": "RIGHT",
      "elk.edgeRouting": "ORTHOGONAL",
      "elk.randomSeed": "1",
      "elk.spacing.nodeNode": "50",
      "elk.layered.spacing.nodeNodeBetweenLayers": "100",
      "elk.layered.considerModelOrder.strategy": "NODES_AND_EDGES",
    },
    children: sorted.nodes.map((node) => ({
      id: node.name,
      width: ECOSYSTEM_NODE_WIDTH,
      height: ECOSYSTEM_NODE_HEIGHT,
    })),
    edges: sorted.edges.map((edge, index) => ({
      id: `${edge.source}->${edge.target}:${index}`,
      sources: [edge.source],
      targets: [edge.target],
    })),
  }
}

/** Direct ELK execution used by the worker and by tests/SSR fallback. */
export async function runElkLayout(
  request: EcosystemLayoutRequest
): Promise<EcosystemLayout> {
  const { default: ELK } = await import("elkjs/lib/elk.bundled.js")
  const elk = new ELK()
  const graph = await elk.layout(elkGraph(request))
  const positions = (graph.children ?? [])
    .map((node) => ({ id: node.id, x: node.x ?? 0, y: node.y ?? 0 }))
    .sort((a, b) => a.id.localeCompare(b.id))
  return {
    positions,
    width: graph.width ?? ECOSYSTEM_NODE_WIDTH,
    height: graph.height ?? ECOSYSTEM_NODE_HEIGHT,
  }
}

/** Deterministic synchronous fallback for Bun/Vitest/SSR and worker startup
 * failure. Kahn layers preserve graph direction; cycles settle into the last
 * layer instead of causing unbounded relaxation. */
export function fallbackEcosystemLayout(
  request: EcosystemLayoutRequest
): EcosystemLayout {
  const sorted = sortedRequest(request)
  const names = sorted.nodes.map((node) => node.name)
  const known = new Set(names)
  const incoming = new Map(names.map((name) => [name, 0]))
  const outgoing = new Map(names.map((name) => [name, [] as string[]]))
  for (const edge of sorted.edges) {
    if (!known.has(edge.source) || !known.has(edge.target)) continue
    outgoing.get(edge.source)?.push(edge.target)
    incoming.set(edge.target, (incoming.get(edge.target) ?? 0) + 1)
  }
  const depth = new Map(names.map((name) => [name, 0]))
  const queue = names.filter((name) => incoming.get(name) === 0)
  const visited = new Set<string>()
  while (queue.length > 0) {
    const name = queue.shift()!
    visited.add(name)
    for (const target of outgoing.get(name) ?? []) {
      depth.set(
        target,
        Math.max(depth.get(target) ?? 0, (depth.get(name) ?? 0) + 1)
      )
      const remaining = (incoming.get(target) ?? 0) - 1
      incoming.set(target, remaining)
      if (remaining === 0) queue.push(target)
    }
    queue.sort()
  }
  const terminalDepth = Math.max(0, ...depth.values())
  for (const name of names) {
    if (!visited.has(name)) depth.set(name, terminalDepth)
  }

  const groups = new Map<number, string[]>()
  for (const name of names) {
    const level = depth.get(name) ?? 0
    groups.set(level, [...(groups.get(level) ?? []), name])
  }
  const xGap = ECOSYSTEM_NODE_WIDTH + 100
  const yGap = ECOSYSTEM_NODE_HEIGHT + 50
  const positions = [...groups.entries()]
    .flatMap(([level, group]) =>
      group.sort().map((id, index) => ({
        id,
        x: 20 + level * xGap,
        y: 20 + index * yGap,
      }))
    )
    .sort((a, b) => a.id.localeCompare(b.id))
  const maxDepth = Math.max(0, ...groups.keys())
  const maxRows = Math.max(
    1,
    ...[...groups.values()].map((group) => group.length)
  )
  return {
    positions,
    width: names.length === 0 ? 0 : 40 + maxDepth * xGap + ECOSYSTEM_NODE_WIDTH,
    height:
      names.length === 0
        ? 0
        : 40 + (maxRows - 1) * yGap + ECOSYSTEM_NODE_HEIGHT,
  }
}
