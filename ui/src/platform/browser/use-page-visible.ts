import { useEffect, useState } from "react"

// Platform visibility adapter (Plan 100 provisional).
// Plan 153 hardens document/visibility failure and SSR semantics.

/** True when the document is visible (or during SSR where document is absent). */
export function usePageVisible(): boolean {
  const [visible, setVisible] = useState(
    () => typeof document === "undefined" || !document.hidden
  )
  useEffect(() => {
    const onChange = () => setVisible(!document.hidden)
    document.addEventListener("visibilitychange", onChange)
    return () => document.removeEventListener("visibilitychange", onChange)
  }, [])
  return visible
}
