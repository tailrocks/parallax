import type { ErrorComponentProps } from "@tanstack/react-router"
import {
  IconAlertTriangleFilled,
  IconMapQuestion,
} from "@tabler/icons-react"

import { Button } from "@/components/ui/button"
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty"
import { Skeleton } from "@/components/ui/skeleton"
import { Spinner } from "@/components/ui/spinner"

function safeErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message
  }
  if (typeof error === "string" && error) {
    return error
  }
  return "Route failed before Parallax could load this surface."
}

export function RouteErrorPanel({ error, reset }: ErrorComponentProps) {
  return (
    <Empty className="max-w-3xl">
      <EmptyHeader>
        <EmptyMedia
          variant="icon"
          className="bg-rose-500/10 text-rose-600 shadow-[var(--custom-shadow-rose)] dark:bg-rose-500/15 dark:text-rose-300"
        >
          <IconAlertTriangleFilled />
        </EmptyMedia>
        <EmptyTitle>Parallax API did not answer</EmptyTitle>
        <EmptyDescription>
          The app shell is running, but this route could not load data from the
          local API. Verify the server at{" "}
          <code className="rounded-md bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">
            127.0.0.1:4000
          </code>
          .
        </EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <pre className="max-h-48 w-full overflow-auto rounded-2xl bg-muted p-3 text-left font-mono text-xs text-rose-600 dark:text-rose-300">
          {safeErrorMessage(error)}
        </pre>
        <Button variant="outline" onClick={reset}>
          Retry route
        </Button>
      </EmptyContent>
    </Empty>
  )
}

export function RoutePendingPanel() {
  return (
    <section className="grid gap-6">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div className="grid gap-2">
          <Skeleton className="h-5 w-48" />
          <Skeleton className="h-4 w-80 max-w-full" />
        </div>
        <Spinner />
      </div>
      <div className="grid gap-4 md:grid-cols-3">
        <Skeleton className="h-32" />
        <Skeleton className="h-32" />
        <Skeleton className="h-32" />
      </div>
    </section>
  )
}

export function RouteNotFoundPanel() {
  return (
    <Empty className="max-w-2xl">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <IconMapQuestion />
        </EmptyMedia>
        <EmptyTitle>Nothing is mounted here</EmptyTitle>
        <EmptyDescription>
          Pick a Parallax surface from the navigation.
        </EmptyDescription>
      </EmptyHeader>
    </Empty>
  )
}
