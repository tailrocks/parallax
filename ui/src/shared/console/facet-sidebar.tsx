import { useMemo, useState } from "react"
import { IconChevronDown, IconChevronRight, IconX } from "@tabler/icons-react"

import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Input } from "@/components/ui/input"
import { ServiceDot } from "@/shared/console/service-dot"
import { cn } from "@/lib/utils"

/** Plan 164 facet model: one bounded dimension with per-value counts.
 * Semantics: multi-select ORs within a facet, ANDs across facets. */
export type FacetValue = {
  value: string
  count: number
}

export type Facet = {
  dimension: string
  label: string
  values: FacetValue[]
  /** Render a ServiceDot swatch next to each value (service facets). */
  serviceDots?: boolean
  /** Show an inline search box when the value list is long. */
  searchable?: boolean
}

const DEFAULT_MAX_VISIBLE = 8

export function FacetSection({
  facet,
  selected,
  onToggle,
  maxVisible = DEFAULT_MAX_VISIBLE,
}: {
  facet: Facet
  selected: string[]
  onToggle: (dimension: string, value: string) => void
  maxVisible?: number
}) {
  const [collapsed, setCollapsed] = useState(false)
  const [expanded, setExpanded] = useState(false)
  const [search, setSearch] = useState("")

  const filtered = useMemo(() => {
    if (!search) return facet.values
    const needle = search.toLowerCase()
    return facet.values.filter((entry) => entry.value.toLowerCase().includes(needle))
  }, [facet.values, search])

  // Selected values stay visible even beyond the maxVisible cut.
  const visible = expanded
    ? filtered
    : filtered.filter((entry, index) => index < maxVisible || selected.includes(entry.value))
  const hiddenCount = filtered.length - visible.length

  const Chevron = collapsed ? IconChevronRight : IconChevronDown
  return (
    <section className="space-y-1.5">
      <button
        type="button"
        className="flex w-full items-center gap-1 text-xs font-medium text-muted-foreground uppercase"
        onClick={() => setCollapsed((prev) => !prev)}
        aria-expanded={!collapsed}
      >
        <Chevron className="size-3.5" />
        {facet.label}
        {selected.length > 0 ? (
          <span className="ml-auto rounded-full bg-muted px-1.5 normal-case tabular-nums">
            {selected.length}
          </span>
        ) : null}
      </button>
      {collapsed ? null : (
        <div className="space-y-0.5 pl-1">
          {facet.searchable && facet.values.length > maxVisible ? (
            <Input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder={`Filter ${facet.label.toLowerCase()}`}
              className="mb-1 h-7 text-xs"
            />
          ) : null}
          {visible.map((entry) => {
            const checked = selected.includes(entry.value)
            return (
              <label
                key={entry.value}
                className={cn(
                  "flex cursor-pointer items-center gap-2 rounded px-1 py-0.5 text-sm hover:bg-muted/60",
                  checked && "font-medium"
                )}
              >
                <Checkbox
                  checked={checked}
                  onCheckedChange={() => onToggle(facet.dimension, entry.value)}
                  className="size-3.5"
                />
                {facet.serviceDots ? <ServiceDot name={entry.value} /> : null}
                <span className="min-w-0 flex-1 truncate">{entry.value}</span>
                <span className="text-xs text-muted-foreground tabular-nums">
                  {entry.count.toLocaleString()}
                </span>
              </label>
            )
          })}
          {hiddenCount > 0 ? (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-6 px-1 text-xs text-muted-foreground"
              onClick={() => setExpanded(true)}
            >
              Show {hiddenCount} more
            </Button>
          ) : null}
          {expanded && filtered.length > maxVisible ? (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-6 px-1 text-xs text-muted-foreground"
              onClick={() => setExpanded(false)}
            >
              Show less
            </Button>
          ) : null}
        </div>
      )}
    </section>
  )
}

/** Sidebar of facet sections. `selections` maps dimension to selected values.
 * Parent owns state (URL search params); this component is fully controlled. */
export function FacetSidebar({
  facets,
  selections,
  onToggle,
  onClear,
  className,
}: {
  facets: Facet[]
  selections: Record<string, string[]>
  onToggle: (dimension: string, value: string) => void
  onClear?: () => void
  className?: string
}) {
  const anySelected = Object.values(selections).some((values) => values.length > 0)
  return (
    <aside className={cn("w-56 shrink-0 space-y-4", className)}>
      {onClear && anySelected ? (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-6 px-1 text-xs"
          onClick={onClear}
        >
          <IconX className="size-3.5" />
          Clear all filters
        </Button>
      ) : null}
      {facets.map((facet) => (
        <FacetSection
          key={facet.dimension}
          facet={facet}
          selected={selections[facet.dimension] ?? []}
          onToggle={onToggle}
        />
      ))}
    </aside>
  )
}

/** URL codec for facet selections: `dim:value` pairs, comma-joined. */
export function facetSelectionsToParam(selections: Record<string, string[]>): string | undefined {
  const parts = Object.entries(selections)
    .flatMap(([dimension, values]) =>
      values.map((value) => `${dimension}:${encodeURIComponent(value)}`)
    )
    .sort()
  return parts.length > 0 ? parts.join(",") : undefined
}

export function facetSelectionsFromParam(raw: string | undefined): Record<string, string[]> {
  if (!raw) return {}
  const selections: Record<string, string[]> = {}
  for (const part of raw.split(",")) {
    const separator = part.indexOf(":")
    if (separator <= 0) continue
    const dimension = part.slice(0, separator)
    const value = decodeURIComponent(part.slice(separator + 1))
    if (!value) continue
    const existing = selections[dimension] ?? []
    if (!existing.includes(value)) existing.push(value)
    selections[dimension] = existing
  }
  return selections
}

export function toggleFacetValue(
  selections: Record<string, string[]>,
  dimension: string,
  value: string
): Record<string, string[]> {
  const current = selections[dimension] ?? []
  const next = current.includes(value)
    ? current.filter((entry) => entry !== value)
    : [...current, value]
  const result = { ...selections }
  if (next.length === 0) {
    delete result[dimension]
  } else {
    result[dimension] = next
  }
  return result
}
