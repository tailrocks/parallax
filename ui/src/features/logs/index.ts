// Public facade for logs (Plan 141). Named exports only.

export {
  ColumnMenu,
  LogsPage,
  SavedViewsMenu,
  loadLogs,
} from "@/features/logs/components/logs-page"
export {
  LogsTable,
  OPTIONAL_LOG_COLUMNS,
  parseLogColumns,
  serializeLogColumns,
  severityVariant,
} from "@/features/logs/components/logs-table"
export type {
  LogDoc,
  OptionalLogColumn,
} from "@/features/logs/components/logs-table"
export {
  contextWindow,
  stepSecondsForRange,
} from "@/features/logs/model/logs-range"
export {
  parseSavedViewState,
  serializeLogsSearch,
  validateLogsSearch,
} from "@/features/logs/model/logs-search"
export type { LogsSearch } from "@/features/logs/model/logs-search"
export { LogsError } from "@/features/logs/model/logs-error"
