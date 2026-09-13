import { Link, useNavigate, useRouterState } from "@tanstack/react-router"
import { IconAffiliate, IconBug, IconUsers, IconWorld } from "@tabler/icons-react"
import { useMemo } from "react"
import { CartesianGrid, Line, LineChart, XAxis, YAxis } from "recharts"

import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { RangePicker } from "@/features/time-range"
import {
  formatVitalValue,
  ratingLabel,
  VITAL_SPECS,
  type VitalRating,
} from "@/features/rum/model/rum-vitals"
import type {
  RumData,
  RumIssueRow,
  RumSessionDetailData,
  RumSessionErrorRow,
  RumSessionRow,
  RumSessionVitalRow,
  RumSessionViewRow,
  RumTraceData,
  RumTraceRow,
  RumVitalData,
} from "@/features/rum/model/rum-overview"
import {
  patchRumSearch,
  type RumSearch,
  type RumSearchPatch,
} from "@/features/rum/model/rum-search"
import {
  rangeLinkSearch,
  resolveRangeSearch,
  updateRangeSearch,
  type ResolvedRange,
} from "@/domain/time-range/range"
import { WhereClauseEditor } from "@/shared/console/where-clause-editor"
import { ClearFiltersButton, FilterSelect, Toolbar } from "@/shared/console/data-table"
import { EmptyState } from "@/shared/console/empty-state"
import { useDelayedLoading } from "@/shared/console/hooks"
import { RelativeTime } from "@/shared/console/relative-time"
import { TableSkeleton } from "@/shared/console/skeletons"
import { PageHeader } from "@/shared/components/page-header"
import { formatCount, formatDurationNs } from "@/shared/format"
import {
  serializeWhereClause,
  whereClauseFromSearch,
  type WhereFilter,
} from "@/shared/where-clause"

function ratingVariant(rating: VitalRating): "emerald" | "amber" | "rose" {
  switch (rating) {
    case "good":
      return "emerald"
    case "needs-improvement":
      return "amber"
    case "poor":
      return "rose"
  }
}

export function RumPage({
  data,
  vital,
  trace,
  sessions,
  session,
  search,
}: {
  data: RumData
  vital: RumVitalData | null
  trace: RumTraceData | null
  sessions: readonly RumSessionRow[]
  session: RumSessionDetailData | null
  search: RumSearch
}) {
  const navigate = useNavigate({ from: "/rum/" })
  const range = resolveRangeSearch(search)
  const routerLoading = useRouterState({
    select: (state) => state.status === "pending",
  })
  const loading = useDelayedLoading(routerLoading)

  const setSearch = (patch: RumSearchPatch) =>
    void navigate({ search: patchRumSearch(search, patch) })

  return (
    <RumContent
      data={data}
      vital={vital}
      trace={trace}
      sessions={sessions}
      session={session}
      search={search}
      range={range}
      loading={loading}
      onSearch={setSearch}
    />
  )
}

