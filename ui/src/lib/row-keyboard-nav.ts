import { useEffect, useRef, useState } from "react"

/**
 * j/k + arrow + Enter navigation over rendered table rows.
 *
 * One instance per table; `scope` isolates tables that share a page. Rows opt
 * in with {@link rowKeyboardAttrs}. Ignores keystrokes from inputs, dialogs,
 * and modified shortcuts so it composes with editors and the command palette.
 */
export function useRowKeyboardNav({
  scope,
  count,
  onOpen,
  enabled = true,
}: {
  scope: string
  /** Number of navigable (rendered) rows. */
  count: number
  onOpen: (index: number) => void
  enabled?: boolean | undefined
}): number {
  const [active, setActive] = useState(-1)
  const activeRef = useRef(-1)
  const onOpenRef = useRef(onOpen)
  onOpenRef.current = onOpen

  useEffect(() => {
    setActive((prev) => {
      const next = prev >= count ? count - 1 : prev
      activeRef.current = next
      return next
    })
  }, [count])

  useEffect(() => {
    if (!enabled) {
      activeRef.current = -1
      setActive(-1)
    }
  }, [enabled])

  useEffect(() => {
    if (!enabled || count === 0) return
    const move = (next: number) => {
      const clamped = Math.min(Math.max(next, 0), count - 1)
      activeRef.current = clamped
      setActive(clamped)
      document
        .querySelector(`[data-kb-scope="${scope}"][data-kb-index="${clamped}"]`)
        ?.scrollIntoView({ block: "nearest" })
    }
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target
      if (
        target instanceof HTMLElement &&
        target.closest("input, textarea, select, [contenteditable], dialog, [role='dialog']")
      ) {
        return
      }
      if (event.metaKey || event.ctrlKey || event.altKey) return
      switch (event.key) {
        case "j":
        case "ArrowDown":
          event.preventDefault()
          move(activeRef.current + 1)
          break
        case "k":
        case "ArrowUp":
          event.preventDefault()
          move(activeRef.current - 1)
          break
        case "Enter":
          if (activeRef.current >= 0) {
            event.preventDefault()
            onOpenRef.current(activeRef.current)
          }
          break
        case "Escape":
          activeRef.current = -1
          setActive(-1)
          break
      }
    }
    window.addEventListener("keydown", onKeyDown)
    return () => window.removeEventListener("keydown", onKeyDown)
  }, [enabled, count, scope])

  return active
}

/** Data attributes marking one navigable row. Spread onto the row element. */
export function rowKeyboardAttrs(scope: string, index: number) {
  return {
    "data-kb-scope": scope,
    "data-kb-index": index,
  }
}
