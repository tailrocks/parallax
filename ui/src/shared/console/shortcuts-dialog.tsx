import { useState } from "react"

import { SHORTCUT_LIST, useShortcut } from "@/shared/keyboard"
import { Kbd } from "@/components/ui/kbd"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog"

/** `?` shortcuts help, rendered from the `shared/keyboard.ts` registry so
 * docs and behavior cannot drift. */
export function ShortcutsDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>Keyboard shortcuts</DialogTitle>
        </DialogHeader>
        <dl className="grid gap-2">
          {SHORTCUT_LIST.map((shortcut) => (
            <div key={shortcut.id} className="flex items-center justify-between gap-4">
              <dt className="text-sm text-muted-foreground">{shortcut.label}</dt>
              <dd className="flex shrink-0 items-center gap-1">
                {shortcut.keys.map((key) => (
                  <Kbd key={key}>{key}</Kbd>
                ))}
              </dd>
            </div>
          ))}
        </dl>
      </DialogContent>
    </Dialog>
  )
}

/** Owns the dialog's open state and wires `?`. Host once (app shell). */
export function useShortcutsDialog() {
  const [open, setOpen] = useState(false)
  useShortcut("show-shortcuts", () => setOpen(true), { enabled: !open })
  return { open, setOpen }
}