export function RumContent({
  data,
  vital,
  trace,
  sessions,
  session,
  search,
  range,
  loading,
  onSearch,
}: {
  data: RumData
  vital: RumVitalData | null
  trace: RumTraceData | null
  sessions: readonly RumSessionRow[]
  session: RumSessionDetailData | null
  search: RumSearch
  range: ResolvedRange
  loading?: boolean
  onSearch: (patch: RumSearchPatch) => void
}) {
  const hasFilters = Boolean(search.service || search.where)
  const whereFilters = whereClauseFromSearch(search.where)
  const applyWhere = (filters: WhereFilter[]) =>
    onSearch({ where: serializeWhereClause(filters) || undefined })

  const trendRows = useMemo(() => {
    if (!vital) return []
    return vital.trend.map((point) => ({
      time: new Date(Number(BigInt(point.tsNanos) / 1_000_000n)).toLocaleTimeString(),
      p75: point.value,
    }))
  }, [vital])
  const trendConfig = {
    p75: { label: "p75", color: "var(--chart-1)" },
  } satisfies ChartConfig

  return (
    <div className="space-y-4">
      <PageHeader
        icon={IconWorld}
        iconClassName="text-sky-500"
        title="RUM"
        description="Browser sessions, vitals, errors, and journeys over live spans, logs, metrics, and issues."
        actions={
          <RangePicker value={range} onChange={(next) => onSearch(updateRangeSearch(next))} />
        }
      />

      <Toolbar className="justify-between">
        <div className="flex flex-wrap items-center gap-2">
          <FilterSelect
            {...(search.service ? { value: search.service } : {})}
            onChange={(service) =>
              onSearch({ service, vital: undefined, traceId: undefined, sessionId: undefined })
            }
            placeholder="All services"
            options={data.services.map((service) => ({
              value: service,
              label: service,
            }))}
          />
          <WhereClauseEditor
            filters={whereFilters}
            onApply={applyWhere}
            className="min-w-64 flex-1"
          />
          {hasFilters ? (
            <ClearFiltersButton
              onClick={() =>
                onSearch({
                  service: undefined,
                  where: undefined,
                  vital: undefined,
                  traceId: undefined,
                  sessionId: undefined,
                })
              }
            />
          ) : null}
        </div>
        <div className="text-sm text-muted-foreground tabular-nums">
          {formatCount(data.vitals.length)} vitals
        </div>
      </Toolbar>

      {loading ? (
        <TableSkeleton rows={8} />
      ) : (
        <>
          <Card>
            <CardHeader>
              <CardTitle className="text-sm">
                Web vitals {data.service ? `· ${data.service}` : "· all services"} · p75
              </CardTitle>
            </CardHeader>
            <CardContent>
              {data.vitals.length === 0 ? (
                <EmptyState
                  icon={IconWorld}
                  title="No web vitals yet"
                  description="Emit OTLP metrics named like lcp, inp, or cls (or browser.* variants) so Parallax can rate real-user experience."
                />
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Vital</TableHead>
                      <TableHead>Metric</TableHead>
                      <TableHead>p75</TableHead>
                      <TableHead>Rating</TableHead>
                      <TableHead>Samples</TableHead>
                      <TableHead>Last datapoint</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {data.vitals.map((row) => (
                      <TableRow
                        key={row.name}
                        data-testid={`vital-row-${row.name}`}
                        className={search.vital === row.name ? "bg-muted/50" : "cursor-pointer"}
                        onClick={() =>
                          onSearch({
                            vital: search.vital === row.name ? undefined : row.name,
                          })
                        }
                      >
                        <TableCell>
                          <span className="font-medium">{row.vital}</span>
                          <span className="ml-2 text-xs text-muted-foreground">
                            {VITAL_SPECS[row.vital].label}
                          </span>
                        </TableCell>
                        <TableCell className="font-mono text-xs">{row.name}</TableCell>
                        <TableCell className="tabular-nums">
                          {row.p75 == null ? "—" : formatVitalValue(row.vital, row.p75)}
                        </TableCell>
                        <TableCell>
                          {row.rating == null ? (
                            "—"
                          ) : (
                            <Badge variant={ratingVariant(row.rating)}>
                              {ratingLabel(row.rating)}
                            </Badge>
                          )}
                        </TableCell>
                        <TableCell className="tabular-nums">
                          {formatCount(Number(row.pointCount) || 0)}
                        </TableCell>
                        <TableCell className="text-xs text-muted-foreground">
                          <RelativeTime nanos={row.lastDatapointNanos} />
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>

          {search.vital ? (
            vital ? (
              <VitalDetailCard
                vital={vital}
                range={range}
                trendRows={trendRows}
                trendConfig={trendConfig}
              />
            ) : (
              <EmptyState
                icon={IconWorld}
                title="Vital not found"
                description={`No vital metric named ${search.vital} in this range.`}
              />
            )
          ) : null}

          <Card>
            <CardHeader>
              <CardTitle className="text-sm">
                Sessions · {formatCount(sessions.length)}
              </CardTitle>
            </CardHeader>
            <CardContent>
              {sessions.length === 0 ? (
                <EmptyState
                  icon={IconUsers}
                  title="No sessions"
                  description="Browser sessions grouped by session.id appear here once RUM spans arrive."
                />
              ) : (
                <SessionTable
                  rows={sessions}
                  selected={search.sessionId}
                  onSelect={(sessionId) => onSearch({ sessionId })}
                />
              )}
            </CardContent>
          </Card>

          {search.sessionId ? (
            session ? (
              <SessionDetailCard session={session} range={range} />
            ) : (
              <EmptyState
                icon={IconUsers}
                title="Session unavailable"
                description={`No timeline data for ${search.sessionId}.`}
              />
            )
          ) : null}

          <div className="grid gap-4 xl:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle className="text-sm">
                  Browser errors · {formatCount(data.issueTotal)} issues
                </CardTitle>
              </CardHeader>
              <CardContent>
                {data.issues.length === 0 ? (
                  <EmptyState
                    icon={IconBug}
                    title="No issues"
                    description="Grouped browser errors for this service appear here."
                  />
                ) : (
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Error</TableHead>
                        <TableHead>Events</TableHead>
                        <TableHead>Last seen</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {data.issues.map((issue) => (
                        <IssueRow
                          key={`${issue.service}-${issue.fingerprint}`}
                          issue={issue}
                          range={range}
                        />
                      ))}
                    </TableBody>
                  </Table>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-sm">
                  Error traces · {formatCount(Number(data.errorTraceTotal) || 0)}
                </CardTitle>
              </CardHeader>
              <CardContent>
                {data.errorTraces.length === 0 ? (
                  <EmptyState
                    icon={IconAffiliate}
                    title="No error traces"
                    description="Error-only traces for this service appear here."
                  />
                ) : (
                  <TraceTable
                    rows={data.errorTraces}
                    range={range}
                    selected={search.traceId}
                    onSelect={(traceId) => onSearch({ traceId })}
                  />
                )}
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle className="text-sm">
                Journeys · {formatCount(Number(data.journeyTotal) || 0)} traces
              </CardTitle>
            </CardHeader>
            <CardContent>
              {data.journeys.length === 0 ? (
                <EmptyState
                  icon={IconAffiliate}
                  title="No journeys"
                  description="Recent traces for this service appear here; add where-filters (e.g. rum.interaction CONTAINS rage_click) to slice by RUM attributes."
                />
              ) : (
                <TraceTable
                  rows={data.journeys}
                  range={range}
                  selected={search.traceId}
                  onSelect={(traceId) => onSearch({ traceId })}
                />
              )}
            </CardContent>
          </Card>

          {search.traceId ? (
            trace ? (
              <TraceCorrelationCard trace={trace} range={range} />
            ) : (
              <EmptyState
                icon={IconAffiliate}
                title="Trace unavailable"
                description={`No correlation data for ${search.traceId}.`}
              />
            )
          ) : null}
        </>
      )}
    </div>
  )
}

function VitalDetailCard({
  vital,
  range,
  trendRows,
  trendConfig,
}: {
  vital: RumVitalData
  range: ResolvedRange
  trendRows: Array<{ time: string; p75: number }>
  trendConfig: ChartConfig
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">
          {vital.row.vital} · <span className="font-mono">{vital.row.name}</span>
          {vital.row.rating == null ? null : (
            <Badge variant={ratingVariant(vital.row.rating)} className="ml-2">
              {vital.row.p75 == null
                ? ratingLabel(vital.row.rating)
                : `${ratingLabel(vital.row.rating)} · ${formatVitalValue(vital.row.vital, vital.row.p75)}`}
            </Badge>
          )}
          <Link
            to="/metrics/$metricName"
            params={{ metricName: vital.row.name }}
            search={rangeLinkSearch(range)}
            className="ml-2 text-xs font-normal text-muted-foreground underline"
          >
            full metric detail
          </Link>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {trendRows.length === 0 ? (
          <p className="text-sm text-muted-foreground">No p75 trend in this range.</p>
        ) : (
          <ChartContainer config={trendConfig} className="h-[180px] w-full">
            <LineChart data={trendRows} margin={{ left: 8, right: 8, top: 8 }}>
              <CartesianGrid vertical={false} />
              <XAxis dataKey="time" tickLine={false} axisLine={false} minTickGap={32} />
              <YAxis tickLine={false} axisLine={false} width={48} />
              <ChartTooltip content={<ChartTooltipContent />} />
              <Line
                dataKey="p75"
                stroke="var(--color-p75)"
                dot={{ r: 2, strokeWidth: 0, fill: "var(--color-p75)" }}
              />
            </LineChart>
          </ChartContainer>
        )}
        <div>
          <h4 className="mb-2 text-xs font-medium text-muted-foreground">
            Trace exemplars {vital.exemplars.length > 0 ? `· ${vital.exemplars.length}` : ""}
          </h4>
          {vital.exemplars.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              No trace exemplar attached; showing trend only.
            </p>
          ) : (
            <ul className="space-y-1 text-sm">
              {vital.exemplars.map((exemplar) => (
                <li key={`${exemplar.traceId}-${exemplar.spanId}-${exemplar.tsNanos}`}>
                  <Link
                    to="/traces/$traceId"
                    params={{ traceId: exemplar.traceId }}
                    data-testid={`trace-link-${exemplar.traceId}`}
                  >
                    {exemplar.traceId} · {exemplar.value}
                  </Link>
                </li>
              ))}
            </ul>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

function sessionDurationNs(row: RumSessionRow): string {
  const duration = BigInt(row.endNanos) - BigInt(row.startNanos)
  return (duration < 0n ? 0n : duration).toString()
}

function SessionTable({
  rows,
  selected,
  onSelect,
}: {
  rows: readonly RumSessionRow[]
  selected: string | undefined
  onSelect: (sessionId: string | undefined) => void
}) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Session</TableHead>
          <TableHead>Duration</TableHead>
          <TableHead>Views</TableHead>
          <TableHead>Vitals</TableHead>
          <TableHead>Errors</TableHead>
          <TableHead>Started</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {rows.map((row) => (
          <TableRow
            key={row.sessionId}
            data-testid={`session-row-${row.sessionId}`}
            className={selected === row.sessionId ? "bg-muted/50" : "cursor-pointer"}
            onClick={() => onSelect(selected === row.sessionId ? undefined : row.sessionId)}
          >
            <TableCell>
              <span className="font-mono text-xs">{row.sessionId}</span>
              <div className="flex items-center gap-1 text-xs text-muted-foreground">
                <span>{row.service}</span>
                {row.hasError ? <Badge variant="rose">error</Badge> : null}
              </div>
            </TableCell>
            <TableCell className="tabular-nums">{formatDurationNs(sessionDurationNs(row))}</TableCell>
            <TableCell className="tabular-nums">{formatCount(row.viewCount)}</TableCell>
            <TableCell className="tabular-nums">{formatCount(row.vitalCount)}</TableCell>
            <TableCell className="tabular-nums">{formatCount(row.errorCount)}</TableCell>
            <TableCell className="text-xs text-muted-foreground">
              <RelativeTime nanos={row.startNanos} />
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}

type SessionTimelineEntry =
  | { kind: "view"; tsNanos: string; row: RumSessionViewRow }
  | { kind: "vital"; tsNanos: string; row: RumSessionVitalRow }
  | { kind: "error"; tsNanos: string; row: RumSessionErrorRow }

function SessionDetailCard({
  session,
  range,
}: {
  session: RumSessionDetailData
  range: ResolvedRange
}) {
  const timeline = useMemo(() => {
    const entries: SessionTimelineEntry[] = [
      ...session.views.map((row): SessionTimelineEntry => ({ kind: "view", tsNanos: row.tsNanos, row })),
      ...session.vitals.map((row): SessionTimelineEntry => ({ kind: "vital", tsNanos: row.tsNanos, row })),
      ...session.errors.map((row): SessionTimelineEntry => ({ kind: "error", tsNanos: row.tsNanos, row })),
    ]
    entries.sort((a, b) => (BigInt(a.tsNanos) < BigInt(b.tsNanos) ? -1 : 1))
    return entries
  }, [session])

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">
          Session timeline · <span className="font-mono">{session.session.sessionId}</span>
          {session.session.hasError ? (
            <Badge variant="rose" className="ml-2">
              {formatCount(session.session.errorCount)} errors
            </Badge>
          ) : null}
        </CardTitle>
      </CardHeader>
      <CardContent>
        {timeline.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            No page views, vitals, or errors in this session.
          </p>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Time</TableHead>
                <TableHead>Event</TableHead>
                <TableHead>Detail</TableHead>
                <TableHead>Trace</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {timeline.map((entry, index) => (
                <TableRow key={`${entry.kind}-${entry.tsNanos}-${index}`}>
                  <TableCell className="text-xs text-muted-foreground">
                    <RelativeTime nanos={entry.tsNanos} />
                  </TableCell>
                  <TableCell>
                    <SessionTimelineKind entry={entry} />
                  </TableCell>
                  <TableCell className="text-sm">
                    <SessionTimelineDetail entry={entry} />
                  </TableCell>
                  <TableCell>
                    <Link
                      to="/traces/$traceId"
                      params={{ traceId: entry.row.traceId }}
                      search={rangeLinkSearch(range)}
                      data-testid={`trace-link-${entry.row.traceId}`}
                      className="font-mono text-xs"
                    >
                      {entry.row.traceId.slice(0, 12)}
                    </Link>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </CardContent>
    </Card>
  )
}

function SessionTimelineKind({ entry }: { entry: SessionTimelineEntry }) {
  switch (entry.kind) {
    case "view":
      return <Badge variant="outline">view</Badge>
    case "vital":
      return <Badge variant="blue">vital</Badge>
    case "error":
      return <Badge variant="rose">error</Badge>
  }
}

function SessionTimelineDetail({ entry }: { entry: SessionTimelineEntry }) {
  switch (entry.kind) {
    case "view":
      return (
        <span>
          <span className="font-medium">{entry.row.screen}</span>
          {entry.row.path ? (
            <span className="ml-2 font-mono text-xs text-muted-foreground">{entry.row.path}</span>
          ) : null}
        </span>
      )
    case "vital":
      return (
        <span>
          <span className="font-medium">{entry.row.name}</span>
          <span className="ml-2 tabular-nums">{entry.row.value}</span>
          {entry.row.rating ? (
            <span className="ml-2 text-xs text-muted-foreground">{entry.row.rating}</span>
          ) : null}
        </span>
      )
    case "error":
      return (
        <span>
          <span className="font-medium">{entry.row.errorType || entry.row.name}</span>
          {entry.row.message ? (
            <span className="ml-2 max-w-64 truncate text-xs text-muted-foreground">
              {entry.row.message}
            </span>
          ) : null}
        </span>
      )
  }
}

function IssueRow({ issue, range }: { issue: RumIssueRow; range: ResolvedRange }) {
  return (
    <TableRow>
      <TableCell>
        <Link
          to="/issues/$service/$fingerprint"
          params={{ service: issue.service, fingerprint: issue.fingerprint }}
          search={rangeLinkSearch(range)}
          className="font-medium"
        >
          {issue.errorType || issue.title}
        </Link>
        <div className="max-w-64 truncate text-xs text-muted-foreground">{issue.title}</div>
      </TableCell>
      <TableCell className="tabular-nums">{formatCount(issue.eventCount)}</TableCell>
      <TableCell className="text-xs text-muted-foreground">
        <RelativeTime nanos={issue.lastSeenNanos} />
      </TableCell>
    </TableRow>
  )
}

function TraceTable({
  rows,
  range,
  selected,
  onSelect,
}: {
  rows: readonly RumTraceRow[]
  range: ResolvedRange
  selected: string | undefined
  onSelect: (traceId: string | undefined) => void
}) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Trace</TableHead>
          <TableHead>Duration</TableHead>
          <TableHead>Spans</TableHead>
          <TableHead>Started</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {rows.map((row) => (
          <TableRow
            key={row.traceId}
            data-testid={`trace-row-${row.traceId}`}
            className={selected === row.traceId ? "bg-muted/50" : "cursor-pointer"}
            onClick={() => onSelect(selected === row.traceId ? undefined : row.traceId)}
          >
            <TableCell>
              <Link
                to="/traces/$traceId"
                params={{ traceId: row.traceId }}
                search={rangeLinkSearch(range)}
                data-testid={`trace-link-${row.traceId}`}
                onClick={(event) => event.stopPropagation()}
              >
                {row.rootName || row.traceId}
              </Link>
              <div className="flex items-center gap-1 text-xs text-muted-foreground">
                <span className="font-mono">{row.traceId.slice(0, 12)}</span>
                {row.hasError ? <Badge variant="rose">error</Badge> : null}
              </div>
            </TableCell>
            <TableCell className="tabular-nums">{formatDurationNs(row.durationNs)}</TableCell>
            <TableCell className="tabular-nums">{formatCount(row.spanCount)}</TableCell>
            <TableCell className="text-xs text-muted-foreground">
              <RelativeTime nanos={row.startNanos} />
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}

function TraceCorrelationCard({ trace, range }: { trace: RumTraceData; range: ResolvedRange }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">
          Trace correlation · <span className="font-mono">{trace.traceId}</span>
        </CardTitle>
      </CardHeader>
      <CardContent className="grid gap-4 xl:grid-cols-2">
        <div>
          <h4 className="mb-2 text-xs font-medium text-muted-foreground">
            Linked traces (cross-tier) {trace.linked.length > 0 ? `· ${trace.linked.length}` : ""}
          </h4>
          {trace.linked.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              No linked traces; the journey stays in one tier.
            </p>
          ) : (
            <ul className="space-y-1 text-sm">
              {trace.linked.map((row) => (
                <li key={row.traceId}>
                  <Link
                    to="/traces/$traceId"
                    params={{ traceId: row.traceId }}
                    search={rangeLinkSearch(range)}
                    data-testid={`trace-link-${row.traceId}`}
                  >
                    {row.service} · {row.rootName || row.traceId}
                  </Link>
                </li>
              ))}
            </ul>
          )}
        </div>
        <div>
          <h4 className="mb-2 text-xs font-medium text-muted-foreground">
            Logs {trace.logs.length > 0 ? `· ${trace.logs.length}` : ""}
          </h4>
          {trace.logs.length === 0 ? (
            <p className="text-sm text-muted-foreground">No logs on this trace.</p>
          ) : (
            <ul className="space-y-1 text-sm">
              {trace.logs.slice(0, 20).map((log, index) => (
                <li key={`${log.tsNanos}-${index}`} className="flex items-baseline gap-2">
                  <Badge variant="outline">{log.severityText}</Badge>
                  <span className="min-w-0 truncate text-muted-foreground">{log.body}</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
