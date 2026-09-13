// Public facade for rum. Named exports only.

export { RumContent, RumPage } from "@/features/rum/components/rum-page"
export { loadRum, loadRumTrace, loadRumVital } from "@/features/rum/api/rum-api"
export {
  formatVitalValue,
  matchVital,
  normalizeVitalValue,
  rateVital,
  ratingLabel,
  VITAL_SPECS,
} from "@/features/rum/model/rum-vitals"
export type { VitalId, VitalRating, VitalSpec } from "@/features/rum/model/rum-vitals"
export type {
  RumData,
  RumExemplar,
  RumIssueRow,
  RumLogRow,
  RumPoint,
  RumTraceData,
  RumTraceRow,
  RumVitalData,
  RumVitalRow,
} from "@/features/rum/model/rum-overview"
export { patchRumSearch, validateRumSearch } from "@/features/rum/model/rum-search"
export type { RumSearch, RumSearchPatch } from "@/features/rum/model/rum-search"
export { RumError } from "@/features/rum/model/rum-error"
