import { IconChevronDown, IconChevronUp, IconSearch, IconX } from "@tabler/icons-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { cn } from "@/lib/utils"

export type SortDirection = "asc" | "desc"
export type SortParam = `${string}:${SortDirection}`

export function parseSortParam(value?: string | null) {
  if (!value) return null
  const [key, direction] = value.split(":")
  if (!key || (direction !== "asc" && direction !== "desc")) return null
  return { key, direction }
}

export function cycleSortParam(current: string | undefined, key: string) {
  const parsed = parseSortParam(current)
  if (!parsed || parsed.key !== key) return `${key}:desc`
  if (parsed.direction === "desc") return `${key}:asc`
  return undefined
}

export function sortRows<T>(
  rows: T[],
  sort: string | undefined,
  accessors: Record<string, (row: T) => string | number | null | undefined>
) {
  const parsed = parseSortParam(sort)
  if (!parsed) return rows
  const accessor = accessors[parsed.key]
  if (!accessor) return rows
  return rows
    .map((row, index) => ({ row, index }))
    .sort((a, b) => {
      const av = accessor(a.row)
      const bv = accessor(b.row)
      if (av == null && bv == null) return a.index - b.index
      if (av == null) return 1
      if (bv == null) return -1
      const result = av < bv ? -1 : av > bv ? 1 : a.index - b.index
      return parsed.direction === "asc" ? result : -result
    })
    .map(({ row }) => row)
}

export function pageWindow(page: number, total: number) {
  if (total <= 7) return Array.from({ length: total }, (_, index) => index + 1)
  const window = new Set([1, total, page - 1, page, page + 1])
  return Array.from(window)
    .filter((value) => value >= 1 && value <= total)
    .sort((a, b) => a - b)
    .flatMap((value, index, values) =>
      index > 0 && value - values[index - 1]! > 1 ? ["..." as const, value] : [value]
    )
}

export function Toolbar({ className, ...props }: React.ComponentProps<"div">) {
  return <div className={cn("flex flex-wrap items-center gap-2", className)} {...props} />
}

export function SearchInput({
  value,
  onChange,
  placeholder = "Search",
}: {
  value: string
  onChange: (value: string) => void
  placeholder?: string
}) {
  return (
    <div className="relative w-56">
      <IconSearch className="pointer-events-none absolute top-1/2 left-3 size-3.5 -translate-y-1/2 text-muted-foreground" />
      <Input
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        className="h-8 rounded-full px-8 dark:bg-input/20"
      />
      {value ? (
        <Button
          type="button"
          variant="ghost"
          size="icon-xs"
          className="absolute top-1/2 right-1 -translate-y-1/2"
          onClick={() => onChange("")}
        >
          <IconX />
          <span className="sr-only">Clear search</span>
        </Button>
      ) : null}
    </div>
  )
}

export function ToggleChip({
  active,
  children,
  ...props
}: React.ComponentProps<typeof Button> & { active?: boolean }) {
  return (
    <Button
      variant="outline"
      size="sm"
      className={cn(
        "rounded-full",
        active && "bg-rose-500/10 text-rose-600 dark:bg-rose-500/15 dark:text-rose-400"
      )}
      {...props}
    >
      {children}
    </Button>
  )
}

export function FilterSelect({
  value,
  onChange,
  options,
  placeholder,
}: {
  value?: string
  onChange: (value: string | undefined) => void
  options: Array<{ value: string; label: string }>
  placeholder: string
}) {
  return (
    <Select
      value={value ?? "__all"}
      onValueChange={(next) => onChange(next == null || next === "__all" ? undefined : next)}
    >
      <SelectTrigger size="sm" className="rounded-full" aria-label={placeholder}>
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="__all">All</SelectItem>
        {options.map((option) => (
          <SelectItem key={option.value} value={option.value}>
            {option.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}

export function ClearFiltersButton({ onClick }: { onClick: () => void }) {
  return (
    <Button variant="ghost" size="sm" onClick={onClick}>
      <IconX />
      Clear
    </Button>
  )
}

export function SortableHead({
  sort,
  sortKey,
  onSort,
  children,
}: {
  sort?: string
  sortKey: string
  onSort: (next: string | undefined) => void
  children: React.ReactNode
}) {
  const parsed = parseSortParam(sort)
  const active = parsed?.key === sortKey
  const Icon = parsed?.direction === "asc" ? IconChevronUp : IconChevronDown
  return (
    <button
      type="button"
      className="flex w-full items-center gap-1 text-left"
      onClick={() => onSort(cycleSortParam(sort, sortKey))}
    >
      {children}
      <Icon className={cn("size-3.5", active ? "opacity-100" : "opacity-30")} />
    </button>
  )
}
