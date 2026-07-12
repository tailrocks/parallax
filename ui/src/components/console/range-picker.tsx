import { useState } from "react"
import { IconCalendarEventFilled, IconChevronDown } from "@tabler/icons-react"
import type { DateRange } from "react-day-picker"

import { RANGE_PRESETS, customRange, formatRangeLabel } from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { Button } from "@/components/ui/button"
import { Calendar } from "@/components/ui/calendar"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover"

export function RangePicker({
  value,
  onChange,
}: {
  value: ResolvedRange
  onChange: (value: ResolvedRange) => void
}) {
  const [draft, setDraft] = useState<DateRange | undefined>(undefined)
  const defaultMonth = new Date(Number(BigInt(value.toNanos) / 1_000_000n))
  const selectDraft = (next: DateRange | undefined) => {
    setDraft(next)
    if (!next?.from || !next.to) return
    const from = new Date(
      next.from.getFullYear(),
      next.from.getMonth(),
      next.from.getDate(),
      0,
      0,
      0,
      0
    )
    const to = new Date(
      next.to.getFullYear(),
      next.to.getMonth(),
      next.to.getDate(),
      23,
      59,
      59,
      999
    )
    onChange(
      customRange(
        (BigInt(from.getTime()) * 1_000_000n).toString(),
        (BigInt(to.getTime()) * 1_000_000n).toString()
      )
    )
    setDraft(undefined)
  }

  return (
    <Popover>
      <PopoverTrigger render={<Button variant="outline" />}>
        <IconCalendarEventFilled />
        {formatRangeLabel(value)}
        <IconChevronDown />
      </PopoverTrigger>
      <PopoverContent align="end" className="flex w-auto gap-3 p-3">
        <div className="grid w-40 gap-1">
          {RANGE_PRESETS.map((preset) => (
            <Button
              key={preset.key}
              type="button"
              variant={value.key === preset.key ? "secondary" : "ghost"}
              size="sm"
              onClick={() =>
                onChange({
                  key: preset.key,
                  fromNanos: (
                    BigInt(Date.now() - preset.ms) * 1_000_000n
                  ).toString(),
                  toNanos: (BigInt(Date.now()) * 1_000_000n).toString(),
                })
              }
            >
              {preset.label}
            </Button>
          ))}
        </div>
        <Calendar
          mode="range"
          numberOfMonths={2}
          disabled={{ after: new Date() }}
          defaultMonth={defaultMonth}
          selected={draft}
          onSelect={selectDraft}
        />
      </PopoverContent>
    </Popover>
  )
}
