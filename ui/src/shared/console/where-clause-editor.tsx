import { useEffect, useMemo, useRef, useState } from "react"
import { IconFilter, IconX } from "@tabler/icons-react"

import { Button } from "@/components/ui/button"
import { Kbd } from "@/components/ui/kbd"
import {
  parseWhereClause,
  serializeWhereClause,
  WHERE_OPS,
  type WhereFilter,
} from "@/lib/where-clause"
import { cn } from "@/lib/utils"

/** Plan 164 where-clause editor (preliminary). Monospace input, live parse
 * with inline error, autocomplete over keys/operators/values, ⌘Enter apply.
 * Controlled: parent owns the applied filters (URL search params) and
 * supplies autocomplete sources (field_keys / field_stats.topValues). */
export function WhereClauseEditor({
  filters,
  onApply,
  keySuggestions = [],
  valueSuggestionsFor,
  className,
  autoFocus,
}: {
  filters: WhereFilter[]
  onApply: (filters: WhereFilter[]) => void
  keySuggestions?: string[]
  /** Top values for a key, from field_stats; sync cache lookup. */
  valueSuggestionsFor?: (key: string) => string[]
  className?: string
  autoFocus?: boolean
}) {
  const [text, setText] = useState(() => serializeWhereClause(filters))
  const [open, setOpen] = useState(false)
  const [highlightIndex, setHighlightIndex] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    setText(serializeWhereClause(filters))
  }, [filters])

  const parsed = useMemo(() => parseWhereClause(text), [text])
  const error = parsed.ok ? null : parsed.error

  // Autocomplete context: what token is being typed at the end of the input.
  const suggestions = useMemo(() => {
    const trimmed = text.replace(/\s+$/, "")
    const tokens = trimmed === "" ? [] : trimmed.split(/\s+/)
    const endsWithSpace = text !== trimmed
    const position = tokens.length % 4 // key, op, value, AND
    const current = endsWithSpace ? "" : (tokens[tokens.length - 1] ?? "")
    const slot = endsWithSpace ? position : Math.max(0, position - 1) % 4
    let pool: string[] = []
    if (slot === 0) {
      pool = keySuggestions
    } else if (slot === 1) {
      pool = [...WHERE_OPS]
    } else if (slot === 2) {
      const key = tokens[tokens.length - (endsWithSpace ? 2 : 3)]
      pool = key && valueSuggestionsFor ? valueSuggestionsFor(key) : []
    } else {
      pool = ["AND"]
    }
    const needle = current.toLowerCase()
    return pool
      .filter((entry) => entry.toLowerCase().startsWith(needle))
      .slice(0, 8)
      .map((entry) => ({ entry, current }))
  }, [text, keySuggestions, valueSuggestionsFor])

  const accept = (entry: string, current: string) => {
    const base = current ? text.slice(0, text.length - current.length) : text
    const quoted = /\s/.test(entry) ? `"${entry}"` : entry
    setText(`${base}${quoted} `)
    setOpen(true)
    setHighlightIndex(0)
    inputRef.current?.focus()
  }

  const apply = () => {
    if (parsed.ok) {
      onApply(parsed.filters)
      setOpen(false)
    }
  }

  return (
    <div className={cn("space-y-1", className)}>
      <div className="relative">
        <IconFilter className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
        <input
          ref={inputRef}
          value={text}
          autoFocus={autoFocus}
          spellCheck={false}
          placeholder='service = "checkout" AND http.request.method != "GET"'
          aria-label="Where clause"
          aria-invalid={error != null}
          className={cn(
            "h-8 w-full rounded-md border bg-transparent pr-20 pl-8 font-mono text-xs outline-none focus:ring-1 focus:ring-ring",
            error && text.trim() !== "" && "border-destructive"
          )}
          onChange={(event) => {
            setText(event.target.value)
            setOpen(true)
            setHighlightIndex(0)
          }}
          onKeyDown={(event) => {
            if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
              event.preventDefault()
              apply()
              return
            }
            if (event.key === " " && event.ctrlKey) {
              event.preventDefault()
              setOpen(true)
              return
            }
            if (!open || suggestions.length === 0) {
              if (event.key === "Enter") {
                event.preventDefault()
                apply()
              }
              return
            }
            if (event.key === "ArrowDown") {
              event.preventDefault()
              setHighlightIndex((prev) => (prev + 1) % suggestions.length)
            } else if (event.key === "ArrowUp") {
              event.preventDefault()
              setHighlightIndex(
                (prev) => (prev - 1 + suggestions.length) % suggestions.length
              )
            } else if (event.key === "Tab" || event.key === "Enter") {
              event.preventDefault()
              const chosen = suggestions[highlightIndex]
              if (chosen) accept(chosen.entry, chosen.current)
            } else if (event.key === "Escape") {
              setOpen(false)
            }
          }}
          onBlur={() => setTimeout(() => setOpen(false), 150)}
        />
        <span className="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2">
          <Kbd>⌘⏎</Kbd>
        </span>
        {open && suggestions.length > 0 ? (
          <ul
            role="listbox"
            aria-label="Autocomplete suggestions"
            className="absolute z-50 mt-1 max-h-56 w-full overflow-auto rounded-md border bg-popover p-1 font-mono text-xs shadow-md"
          >
            {suggestions.map((suggestion, index) => (
              <li
                key={suggestion.entry}
                role="option"
                aria-selected={index === highlightIndex}
                className={cn(
                  "cursor-pointer rounded px-2 py-1",
                  index === highlightIndex && "bg-muted"
                )}
                onMouseDown={(event) => {
                  event.preventDefault()
                  accept(suggestion.entry, suggestion.current)
                }}
              >
                {suggestion.entry}
              </li>
            ))}
          </ul>
        ) : null}
      </div>
      {error && text.trim() !== "" ? (
        <p className="font-mono text-xs text-destructive">
          {error.message} (at {error.start})
        </p>
      ) : null}
    </div>
  )
}

/** Applied where-clause shown as removable chips. */
export function WhereClauseChips({
  filters,
  onRemove,
  className,
}: {
  filters: WhereFilter[]
  onRemove: (index: number) => void
  className?: string
}) {
  if (filters.length === 0) return null
  return (
    <div className={cn("flex flex-wrap items-center gap-1.5", className)}>
      {filters.map((filter, index) => (
        <span
          key={`${filter.key}-${filter.op}-${filter.value}-${index}`}
          className="inline-flex items-center gap-1 rounded-full border bg-muted/50 py-0.5 pr-1 pl-2 font-mono text-xs"
        >
          {filter.key} {filter.op} {filter.value}
          <Button
            type="button"
            variant="ghost"
            size="icon-xs"
            className="size-4"
            aria-label={`Remove filter ${filter.key}`}
            onClick={() => onRemove(index)}
          >
            <IconX className="size-3" />
          </Button>
        </span>
      ))}
    </div>
  )
}
