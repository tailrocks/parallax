import { describe, expect, it } from "vitest"

import type { ServiceMapEdge, ServiceMapNode } from "@/lib/api"
import { applyEcosystemFocus, focusNeighborhood } from "@/lib/ecosystem-focus"

const node = (name: string): ServiceMapNode => ({
  name,
  kind: "service",
  lastSeenNanos: "0",
  spanCount: "1",
  errorCount: "0",
  p95Ms: 1,
})

const edge = (
  source: string,
  target: string,
  callCount: number
): ServiceMapEdge => ({
  source,
  target,
  callCount: String(callCount),
  errorCount: "0",
  p50Ms: 1,
  p95Ms: 2,
})

const nodes = ["browser", "checkout", "inventory", "postgres", "email"].map(
  node
)
const edges = [
  edge("browser", "checkout", 1_000),
  edge("checkout", "inventory", 500),
  edge("inventory", "postgres", 100),
  edge("checkout", "email", 5),
]

describe("focusNeighborhood", () => {
  it("includes callers and callees at one hop", () => {
    expect([...focusNeighborhood(nodes, edges, "checkout", 1)].sort()).toEqual([
      "browser",
      "checkout",
      "email",
      "inventory",
    ])
  })

  it("expands through both directions at two hops", () => {
    expect([...focusNeighborhood(nodes, edges, "checkout", 2)].sort()).toEqual([
      "browser",
      "checkout",
      "email",
      "inventory",
      "postgres",
    ])
  })

  it("ignores a stale unknown focus instead of blanking the graph", () => {
    expect(focusNeighborhood(nodes, edges, "deleted", 1).size).toBe(
      nodes.length
    )
  })
})

describe("applyEcosystemFocus", () => {
  it("dims outside nodes while preserving the topology", () => {
    const result = applyEcosystemFocus(nodes, edges, {
      focus: "checkout",
      hops: 1,
      mode: "dim",
      minTraffic: 0,
    })
    expect(result.nodes).toHaveLength(5)
    expect(result.edges).toHaveLength(4)
    expect(result.nodes.find((item) => item.name === "postgres")?.dimmed).toBe(
      true
    )
    expect(result.hiddenNodeCount).toBe(0)
  })

  it("hides outside nodes and every incident edge", () => {
    const result = applyEcosystemFocus(nodes, edges, {
      focus: "checkout",
      hops: 1,
      mode: "hide",
      minTraffic: 0,
    })
    expect(result.nodes.map((item) => item.name).sort()).toEqual([
      "browser",
      "checkout",
      "email",
      "inventory",
    ])
    expect(result.edges.map((item) => item.target).sort()).toEqual([
      "checkout",
      "email",
      "inventory",
    ])
    expect(result.hiddenNodeCount).toBe(1)
    expect(result.hiddenEdgeCount).toBe(1)
  })

  it("filters traffic relative to the busiest edge and counts hidden edges", () => {
    const result = applyEcosystemFocus(nodes, edges, {
      focus: null,
      hops: 1,
      mode: "dim",
      minTraffic: 0.01,
    })
    expect(result.edges.map((item) => item.callCount)).toEqual([
      "1000",
      "500",
      "100",
    ])
    expect(result.hiddenEdgeCount).toBe(1)
  })

  it("treats malformed counts as zero and clamps threshold presets", () => {
    const malformed = { ...edges[0]!, callCount: "not-a-number" }
    const result = applyEcosystemFocus(nodes, [malformed, ...edges.slice(1)], {
      focus: null,
      hops: 1,
      mode: "hide",
      minTraffic: 4,
    })
    expect(result.edges).toHaveLength(0)
    expect(result.hiddenEdgeCount).toBe(4)
  })
})
