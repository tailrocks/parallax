import { describe, expect, it } from "vitest"

import { targetForCell } from "@/features/sql/model/sql-cell-target"
import { SQL_EXAMPLES } from "@/features/sql/model/sql-examples"

describe("SQL result helpers", () => {
  it("keeps SQL examples on real table names", () => {
    const banned = /\botel_spans\b|\botel_logs\b|\botel_metrics_points\b/
    for (const example of SQL_EXAMPLES) {
      expect(example.sql).not.toMatch(banned)
    }
  })

  it("maps supported id columns to route targets", () => {
    expect(targetForCell("trace_id", "trace-a", {})).toEqual({
      to: "/traces/$traceId",
      params: { traceId: "trace-a" },
    })
    expect(targetForCell("span_id", "span-a", { trace_id: "trace-a" })).toEqual({
      to: "/traces/$traceId",
      params: { traceId: "trace-a" },
    })
    expect(targetForCell('"cli.invocation.id"', "run-a", {})).toEqual({
      to: "/invocations/$invocationId",
      params: { invocationId: "run-a" },
    })
    expect(targetForCell("invocation_id", "run-b", {})).toEqual({
      to: "/invocations/$invocationId",
      params: { invocationId: "run-b" },
    })
    expect(targetForCell("span_id", "span-a", {})).toBeNull()
    expect(targetForCell("trace_id", "null", {})).toBeNull()
  })
})
