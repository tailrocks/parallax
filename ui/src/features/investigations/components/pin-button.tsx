import { useEffect, useMemo, useState } from "react"
import { useRouter, useRouterState } from "@tanstack/react-router"
import { IconCheck, IconPin, IconPlus } from "@tabler/icons-react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import {
  Popover,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
} from "@/components/ui/popover"
import {
  appendInvestigationPin,
  buildInvestigationPin,
  emptyInvestigationState,
  parseInvestigationState,
  serializeInvestigationState,
  windowFromHref,
} from "@/features/investigations/model/investigation-state"
import type { InvestigationPinKind } from "@/features/investigations/model/investigation-state"
import type { Investigation } from "@/features/investigations/model/investigation"
import {
  loadInvestigationPinOptions,
  saveInvestigation,
} from "@/features/investigations/api/investigation-api"
import { investigationErrorMessage } from "@/features/investigations/model/investigation-error"

export function PinButton({
  kind,
  label,
}: {
  kind: InvestigationPinKind
  label: string
}) {
  const router = useRouter()
  const currentHref = useRouterState({
    select: (state) => state.location.href,
  })
  const [open, setOpen] = useState(false)
  const [investigations, setInvestigations] = useState<Investigation[]>([])
  const [newName, setNewName] = useState("")
  const [loading, setLoading] = useState(false)
  const [savingId, setSavingId] = useState<string | null>(null)
  const [message, setMessage] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const pin = useMemo(
    () => buildInvestigationPin(kind, label, currentHref),
    [currentHref, kind, label]
  )

  useEffect(() => {
    if (!open) return
    let ignore = false
    setLoading(true)
    setError(null)
    void loadInvestigationPinOptions()
      .then((rows) => {
        if (!ignore) setInvestigations(rows)
      })
      .catch((err: unknown) => {
        if (!ignore) setError(investigationErrorMessage(err))
      })
      .finally(() => {
        if (!ignore) setLoading(false)
      })
    return () => {
      ignore = true
    }
  }, [open])

  async function saveToInvestigation(investigation: Investigation) {
    setSavingId(investigation.id)
    setError(null)
    setMessage(null)
    try {
      const state = appendInvestigationPin(
        parseInvestigationState(investigation.state),
        pin
      )
      await saveInvestigation({
        id: investigation.id,
        name: investigation.name,
        state: serializeInvestigationState(state),
      })
      setMessage(`Pinned to ${investigation.name}`)
      await router.invalidate()
    } catch (err) {
      setError(investigationErrorMessage(err))
    } finally {
      setSavingId(null)
    }
  }

  async function createInvestigation() {
    const name = newName.trim()
    if (!name) return
    setSavingId("__new__")
    setError(null)
    setMessage(null)
    try {
      const state = appendInvestigationPin(
        {
          ...emptyInvestigationState(),
          window: windowFromHref(currentHref),
        },
        pin
      )
      const created = await saveInvestigation({
        name,
        state: serializeInvestigationState(state),
      })
      setInvestigations((current) => [created, ...current])
      setNewName("")
      setMessage(`Pinned to ${created.name}`)
      await router.invalidate()
    } catch (err) {
      setError(investigationErrorMessage(err))
    } finally {
      setSavingId(null)
    }
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger render={<Button size="sm" variant="outline" />}>
        <IconPin data-icon="inline-start" />
        Pin
      </PopoverTrigger>
      <PopoverContent align="end" className="w-80">
        <PopoverHeader>
          <PopoverTitle>Pin to investigation</PopoverTitle>
          <PopoverDescription>
            Capture this page with its current URL state.
          </PopoverDescription>
        </PopoverHeader>
        <div className="flex flex-col gap-3">
          <div className="flex items-center justify-between gap-2 rounded-lg border bg-muted/30 px-3 py-2">
            <span className="truncate text-sm">{pin.label}</span>
            <Badge variant="secondary">{pin.kind}</Badge>
          </div>
          {loading ? (
            <p className="text-sm text-muted-foreground">Loading...</p>
          ) : investigations.length ? (
            <div className="flex max-h-52 flex-col gap-1 overflow-auto">
              {investigations.map((investigation) => (
                <Button
                  key={investigation.id}
                  type="button"
                  variant="ghost"
                  className="justify-between"
                  disabled={savingId !== null}
                  onClick={() => void saveToInvestigation(investigation)}
                >
                  <span className="truncate">{investigation.name}</span>
                  {savingId === investigation.id ? (
                    <IconCheck data-icon="inline-end" />
                  ) : null}
                </Button>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">
              No investigations yet.
            </p>
          )}
          <FieldGroup className="gap-3">
            <Field>
              <FieldLabel htmlFor="new-investigation-name">
                New investigation
              </FieldLabel>
              <Input
                id="new-investigation-name"
                value={newName}
                onChange={(event) => setNewName(event.target.value)}
                placeholder="Checkout outage"
              />
            </Field>
            <Button
              type="button"
              variant="outline"
              disabled={!newName.trim() || savingId !== null}
              onClick={() => void createInvestigation()}
            >
              <IconPlus data-icon="inline-start" />
              Create and pin
            </Button>
          </FieldGroup>
          {message ? (
            <p className="text-sm text-muted-foreground">{message}</p>
          ) : null}
          {error ? <p className="text-sm text-destructive">{error}</p> : null}
        </div>
      </PopoverContent>
    </Popover>
  )
}
