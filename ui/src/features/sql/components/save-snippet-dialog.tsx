import { IconDeviceFloppy } from "@tabler/icons-react"

import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"

export function SaveSnippetDialog({
  open,
  name,
  saving,
  onOpenChange,
  onNameChange,
  onSave,
}: {
  open: boolean
  name: string
  saving: boolean
  onOpenChange: (open: boolean) => void
  onNameChange: (name: string) => void
  onSave: () => void
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Save snippet</DialogTitle>
        </DialogHeader>
        <Input
          value={name}
          onChange={(event) => onNameChange(event.target.value)}
          placeholder="Snippet name"
          autoFocus
        />
        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            onClick={() => onOpenChange(false)}
          >
            Cancel
          </Button>
          <Button
            type="button"
            onClick={onSave}
            disabled={saving || !name.trim()}
          >
            <IconDeviceFloppy />
            Save
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
