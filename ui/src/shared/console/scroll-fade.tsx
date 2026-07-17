import { cn } from "@/lib/utils"

export function ScrollFade({ className, children }: React.ComponentProps<"div">) {
  return (
    <div className={cn("relative overflow-hidden", className)}>
      <div className="overflow-auto">{children}</div>
      <div className="pointer-events-none absolute inset-x-0 bottom-0 h-8 bg-linear-to-t from-background to-transparent" />
    </div>
  )
}
