import { IconLayoutGridFilled, IconTableFilled } from "@tabler/icons-react"
import { motion } from "motion/react"

import { cn } from "@/lib/utils"

export type ViewMode = "cards" | "table"

export function ViewToggle({
  value,
  onChange,
}: {
  value: ViewMode
  onChange: (value: ViewMode) => void
}) {
  const options = [
    { value: "cards" as const, icon: IconLayoutGridFilled, label: "Cards" },
    { value: "table" as const, icon: IconTableFilled, label: "Table" },
  ]
  return (
    <div className="inline-flex rounded-full bg-muted p-1">
      {options.map((option) => {
        const Icon = option.icon
        const active = value === option.value
        return (
          <button
            key={option.value}
            type="button"
            className={cn("relative flex size-7 items-center justify-center rounded-full", active ? "text-foreground" : "text-muted-foreground")}
            onClick={() => onChange(option.value)}
          >
            {active ? <motion.span layoutId="view-toggle" className="absolute inset-0 rounded-full bg-background shadow-(--custom-shadow)" /> : null}
            <Icon className="relative z-10 size-4" />
            <span className="sr-only">{option.label}</span>
          </button>
        )
      })}
    </div>
  )
}
