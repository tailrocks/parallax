import { createFileRoute, notFound } from "@tanstack/react-router"

import { navItem } from "@/shared/navigation"
import {
  InvestigationDetailPage,
  loadInvestigationDetail,
} from "@/features/investigations"

export const Route = createFileRoute("/investigations/$investigationId")({
  loader: async ({ params }) => {
    const investigation = await loadInvestigationDetail(params.investigationId)
    if (!investigation) throw notFound()
    return { investigation }
  },
  component: InvestigationDetailRoute,
})

function InvestigationDetailRoute() {
  const { investigation } = Route.useLoaderData()
  const back = navItem("/investigations")
  return (
    <InvestigationDetailPage
      investigation={investigation}
      {...(back ? { back } : {})}
    />
  )
}
