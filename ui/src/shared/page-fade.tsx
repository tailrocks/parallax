import { useRef } from "react"

let consumed = false

/** Once-per-boot gate for the 75ms opacity page fade (plan 172).
 * First call returns whether to play; later calls are a no-op (`false`).
 * Reduced-motion short-circuits to `false` and still consumes. */
export function shouldPlayPageFade(): boolean {
  if (consumed) return false
  consumed = true
  if (
    typeof window !== "undefined" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches
  ) {
    return false
  }
  return true
}

export function resetPageFadeForTests(): void {
  consumed = false
}

export function PageFade({ children }: { children: React.ReactNode }) {
  const play = useRef<boolean | null>(null)
  if (play.current === null) {
    play.current = typeof window === "undefined" ? true : shouldPlayPageFade()
  }
  return (
    <div className={play.current ? "page-fade" : undefined} suppressHydrationWarning>
      {children}
    </div>
  )
}
