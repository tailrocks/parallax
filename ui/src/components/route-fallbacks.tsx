import type { ErrorComponentProps } from "@tanstack/react-router"
import { AlertTriangleIcon, LoaderCircleIcon, MapIcon } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Skeleton } from "@/components/ui/skeleton"

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
    <section className="parallax-panel parallax-glow-border max-w-3xl p-6">
      <div className="flex items-start gap-4">
        <span className="grid size-10 shrink-0 place-items-center rounded-2xl bg-[color-mix(in_srgb,var(--brand-rose),transparent_16%)] text-white shadow-[0_0_24px_color-mix(in_srgb,var(--brand-rose),transparent_72%)]">
          <AlertTriangleIcon />
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
            Surface unavailable
          </p>
          <h1 className="mt-1 text-2xl font-semibold tracking-tight">
            Parallax API did not answer
          </h1>
          <p className="mt-2 max-w-xl text-sm text-muted-foreground">
            The app shell is running, but this route could not load data from
            the local API. Verify the server at{" "}
            <code className="rounded-md bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">
              127.0.0.1:4000
            </code>
            .
          </p>
          <pre className="mt-4 overflow-auto rounded-xl border border-border/70 bg-background/80 p-3 font-mono text-xs text-(--brand-rose)">
            {safeErrorMessage(error)}
          </pre>
          <Button className="mt-4 rounded-full" size="sm" onClick={reset}>
            Retry route
          </Button>
        </div>
      </div>
    </section>
  )
}

export function RoutePendingPanel() {
  return (
    <section className="grid gap-4">
      <div className="parallax-panel p-6">
        <div className="flex items-center gap-3">
          <span className="grid size-9 place-items-center rounded-2xl bg-[color-mix(in_srgb,var(--brand-blue),transparent_18%)] text-white">
            <LoaderCircleIcon className="animate-spin" />
          </span>
          <div>
            <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
              Loading surface
            </p>
            <h1 className="text-xl font-semibold tracking-tight">
              Building latest context
            </h1>
          </div>
        </div>
      </div>
      <div className="grid gap-4 md:grid-cols-3">
        <Skeleton className="h-32 rounded-2xl" />
        <Skeleton className="h-32 rounded-2xl" />
        <Skeleton className="h-32 rounded-2xl" />
      </div>
    </section>
  )
}

export function RouteNotFoundPanel() {
  return (
    <section className="parallax-panel max-w-2xl p-6">
      <div className="flex items-start gap-4">
        <span className="grid size-10 shrink-0 place-items-center rounded-2xl bg-[color-mix(in_srgb,var(--brand-violet),transparent_16%)] text-white">
          <MapIcon />
        </span>
        <div>
          <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
            Missing route
          </p>
          <h1 className="mt-1 text-2xl font-semibold tracking-tight">
            Nothing is mounted here
          </h1>
          <p className="mt-2 text-sm text-muted-foreground">
            Pick a Parallax surface from the navigation.
          </p>
        </div>
      </div>
    </section>
  )
}
