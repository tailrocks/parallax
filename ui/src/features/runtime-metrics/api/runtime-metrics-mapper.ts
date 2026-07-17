// Map decoded RuntimeMetricStrip operation data → presentation panels.

import type { RuntimeMetricStripQuery } from "@/features/runtime-metrics/api/runtime-metrics.generated"

export type StripMetricPoint = {
  readonly tsNanos: string
  readonly value: number
}

export type StripPanel = {
  readonly title: string
  readonly unit: string
  readonly key: "cpu" | "memory" | "tasks"
  readonly points: readonly StripMetricPoint[]
}

export function mapRuntimeMetricStrip(data: RuntimeMetricStripQuery): StripPanel[] {
  return [
    {
      title: "CPU",
      unit: "%",
      key: "cpu",
      points: (data.cpu[0]?.points ?? []).map((point) => ({
        tsNanos: point.tsNanos,
        value: point.value * 100,
      })),
    },
    {
      title: "Memory",
      unit: "bytes",
      key: "memory",
      points: data.memory[0]?.points ?? [],
    },
    {
      title: "Tokio alive tasks",
      unit: "",
      key: "tasks",
      points: data.tasks[0]?.points ?? [],
    },
  ]
}
