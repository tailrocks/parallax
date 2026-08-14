import { Bar, BarChart, CartesianGrid, XAxis, YAxis } from "recharts"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import type { TrendPoint } from "@/features/issues/model/issue-summary"
import { formatTimeInRange } from "@/shared/format"

const trendConfig = {
  count: { label: "events", color: "var(--destructive)" },
} satisfies ChartConfig

export function TrendChart({
  trend,
  onBucket,
  activeBucket,
}: {
  trend: readonly TrendPoint[]
  onBucket: (tsNanos: string | null) => void
  activeBucket: string | null
}) {
  if (trend.length === 0) return null
  const data = trend.map((point) => ({
    ...point,
    time: formatTimeInRange(point.tsNanos, {
      fromNanos: trend[0]?.tsNanos ?? point.tsNanos,
      toNanos: trend.at(-1)?.tsNanos ?? point.tsNanos,
    }),
  }))
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Occurrence trend</CardTitle>
      </CardHeader>
      <CardContent>
        <ChartContainer config={trendConfig} className="h-[180px] w-full">
          <BarChart
            data={data}
            margin={{ left: 8, right: 8, top: 8 }}
            onClick={(state) => {
              const payloadState = state as {
                activePayload?: Array<{ payload?: { tsNanos?: unknown } }>
              }
              const ts = payloadState.activePayload?.[0]?.payload?.tsNanos as string | undefined
              if (ts) void onBucket(ts === activeBucket ? null : ts)
            }}
          >
            <CartesianGrid vertical={false} />
            <XAxis dataKey="time" tickLine={false} axisLine={false} minTickGap={48} />
            <YAxis tickLine={false} axisLine={false} width={40} />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Bar dataKey="count" fill="var(--color-count)" radius={3} />
          </BarChart>
        </ChartContainer>
      </CardContent>
    </Card>
  )
}
