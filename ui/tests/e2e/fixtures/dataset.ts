/** Deterministic dataset identity for foundation fixtures (no product seed yet). */

export type DatasetId = `foundation-${string}`

export function foundationDatasetId(seed = "default"): DatasetId {
  return `foundation-${seed}`
}
