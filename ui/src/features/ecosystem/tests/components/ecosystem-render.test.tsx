/* @vitest-environment jsdom */

import { screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"

import { EcosystemGraph } from "@/features/ecosystem/components/ecosystem-graph"
import type { ServiceMapNode } from "@/features/ecosystem/model/service-map"
import { customRange } from "@/domain/time-range/range"
import { renderTestRouter } from "@/test/router"

const base: ServiceMapNode = {
  name: "svc",
  kind: "service",
  system: null,
  lastSeenNanos: "0",
  spanCount: "10",
  errorCount: "0",
  p95Ms: 20,
}

function serviceNode(name: string, errorCount = "0"): ServiceMapNode {
  return { ...base, name, errorCount }
}

describe("EcosystemGraph investigation rendering", () => {
  it("renders traffic bands, error encoding, and the complete legend", async () => {
    renderTestRouter(
      <EcosystemGraph
        nodes={[
          serviceNode("svc"),
          serviceNode("low", "1"),
          serviceNode("medium"),
          serviceNode("high"),
        ]}
        edges={[
          { source: "svc", target: "low", callCount: "2", errorCount: "1", p50Ms: 1, p95Ms: 2 },
          { source: "svc", target: "medium", callCount: "10", errorCount: "0", p50Ms: 1, p95Ms: 2 },
          { source: "svc", target: "high", callCount: "100", errorCount: "0", p50Ms: 1, p95Ms: 2 },
        ]}
        range={customRange("0", "200")}
      />,
      { targetPaths: ["/services/$service", "/traces"] }
    )

    const legend = await screen.findByLabelText("Investigation legend")
    expect([
      ["service", "cli", "browser", "database", "queue", "external", "healthy", "errors"].every(
        (value) => screen.getAllByText(value).length > 0
      ),
      ["low", "medium", "high"].every((value) => screen.getAllByText(value).length > 0),
      Boolean(legend.querySelector('line[stroke="var(--chart-error)"]')),
      ["low", "medium", "high"].every((band) =>
        Boolean(legend.querySelector(`.service-map-edge--traffic-${band}[stroke-width]`))
      ),
      legend.textContent,
    ]).toEqual([
      true,
      true,
      true,
      true,
      expect.stringContaining("Investigation legend") as unknown as string,
    ])
  })

  it("keeps the empty state actionable", async () => {
    renderTestRouter(<EcosystemGraph nodes={[]} edges={[]} range={customRange("0", "200")} />, {
      targetPaths: ["/services/$service", "/traces"],
    })
    expect(await screen.findByText("No service edges.")).toBeTruthy()
  })

  it("keeps long dependency identities and metrics readable", async () => {
    const longName = `${"dependency-".repeat(18)}database`
    renderTestRouter(
      <EcosystemGraph
        nodes={[
          {
            ...base,
            name: longName,
            kind: "database",
            system: "postgresql",
            spanCount: "1",
            p95Ms: null,
          },
        ]}
        edges={[]}
        range={customRange("0", "200")}
      />,
      { targetPaths: ["/services/$service", "/traces"] }
    )
    const dependency = await screen.findByRole("group", {
      name: `database dependency ${longName}, system postgresql`,
    })
    expect([screen.getByText(longName).className, dependency.getAttribute("tabindex")]).toEqual([
      expect.stringContaining("truncate") as unknown as string,
      "0",
    ])
    expect(dependency.textContent).toContain("0 errors")
  })
})
