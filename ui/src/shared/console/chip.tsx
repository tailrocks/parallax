import { Badge } from "@/components/ui/badge"
import { cn } from "@/lib/utils"

export function Chip({
  className,
  variant = "outline",
  size = "md",
  ...props
}: React.ComponentProps<typeof Badge>) {
  return (
    <Badge
      variant={variant}
      size={size}
      className={cn("font-mono normal-case", className)}
      {...props}
    />
  )
}
