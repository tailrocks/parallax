import { useEffect, useState } from "react"

import { formatDateTime, formatRelative } from "@/lib/format"

type TickSubscriber = () => void

const subscribers = new Set<TickSubscriber>()
let timer: number | null = null

function notifySubscribers() {
  // Guard: vitest/jsdom may tear down `window` while a pending tick still runs.
  if (typeof window === "undefined") {
    stopTimer()
    return
  }
  for (const subscriber of subscribers) subscriber()
}

function pageIsVisible(): boolean {
  return typeof document === "undefined" || !document.hidden
}

function ensureTimer() {
  if (timer !== null || typeof window === "undefined") return
  if (!pageIsVisible()) return
  timer = window.setInterval(notifySubscribers, 15_000)
}

function stopTimer() {
  if (timer !== null && typeof window !== "undefined") {
    window.clearInterval(timer)
  }
  timer = null
}

function onVisibilityChange() {
  if (typeof document === "undefined" || typeof window === "undefined") {
    stopTimer()
    return
  }
  if (pageIsVisible()) {
    if (subscribers.size > 0) {
      ensureTimer()
      notifySubscribers()
    }
  } else {
    stopTimer()
  }
}

if (typeof document !== "undefined") {
  document.addEventListener("visibilitychange", onVisibilityChange)
}

function subscribeToTicker(subscriber: TickSubscriber) {
  subscribers.add(subscriber)
  ensureTimer()
  return () => {
    subscribers.delete(subscriber)
    if (subscribers.size === 0) stopTimer()
  }
}

export function RelativeTime({ nanos }: { nanos: string }) {
  const [, setTick] = useState(0)
  useEffect(() => {
    return subscribeToTicker(() => {
      if (typeof window === "undefined") return
      setTick((tick) => tick + 1)
    })
  }, [])

  return <time title={formatDateTime(nanos)}>{formatRelative(nanos)}</time>
}
