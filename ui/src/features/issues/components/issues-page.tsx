import { useNavigate, useRouterState } from "@tanstack/react-router"
import { IconBug, IconTerminal2 } from "@tabler/icons-react"

import { ClearFiltersButton, FilterSelect, SearchInput } from "@/shared/console/data-table"
import { EmptyState } from "@/shared/console/empty-state"
import { QueryBar, QueryBarCount, QueryBarRow } from "@/shared/console/query-bar"
import { SnippetTabs } from "@/shared/console/snippet-tabs"
import { useDelayedLoading } from "@/shared/console/hooks"
import { TableSkeleton } from "@/shared/console/skeletons"

import type { IssueRow, IssuesData } from "@/features/issues/model/issue-summary"
import {
  patchIssuesSearch,
  type IssueSort,
  type IssuesSearch,
  type IssuesSearchPatch,
} from "@/features/issues/model/issues-search"
import { RangePicker } from "@/features/time-range"
import {
  rangeLinkSearch,
  resolveRangeSearch,
  updateRangeSearch,
  type ResolvedRange,
} from "@/domain/time-range/range"
import { PageHeader } from "@/shared/components/page-header"
import { IssuesTable } from "@/features/issues/components/issues-table"

export function IssuesPage({ data, search }: { data: IssuesData; search: IssuesSearch }) {
  const navigate = useNavigate({ from: "/issues/" })
  const range = resolveRangeSearch(search)
  const routerLoading = useRouterState({
    select: (state) => state.status === "pending",
  })
  const loading = useDelayedLoading(routerLoading)

  const setSearch = (patch: IssuesSearchPatch) =>
    void navigate({ search: patchIssuesSearch(search, patch) })

  return (
    <IssuesContent
      data={data}
      search={search}
      range={range}
      loading={loading}
      onSearch={setSearch}
      onIssue={(issue) =>
        void navigate({
          to: "/issues/$service/$fingerprint",
          params: { service: issue.service, fingerprint: issue.fingerprint },
          search: rangeLinkSearch(range),
        })
      }
    />
  )
}

export function IssuesContent({
  data,
  search,
  range,
  loading,
  onSearch,
  onIssue,
}: {
  data: IssuesData
  search: IssuesSearch
  range: ResolvedRange
  loading?: boolean
  onSearch: (patch: IssuesSearchPatch) => void
  onIssue: (issue: IssueRow) => void
}) {
  const hasFilters = Boolean(search.q || search.service || search.status)
  const sort = search.sort ?? "LAST_SEEN"

  return (
    <div className="space-y-4">
      <PageHeader
        icon={IconBug}
        iconClassName="text-rose-500"
        title="Issues"
        description="Grouped errors by service, message, and culprit."
        actions={
          <RangePicker value={range} onChange={(next) => onSearch(updateRangeSearch(next))} />
        }
      />

      <QueryBar>
        <QueryBarRow>
          <SearchInput
            value={search.q ?? ""}
            onChange={(q) => onSearch({ q })}
            placeholder="Search message/type"
          />
          <FilterSelect
            {...(search.service ? { value: search.service } : {})}
            onChange={(service) => onSearch({ service })}
            placeholder="All services"
            options={data.services.map((service) => ({
              value: service,
              label: service,
            }))}
          />
        </QueryBarRow>
        <QueryBarRow>
          <FilterSelect
            {...(search.status ? { value: search.status } : {})}
            onChange={(status) =>
              onSearch({
                status:
                  status === "open" || status === "resolved" || status === "regressed"
                    ? status
                    : undefined,
              })
            }
            placeholder="Any status"
            options={[
              { value: "open", label: "Open" },
              { value: "regressed", label: "Regressed" },
              { value: "resolved", label: "Resolved" },
            ]}
          />
          <FilterSelect
            value={sort}
            onChange={(next) => onSearch({ sort: next as IssueSort })}
            placeholder="Sort"
            options={[
              { value: "LAST_SEEN", label: "Last seen" },
              { value: "FIRST_SEEN", label: "First seen" },
              { value: "EVENTS", label: "Events" },
              { value: "TREND", label: "Trend" },
            ]}
          />
          {hasFilters ? (
            <ClearFiltersButton
              onClick={() =>
                onSearch({
                  q: undefined,
                  service: undefined,
                  status: undefined,
                })
              }
            />
          ) : null}
          <QueryBarCount shown={data.issues.items.length} total={data.issues.total} unit="issues" />
        </QueryBarRow>
      </QueryBar>

      {loading ? (
        <TableSkeleton rows={8} />
      ) : data.issues.items.length === 0 ? (
        <EmptyState
          icon={IconTerminal2}
          title={hasFilters ? "No issues match filters" : "No issues ingested yet"}
          description={hasFilters ? "Loosen query, service, status, or range." : <SnippetTabs />}
        />
      ) : (
        <IssuesTable
          items={data.issues.items}
          range={range}
          sort={sort}
          onSearch={onSearch}
          onIssue={onIssue}
        />
      )}
    </div>
  )
}
