/** Stable dataset IDs shared with the Rust browser seed facade. */

export type ProductDatasetId = "shell-empty" | "investigations-pilot"

export const DATASET_CATALOG = {
  "shell-empty": {
    id: "shell-empty",
    owner: "layout/shell",
    description: "Empty shell: no telemetry, no investigations",
  },
  "investigations-pilot": {
    id: "investigations-pilot",
    owner: "features/investigations",
    description: "One seeded investigation with pin + note",
  },
} as const satisfies Record<
  ProductDatasetId,
  { id: ProductDatasetId; owner: string; description: string }
>
