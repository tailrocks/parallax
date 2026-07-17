// Public facade for issues (Plan 139). Named exports only.

export { IssuesContent, IssuesPage, MiniSparkline } from "@/features/issues/components/issues-page"
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
  BreadcrumbLog,
  IssueDetail,
  IssueDetailData,
  IssueEvent,
} from "@/features/issues/model/issue-detail"
export { parseStacktrace, structuredFrameCount } from "@/features/issues/model/stacktrace"
export type { Frame } from "@/features/issues/model/stacktrace"
export { IssuesError } from "@/features/issues/model/issues-error"
export type { ErrorEvent, Issue } from "@/features/issues/model/issue-wire"
