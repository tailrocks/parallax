import type { Icon } from "@tabler/icons-react"

import { cn } from "@/lib/utils"

export function NavIcon({
  icon: OutlineIcon,
  activeIcon: ActiveIcon,
  active,
  className,
}: {
  icon: Icon
  activeIcon: Icon
  active: boolean
  className?: string | undefined
}) {
  return (
    <span
      className={cn(
        "grid size-4.5 place-items-center [&_svg]:size-full!",
        className
      )}
    >
      <OutlineIcon
        aria-hidden="true"
        className={cn(
          "transition-opacity duration-100 ease-in-out [grid-area:1/1]",
          active ? "opacity-0" : "opacity-100"
        )}
      />
      <ActiveIcon
        aria-hidden="true"
        className={cn(
          "transition-opacity duration-100 ease-in-out [grid-area:1/1]",
          active ? "opacity-100" : "opacity-0"
        )}
      />
    </span>
  )
}
