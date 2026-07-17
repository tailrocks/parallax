import { z } from "zod"

import { rangeSearchSchema } from "@/domain/time-range/range"

export type ServiceSort =
  | "name:asc"
  | "version:desc"
  | "version:asc"
  | "runtime:desc"
  | "runtime:asc"
  | "env:desc"
  | "env:asc"
  | "spans:desc"
  | "spans:asc"
  | "errors:desc"
  | "errors:asc"
  | "errorRate:desc"
  | "errorRate:asc"
  | "p95:desc"
  | "p95:asc"
  | "lastSeen:desc"
  | "lastSeen:asc"

export interface ServicesSearch {
  q?: string
  range?: string
  from?: string
  to?: string
  sort?: ServiceSort
}

export type ServicesSearchPatch = {
  [K in keyof ServicesSearch]?: ServicesSearch[K] | undefined
}

const serviceSorts = new Set<ServiceSort>([
  "name:asc",
  "version:desc",
  "version:asc",
  "runtime:desc",
  "runtime:asc",
  "env:desc",
  "env:asc",
  "spans:desc",
  "spans:asc",
  "errors:desc",
  "errors:asc",
  "errorRate:desc",
  "errorRate:asc",
  "p95:desc",
  "p95:asc",
  "lastSeen:desc",
  "lastSeen:asc",
])

const servicesSearchSchema = rangeSearchSchema.extend({
  q: z.unknown().optional(),
  sort: z.unknown().optional(),
})

export function validateServicesSearch(search: Record<string, unknown>): ServicesSearch {
  const parsed = servicesSearchSchema.parse(search)
  const result: ServicesSearch = {}
  if (typeof parsed.q === "string" && parsed.q) result.q = parsed.q
  if (parsed.range) result.range = parsed.range
  if (parsed.from) result.from = parsed.from
  if (parsed.to) result.to = parsed.to
  if (serviceSorts.has(parsed.sort as ServiceSort)) {
    result.sort = parsed.sort as ServiceSort
  }
  return result
}

export function patchServicesSearch(
  current: ServicesSearch,
  patch: ServicesSearchPatch
): ServicesSearch {
  const raw = { ...current, ...patch }
  const next: ServicesSearch = {}
  for (const key of Object.keys(raw) as Array<keyof ServicesSearch>) {
    const value = raw[key]
    if (value != null && value !== "") {
      Object.assign(next, { [key]: value })
    }
  }
  return next
}
