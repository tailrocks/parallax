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
export type { LogDoc, OptionalLogColumn } from "@/features/logs/components/logs-table"
export { contextWindow, stepSecondsForRange } from "@/features/logs/model/logs-range"
export {
  parseSavedViewState,
  serializeLogsSearch,
  validateLogsSearch,
} from "@/features/logs/model/logs-search"
export type { LogsSearch } from "@/features/logs/model/logs-search"
export { LogsError } from "@/features/logs/model/logs-error"

export { LOG_FIELDS, type LogRecord } from "@/features/logs/model/wire"
export {
  DEFAULT_HISTOGRAM_BUCKETS,
  buildUniformBuckets,
  pxToTime,
  snapBrushToBuckets,
  timeToPx,
  type HistogramBucket,
} from "@/features/logs/model/log-histogram-brush"
export {
  bodyMatchesTemplate,
  filterBodiesByTemplate,
  templateStableFragment,
  templateToRegExp,
} from "@/features/logs/model/log-pattern-match"
export {
  DEFAULT_LOG_PATTERNS_URL,
  decodeLogPatternsUrl,
  encodeLogPatternsUrl,
  encodePatternsFlag,
  mergeLogPatternsParams,
  parsePatternsFlag,
} from "@/features/logs/model/log-patterns-url"
export {
  DEFAULT_LOG_DENSITY,
  decodePinnedColumns,
  encodeLogWrap,
  encodePinnedColumns,
  logDensityClass,
  parseLogDensity,
  parseLogWrap,
  pinColumn,
  togglePinnedColumn,
  unpinColumn,
} from "@/features/logs/model/log-table-prefs"
