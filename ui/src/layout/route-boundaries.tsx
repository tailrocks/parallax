import { useState } from "react"
import type { ErrorComponentProps } from "@tanstack/react-router"
import { IconAlertTriangleFilled, IconKey } from "@tabler/icons-react"

import { Button } from "@/components/ui/button"
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import { Spinner } from "@/components/ui/spinner"
import { setApiToken } from "@/platform/auth/api-token"

export { RouteNotFoundPanel } from "@/shared/route-not-found"

function safeErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message
  }
  if (typeof error === "string" && error) {
    return error
  }
  return "Route failed before Parallax could load this surface."
}

/** The API rejected the request as unauthenticated (plan 109 bearer token). */
export function isUnauthorizedError(error: unknown): boolean {
  if (typeof error === "object" && error !== null && "status" in error) {
    if ((error as { status: unknown }).status === 401) return true
  }
  return /401|unauthorized/i.test(safeErrorMessage(error))
}

/** Token entry for token-protected servers; saving reloads so every query re-authenticates. */
function ApiTokenPanel({ onRetry }: { onRetry: () => void }) {
  const [token, setToken] = useState("")
  return (
    <Empty className="max-w-3xl">
      <EmptyHeader>
        <EmptyMedia
          variant="icon"
          className="bg-amber-500/10 text-amber-600 shadow-[var(--custom-shadow-amber)] dark:bg-amber-500/15 dark:text-amber-300"
        >
          <IconKey />
        </EmptyMedia>
        <EmptyTitle>This Parallax server requires an API token</EmptyTitle>
        <EmptyDescription>
          The local API answered 401. Paste the server&rsquo;s{" "}
          <code className="rounded-md bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">
            api_token
          </code>{" "}
          to unlock the UI. It is stored only in this browser.
        </EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <form
          className="flex max-w-md items-center gap-2"
          onSubmit={(event) => {
            event.preventDefault()
            setApiToken(token)
            window.location.reload()
          }}
        >
          <Input
            type="password"
            value={token}
            placeholder="API token"
            onChange={(event) => setToken(event.target.value)}
            autoFocus
          />
          <Button type="submit" disabled={!token.trim()}>
            Save
          </Button>
          <Button type="button" variant="ghost" onClick={onRetry}>
            Retry
          </Button>
        </form>
      </EmptyContent>
    </Empty>
  )
}

export function RouteErrorPanel({ error, reset }: ErrorComponentProps) {
  if (isUnauthorizedError(error)) {
    return <ApiTokenPanel onRetry={reset} />
  }
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
          The app shell is running, but this route could not load data from the local API. Verify
          the server at{" "}
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
