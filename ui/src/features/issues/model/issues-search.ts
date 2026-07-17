import { z } from "zod"

import { rangeSearchSchema } from "@/lib/range"

export type IssueSort = "LAST_SEEN" | "FIRST_SEEN" | "EVENTS" | "TREND"

export interface IssuesSearch {
  q?: string
  service?: string
  status?: "open" | "resolved"
  sort?: IssueSort
  range?: string
  from?: string
  to?: string
}

export type IssuesSearchPatch = {
  [K in keyof IssuesSearch]?: IssuesSearch[K] | undefined
}

const SORTS = ["LAST_SEEN", "FIRST_SEEN", "EVENTS", "TREND"] as const

const issuesSearchSchema = rangeSearchSchema.extend({
  q: z.unknown().optional(),
  service: z.unknown().optional(),
  status: z.unknown().optional(),
  sort: z.unknown().optional(),
})

export function validateIssuesSearch(
  search: Record<string, unknown>
): IssuesSearch {
  const parsed = issuesSearchSchema.parse(search)
  const result: IssuesSearch = {}
  if (typeof parsed.q === "string" && parsed.q) result.q = parsed.q
  if (typeof parsed.service === "string" && parsed.service) {
    result.service = parsed.service
  }
  if (parsed.status === "open" || parsed.status === "resolved") {
    result.status = parsed.status
  }
  if (SORTS.includes(parsed.sort as IssueSort)) {
    result.sort = parsed.sort as IssueSort
  }
  if (parsed.range) result.range = parsed.range
  if (parsed.from) result.from = parsed.from
  if (parsed.to) result.to = parsed.to
  return result
}

export function patchIssuesSearch(
  current: IssuesSearch,
  patch: IssuesSearchPatch
): IssuesSearch {
  const raw = { ...current, ...patch }
  const next: IssuesSearch = {}
  for (const key of Object.keys(raw) as Array<keyof IssuesSearch>) {
    const value = raw[key]
    if (value != null && value !== "") Object.assign(next, { [key]: value })
  }
  return next
}
