/* @vitest-environment jsdom */

import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"

import type { RuntimeMetric } from "@/domain/runtime-metrics/runtime-metric"
import { RuntimeSnapshotCard } from "@/features/runtime-metrics"

const metrics: RuntimeMetric[] = [
  {
    family: "process",
    metric: "process.memory.usage",
    unit: "bytes",
    points: [
      { tsNanos: "1000000000", value: 2 * 1024 * 1024 },
      { tsNanos: "2000000000", value: 4 * 1024 * 1024 },
    ],
  },
  {
    family: "tokio",
    metric: "tokio.runtime.alive_tasks",
    unit: null,
    points: [{ tsNanos: "1000000000", value: 3 }],
  },
  {
    family: "empty",
    metric: "empty.series",
    unit: null,
    points: [],
  },
]

describe("RuntimeSnapshotCard", () => {
  it("groups non-empty families and converts byte units", () => {
    render(<RuntimeSnapshotCard metrics={metrics} />)
    expect(screen.getByText("Runtime")).toBeTruthy()
    expect(screen.getByText("process")).toBeTruthy()
    expect(screen.getByText("tokio")).toBeTruthy()
    expect(screen.queryByText("empty")).toBeNull()
    expect(screen.getByText("MiB")).toBeTruthy()
    expect(screen.getByText("process memory usage")).toBeTruthy()
    expect(screen.getByText("tokio alive_tasks")).toBeTruthy()
  })

  it("renders nothing when every series is empty", () => {
    const { container } = render(
      <RuntimeSnapshotCard
        metrics={[
          {
            family: "process",
            metric: "process.cpu.utilization",
            unit: "ratio",
            points: [],
          },
        ]}
      />
    )
    expect(container.firstChild).toBeNull()
  })
})
