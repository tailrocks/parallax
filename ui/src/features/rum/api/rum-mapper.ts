// Wire-to-model mapping for the RUM surface. All data comes from live
// Parallax GraphQL primitives (metricCatalog, histogramQuantile,
// metricExemplars, issues, tracesPage, linkedTraces, logsByTrace,
// rumSessions, rumSession).

import type { RumCatalogQuery } from "@/features/rum/api/rum-catalog.generated"
import type { RumErrorsQuery } from "@/features/rum/api/rum-errors.generated"
import type { RumJourneysQuery } from "@/features/rum/api/rum-journeys.generated"
import type { RumSessionQuery } from "@/features/rum/api/rum-session.generated"
import type { RumSessionsQuery } from "@/features/rum/api/rum-sessions.generated"
import type { RumTraceQuery } from "@/features/rum/api/rum-trace.generated"
import type { RumVitalDetailQuery } from "@/features/rum/api/rum-vital-detail.generated"
import type { RumVitalStatsQuery } from "@/features/rum/api/rum-vital-stats.generated"
import type {
  RumExemplar,
  RumIssueRow,
  RumLogRow,
  RumPoint,
  RumSessionDetailData,
  RumSessionRow,
  RumTraceRow,
  RumVitalRow,
} from "@/features/rum/model/rum-overview"
import {
  matchVital,
  normalizeVitalValue,
  rateVital,
  type VitalId,
} from "@/features/rum/model/rum-vitals"

export interface CatalogVital {
  readonly name: string
  readonly vital: VitalId
  readonly kind: string
  readonly unit: string | null
  readonly services: readonly string[]
  readonly lastDatapointNanos: string
  readonly pointCount: string
}

export function mapCatalogVitals(data: RumCatalogQuery): CatalogVital[] {
  const rows: CatalogVital[] = []
  for (const row of data.metricCatalog) {
    const vital = matchVital(row.name)
    if (!vital) continue
    rows.push({
      name: row.name,
      vital,
      kind: row.kind,
      unit: row.unit,
      services: [...row.services],
      lastDatapointNanos: row.lastDatapointNanos,
      pointCount: row.pointCount,
    })
  }
  rows.sort((a, b) => a.vital.localeCompare(b.vital) || a.name.localeCompare(b.name))
  return rows
}

export function mapVitalRow(catalog: CatalogVital, stats: RumVitalStatsQuery | null): RumVitalRow {
  const points = stats?.histogramQuantile ?? []
  const latest = points.length > 0 ? points[points.length - 1] : null
  const p75 = latest == null ? null : normalizeVitalValue(latest.value, catalog.unit)
  return {
    ...catalog,
    p75,
    rating: p75 == null ? null : rateVital(catalog.vital, p75),
  }
}

export function mapVitalTrend(stats: RumVitalStatsQuery): RumPoint[] {
  return stats.histogramQuantile.map((point) => ({ ...point }))
}

export function mapExemplars(data: RumVitalDetailQuery): RumExemplar[] {
  return data.metricExemplars.map((exemplar) => ({ ...exemplar }))
}

function mapTraceRow(row: RumErrorsQuery["tracesPage"]["items"][number]): RumTraceRow {
  return { ...row }
}

export function mapErrorTraces(data: RumErrorsQuery): RumTraceRow[] {
  return data.tracesPage.items.map(mapTraceRow)
}

export function mapIssues(data: RumErrorsQuery): RumIssueRow[] {
  return data.issues.items.map((issue) => ({
    fingerprint: issue.fingerprint,
    title: issue.title,
    errorType: issue.errorType,
    culprit: issue.culprit,
    service: issue.service,
    status: issue.status,
    lastSeenNanos: issue.lastSeenNanos,
    eventCount: issue.eventCount,
    lastTraceId: issue.lastTraceId,
  }))
}

export function mapJourneys(data: RumJourneysQuery): RumTraceRow[] {
  return data.tracesPage.items.map((row) => ({ ...row }))
}

export function mapLinkedTraces(data: RumTraceQuery): RumTraceRow[] {
  return data.linkedTraces.map((row) => ({ ...row }))
}

export function mapTraceLogs(data: RumTraceQuery): RumLogRow[] {
  return data.logsByTrace.map((log) => ({ ...log }))
}

export function mapRumSessions(data: RumSessionsQuery): RumSessionRow[] {
  return data.rumSessions.map((row) => ({ ...row }))
}

export function mapRumSession(data: RumSessionQuery): RumSessionDetailData | null {
  const detail = data.rumSession
  if (!detail) return null
  return {
    session: { ...detail.session },
    views: detail.views.map((view) => ({ ...view })),
    vitals: detail.vitals.map((vital) => ({ ...vital })),
    errors: detail.errors.map((error) => ({ ...error })),
  }
}
