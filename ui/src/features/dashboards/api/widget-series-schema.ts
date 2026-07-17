// Plan 152 — strict per-series Zod decoder for dynamic dashboard batches.

import { z } from "zod"

/** One `metricSeries` alias value: matches SDL nullability exactly. */
export const widgetSeriesSchema = z.object({
  groupValue: z.string().nullable(),
  points: z.array(
    z.object({
      tsNanos: z.string(),
      value: z.number(),
    })
  ),
})

export type WidgetSeries = z.infer<typeof widgetSeriesSchema>

export const widgetSeriesListSchema = z.array(widgetSeriesSchema)
