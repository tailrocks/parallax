import { useMemo, useState } from "react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { cn } from "@/lib/utils"

export function Combobox({
  value,
  options,
  placeholder = "Search",
  onChange,
  className,
}: {
  value: string
  options: string[]
  placeholder?: string
  onChange: (value: string) => void
  className?: string
}) {
  const [query, setQuery] = useState(value)
  const matches = useMemo(() => {
    const lower = query.toLowerCase()
    return options
      .filter((option) => option.toLowerCase().includes(lower))
      .slice(0, 8)
  }, [options, query])

  return (
    <div className={cn("flex min-w-56 flex-col gap-1", className)}>
      <Input
        value={query}
        onChange={(event) => {
          setQuery(event.target.value)
          onChange(event.target.value)
        }}
        placeholder={placeholder}
        className="h-8 font-mono text-xs"
      />
      {query && matches.length > 0 ? (
        <div className="rounded-lg border bg-popover p-1 shadow-(--custom-shadow)">
          {matches.map((option) => (
            <Button
              key={option}
              type="button"
              variant="ghost"
              size="xs"
              className="w-full justify-start font-mono"
              onClick={() => {
                setQuery(option)
                onChange(option)
              }}
            >
              {option}
            </Button>
          ))}
        </div>
      ) : null}
    </div>
  )
}
