import { Link } from "@tanstack/react-router"
import type { Icon } from "@tabler/icons-react"
import { IconChevronRight } from "@tabler/icons-react"

import { cn } from "@/lib/utils"

export type PageHeaderBack = {
  href: string
  label: string
  icon: Icon
  iconClassName?: string
}

/** Product-neutral page chrome. Typed title/back/actions only — no nav registry. */
export function PageHeader({
  title,
  titleLeading,
  titleTrailing,
  description,
  actions,
  back,
  icon: TitleIcon,
  iconClassName,
}: {
  title: string
  titleLeading?: React.ReactNode
  titleTrailing?: React.ReactNode
  description?: React.ReactNode
  actions?: React.ReactNode
  back?: PageHeaderBack
  icon?: Icon
  iconClassName?: string
}) {
  const BackIcon = back?.icon

  return (
    <div className="flex flex-wrap items-end justify-between gap-4">
      <div className="flex min-w-0 flex-col gap-1.5">
        {back && BackIcon ? (
          <h1 className="flex items-center gap-1.5 text-base font-medium tracking-tight">
            <Link
              to={back.href}
              className="flex shrink-0 items-center gap-2 text-muted-foreground transition-colors hover:text-foreground"
            >
              <BackIcon className={cn("size-4.5 shrink-0", back.iconClassName)} />
              {back.label}
            </Link>
            <IconChevronRight className="size-4 shrink-0 stroke-[1.5px] text-muted-foreground/50" />
            {titleLeading}
            <span className="truncate">{title}</span>
            {titleTrailing}
          </h1>
        ) : (
          <h1 className="flex items-center gap-2 text-base font-medium tracking-tight">
            {TitleIcon ? <TitleIcon className={cn("size-4.5 shrink-0", iconClassName)} /> : null}
            {titleLeading}
            <span className="truncate">{title}</span>
            {titleTrailing}
          </h1>
        )}
        {description ? <p className="text-sm text-muted-foreground">{description}</p> : null}
      </div>
      {actions ? <div className="flex items-center gap-2">{actions}</div> : null}
    </div>
  )
}
