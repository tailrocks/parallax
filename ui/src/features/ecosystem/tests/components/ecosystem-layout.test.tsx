/* @vitest-environment jsdom */

import { screen } from "@testing-library/react"
import { expect, it } from "vitest"

import { EcosystemGraph } from "@/features/ecosystem/components/ecosystem-graph"
import { customRange } from "@/domain/time-range/range"
import { renderTestRouter } from "@/test/router"

it("D-014 eco-full: a 9-node column grows the canvas instead of overlapping cards", async () => {
  const nodes = Array.from({ length: 9 }, (_, index) => ({
    name: `svc-${index}`,
    kind: "service",
    system: null,
    lastSeenNanos: "0",
    spanCount: "1",
    errorCount: "0",
    p95Ms: null,
  }))
  renderTestRouter(<EcosystemGraph nodes={nodes} edges={[]} range={customRange("0", "200")} />, {
    targetPaths: ["/services/$service", "/traces"],
  })
  const cards = await screen.findAllByText(/svc-/)
  const container = document.querySelector('[aria-label="service dependency graph"]') as HTMLElement
  expect([cards.length, Number.parseInt(container.style.height, 10)]).toEqual([
    9,
    expect.any(Number) as number,
  ])
})
