import { describe, expect, it } from "vitest"

import type { InvocationRow } from "@/features/invocations/model/invocation"
import {
  INVOCATION_FACET_VALUES_CAP,
  invocationFacetCounts,
} from "@/features/invocations/model/invocation-facets"

const NOW_MS = 1_700_000_000_000

function row(overrides: Partial<InvocationRow> = {}): InvocationRow {
  return {
    invocationId: "inv-1",
    source: "cli",
    command: "deploy",
    appMode: "one_shot",
    outcome: "success",
    service: "cli",
    registeredStatus: null,
    exitCode: 0,
    startedAtNanos: `${BigInt(NOW_MS) * 1_000_000n}`,
    endedAtNanos: `${BigInt(NOW_MS) * 1_000_000n + 1_000n}`,
    lastNanos: `${BigInt(NOW_MS) * 1_000_000n + 1_000n}`,
    errorCount: 0,
    traceCount: 1,
    sessionCount: 0,
    spanCount: 1,
    logCount: 0,
    ...overrides,
  }
}

describe("invocationFacetCounts (plan 164)", () => {
  it("counts values per dimension, count-desc then value-asc", () => {
    const rows = [
      row(),
      row({ invocationId: "inv-2" }),
      row({ invocationId: "inv-3", appMode: "daemon", command: "serve" }),
    ]
    const facets = invocationFacetCounts(rows, NOW_MS)
    const facet = (dimension: string) => facets.find((f) => f.dimension === dimension)?.values
    expect(facet("mode")).toEqual([
      { value: "one_shot", count: 2 },
      { value: "daemon", count: 1 },
    ])
    expect(facet("command")).toEqual([
      { value: "deploy", count: 2 },
      { value: "serve", count: 1 },
    ])
  })

  it("derives status through invocationStatus (failed exit codes)", () => {
    const rows = [row(), row({ invocationId: "inv-2", exitCode: 1 })]
    const facets = invocationFacetCounts(rows, NOW_MS)
    expect(facets.find((f) => f.dimension === "status")?.values).toEqual([
      { value: "failed", count: 1 },
      { value: "finished", count: 1 },
    ])
  })

  it("skips missing values and caps the list", () => {
    const rows = [
      row({ appMode: null, outcome: null }),
      ...Array.from({ length: 30 }, (_, index) =>
        row({ invocationId: `inv-${index}`, command: `cmd-${index}` })
      ),
    ]
    const facets = invocationFacetCounts(rows, NOW_MS)
    const modes = facets.find((f) => f.dimension === "mode")?.values ?? []
    expect(modes.reduce((sum, entry) => sum + entry.count, 0)).toBe(30)
    const commands = facets.find((f) => f.dimension === "command")?.values
    expect(commands).toHaveLength(INVOCATION_FACET_VALUES_CAP)
  })
})
