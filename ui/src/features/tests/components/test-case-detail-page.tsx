import { Link, useNavigate } from "@tanstack/react-router"
import { IconFlask, IconHash } from "@tabler/icons-react"

import { EmptyState } from "@/shared/console/empty-state"
import { RelativeTime } from "@/shared/console/relative-time"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { identitySourceLabel, type TestCaseDetailData } from "@/features/tests/model/test-detail"
import {
  flakyLabel,
  rollupLabel,
  suiteLabel,
  type TestResultRef,
  type TestRollup,
} from "@/features/tests/model/test-summary"
import type { TestsSearch } from "@/features/tests/model/tests-search"
import { RangePicker } from "@/features/time-range"
import { mergeRangeSearch, resolveRangeSearch, type ResolvedRange } from "@/domain/time-range/range"
import { cn } from "@/lib/utils"
import { PageHeader } from "@/shared/components/page-header"
import { navItem } from "@/shared/navigation"

function statusToRollup(status: TestResultRef["status"]): TestRollup {
  switch (status) {
    case "PASSED":
      return "PASSED"
    case "FAILED":
      return "FAILED"
    case "BROKEN":
      return "BROKEN"
    case "SKIPPED":
      return "SKIPPED"
    case "UNKNOWN":
      return "UNKNOWN"
  }
}

function statusTone(status: TestResultRef["status"]): string {
  switch (status) {
    case "FAILED":
    case "BROKEN":
      return "bg-destructive/15 text-destructive"
    case "PASSED":
      return "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300"
    case "SKIPPED":
    case "UNKNOWN":
      return "bg-muted text-muted-foreground"
  }
}

export function TestCaseDetailRoutePage({
  data,
  search,
}: {
  data: TestCaseDetailData
  search: TestsSearch
}) {
  const navigate = useNavigate({ from: "/tests/$caseKey" })
  const range = resolveRangeSearch(search)
  return (
    <TestCaseDetailContent
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

export function TestCaseDetailContent({
  data,
  range,
  onRange,
}: {
  data: TestCaseDetailData
  range: ResolvedRange
  onRange: (range: ResolvedRange) => void
}) {
  const testsBack = navItem("/tests")
  const detail = data.case

  if (!detail) {
    return (
      <EmptyState
        icon={IconFlask}
        title="Test case not found"
        description="No registry case matches this versioned key."
      />
    )
  }

  return (
    <div className="space-y-4">
      <PageHeader
        icon={IconFlask}
        iconClassName="text-violet-500"
        title={detail.name}
        description={suiteLabel(detail.suitePath)}
        {...(testsBack ? { back: testsBack } : {})}
        actions={<RangePicker value={range} onChange={onRange} />}
      />

      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Identity</CardTitle>
          </CardHeader>
          <CardContent className="text-sm">
            {identitySourceLabel(detail.identitySource)}
            {detail.explicitId ? (
              <div className="mt-1 flex items-center gap-1 truncate text-muted-foreground">
                <IconHash className="size-3.5 shrink-0" aria-hidden />
                <span className="truncate">{detail.explicitId}</span>
              </div>
            ) : null}
            {detail.codeReference ? (
              <div className="mt-1 truncate text-muted-foreground">{detail.codeReference}</div>
            ) : null}
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Variants</CardTitle>
          </CardHeader>
          <CardContent className="text-2xl font-semibold tabular-nums">
            {detail.variants.length}
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">First seen</CardTitle>
          </CardHeader>
          <CardContent className="text-sm">
            <RelativeTime nanos={detail.firstSeenNanos} />
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Last seen</CardTitle>
          </CardHeader>
          <CardContent className="text-sm">
            <RelativeTime nanos={detail.lastSeenNanos} />
          </CardContent>
        </Card>
      </div>

      {detail.variants.length === 0 ? (
        <EmptyState
          icon={IconFlask}
          title="No variants"
          description="This case has no stored variants yet."
        />
      ) : (
        detail.variants.map((variant) => (
          <Card key={variant.variantKey}>
            <CardHeader className="gap-2 sm:flex-row sm:items-center sm:justify-between">
              <div className="space-y-1">
                <CardTitle className="text-base">Variant</CardTitle>
                <div className="text-sm text-muted-foreground">
                  {variant.parameters.length === 0
                    ? "default configuration"
                    : variant.parameters
                        .filter((parameter) => !parameter.excluded)
                        .map((parameter) => `${parameter.name}=${parameter.value}`)
                        .join(", ")}
                </div>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                {variant.flaky ? (
                  <Badge variant="secondary" className="capitalize">
                    {flakyLabel(variant.flaky.state)}
                  </Badge>
                ) : null}
                <span className="text-sm text-muted-foreground tabular-nums">
                  {variant.history.length} attempts
                </span>
              </div>
            </CardHeader>
            <CardContent>
              {variant.history.length === 0 ? (
                <p className="text-sm text-muted-foreground">
                  No attempt history for this variant.
                </p>
              ) : (
                <div className="overflow-hidden rounded-lg border">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Attempt</TableHead>
                        <TableHead>Status</TableHead>
                        <TableHead>Service</TableHead>
                        <TableHead>Invocation</TableHead>
                        <TableHead>Trace</TableHead>
                        <TableHead>Issue</TableHead>
                        <TableHead className="text-right">When</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {variant.history.map((attempt) => (
                        <TableRow
                          key={`${attempt.invocationId}:${attempt.attempt}:${attempt.traceId}`}
                        >
                          <TableCell className="tabular-nums">{attempt.attempt}</TableCell>
                          <TableCell>
                            <Badge
                              variant="secondary"
                              className={cn("capitalize", statusTone(attempt.status))}
                            >
                              {rollupLabel(statusToRollup(attempt.status))}
                            </Badge>
                          </TableCell>
                          <TableCell className="text-sm">
                            {attempt.service}
                            {attempt.serviceVersion ? (
                              <span className="text-muted-foreground">
                                {" "}
                                · {attempt.serviceVersion}
                              </span>
                            ) : null}
                          </TableCell>
                          <TableCell className="max-w-[10rem] truncate text-sm">
                            {attempt.invocationId ? (
                              <Link
                                to="/invocations/$invocationId"
                                params={{ invocationId: attempt.invocationId }}
                                className="text-primary hover:underline"
                              >
                                {attempt.invocationId}
                              </Link>
                            ) : (
                              "—"
                            )}
                          </TableCell>
                          <TableCell className="max-w-[10rem] truncate text-sm">
                            {attempt.traceId ? (
                              <Link
                                to="/traces/$traceId"
                                params={{ traceId: attempt.traceId }}
                                className="text-primary hover:underline"
                              >
                                {attempt.traceId}
                              </Link>
                            ) : (
                              "—"
                            )}
                          </TableCell>
                          <TableCell className="max-w-[10rem] truncate text-sm">
                            {attempt.failureFingerprint ? (
                              <Link
                                to="/issues/$fingerprint"
                                params={{ fingerprint: attempt.failureFingerprint }}
                                className="text-primary hover:underline"
                              >
                                {attempt.failureFingerprint}
                              </Link>
                            ) : (
                              "—"
                            )}
                          </TableCell>
                          <TableCell className="text-right text-sm">
                            <RelativeTime nanos={attempt.endedAtNanos} />
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              )}
            </CardContent>
          </Card>
        ))
      )}
    </div>
  )
}
