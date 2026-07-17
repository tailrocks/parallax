import { useEffect, useState } from "react"

import { loadRuntimeMetricStrip } from "@/features/runtime-metrics/api/load-runtime-metrics"
import type { StripPanel } from "@/features/runtime-metrics/api/runtime-metrics-mapper"
import { isRuntimeMetricsAbort } from "@/features/runtime-metrics/model/runtime-metrics-error"
import { usePageVisible } from "@/platform/visibility/use-page-visible"

export type UseRuntimeMetricsArgs = {
  readonly service?: string | undefined
  readonly invocationId?: string | undefined
  readonly fromNanos: string
  readonly toNanos: string
  readonly stepSeconds: number
  readonly live?: boolean
}

/** Fetch + live poll for MetricStrip. Preserves abort, visibility, 5s interval. */
export function useRuntimeMetrics({
  service,
  invocationId,
  fromNanos,
  toNanos,
  stepSeconds,
  live = false,
}: UseRuntimeMetricsArgs): StripPanel[] | null {
  const [panels, setPanels] = useState<StripPanel[] | null>(null)
  const pageVisible = usePageVisible()

  useEffect(() => {
    let cancelled = false
    let activeController: AbortController | null = null
    const fetchPanels = () => {
      activeController?.abort()
      const controller = new AbortController()
      activeController = controller
      const to = live ? ((BigInt(Date.now()) + 30_000n) * 1_000_000n).toString() : toNanos
      void loadRuntimeMetricStrip({
        service,
        invocationId,
        fromNanos,
        toNanos: to,
        stepSeconds,
        signal: controller.signal,
      })
        .then((next) => {
          if (cancelled || controller.signal.aborted) return
          setPanels(next)
        })
        .catch((error: unknown) => {
          if (cancelled || isRuntimeMetricsAbort(error)) return
          setPanels([])
        })
    }
    if (pageVisible) fetchPanels()
    const timer = live && pageVisible ? setInterval(fetchPanels, 5000) : undefined
    return () => {
      cancelled = true
      activeController?.abort()
      if (timer) clearInterval(timer)
    }
  }, [service, invocationId, fromNanos, toNanos, stepSeconds, live, pageVisible])

  return panels
}
