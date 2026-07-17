import { serviceColor } from "@/lib/colors"
import { cn } from "@/lib/utils"

/** Squircle service-identity dot (plan 162). Identity only — never state.
 * Decorative (`aria-hidden`): it always sits next to the service name text,
 * which carries the information. */
export function ServiceDot({
  name,
  className,
}: {
  name: string
  className?: string
}) {
  return (
    <span
      aria-hidden
      className={cn("inline-block size-2 shrink-0 rounded-[30%]", className)}
      style={{ backgroundColor: serviceColor(name).color }}
    />
  )
}
