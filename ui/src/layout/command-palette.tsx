import { useNavigate } from "@tanstack/react-router"
import { ServiceDot } from "@/components/console/service-dot"
import {
  IconAffiliate,
  IconBug,
  IconHash,
  IconSearch,
  IconServer,
  IconTerminal2,
} from "@tabler/icons-react"
import { useEffect, useMemo, useState } from "react"

import { NavIcon } from "@/layout/nav-icon"
import { nav } from "@/shared/navigation"
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandShortcut,
} from "@/components/ui/command"
import { formatRelative } from "@/lib/format"
import { graphql } from "@/lib/api"
import { guessId } from "@/features/quick-navigation"
import type { IdGuess } from "@/features/quick-navigation"

type StaticRoute =
  | "/"
  | "/issues"
  | "/traces"
  | "/ecosystem"
  | "/logs"
  | "/services"
  | "/invocations"
  | "/dashboards"
  | "/investigations"
  | "/sql"

interface RecentTrace {
  traceId: string
  rootName: string
  service: string
  startNanos: string
  hasError: boolean
}

interface RecentRun {
  invocationId: string
  command: string | null
  status: string
  startedAtNanos: string
  endedAtNanos: string | null
  errorCount: number
}

type LoadState<T> =
  | { status: "idle"; data: T }
  | { status: "loading"; data: T }
  | { status: "ready"; data: T }
  | { status: "error"; data: T }

const emptyRecent = { traces: [] as RecentTrace[], runs: [] as RecentRun[] }

function isCommandK(event: KeyboardEvent) {
  return event.key.toLowerCase() === "k" && (event.metaKey || event.ctrlKey)
}

function pageRoute(href: string): StaticRoute | null {
  switch (href) {
    case "/":
    case "/issues":
    case "/traces":
    case "/ecosystem":
    case "/logs":
    case "/services":
    case "/invocations":
    case "/dashboards":
    case "/investigations":
    case "/sql":
      return href
    default:
      return null
  }
}

function idLabel(guess: IdGuess) {
  switch (guess.kind) {
    case "trace":
      return `Open trace ${guess.id}`
    case "invocation":
      return `Open invocation ${guess.id}`
    case "fingerprint":
      return `Open issue ${guess.id}`
    case "span-in-trace":
      return `Search traces for span ${guess.id}`
  }
}

function idIcon(guess: IdGuess) {
  switch (guess.kind) {
    case "trace":
      return IconAffiliate
    case "invocation":
      return IconTerminal2
    case "fingerprint":
      return IconBug
    case "span-in-trace":
      return IconHash
  }
}

