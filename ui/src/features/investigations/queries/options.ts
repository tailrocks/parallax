import { queryOptions } from "@tanstack/react-query"

import {
  loadInvestigationDetail,
  loadInvestigationPinOptions,
  loadInvestigationsList,
} from "@/features/investigations/api/investigation-api"
import { investigationKeys } from "@/features/investigations/queries/keys"

export function investigationsListQueryOptions() {
  return queryOptions({
    queryKey: investigationKeys.list(),
    queryFn: () => loadInvestigationsList(),
  })
}

export function investigationDetailQueryOptions(id: string) {
  return queryOptions({
    queryKey: investigationKeys.detail(id),
    queryFn: () => loadInvestigationDetail(id),
  })
}

export function investigationPinOptionsQueryOptions() {
  return queryOptions({
    queryKey: investigationKeys.pinOptions(),
    queryFn: () => loadInvestigationPinOptions(),
  })
}
