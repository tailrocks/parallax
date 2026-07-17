import { beforeEach, describe, expect, it } from "vitest"

import type { ServiceMapEdge, ServiceMapNode } from "@/lib/api"
import {
  clearEcosystemLayoutCache,
  ecosystemTopologyKey,
  fallbackEcosystemLayout,
  layoutEcosystem,
} from "@/lib/ecosystem-layout"

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

beforeEach(clearEcosystemLayoutCache)

describe("ecosystemTopologyKey", () => {
  it("is stable across input order and metric changes", () => {
    const reordered = {
      nodes: [node("browser", "999"), node("inventory"), node("checkout")],
      edges: [...request.edges].reverse(),
    }
    expect(ecosystemTopologyKey(reordered)).toBe(ecosystemTopologyKey(request))
  })

  it("changes with topology", () => {
    expect(
      ecosystemTopologyKey({
        ...request,
        edges: [...request.edges, edge("inventory", "postgres")],
      })
    ).not.toBe(ecosystemTopologyKey(request))
  })
})

describe("ELK ecosystem layout", () => {
  it("is deterministic across fallback runs and input order", () => {
    const first = fallbackEcosystemLayout(request)
    const second = fallbackEcosystemLayout({
      nodes: [...request.nodes].reverse(),
      edges: [...request.edges].reverse(),
    })
    expect(second).toEqual(first)
  })

  it("lays a directed chain from left to right", async () => {
    const layout = await layoutEcosystem(request)
    const positions = new Map(layout.positions.map((item) => [item.id, item]))
    expect(positions.get("browser")!.x).toBeLessThan(
      positions.get("checkout")!.x
    )
    expect(positions.get("checkout")!.x).toBeLessThan(
      positions.get("inventory")!.x
    )
    expect(layout.width).toBeGreaterThan(0)
    expect(layout.height).toBeGreaterThan(0)
  })

  it("memoizes identical topology", () => {
    const first = layoutEcosystem(request)
    const second = layoutEcosystem({
      nodes: [...request.nodes].reverse(),
      edges: [...request.edges].reverse(),
    })
    expect(second).toBe(first)
  })

  it("handles an empty graph", () => {
    const layout = fallbackEcosystemLayout({ nodes: [], edges: [] })
    expect(layout.positions).toEqual([])
    expect(layout.width).toBeGreaterThanOrEqual(0)
    expect(layout.height).toBeGreaterThanOrEqual(0)
  })
})
