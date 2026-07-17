import type { FocusMode, TrafficPreset } from "@/features/ecosystem/model/ecosystem-topology"
import { rangeSearchSchema } from "@/domain/time-range/range"

export type EcosystemSearch = {
  range?: string | undefined
  from?: string | undefined
  to?: string | undefined
  focus?: string | undefined
  hops?: 1 | 2 | undefined
  focusMode?: FocusMode | undefined
  minTraffic?: TrafficPreset | undefined
}

const TRAFFIC_VALUES = new Set<TrafficPreset>(["all", "0.1%", "1%", "5%"])

export function validateEcosystemSearch(search: Record<string, unknown>): EcosystemSearch {
  const parsed = rangeSearchSchema.parse(search)
  const hops = Number(search["hops"])
  const minTraffic = search["minTraffic"]
  return {
    range: typeof parsed.range === "string" ? parsed.range : undefined,
    from: typeof parsed.from === "string" ? parsed.from : undefined,
    to: typeof parsed.to === "string" ? parsed.to : undefined,
    focus:
      typeof search["focus"] === "string" && search["focus"].length > 0
        ? search["focus"]
        : undefined,
    hops: hops === 2 ? 2 : undefined,
    focusMode: search["focusMode"] === "hide" ? "hide" : undefined,
    minTraffic:
      typeof minTraffic === "string" &&
      TRAFFIC_VALUES.has(minTraffic as TrafficPreset) &&
      minTraffic !== "all"
        ? (minTraffic as TrafficPreset)
        : undefined,
  }
}
