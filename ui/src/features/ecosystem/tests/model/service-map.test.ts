import { describe, expect, it } from "vitest"

import { serviceMapEdgePresentation } from "@/features/ecosystem/model/ecosystem-topology"
import { mapServiceMap } from "@/features/ecosystem/model/service-map"

const raw = (kind: string, system: string | null = null) => ({
  name: `${kind}-node`,
  kind,
  system,
  lastSeenNanos: "0",
  spanCount: "1",
  errorCount: "0",
  p95Ms: null,
})

describe("service-map wire mapping", () => {
  it("preserves every backend kind and system without downgrades", () => {
    const mapped = mapServiceMap({
      nodes: [
        raw("service"),
        raw("cli"),
        raw("browser"),
        raw("database", "postgresql"),
        raw("queue", "rabbitmq/fulfillment"),
        raw("external", "api.stripe.test"),
        raw("cache"),
      ],
      edges: [],
    })

    expect(mapped.nodes.map((node) => [node.kind, node.system])).toEqual([
      ["service", null],
      ["cli", null],
      ["browser", null],
      ["database", "postgresql"],
      ["queue", "rabbitmq/fulfillment"],
      ["external", "api.stripe.test"],
      ["cache", null],
    ])
  })

  it("encodes traffic bands and errors deterministically", () => {
    const low = serviceMapEdgePresentation({ callCount: 2, errorCount: 1 })
    const medium = serviceMapEdgePresentation({ callCount: 10, errorCount: 0 })
    const high = serviceMapEdgePresentation({ callCount: 100, errorCount: 0 })
    expect([low, medium.band, high.band, low.width < high.width]).toEqual([
      {
        band: "low",
        className: "service-map-edge--traffic-low",
        hasError: true,
        width: expect.any(Number) as number,
      },
      "medium",
      "high",
      true,
    ])
  })
})
