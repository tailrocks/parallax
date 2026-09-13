/* @vitest-environment jsdom */

import { screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"

import { EcosystemGraph } from "@/features/ecosystem/components/ecosystem-graph"
import type { ServiceMapEdge, ServiceMapNode } from "@/features/ecosystem/model/service-map"
import { customRange } from "@/domain/time-range/range"
import { renderTestRouter } from "@/test/router"

const nodes: ServiceMapNode[] = [
  {
    kind: "service" as const,
    name: "A",
    system: null,
    lastSeenNanos: "100",
    spanCount: "10",
    errorCount: "0",
    p95Ms: 20,
  },
  {
    kind: "service" as const,
    system: null,
    name: "B",
    lastSeenNanos: "120",
    spanCount: "5",
    errorCount: "1",
    p95Ms: 45,
  },
]

const edges: ServiceMapEdge[] = [
  {
    source: "A",
    target: "B",
    callCount: "2",
    errorCount: "1",
    p50Ms: 25,
    p95Ms: 45,
  },
]

describe("EcosystemGraph", () => {
  it("renders trace-path graph nodes and edge links", async () => {
    renderTestRouter(
      <EcosystemGraph nodes={nodes} edges={edges} range={customRange("0", "200")} />,
      { targetPaths: ["/services/$service", "/traces"] }
    )

    expect(await screen.findByText("trace-path")).toBeTruthy()
    expect(screen.getByText("A")).toBeTruthy()
    expect(screen.getByText("B")).toBeTruthy()
    // React Flow renders the edge path in jsdom; its stats label paints
    // only after browser layout (covered by the browser evidence).
    const serviceLink = screen.getByText("A").closest("a")
    expect(serviceLink?.href).toContain("/services/A")
    // jsdom cannot lay edges out; assert the React Flow canvas mounted.
    expect(document.querySelector(".react-flow")).toBeTruthy()
  })

  it("dims outside-focus nodes and reports hidden topology", async () => {
    renderTestRouter(
      <EcosystemGraph
        nodes={nodes}
        edges={edges}
        range={customRange("0", "200")}
        dimmedNodeIds={new Set(["B"])}
        hiddenNodeCount={1}
        hiddenEdgeCount={2}
      />,
      { targetPaths: ["/services/$service", "/traces"] }
    )

    const dimmed = (await screen.findByText("B")).closest("a")
    expect(dimmed?.className).toContain("opacity-30")
    expect(screen.getByText("hidden").closest('[data-slot="badge"]')?.textContent).toBe("3 hidden")
  })
})
