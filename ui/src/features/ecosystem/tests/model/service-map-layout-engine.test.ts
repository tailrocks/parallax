import { describe, expect, it, vi } from "vitest"

vi.mock("elkjs/lib/elk.bundled.js", () => ({
  default: class ELK {
    async layout(graph: { children?: Array<{ id: string }> }) {
      return {
        width: 600,
        height: 80,
        children: (graph.children ?? []).map((node, index) => ({
          id: node.id,
          x: index * 200,
          y: 0,
        })),
      }
    }
  },
}))

import type { ServiceMapEdge, ServiceMapNode } from "@/features/ecosystem/model/service-map"
import {
  ECOSYSTEM_NODE_HEIGHT,
  ECOSYSTEM_NODE_WIDTH,
  ecosystemTopologyKey,
  fallbackEcosystemLayout,
  runElkLayout,
} from "@/features/ecosystem/model/service-map-layout-engine"

const node = (name: string, spanCount = "1"): ServiceMapNode => ({
  name,
  kind: "service",
  lastSeenNanos: "0",
  spanCount,
  errorCount: "0",
  p95Ms: 1,
})

const edge = (source: string, target: string): ServiceMapEdge => ({
  source,
  target,
  callCount: "1",
  errorCount: "0",
  p50Ms: 1,
  p95Ms: 1,
})

const request = {
  nodes: [node("inventory"), node("checkout"), node("browser")],
  edges: [edge("checkout", "inventory"), edge("browser", "checkout")],
}

function boxesOverlap(
  a: { x: number; y: number },
  b: { x: number; y: number },
  width = ECOSYSTEM_NODE_WIDTH,
  height = ECOSYSTEM_NODE_HEIGHT
): boolean {
  return a.x < b.x + width && b.x < a.x + width && a.y < b.y + height && b.y < a.y + height
}

describe("service-map-layout-engine", () => {
  it("runElkLayout is deterministic", async () => {
    const first = await runElkLayout(request)
    const second = await runElkLayout({
      nodes: [...request.nodes].reverse(),
      edges: [...request.edges].reverse(),
    })
    expect(second).toEqual(first)
  })

  it("runElkLayout boxes do not overlap", async () => {
    const layout = await runElkLayout(request)
    for (let i = 0; i < layout.positions.length; i += 1) {
      for (let j = i + 1; j < layout.positions.length; j += 1) {
        expect(boxesOverlap(layout.positions[i]!, layout.positions[j]!)).toBe(false)
      }
    }
  })

  it("fallback is deterministic and non-overlapping", () => {
    const first = fallbackEcosystemLayout(request)
    const second = fallbackEcosystemLayout({
      nodes: [...request.nodes].reverse(),
      edges: [...request.edges].reverse(),
    })
    expect(second).toEqual(first)
    for (let i = 0; i < first.positions.length; i += 1) {
      for (let j = i + 1; j < first.positions.length; j += 1) {
        expect(boxesOverlap(first.positions[i]!, first.positions[j]!)).toBe(false)
      }
    }
  })

  it("topology key ignores metrics", () => {
    expect(ecosystemTopologyKey({ nodes: [node("a", "9")], edges: [] })).toBe(
      ecosystemTopologyKey({ nodes: [node("a", "1")], edges: [] })
    )
  })
})
