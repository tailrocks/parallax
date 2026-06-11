import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/dashboards")({ component: Dashboards })

/** Service overview + user-defined dashboards land with the metricSeries
 *  API slice (spec §8/§9); this page is the placeholder shell until then. */
function Dashboards() {
  return (
    <div className="space-y-2">
      <h1 className="text-lg font-semibold">Dashboards</h1>
      <p className="text-sm text-muted-foreground">
        Service overview (CPU, memory, request rate/latency) and custom
        dashboards built from your metrics arrive with the metric-series API.
      </p>
    </div>
  )
}
