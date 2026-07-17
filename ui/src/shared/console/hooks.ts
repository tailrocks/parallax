import { useEffect, useState } from "react"

export function useDelayedLoading(loading: boolean, delay = 700) {
  const [delayed, setDelayed] = useState(false)

  useEffect(() => {
    if (!loading) {
      setDelayed(false)
      return
    }
    const timer = setTimeout(() => setDelayed(true), delay)
    return () => clearTimeout(timer)
  }, [delay, loading])

  return delayed
}

export function useDebouncedValue<T>(value: T, delay = 300) {
  const [debounced, setDebounced] = useState(value)
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delay)
    return () => clearTimeout(timer)
  }, [delay, value])
  return debounced
}
