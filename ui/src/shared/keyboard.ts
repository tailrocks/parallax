import { useEffect, useRef, useState } from "react"

/** Canonical keyboard shortcuts (workstream D).
 *
 * One registry kills the per-page copy-pasted `keydown` handlers (logs +
 * traces each owned an identical `f` handler). Pages consume via
 * `useShortcut` / `useFilterFocusShortcut`; `?` help renders from
 * `SHORTCUT_LIST` in `shared/console/shortcuts-dialog.tsx`.
 *
 * Rules: plain-letter shortcuts never fire while typing; mod (Cmd/Ctrl)
 * and Escape shortcuts fire everywhere. */

export type ShortcutId =
  | "palette"
  | "focus-filter"
  | "focus-search"
  | "show-shortcuts"
  | "clear-close"

export interface KeySpec {
  /** Lowercase `event.key`, except named keys (`Escape`, `/`, `?`). */
  key: string
  /** Cmd on macOS, Ctrl elsewhere. */
  mod?: boolean
  shift?: boolean
}

export interface ShortcutDef {
  id: ShortcutId
  /** Plain-language label for the `?` dialog. */
  label: string
  /** Glyphs rendered inside `<Kbd>`. */
  keys: string[]
  spec: KeySpec
}

export const SHORTCUTS: Record<ShortcutId, ShortcutDef> = {
  palette: {
    id: "palette",
    label: "Open command palette",
    keys: ["⌘", "K"],
    spec: { key: "k", mod: true },
  },
  "focus-filter": {
    id: "focus-filter",
    label: "Focus structured filter",
    keys: ["F"],
    spec: { key: "f" },
  },
  "focus-search": {
    id: "focus-search",
    label: "Focus search",
    keys: ["/"],
    spec: { key: "/" },
  },
  "show-shortcuts": {
    id: "show-shortcuts",
    label: "Show keyboard shortcuts",
    keys: ["?"],
    spec: { key: "?" },
  },
  "clear-close": {
    id: "clear-close",
    label: "Clear input / close dialog",
    keys: ["Esc"],
    spec: { key: "Escape" },
  },
}

export const SHORTCUT_LIST: ShortcutDef[] = [
  SHORTCUTS.palette,
  SHORTCUTS["focus-search"],
  SHORTCUTS["focus-filter"],
  SHORTCUTS["show-shortcuts"],
  SHORTCUTS["clear-close"],
]

export function matchesShortcut(spec: KeySpec, event: KeyboardEvent): boolean {
  const mod = event.metaKey || event.ctrlKey
  if (Boolean(spec.mod) !== mod) return false
  if (spec.shift && !event.shiftKey) return false
  if (event.altKey) return false
  return event.key.length === 1
    ? event.key.toLowerCase() === spec.key.toLowerCase()
    : event.key === spec.key
}

/** True when the keydown target is a text-entry control. Plain-letter
 * shortcuts must not fire there. Mirrors the guard the old per-page
 * handlers used. */
export function isTypingTarget(event: KeyboardEvent): boolean {
  const target = event.target
  return (
    target instanceof HTMLElement &&
    target.closest("input, textarea, select, [contenteditable]") !== null
  )
}

/** Subscribe to one registry shortcut. Plain-letter shortcuts skip while
 * typing; mod/Escape shortcuts fire everywhere. */
export function useShortcut(
  id: ShortcutId,
  handler: () => void,
  options?: { enabled?: boolean }
): void {
  const enabled = options?.enabled !== false
  const handlerRef = useRef(handler)
  handlerRef.current = handler
  useEffect(() => {
    if (!enabled) return
    const def = SHORTCUTS[id]
    const global = def.spec.mod === true || def.spec.key === "Escape"
    const onKeyDown = (event: KeyboardEvent) => {
      if (!global && isTypingTarget(event)) return
      if (!matchesShortcut(def.spec, event)) return
      event.preventDefault()
      handlerRef.current()
    }
    window.addEventListener("keydown", onKeyDown)
    return () => window.removeEventListener("keydown", onKeyDown)
  }, [id, enabled])
}

/** Focus-request counter for filter/search inputs: pass `focusKey` as the
 * input's `key` and `autoFocus={focusKey > 0}`. Replaces the ad-hoc
 * `whereFocusKey` state in logs/traces pages. */
export function useFocusRequest(): [number, () => void] {
  const [focusKey, setFocusKey] = useState(0)
  return [focusKey, () => setFocusKey((current) => current + 1)]
}

/** Drop-in for the old per-page `f` handlers:
 * `useFilterFocusShortcut(requestFilterFocus)`. */
export function useFilterFocusShortcut(requestFocus: () => void): void {
  useShortcut("focus-filter", requestFocus)
}
