import { IconDeviceLaptop, IconMoon, IconSun } from "@tabler/icons-react"
import { motion } from "motion/react"
import { useTheme } from "next-themes"
import { useEffect, useId, useState } from "react"

import { cn } from "@/lib/utils"

function useMounted() {
  const [mounted, setMounted] = useState(false)
  useEffect(() => setMounted(true), [])
  return mounted
}

const MORPH = { type: "spring", stiffness: 400, damping: 38 } as const

const THEME_OPTIONS = [
  { value: "system", label: "System", icon: IconDeviceLaptop },
  { value: "light", label: "Light", icon: IconSun },
  { value: "dark", label: "Dark", icon: IconMoon },
] as const

export function ThemeSwitcher() {
  const { theme, setTheme } = useTheme()
  const mounted = useMounted()
  const pillId = useId()

  return (
    <div className="inline-flex items-center rounded-full bg-input/20 p-1">
      {THEME_OPTIONS.map((opt) => {
        const active = mounted && theme === opt.value
        const Icon = opt.icon

        return (
          <button
            key={opt.value}
            type="button"
            aria-label={opt.label}
            aria-pressed={active}
            onClick={() => setTheme(opt.value)}
            className={cn(
              "relative flex size-7 cursor-pointer items-center justify-center rounded-full transition-colors",
              active ? "text-foreground" : "text-muted-foreground/60 hover:text-foreground"
            )}
          >
            {active ? (
              <motion.span
                layoutId={pillId}
                transition={MORPH}
                className="absolute inset-0 rounded-full bg-muted shadow-(--custom-shadow) dark:bg-input/50"
              />
            ) : null}
            <Icon className="relative z-10 size-4" />
          </button>
        )
      })}
    </div>
  )
}
