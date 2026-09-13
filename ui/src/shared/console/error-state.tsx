import { IconAlertTriangleFilled, IconRefresh } from "@tabler/icons-react"

import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"

/** Section-level load failure. Companion to `EmptyState`: same voice, plus
 * retry. Announced via `role=alert`. Pages must use this (or `SectionError`)
 * instead of bare destructive text. */
export function ErrorState({
  title,
  message,
  onRetry,
  className,
}: {
  title: string
  message?: React.ReactNode
  onRetry?: () => void
  className?: string
}) {
  return (
    <Empty className={cn("min-h-48 border-destructive/30", className)} role="alert">
      <EmptyHeader>
        <EmptyMedia>
          <IconAlertTriangleFilled className="size-8 text-destructive opacity-80" />
        </EmptyMedia>
        <EmptyTitle>{title}</EmptyTitle>
        {message ? <EmptyDescription>{message}</EmptyDescription> : null}
      </EmptyHeader>
      {onRetry ? (
        <EmptyContent>
          <Button type="button" variant="outline" size="sm" onClick={onRetry}>
            <IconRefresh />
            Retry
          </Button>
        </EmptyContent>
      ) : (
        <EmptyContent />
      )}
    </Empty>
  )
}

/** Compact inline failure for user-triggered fetches (load-older, live
 * reconnect, saved-view IO). One row: icon + message + retry. */
export function SectionError({
  message,
  onRetry,
  className,
}: {
  message: string
  onRetry?: () => void
  className?: string
}) {
  return (
    <p
      role="alert"
      className={cn("flex flex-wrap items-center gap-2 text-sm text-destructive", className)}
    >
      <IconAlertTriangleFilled className="size-4 shrink-0" aria-hidden />
      <span className="min-w-0 flex-1">{message}</span>
      {onRetry ? (
        <Button type="button" variant="ghost" size="sm" onClick={onRetry}>
          <IconRefresh />
          Retry
        </Button>
      ) : null}
    </p>
  )
}
