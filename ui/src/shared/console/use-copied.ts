import { useCallback, useState } from "react"

export function useCopied(timeout = 1200) {
  const [copied, setCopied] = useState(false)
  const copy = useCallback(
    async (value: string) => {
      await navigator.clipboard.writeText(value)
      setCopied(true)
      window.setTimeout(() => setCopied(false), timeout)
    },
    [timeout]
  )
  return { copied, copy }
}
