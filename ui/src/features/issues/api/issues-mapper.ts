import type { IssueDetailQuery } from "@/features/issues/api/issue-detail.generated"
import type { IssuesListQuery } from "@/features/issues/api/issues-list.generated"
import type {
  BreadcrumbLog,
  IssueDetail,
  IssueDetailData,
  IssueEvent,
} from "@/features/issues/model/issue-detail"
import type { IssueRow, IssuesData, TrendPoint } from "@/features/issues/model/issue-summary"

function mapTrend(
  points: ReadonlyArray<{ readonly tsNanos: string; readonly count: number }>
): TrendPoint[] {
  return points.map((point) => ({
    tsNanos: point.tsNanos,
    count: point.count,
  }))
}

function mapIssueRow(row: IssuesListQuery["issues"]["items"][number]): IssueRow {
  return {
    fingerprint: row.fingerprint,
    title: row.title,
    errorType: row.errorType,
    culprit: row.culprit,
    service: row.service,
    status: row.status,
    firstSeenNanos: row.firstSeenNanos,
    lastSeenNanos: row.lastSeenNanos,
    eventCount: row.eventCount,
    lastTraceId: row.lastTraceId,
    tags: row.tags,
    trend: mapTrend(row.trend),
  }
}

export function mapIssueEvents(
  events: ReadonlyArray<{
    readonly tsNanos: string
    readonly service: string
    readonly message: string
    readonly stacktrace: string | null
    readonly source: string
    readonly traceId: string
    readonly spanId: string
    readonly attributes: string
  }>
): IssueEvent[] {
  return events.map((event) => ({
    tsNanos: event.tsNanos,
    service: event.service,
    message: event.message,
    stacktrace: event.stacktrace,
    source: event.source,
    traceId: event.traceId,
    spanId: event.spanId,
    attributes: event.attributes,
  }))
}

export function mapIssuesList(data: IssuesListQuery): IssuesData {
  return {
    issues: {
      total: data.issues.total,
      items: data.issues.items.map(mapIssueRow),
    },
    services: [...data.services],
  }
}

export function mapIssueDetail(
  data: IssueDetailQuery,
  extras: {
    resource: Record<string, unknown>
    breadcrumbs: readonly BreadcrumbLog[]
    traceRunId: string | null
    releaseVersion: string | null
  }
): IssueDetailData {
  const issue: IssueDetail | null = data.issue
    ? {
        fingerprint: data.issue.fingerprint,
        title: data.issue.title,
        errorType: data.issue.errorType,
        culprit: data.issue.culprit,
        service: data.issue.service,
        status: data.issue.status,
        firstSeenNanos: data.issue.firstSeenNanos,
        lastSeenNanos: data.issue.lastSeenNanos,
        eventCount: data.issue.eventCount,
        lastTraceId: data.issue.lastTraceId,
        tags: data.issue.tags,
        groupingExplanation: data.issue.groupingExplanation,
        events: mapIssueEvents(data.issue.events),
      }
    : null
  return {
    issue,
    issueTrend: mapTrend(data.issueTrend),
    resource: extras.resource,
    breadcrumbs: extras.breadcrumbs,
    traceRunId: extras.traceRunId,
    releaseVersion: extras.releaseVersion,
  }
}