export function CommandPalette({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const navigate = useNavigate()
  const [query, setQuery] = useState("")
  const [services, setServices] = useState<LoadState<string[]>>({
    status: "idle",
    data: [],
  })
  const [recent, setRecent] = useState<LoadState<typeof emptyRecent>>({
    status: "idle",
    data: emptyRecent,
  })
  const idGuesses = useMemo(() => guessId(query), [query])

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (!isCommandK(event)) return
      event.preventDefault()
      onOpenChange(!open)
    }
    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [onOpenChange, open])

  useEffect(() => {
    if (open) return
    setQuery("")
  }, [open])

  useEffect(() => {
    if (!open) return
    const controller = new AbortController()
    setServices({ status: "loading", data: [] })
    void graphql<{ services: string[] }>(
      `
        {
          services
        }
      `,
      {
        signal: controller.signal,
      }
    )
      .then((data) =>
        setServices({
          status: "ready",
          data: data.services.slice(0, 20),
        })
      )
      .catch(() => setServices({ status: "error", data: [] }))
    return () => controller.abort()
  }, [open])

  useEffect(() => {
    if (!open) return
    const controller = new AbortController()
    setRecent({ status: "loading", data: emptyRecent })
    void graphql<{
      tracesPage: { items: RecentTrace[] }
      invocations: RecentRun[]
    }>(
      `
        {
          tracesPage(limit: 5, sort: START_DESC) {
            items {
              traceId
              rootName
              service
              startNanos
              hasError
            }
          }
          invocations(limit: 5) {
            invocationId
            command
            status
            startedAtNanos
            endedAtNanos
            errorCount
          }
        }
      `,
      { signal: controller.signal }
    )
      .then((data) =>
        setRecent({
          status: "ready",
          data: {
            traces: data.tracesPage.items,
            runs: data.invocations,
          },
        })
      )
      .catch(() => setRecent({ status: "error", data: emptyRecent }))
    return () => controller.abort()
  }, [open])

  const close = () => onOpenChange(false)

  const goPage = (href: string) => {
    const route = pageRoute(href)
    if (!route) return
    close()
    void navigate({ to: route, search: {} })
  }

  const goId = (guess: IdGuess) => {
    close()
    switch (guess.kind) {
      case "trace":
        void navigate({
          to: "/traces/$traceId",
          params: { traceId: guess.id },
        })
        break
      case "invocation":
        void navigate({
          to: "/invocations/$invocationId",
          params: { invocationId: guess.id },
        })
        break
      case "fingerprint":
        void navigate({
          to: "/issues/$fingerprint",
          params: { fingerprint: guess.id },
        })
        break
      case "span-in-trace":
        void navigate({ to: "/traces", search: { q: guess.id } })
        break
    }
  }

  return (
    <CommandDialog open={open} onOpenChange={onOpenChange}>
      <CommandInput
        value={query}
        onValueChange={setQuery}
        placeholder="Search pages, services, traces, runs..."
      />
      <CommandList>
        <CommandEmpty>No commands found.</CommandEmpty>

        {idGuesses.length > 0 ? (
          <CommandGroup heading="Id jump">
            {idGuesses.map((guess) => {
              const Icon = idIcon(guess)
              return (
                <CommandItem
                  key={`${guess.kind}-${guess.id}`}
                  value={`${guess.kind} ${guess.id}`}
                  onSelect={() => goId(guess)}
                >
                  <Icon />
                  <span className="min-w-0 truncate">{idLabel(guess)}</span>
                </CommandItem>
              )
            })}
          </CommandGroup>
        ) : null}

        <CommandGroup heading="Pages">
          {nav.map((item) => (
            <CommandItem
              key={item.href}
              value={`page ${item.label} ${item.href}`}
              onSelect={() => goPage(item.href)}
            >
              <NavIcon
                icon={item.icon}
                activeIcon={item.activeIcon}
                active={false}
                className={item.iconClassName}
              />
              <span>{item.label}</span>
              <CommandShortcut>{item.href}</CommandShortcut>
            </CommandItem>
          ))}
        </CommandGroup>

        <CommandGroup heading="Services">
          {services.status === "loading" ? (
            <CommandItem disabled value="services loading">
              <IconServer />
              Services loading
            </CommandItem>
          ) : null}
          {services.status === "error" ? (
            <CommandItem disabled value="services unavailable">
              <IconServer />
              Services unavailable
            </CommandItem>
          ) : null}
          {services.data.map((service) => (
            <CommandItem
              key={service}
              value={`service ${service}`}
              onSelect={() => {
                close()
                void navigate({
                  to: "/services/$service",
                  params: { service },
                })
              }}
            >
              <ServiceDot name={service} />
              <span className="min-w-0 truncate">{service}</span>
            </CommandItem>
          ))}
        </CommandGroup>

        <CommandGroup heading="Recent">
          {recent.status === "loading" ? (
            <CommandItem disabled value="recent loading">
              <IconSearch />
              Recent loading
            </CommandItem>
          ) : null}
          {recent.status === "error" ? (
            <CommandItem disabled value="recent unavailable">
              <IconSearch />
              Recent unavailable
            </CommandItem>
          ) : null}
          {recent.data.traces.map((trace) => (
            <CommandItem
              key={trace.traceId}
              value={`trace ${trace.rootName} ${trace.service} ${trace.traceId}`}
              onSelect={() => {
                close()
                void navigate({
                  to: "/traces/$traceId",
                  params: { traceId: trace.traceId },
                })
              }}
            >
              <IconAffiliate />
              <span className="min-w-0 flex-1 truncate">
                {trace.rootName || trace.traceId}
              </span>
              {trace.hasError ? <CommandShortcut>error</CommandShortcut> : null}
              <CommandShortcut>
                {formatRelative(trace.startNanos)}
              </CommandShortcut>
            </CommandItem>
          ))}
          {recent.data.runs.map((run) => (
            <CommandItem
              key={run.invocationId}
              value={`run ${run.command ?? ""} ${run.invocationId}`}
              onSelect={() => {
                close()
                void navigate({
                  to: "/invocations/$invocationId",
                  params: { invocationId: run.invocationId },
                })
              }}
            >
              <IconTerminal2 />
              <span className="min-w-0 flex-1 truncate">
                {run.command ?? run.invocationId}
              </span>
              {run.errorCount > 0 ? (
                <CommandShortcut>{run.errorCount} errors</CommandShortcut>
              ) : null}
              <CommandShortcut>
                {formatRelative(run.startedAtNanos)}
              </CommandShortcut>
            </CommandItem>
          ))}
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  )
}
