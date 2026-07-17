export interface TrendPoint {
  readonly tsNanos: string
  readonly count: number
}

export interface IssueSummary {
  readonly fingerprint: string
  readonly title: string
  readonly errorType: string
  readonly culprit: string | null
  readonly service: string
  readonly status: string
  readonly firstSeenNanos: string
  readonly lastSeenNanos: string
  readonly eventCount: number
  readonly lastTraceId: string | null
  readonly tags: string
  readonly trend: readonly TrendPoint[]
}

export type IssueRow = IssueSummary

export interface IssuesData {
  readonly issues: {
    readonly total: number
    readonly items: readonly IssueRow[]
  }
  readonly services: readonly string[]
}

export function topTags(tags: string): Array<{ label: string; rest: number }> {
  try {
    const parsed = JSON.parse(tags) as Record<string, Record<string, number>>
    const labels = Object.entries(parsed).flatMap(([key, values]) => {
      const top = Object.entries(values).sort(([, a], [, b]) => b - a)[0]
      return top ? [`${key}:${top[0]}`] : [key]
    })
    return labels.slice(0, 2).map((label, index) => ({
      label,
      rest: index === 1 ? Math.max(0, labels.length - 2) : 0,
    }))
  } catch {
    return []
  }
}

export function trendEvents(issue: IssueRow): number {
  return issue.trend.reduce((sum, point) => sum + point.count, 0)
}
