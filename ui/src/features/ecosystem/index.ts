// Public facade for the ecosystem service map (Plan 136). Named exports only.

export { EcosystemPage } from "@/features/ecosystem/components/ecosystem-page"
export { EcosystemGraph } from "@/features/ecosystem/components/ecosystem-graph"
export { loadServiceMap } from "@/features/ecosystem/api/service-map-api"
export { validateEcosystemSearch } from "@/features/ecosystem/model/ecosystem-search"
export type { EcosystemSearch } from "@/features/ecosystem/model/ecosystem-search"
export type {
  ServiceMap,
  ServiceMapEdge,
  ServiceMapNode,
} from "@/features/ecosystem/model/service-map"
