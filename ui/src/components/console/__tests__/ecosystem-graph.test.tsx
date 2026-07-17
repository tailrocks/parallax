/* @vitest-environment jsdom */

import { screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"

import { EcosystemGraph } from "@/components/console/ecosystem-graph"
import type { ServiceMapEdge, ServiceMapNode } from "@/lib/api"
import { customRange } from "@/lib/range"
import { renderTestRouter } from "@/test/router"

const nodes: ServiceMapNode[] = [
  {
    kind: "service" as const,
    name: "A",
    lastSeenNanos: "100",
    spanCount: "10",
    errorCount: "0",
    p95Ms: 20,
  },
  {
    kind: "service" as const,
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
      <EcosystemGraph
        nodes={nodes}
        edges={edges}
        range={customRange("0", "200")}
      />,
      { targetPaths: ["/services/$service", "/traces"] }
    )

    expect(await screen.findByText("trace-path")).toBeTruthy()
    expect(screen.getByText("A")).toBeTruthy()
    expect(screen.getByText("B")).toBeTruthy()
    expect(screen.getByText("A -> B")).toBeTruthy()
    expect(screen.getByText(/50% errors/)).toBeTruthy()

    const serviceLink = screen.getByText("A").closest("a")
    expect(serviceLink?.href).toContain("/services/A")
    const edgeLink = screen.getByText("A -> B").closest("a")
    expect(edgeLink?.href).toContain("/traces")
    expect(edgeLink?.href).toContain("service=A")
  })
})

it("D-014 eco-full: a 9-node column grows the canvas instead of overlapping cards", async () => {
  const nodes = Array.from({ length: 9 }, (_, i) => ({
    name: `svc-${i}`,
    kind: "service" as const,
    lastSeenNanos: "0",
    spanCount: "1",
    errorCount: "0",
    p95Ms: null,
  }))
  renderTestRouter(
    <EcosystemGraph nodes={nodes} edges={[]} range={customRange("0", "200")} />,
    { targetPaths: ["/services/$service", "/traces"] }
  )
  const cards = await screen.findAllByText(/svc-/)
  expect(cards.length).toBe(9)
  const container = document.querySelector('[style*="min-height"]')
  const minHeight = Number.parseInt(
    (container as HTMLElement).style.minHeight,
    10
  )
  expect(minHeight).toBeGreaterThanOrEqual(9 * 80)
})
