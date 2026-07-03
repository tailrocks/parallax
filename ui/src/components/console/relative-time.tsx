import { useEffect, useState } from "react"

import { formatDateTime, formatRelative } from "@/lib/format"

export function RelativeTime({ nanos }: { nanos: string }) {
  const [, setTick] = useState(0)
  useEffect(() => {
    const timer = setInterval(() => setTick((tick) => tick + 1), 15_000)
    return () => clearInterval(timer)
  }, [])

  return <time title={formatDateTime(nanos)}>{formatRelative(nanos)}</time>
}
