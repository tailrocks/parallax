import { createFileRoute } from "@tanstack/react-router"

import { InvestigationsPage, loadInvestigationsList } from "@/features/investigations"

export const Route = createFileRoute("/investigations/")({
  loader: () => loadInvestigationsList().then((investigations) => ({ investigations })),
  component: InvestigationsRoute,
})

function InvestigationsRoute() {
  const { investigations } = Route.useLoaderData()
  return <InvestigationsPage investigations={investigations} />
}
