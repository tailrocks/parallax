import {
  encodeAlertGraduationSearch,
  type AlertGraduationSearch,
} from "@/features/alerts/model/alert-rule-form"
import type { LogsSearch } from "@/features/logs/model/logs-search"

/** Graduate the current logs query to a log_count alert, scoped to service. */
export function encodeLogsAlertGraduation(
  search: Pick<LogsSearch, "service">
): AlertGraduationSearch {
  return encodeAlertGraduationSearch({
    signalType: "log_count",
    service: search.service,
  })
}
