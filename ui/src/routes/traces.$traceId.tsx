import { createFileRoute } from "@tanstack/react-router"

import {
  TraceDetailPage,
  loadTraceDetail,
  validateTraceDetailSearch,
} from "@/features/traces"

export const Route = createFileRoute("/traces/$traceId")({
  validateSearch: validateTraceDetailSearch,
  loader: ({ params }) => loadTraceDetail(params.traceId),
  component: TraceDetailRoute,
})

function TraceDetailRoute() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  const { traceId } = Route.useParams()
  return <TraceDetailPage data={data} search={search} traceId={traceId} />
}
