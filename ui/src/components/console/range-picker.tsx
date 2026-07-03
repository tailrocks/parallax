import { IconCalendarEventFilled, IconChevronDown } from "@tabler/icons-react"

import { RANGE_PRESETS, formatRangeLabel  } from "@/lib/range"
import type {ResolvedRange} from "@/lib/range";
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
                  fromNanos: ((BigInt(Date.now() - preset.ms)) * 1_000_000n).toString(),
                  toNanos: (BigInt(Date.now()) * 1_000_000n).toString(),
                })
              }
            >
              {preset.label}
            </Button>
          ))}
        </div>
        <Calendar mode="range" numberOfMonths={2} disabled={{ after: new Date() }} />
      </PopoverContent>
    </Popover>
  )
}
