import { useEffect, useState } from "react"

import { formatDateTime, formatRelative } from "@/lib/format"

type TickSubscriber = () => void

const subscribers = new Set<TickSubscriber>()
let timer: number | null = null

function notifySubscribers() {
  for (const subscriber of subscribers) subscriber()
}

function subscribeToTicker(subscriber: TickSubscriber) {
  subscribers.add(subscriber)
  if (!timer && typeof window !== "undefined") {
    timer = window.setInterval(notifySubscribers, 15_000)
  }
  return () => {
    subscribers.delete(subscriber)
    if (subscribers.size === 0 && timer) {
      clearInterval(timer)
      timer = null
    }
  }
}

export function RelativeTime({ nanos }: { nanos: string }) {
  const [, setTick] = useState(0)
  useEffect(() => {
    return subscribeToTicker(() => setTick((tick) => tick + 1))
  }, [])

  return <time title={formatDateTime(nanos)}>{formatRelative(nanos)}</time>
}
