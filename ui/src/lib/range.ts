// Compatibility reexport — domain owner is `@/domain/time-range/range` (Plan 100).
// Plan 149 consumes the domain facade; Plan 151 deletes this path when callers move.
export {
  RANGE_PRESETS,
  DEFAULT_RANGE_KEY,
  rangeSearchSchema,
  type ResolvedRange,
  resolvePreset,
  customRange,
  resolveRangeSearch,
  updateRangeSearch,
  rangeLinkSearch,
  mergeRangeSearch,
  formatRangeLabel,
} from "@/domain/time-range/range"
