// Public facade for issues (Plan 139). Named exports only.

export { MiniSparkline } from "@/features/issues/components/issues-table"
export { IssuesContent, IssuesPage } from "@/features/issues/components/issues-page"
export {
  IssueDetailContent,
  IssueDetailRoutePage,
} from "@/features/issues/components/issue-detail-page"
export {
  loadIssueDetail,
  loadIssueOccurrences,
  loadIssues,
  setIssueStatus,
} from "@/features/issues/api/issues-api"
export { topTags, trendEvents } from "@/features/issues/model/issue-summary"
export type {
  IssueRow,
  IssueSummary,
  IssuesData,
  TrendPoint,
} from "@/features/issues/model/issue-summary"
export { patchIssuesSearch, validateIssuesSearch } from "@/features/issues/model/issues-search"
export type {
  IssueSort,
  IssuesSearch,
  IssuesSearchPatch,
} from "@/features/issues/model/issues-search"
export type {
  IssueAttributeEntry,
  IssueCorrelation,
  IssueCorrelationLog,
  IssueCorrelationResult,
  IssueDetail,
  IssueDetailData,
  IssueEvent,
} from "@/features/issues/model/issue-detail"
export {
  parseIssueAttributes,
  type ParsedIssueAttributes,
} from "@/features/issues/model/issue-detail"
export { parseStacktrace, structuredFrameCount } from "@/features/issues/model/stacktrace"
export type { Frame } from "@/features/issues/model/stacktrace"
export { IssuesError } from "@/features/issues/model/issues-error"
export type { ErrorEvent, Issue } from "@/features/issues/model/issue-wire"
