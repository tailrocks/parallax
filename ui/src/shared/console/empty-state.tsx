import type { Icon } from "@tabler/icons-react"

import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty"
import { cn } from "@/lib/utils"

export function EmptyState({
  title,
  description,
  icon: Icon,
  className,
}: {
  title: string
  description?: React.ReactNode
  icon?: Icon
  className?: string
}) {
  return (
    <Empty className={cn("min-h-64", className)}>
      <EmptyHeader>
        {Icon ? (
          <EmptyMedia>
            <Icon className="size-8 opacity-40" />
          </EmptyMedia>
        ) : null}
        <EmptyTitle>{title}</EmptyTitle>
        {description ? (
          <EmptyDescription>{description}</EmptyDescription>
        ) : null}
      </EmptyHeader>
      <EmptyContent />
    </Empty>
  )
}
