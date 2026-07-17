import { z } from "zod"

import { rangeSearchSchema } from "@/domain/time-range/range"

export type TestExplorerSort = "LAST_SEEN" | "NAME"
export type TestRollupFilter = "PASSED" | "FLAKY_PASS" | "FAILED" | "BROKEN" | "SKIPPED" | "UNKNOWN"
export type TestFlakyFilter = "HEALTHY" | "FLAKY" | "FIXED" | "BROKEN"

export interface TestsSearch {
  q?: string
  suite?: string
  service?: string
  serviceVersion?: string
  status?: TestRollupFilter
  flakyState?: TestFlakyFilter
  sort?: TestExplorerSort
  range?: string
  from?: string
  to?: string
}

export type TestsSearchPatch = {
  [K in keyof TestsSearch]?: TestsSearch[K] | undefined
}

const SORTS = ["LAST_SEEN", "NAME"] as const
const ROLLUPS = ["PASSED", "FLAKY_PASS", "FAILED", "BROKEN", "SKIPPED", "UNKNOWN"] as const
const FLAKY = ["HEALTHY", "FLAKY", "FIXED", "BROKEN"] as const

const testsSearchSchema = rangeSearchSchema.extend({
  q: z.unknown().optional(),
  suite: z.unknown().optional(),
  service: z.unknown().optional(),
  serviceVersion: z.unknown().optional(),
  status: z.unknown().optional(),
  flakyState: z.unknown().optional(),
  sort: z.unknown().optional(),
})

export function validateTestsSearch(search: Record<string, unknown>): TestsSearch {
  const parsed = testsSearchSchema.parse(search)
  const result: TestsSearch = {}
  if (typeof parsed.q === "string" && parsed.q) result.q = parsed.q
  if (typeof parsed.suite === "string" && parsed.suite) result.suite = parsed.suite
  if (typeof parsed.service === "string" && parsed.service) result.service = parsed.service
  if (typeof parsed.serviceVersion === "string" && parsed.serviceVersion) {
    result.serviceVersion = parsed.serviceVersion
  }
  if (ROLLUPS.includes(parsed.status as TestRollupFilter)) {
    result.status = parsed.status as TestRollupFilter
  }
  if (FLAKY.includes(parsed.flakyState as TestFlakyFilter)) {
    result.flakyState = parsed.flakyState as TestFlakyFilter
  }
  if (SORTS.includes(parsed.sort as TestExplorerSort)) {
    result.sort = parsed.sort as TestExplorerSort
  }
  if (parsed.range) result.range = parsed.range
  if (parsed.from) result.from = parsed.from
  if (parsed.to) result.to = parsed.to
  return result
}

export function patchTestsSearch(current: TestsSearch, patch: TestsSearchPatch): TestsSearch {
  const raw = { ...current, ...patch }
  const next: TestsSearch = {}
  for (const key of Object.keys(raw) as Array<keyof TestsSearch>) {
    const value = raw[key]
    if (value != null && value !== "") Object.assign(next, { [key]: value })
  }
  return next
}
