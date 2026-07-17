import { Link, useRouter } from "@tanstack/react-router"
import { useState } from "react"
import { IconClipboardList, IconPlus, IconTrash } from "@tabler/icons-react"

import { EmptyState } from "@/shared/console/empty-state"
import { RelativeTime } from "@/shared/console/relative-time"
import { PageHeader } from "@/shared/components/page-header"
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
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import {
  emptyInvestigationState,
  parseInvestigationState,
  serializeInvestigationState,
} from "@/features/investigations/model/investigation-state"
import type { Investigation } from "@/features/investigations/model/investigation"
import {
  deleteInvestigation,
  saveInvestigation,
} from "@/features/investigations/api/investigation-api"
import { investigationErrorMessage } from "@/features/investigations/model/investigation-error"
import { formatCount } from "@/lib/format"

export function InvestigationsPage({
  investigations,
}: {
  investigations: readonly Investigation[]
}) {
  const router = useRouter()
  const [open, setOpen] = useState(false)
  const [name, setName] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [deleteError, setDeleteError] = useState<string | null>(null)

  async function create() {
    setError(null)
    try {
      const created = await saveInvestigation({
        name,
        state: serializeInvestigationState(emptyInvestigationState()),
      })
      setName("")
      setOpen(false)
      await router.navigate({
        to: "/investigations/$investigationId",
        params: { investigationId: created.id },
      })
    } catch (err) {
      setError(investigationErrorMessage(err))
    }
  }

  async function remove(id: string) {
    setDeleteError(null)
    try {
      await deleteInvestigation(id)
      await router.invalidate()
    } catch (err) {
      setDeleteError(investigationErrorMessage(err))
    }
  }

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        icon={IconClipboardList}
        iconClassName="text-teal-500"
        title="Investigations"
        description="Saved case files with pinned telemetry pages and notes."
        actions={
          <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger render={<Button />}>
              <IconPlus data-icon="inline-start" />
              New investigation
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>New investigation</DialogTitle>
                <DialogDescription>
                  Name the case file before adding pins and notes.
                </DialogDescription>
              </DialogHeader>
              <Input
                value={name}
                onChange={(event) => setName(event.target.value)}
                placeholder="Checkout outage"
              />
              {error ? (
                <p className="text-sm text-destructive">{error}</p>
              ) : null}
              <DialogFooter>
                <Button disabled={!name.trim()} onClick={() => void create()}>
                  Create
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        }
      />

      {deleteError ? (
        <p className="text-sm text-destructive">{deleteError}</p>
      ) : null}

      {investigations.length === 0 ? (
        <EmptyState
          icon={IconClipboardList}
          title="No investigations"
          description="Create a case file or pin from a trace, issue, or run."
        />
      ) : (
        <ul className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {investigations.map((investigation) => {
            const state = parseInvestigationState(investigation.state)
            return (
              <li key={investigation.id}>
                <Card>
                  <CardHeader className="flex-row items-center justify-between">
                    <CardTitle className="truncate text-sm">
                      <Link
                        to="/investigations/$investigationId"
                        params={{ investigationId: investigation.id }}
                        className="hover:underline"
                      >
                        {investigation.name}
                      </Link>
                    </CardTitle>
                    <AlertDialog>
                      <AlertDialogTrigger
                        render={
                          <Button variant="ghost-destructive" size="icon-xs" />
                        }
                      >
                        <IconTrash />
                        <span className="sr-only">Delete</span>
                      </AlertDialogTrigger>
                      <AlertDialogContent>
                        <AlertDialogHeader>
                          <AlertDialogTitle>
                            Delete investigation?
                          </AlertDialogTitle>
                          <AlertDialogDescription>
                            Delete {investigation.name}. This cannot be undone.
                          </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                          <AlertDialogCancel>Cancel</AlertDialogCancel>
                          <AlertDialogAction
                            variant="destructive"
                            onClick={() => void remove(investigation.id)}
                          >
                            Delete
                          </AlertDialogAction>
                        </AlertDialogFooter>
                      </AlertDialogContent>
                    </AlertDialog>
                  </CardHeader>
                  <CardContent className="flex items-center justify-between gap-2">
                    <Badge variant="secondary">
                      {formatCount(state.pins.length)} pins
                    </Badge>
                    <span className="text-xs text-muted-foreground">
                      <RelativeTime nanos={investigation.updatedAtNanos} />
                    </span>
                  </CardContent>
                </Card>
              </li>
            )
          })}
        </ul>
      )}
    </div>
  )
}
