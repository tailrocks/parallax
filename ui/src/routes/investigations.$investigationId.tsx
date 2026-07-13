import {
  Link,
  createFileRoute,
  notFound,
  useRouter,
} from "@tanstack/react-router"
import { useState } from "react"
import {
  IconAffiliate,
  IconArticle,
  IconBug,
  IconExternalLink,
  IconPencil,
  IconTerminal2,
  IconTrash,
} from "@tabler/icons-react"
import type { Icon } from "@tabler/icons-react"

import { navItem } from "@/components/nav"
import { PageHeader } from "@/components/page-header"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog"
import { Badge } from "@/components/ui/badge"
import { Button, buttonVariants } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field"
import { Textarea } from "@/components/ui/textarea"
import { gqlString, graphql, graphqlCached } from "@/lib/api"
import type { Investigation } from "@/lib/api"
import {
  hrefForPin,
  investigationWindowSearch,
  parseInvestigationState,
  serializeInvestigationState,
} from "@/lib/investigations"
import type { InvestigationPin, InvestigationState } from "@/lib/investigations"

export const Route = createFileRoute("/investigations/$investigationId")({
  loader: async ({ params }) => {
    const { investigation } = await graphqlCached<{
      investigation: Investigation | null
    }>(
      `{ investigation(id: "${gqlString(params.investigationId)}") {
           id name state createdAtNanos updatedAtNanos
         } }`
    )
    if (!investigation) throw notFound()
    return { investigation }
  },
  component: InvestigationDetailPage,
})

const PIN_ICONS: Record<InvestigationPin["kind"], Icon> = {
  trace: IconAffiliate,
  issue: IconBug,
  run: IconTerminal2,
  log: IconArticle,
  view: IconExternalLink,
}

function InvestigationDetailPage() {
  const { investigation } = Route.useLoaderData()
  return <InvestigationDetailContent investigation={investigation} />
}

export function InvestigationDetailContent({
  investigation,
}: {
  investigation: Investigation
}) {
  const router = useRouter()
  const investigationsBack = navItem("/investigations")!
  const [draft, setDraft] = useState<InvestigationState>(() =>
    parseInvestigationState(investigation.state)
  )
  const [error, setError] = useState<string | null>(null)
  const windowSearch = investigationWindowSearch(draft.window)

  async function save() {
    setError(null)
    try {
      await graphql(`mutation { investigationSave(id: "${gqlString(investigation.id)}", name: "${gqlString(investigation.name)}", state: "${gqlString(serializeInvestigationState(draft))}") { id } }`)
      await router.invalidate()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  async function remove() {
    setError(null)
    try {
      await graphql(`mutation { investigationDelete(id: "${gqlString(investigation.id)}") }`)
      await router.navigate({ to: "/investigations" })
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  function updatePin(index: number, patch: Partial<InvestigationPin>) {
    setDraft((current) => ({
      ...current,
      pins: current.pins.map((pin, i) =>
        i === index ? { ...pin, ...patch } : pin
      ),
    }))
  }

  function removePin(index: number) {
    setDraft((current) => ({
      ...current,
      pins: current.pins.filter((_, i) => i !== index),
    }))
  }

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        back={investigationsBack}
        title={investigation.name}
        description="Pinned telemetry pages and working notes."
        actions={
          <>
            <Button size="sm" onClick={() => void save()}>
              <IconPencil data-icon="inline-start" />
              Save
            </Button>
            <AlertDialog>
              <AlertDialogTrigger
                render={<Button size="sm" variant="ghost-destructive" />}
              >
                <IconTrash data-icon="inline-start" />
                Delete
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>Delete investigation?</AlertDialogTitle>
                  <AlertDialogDescription>
                    Delete {investigation.name}. This cannot be undone.
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel>Cancel</AlertDialogCancel>
                  <AlertDialogAction
                    variant="destructive"
                    onClick={() => void remove()}
                  >
                    Delete
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
          </>
        }
      />

      {error ? <p className="text-sm text-destructive">{error}</p> : null}

      <Card>
        <CardHeader className="flex-row items-center justify-between gap-2">
          <CardTitle className="text-sm">Window</CardTitle>
          <Link
            to="/"
            search={windowSearch}
            className={buttonVariants({ variant: "outline", size: "sm" })}
          >
            <IconExternalLink data-icon="inline-start" />
            Open overview
          </Link>
        </CardHeader>
        <CardContent className="flex flex-wrap gap-2">
          <Badge variant="secondary">{draft.window.range}</Badge>
          {draft.window.from && draft.window.to ? (
            <Badge variant="outline">
              {draft.window.from} - {draft.window.to}
            </Badge>
          ) : null}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">Pins</CardTitle>
        </CardHeader>
        <CardContent>
          {draft.pins.length === 0 ? (
            <p className="text-sm text-muted-foreground">No pins yet.</p>
          ) : (
            <ul className="flex flex-col gap-3">
              {draft.pins.map((pin, index) => {
                const PinIcon = PIN_ICONS[pin.kind]
                const href = hrefForPin(pin)
                return (
                  <li
                    key={`${pin.kind}-${pin.ref}-${index}`}
                    className="flex flex-col gap-3 rounded-lg border p-3"
                  >
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <a
                        href={href}
                        className="flex min-w-0 items-center gap-2 hover:underline"
                      >
                        <PinIcon className="size-4 shrink-0 text-muted-foreground" />
                        <span className="truncate">{pin.label}</span>
                      </a>
                      <div className="flex items-center gap-2">
                        <Badge variant="secondary">{pin.kind}</Badge>
                        <Button
                          type="button"
                          variant="ghost-destructive"
                          size="icon-xs"
                          onClick={() => removePin(index)}
                        >
                          <IconTrash />
                          <span className="sr-only">Remove pin</span>
                        </Button>
                      </div>
                    </div>
                    <FieldGroup className="gap-2">
                      <Field>
                        <FieldLabel htmlFor={`pin-note-${index}`}>
                          Note
                        </FieldLabel>
                        <Textarea
                          id={`pin-note-${index}`}
                          value={pin.note ?? ""}
                          onChange={(event) =>
                            updatePin(index, { note: event.target.value })
                          }
                          rows={2}
                        />
                      </Field>
                    </FieldGroup>
                  </li>
                )
              })}
            </ul>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">Notes</CardTitle>
        </CardHeader>
        <CardContent>
          <FieldGroup>
            <Field>
              <FieldLabel htmlFor="investigation-notes">Markdown</FieldLabel>
              <FieldDescription>
                Saved as plain text; rendered nowhere as HTML.
              </FieldDescription>
              <Textarea
                id="investigation-notes"
                value={draft.notes}
                onChange={(event) =>
                  setDraft((current) => ({
                    ...current,
                    notes: event.target.value,
                  }))
                }
                rows={10}
              />
            </Field>
          </FieldGroup>
        </CardContent>
      </Card>
    </div>
  )
}
