import { IconCheck, IconCopy } from "@tabler/icons-react"

import { useCopied } from "@/shared/console/use-copied"
import { Button } from "@/components/ui/button"

export function CopyButton({ value }: { value: string }) {
  const { copied, copy } = useCopied()
  const Icon = copied ? IconCheck : IconCopy
  return (
    <Button
      type="button"
      variant="ghost"
      size="icon-xs"
      className="text-muted-foreground/60 hover:text-foreground"
      onClick={(event) => {
        event.stopPropagation()
        void copy(value)
      }}
    >
      <Icon />
      <span className="sr-only">Copy</span>
    </Button>
  )
}
