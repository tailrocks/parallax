import { describe, expect, it } from "vitest"

import { metricCatalogDetailSearch } from "@/routes/-metrics-table"

describe("metricCatalogDetailSearch", () => {
  it("carries kind and the first service into metric detail search", () => {
    expect(
      metricCatalogDetailSearch({
        name: "http.server.duration",
        kind: "histogram",
        services: ["checkout", "catalog"],
      })
    ).toEqual({ kind: "histogram", service: "checkout" })
  })

  it("omits service when the catalog row has none", () => {
    expect(
      metricCatalogDetailSearch({
        name: "process.cpu.utilization",
        kind: "gauge",
        services: null,
      })
    ).toEqual({ kind: "gauge", service: undefined })
  })
})
