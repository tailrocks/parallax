import { Link, useNavigate } from "@tanstack/react-router"
import {
  IconActivityHeartbeat,
  IconAffiliate,
  IconAlertTriangleFilled,
  IconArticle,
  IconGaugeFilled,
  IconServer,
} from "@tabler/icons-react"

import { EmptyState } from "@/components/console/empty-state"
import { RelativeTime } from "@/components/console/relative-time"
import {
  CardSparkline,
  PillMeter,
  StatCard,
} from "@/components/console/stat-card"
import { navItem } from "@/components/nav"
import { buttonVariants } from "@/components/ui/button"
import { ServiceIdentityCard } from "@/features/services/components/service-identity-card"
import {
  ServiceLatencyChart,
  ServiceRequestsChart,
} from "@/features/services/components/service-red-charts"
import { ServiceRecentTraces } from "@/features/services/components/service-recent-traces"
import { ServiceReleaseStrip } from "@/features/services/components/service-release-strip"
import {
  latestErrorRate,
  latestValue,
  totalSeries,
  type ServiceDetailData,
} from "@/features/services/model/service-detail"
import { RuntimeSnapshotCard } from "@/features/runtime-metrics"
import { RangePicker } from "@/features/time-range"
import { formatCount, formatDurationNs, formatPercent } from "@/lib/format"
import {
  mergeRangeSearch,
  rangeLinkSearch,
  resolveRangeSearch,
  type ResolvedRange,
} from "@/lib/range"
import type { ServicesSearch } from "@/features/services/model/services-search"
import { PageHeader } from "@/shared/components/page-header"

export function ServiceDetailRoutePage({
  service,
  data,
  search,
}: {
  service: string
  data: ServiceDetailData
  search: ServicesSearch
}) {
  const navigate = useNavigate({ from: "/services/$service" })
  const range = resolveRangeSearch(search)

  return (
    <ServiceDetailContent
      service={service}
      data={data}
      range={range}
      onRange={(next) =>
        void navigate({
          search: (current) => mergeRangeSearch(current, next),
        })
      }
    />
  )
}

export function ServiceDetailContent({
  service,
  data,
  range,
  onRange,
}: {
  service: string
  data: ServiceDetailData
  range: ResolvedRange
  onRange: (range: ResolvedRange) => void
}) {
  const hasRed =
    data.red.rate.length > 0 ||
    data.red.errorRate.length > 0 ||
    data.red.p95.length > 0
  const traces = data.tracesPage.items
  const identity = data.serviceCatalog.find((row) => row.name === service)
  const noData =
    !hasRed && traces.length === 0 && data.runtimeSnapshot.length === 0
  const lastSeen = traces[0]?.startNanos
  const servicesBack = navItem("/services")

  if (noData) {
    return (
      <div className="space-y-4">
        <PageHeader
          {...(servicesBack ? { back: servicesBack } : {})}
          title={service}
          actions={<RangePicker value={range} onChange={onRange} />}
        />
        <EmptyState
          icon={IconServer}
          title="Service not found"
          description="No spans, errors, or metrics matched this service in the selected window."
        />
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <PageHeader
        {...(servicesBack ? { back: servicesBack } : {})}
        title={service}
        actions={
          <>
            <Link
              to="/traces"
              search={{ service, ...rangeLinkSearch(range) }}
              className={buttonVariants({ variant: "outline", size: "sm" })}
            >
              <IconAffiliate />
              Traces
            </Link>
            <Link
              to="/logs"
              search={{ service, ...rangeLinkSearch(range) }}
              className={buttonVariants({ variant: "outline", size: "sm" })}
            >
              <IconArticle />
              Logs
            </Link>
            <Link
              to="/issues"
              search={{ service, ...rangeLinkSearch(range) }}
              className={buttonVariants({ variant: "outline", size: "sm" })}
            >
              <IconAlertTriangleFilled />
              Issues
            </Link>
            <RangePicker value={range} onChange={onRange} />
          </>
        }
      />

      <ServiceReleaseStrip releases={data.releases} range={range} />
      <ServiceIdentityCard identity={identity} fallbackLastSeen={lastSeen} />

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatCard
          icon={IconActivityHeartbeat}
          label="Requests"
          value={formatCount(Math.round(totalSeries(data.red.rate)))}
          hint="span-derived"
          chart={<CardSparkline data={[...data.red.rate]} />}
        />
        <StatCard
          icon={IconAlertTriangleFilled}
          iconClassName="text-rose-500"
          label="Error rate"
          value={formatPercent(latestErrorRate(data.red))}
          hint={<PillMeter value={latestErrorRate(data.red)} />}
        />
        <StatCard
          icon={IconGaugeFilled}
          label="p95 latency"
          value={
            latestValue(data.red.p95) == null
              ? "-"
              : formatDurationNs((latestValue(data.red.p95) ?? 0) * 1_000_000)
          }
          hint={
            latestValue(data.red.p50) == null
              ? undefined
              : `p50 ${formatDurationNs((latestValue(data.red.p50) ?? 0) * 1_000_000)}`
          }
          chart={<CardSparkline data={[...data.red.p95]} />}
        />
        <StatCard
          icon={IconAffiliate}
          label="Last seen"
          value={lastSeen ? <RelativeTime nanos={lastSeen} /> : <span>-</span>}
          hint={`${formatCount(traces.length)} recent traces`}
        />
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        <ServiceRequestsChart red={data.red} />
        <ServiceLatencyChart
          red={data.red}
          overview={data.overview}
          exemplars={[
            ...data.httpDurationExemplars,
            ...data.rpcDurationExemplars,
          ]}
          range={range}
        />
      </div>

      <RuntimeSnapshotCard metrics={[...data.runtimeSnapshot]} />
      <ServiceRecentTraces traces={traces} range={range} />
    </div>
  )
}
