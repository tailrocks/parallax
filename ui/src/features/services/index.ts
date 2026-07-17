// Public facade for services (Plan 138). Named exports only.

export {
  ServicesIndexContent,
  ServicesPage,
  ServicesRouteShell,
} from "@/features/services/components/services-page"
export {
  ServiceDetailContent,
  ServiceDetailRoutePage,
} from "@/features/services/components/service-detail-page"
export {
  loadServiceDetail,
  loadServices,
} from "@/features/services/api/services-api"
export {
  serviceErrorRate,
  serviceHref,
  servicesWithCatalog,
  sortedServices,
} from "@/features/services/model/service-summary"
export type {
  ServiceCatalogRow,
  ServiceSummary,
  ServiceTableRow,
  ServicesData,
} from "@/features/services/model/service-summary"
export {
  patchServicesSearch,
  validateServicesSearch,
} from "@/features/services/model/services-search"
export type {
  ServiceSort,
  ServicesSearch,
  ServicesSearchPatch,
} from "@/features/services/model/services-search"
export {
  exemplarMarkers,
  latencyBands,
  latestErrorRate,
  stepSecondsForRange,
  totalSeries,
} from "@/features/services/model/service-detail"
export type {
  MetricExemplar,
  ReleaseWindow,
  ServiceDetailData,
  SpanRed,
} from "@/features/services/model/service-detail"
export { ServicesError } from "@/features/services/model/services-error"
