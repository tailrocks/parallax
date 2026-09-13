import { z } from "zod"

import { rangeSearchSchema } from "@/domain/time-range/range"

export interface RumSearch {
  service?: string
  vital?: string
  traceId?: string
  sessionId?: string
  where?: string
  range?: string
  from?: string
  to?: string
}

export type RumSearchPatch = {
  [K in keyof RumSearch]?: RumSearch[K] | undefined
}

const rumSearchSchema = rangeSearchSchema.extend({
  service: z.unknown().optional(),
  vital: z.unknown().optional(),
  traceId: z.unknown().optional(),
  sessionId: z.unknown().optional(),
  where: z.unknown().optional(),
})

function searchString(value: unknown): string | undefined {
  return typeof value === "string" && value ? value : undefined
}

export function validateRumSearch(search: Record<string, unknown>): RumSearch {
  const parsed = rumSearchSchema.parse(search)
  const result: RumSearch = {}
  const service = searchString(parsed.service)
  const vital = searchString(parsed.vital)
  const traceId = searchString(parsed.traceId)
  const sessionId = searchString(parsed.sessionId)
  const where = searchString(parsed.where)
  if (service) result.service = service
  if (vital) result.vital = vital
  if (traceId) result.traceId = traceId
  if (sessionId) result.sessionId = sessionId
  if (where) result.where = where
  if (parsed.range) result.range = parsed.range
  if (parsed.from) result.from = parsed.from
  if (parsed.to) result.to = parsed.to
  return result
}

export function patchRumSearch(current: RumSearch, patch: RumSearchPatch): RumSearch {
  const raw = { ...current, ...patch }
  const next: RumSearch = {}
  for (const key of Object.keys(raw) as Array<keyof RumSearch>) {
    const value = raw[key]
    if (value != null && value !== "") Object.assign(next, { [key]: value })
  }
  return next
}
