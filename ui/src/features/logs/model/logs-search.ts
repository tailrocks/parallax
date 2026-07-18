import { z } from "zod"

export interface LogsSearch {
  q?: string | undefined
  service?: string | undefined
  sev?: number | undefined
  where?: string | undefined
  range?: string | undefined
  from?: string | undefined
  to?: string | undefined
  live?: boolean | undefined
  cols?: string | undefined
  patterns?: boolean | undefined
  anchor?: string | undefined
}

export type LogsSearchPatch = {
  [K in keyof LogsSearch]?: LogsSearch[K] | undefined
}

const logsSearchSchema = z.object({
  q: z.unknown().optional(),
  service: z.unknown().optional(),
  sev: z.unknown().optional(),
  where: z.unknown().optional(),
  range: z.unknown().optional(),
  from: z.unknown().optional(),
  to: z.unknown().optional(),
  live: z.unknown().optional(),
  cols: z.unknown().optional(),
  patterns: z.unknown().optional(),
  anchor: z.unknown().optional(),
})

export function validateLogsSearch(search: Record<string, unknown>): LogsSearch {
  const parsed = logsSearchSchema.parse(search)
  const severity =
    typeof parsed.sev === "number" || typeof parsed.sev === "string"
      ? Number(parsed.sev)
      : Number.NaN
  return {
    q: typeof parsed.q === "string" && parsed.q ? parsed.q : undefined,
    service: typeof parsed.service === "string" && parsed.service ? parsed.service : undefined,
    sev: Number.isFinite(severity) && severity > 0 ? severity : undefined,
    where: typeof parsed.where === "string" && parsed.where ? parsed.where : undefined,
    range: typeof parsed.range === "string" ? parsed.range : undefined,
    from: typeof parsed.from === "string" ? parsed.from : undefined,
    to: typeof parsed.to === "string" ? parsed.to : undefined,
    live: parsed.live === "1" || parsed.live === true,
    cols: typeof parsed.cols === "string" ? parsed.cols : undefined,
    patterns:
      parsed.patterns === true || parsed.patterns === "1" || parsed.patterns === "true"
        ? true
        : undefined,
    anchor:
      typeof parsed.anchor === "string" && /^\d+$/.test(parsed.anchor) ? parsed.anchor : undefined,
  }
}

export function parseSavedViewState(state: string): LogsSearch {
  const params = new URLSearchParams(state.startsWith("?") ? state.slice(1) : state)
  const raw: Record<string, unknown> = {}
  params.forEach((value, key) => {
    raw[key] = value
  })
  return validateLogsSearch(raw)
}

export function serializeLogsSearch(search: LogsSearch): string {
  const params = new URLSearchParams()
  if (search.q) params.set("q", search.q)
  if (search.service) params.set("service", search.service)
  if (search.sev) params.set("sev", String(search.sev))
  if (search.where) params.set("where", search.where)
  if (search.range) params.set("range", search.range)
  if (search.from) params.set("from", search.from)
  if (search.to) params.set("to", search.to)
  if (search.patterns) params.set("patterns", "1")
  if (search.live) params.set("live", "1")
  if (search.cols) params.set("cols", search.cols)
  if (search.anchor) params.set("anchor", search.anchor)
  const value = params.toString()
  return value ? `?${value}` : ""
}

export function patchLogsSearch(current: LogsSearch, patch: LogsSearchPatch): LogsSearch {
  return { ...current, ...patch }
}
