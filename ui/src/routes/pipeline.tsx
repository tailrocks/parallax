import { createFileRoute } from "@tanstack/react-router"

import { PipelinePage, loadPipelineStatus } from "@/features/pipeline"

export const Route = createFileRoute("/pipeline")({
  loader: () => loadPipelineStatus().then((status) => ({ status })),
  component: PipelineRoute,
})

function PipelineRoute() {
  const { status } = Route.useLoaderData()
  return <PipelinePage status={status} />
}
