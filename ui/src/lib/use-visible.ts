import { useEffect, useState } from "react"

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
