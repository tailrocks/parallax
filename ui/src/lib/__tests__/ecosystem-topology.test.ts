import { describe, expect, it } from "vitest"

import {
  TRAFFIC_PRESETS,
  applyFocus,
  edgeErrorRate,
  edgeWidthFromCalls,
  filterLowTraffic,
  neighborhoodIds,
  resolveExternalNode,
  type TopologyEdge,
  type TopologyNode,
} from "@/lib/ecosystem-topology"

const nodes: TopologyNode[] = [
  { id: "checkout", name: "checkout" },
  { id: "pricing", name: "pricing" },
  { id: "inventory", name: "inventory" },
  { id: "cli", name: "cli", kind: "cli" },
  { id: "kafka", name: "kafka", kind: "queue" },
]

const edges: TopologyEdge[] = [
  { source: "cli", target: "checkout", callCount: 2, errorCount: 0 },
  { source: "checkout", target: "pricing", callCount: 1000, errorCount: 10 },
  { source: "checkout", target: "inventory", callCount: 500, errorCount: 0 },
  { source: "inventory", target: "kafka", callCount: 50, errorCount: 5 },
]

describe("neighborhoodIds", () => {
  it("includes only the focus at 0 hops", () => {
    expect([...neighborhoodIds("checkout", 0, edges)].sort()).toEqual(["checkout"])
  })

  it("expands 1-hop undirected neighborhood", () => {
    const ids = neighborhoodIds("checkout", 1, edges)
    expect(ids.has("checkout")).toBe(true)
    expect(ids.has("pricing")).toBe(true)
    expect(ids.has("inventory")).toBe(true)
    expect(ids.has("cli")).toBe(true)
    expect(ids.has("kafka")).toBe(false)
  })

  it("reaches kafka at 2 hops from checkout", () => {
    expect(neighborhoodIds("checkout", 2, edges).has("kafka")).toBe(true)
  })
})

describe("applyFocus", () => {
  it("hide mode drops outside nodes and edges", () => {
    const result = applyFocus(nodes, edges, {
      focus: "checkout",
      hops: 1,
      mode: "hide",
    })
    expect(result.nodes.map((n) => n.id).sort()).toEqual(
      ["checkout", "cli", "inventory", "pricing"].sort()
    )
    expect(result.edges.every((e) => result.inFocus.has(e.source))).toBe(true)
    expect(result.outside.has("kafka")).toBe(true)
  })

  it("dim mode keeps full graph and tags outside", () => {
    const result = applyFocus(nodes, edges, {
      focus: "checkout",
      hops: 1,
      mode: "dim",
    })
    expect(result.nodes).toHaveLength(nodes.length)
    expect(result.outside.has("kafka")).toBe(true)
    expect(result.inFocus.has("checkout")).toBe(true)
  })

  it("null focus keeps everything in-focus", () => {
    const result = applyFocus(nodes, edges, {
      focus: null,
      hops: 1,
      mode: "hide",
    })
    expect(result.nodes).toHaveLength(nodes.length)
    expect(result.outside.size).toBe(0)
  })
})

describe("filterLowTraffic", () => {
  it("all preset hides nothing", () => {
    const r = filterLowTraffic(edges, TRAFFIC_PRESETS.all)
    expect(r.hiddenCount).toBe(0)
    expect(r.edges).toHaveLength(edges.length)
  })

  it("1% preset hides the low-rate CLI edge", () => {
    // max = 1000; 1% => min 10; cli edge callCount 2 is hidden
    const r = filterLowTraffic(edges, TRAFFIC_PRESETS["1%"])
    expect(r.maxCallCount).toBe(1000)
    expect(r.hiddenCount).toBe(1)
    expect(r.edges.some((e) => e.source === "cli")).toBe(false)
    expect(r.edges.some((e) => e.source === "checkout" && e.target === "pricing")).toBe(
      true
    )
  })

  it("5% keeps edges at exactly the threshold, hides below", () => {
    // max=1000; 5% => minCallCount 50; kafka has 50 (kept); cli has 2 (hidden)
    const r = filterLowTraffic(edges, TRAFFIC_PRESETS["5%"])
    expect(r.minCallCount).toBe(50)
    expect(r.edges.some((e) => e.target === "kafka")).toBe(true)
    expect(r.edges.some((e) => e.source === "cli")).toBe(false)
    expect(r.hiddenCount).toBe(1)
  })
})

describe("resolveExternalNode", () => {
  it("prefers db.system.name ladder for database nodes", () => {
    expect(
      resolveExternalNode({
        "db.system.name": "postgresql",
        "db.namespace": "inventory",
        "server.address": "db.internal",
      })
    ).toEqual({
      kind: "database",
      name: "inventory",
      system: "postgresql",
    })
  })

  it("falls back to legacy db.system and server.address name", () => {
    expect(
      resolveExternalNode({
        "db.system": "mysql",
        "server.address": "mysql.prod",
      })
    ).toEqual({ kind: "database", name: "mysql.prod", system: "mysql" })
  })

  it("resolves messaging as queue", () => {
    expect(
      resolveExternalNode({
        "messaging.system": "kafka",
        "messaging.destination.name": "orders",
      })
    ).toEqual({ kind: "queue", name: "orders", system: "kafka" })
  })

  it("resolves bare server.address as external HTTP", () => {
    expect(resolveExternalNode({ "server.address": "api.stripe.com" })).toEqual({
      kind: "external",
      name: "api.stripe.com",
      system: "api.stripe.com",
    })
  })

  it("returns null without external attributes", () => {
    expect(resolveExternalNode({ "http.method": "GET" })).toBeNull()
  })
})

describe("edge helpers", () => {
  it("computes error rate from counts", () => {
    expect(edgeErrorRate({ callCount: 100, errorCount: 25 })).toBe(0.25)
    expect(edgeErrorRate({ callCount: 0, errorCount: 5 })).toBe(0)
  })

  it("scales width with log2(callCount)", () => {
    expect(edgeWidthFromCalls(0)).toBe(1)
    expect(edgeWidthFromCalls(1)).toBeGreaterThan(1)
    expect(edgeWidthFromCalls(1_000)).toBe(8)
  })
})
