/** Stable dataset IDs shared with the Rust browser seed facade. */

export type ProductDatasetId =
  | "shell-empty"
  | "investigations-pilot"
  | "logs-pilot"
  | "traces-pilot"
  | "dashboards-pilot"
  | "sql-pilot"
  | "alerts-pilot"
  | "metrics-pilot"

export const LOGS_PILOT_BODY = "checkout authorize failed"
export const TRACES_PILOT_TRACE_ID = "cccccccccccccccccccccccccccccccc"
export const TRACES_PILOT_ROOT_NAME = "checkout.authorize"
export const DASHBOARD_PILOT_NAME = "Checkout RED"
export const ALERT_RULE_PILOT_NAME = "High checkout errors"

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
  "logs-pilot": {
    id: "logs-pilot",
    owner: "features/logs",
    description: "Six log rows across checkout/billing and INFO/WARN/ERROR",
  },
  "traces-pilot": {
    id: "traces-pilot",
    owner: "features/traces",
    description: "One named trace with children and one error span",
  },
  "dashboards-pilot": {
    id: "dashboards-pilot",
    owner: "features/dashboards",
    description: "One dashboard with one widget",
  },
  "sql-pilot": {
    id: "sql-pilot",
    owner: "features/sql",
    description: "Minimal logs for SELECT count(*)",
  },
  "alerts-pilot": {
    id: "alerts-pilot",
    owner: "features/overview",
    description: "One alert rule, destination, and resolved incident",
  },
  "metrics-pilot": {
    id: "metrics-pilot",
    owner: "features/overview",
    description: "Gauge plus histogram with known series",
  },
} as const satisfies Record<
  ProductDatasetId,
  { id: ProductDatasetId; owner: string; description: string }
>
