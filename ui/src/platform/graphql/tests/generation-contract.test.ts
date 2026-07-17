import { describe, expect, it } from "vitest"

import {
  GraphqlContractStaticProbeDocument,
  GraphqlContractStaticProbeQuerySchema,
} from "@/platform/graphql/tests/fixtures/static-probe.generated"

// Import base generated types so the module graph stays reachable under Oxc.
import type { Scalars } from "@/platform/graphql/generated/schema-types.generated"

describe("GraphQL generation contract", () => {
  it("probe document is a named TypedDocumentNode with Zod result schema", () => {
    expect(GraphqlContractStaticProbeDocument.kind).toBe("Document")
    const definition = GraphqlContractStaticProbeDocument.definitions[0]
    expect(definition?.kind).toBe("OperationDefinition")
    expect(definition && "name" in definition ? definition.name?.value : null).toBe(
      "GraphqlContractStaticProbe"
    )
    const valid = GraphqlContractStaticProbeQuerySchema.safeParse({
      health: "ok",
      version: "1",
      otlpGrpcPort: 1,
      otlpHttpPort: 2,
      overview: { spanCount: "0", errorRate: 0, activeServices: 0 },
      issue: null,
      metricNames: [],
      signalCountSeries: [],
    })
    expect(valid.success).toBe(true)
    const invalid = GraphqlContractStaticProbeQuerySchema.safeParse({
      health: 1,
    })
    expect(invalid.success).toBe(false)
    // Scalars mapping is present on the base generated file.
    const _id: Scalars["ID"]["output"] = "x"
    expect(_id).toBe("x")
  })

  it("generated modules export the probe operation schema", () => {
    expect(typeof GraphqlContractStaticProbeQuerySchema.safeParse).toBe("function")
  })
})
