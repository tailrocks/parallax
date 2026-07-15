type TimerHandle = ReturnType<typeof globalThis.setTimeout>

export type PendingTimers = Readonly<{
  intervals: number
  timeouts: number
}>

export function pendingTimerMessage(pending: PendingTimers) {
  return pending.intervals === 0 && pending.timeouts === 0
    ? null
    : `test leaked timers: ${pending.timeouts} timeout(s), ${pending.intervals} interval(s)`
}

export function installTimerTracker() {
  const originalSetTimeout = globalThis.setTimeout
  const originalClearTimeout = globalThis.clearTimeout
  const originalSetInterval = globalThis.setInterval
  const originalClearInterval = globalThis.clearInterval
  const timeouts = new Set<TimerHandle>()
  const intervals = new Set<TimerHandle>()
  function trackedSetTimeout<TArguments extends unknown[]>(
    callback: (...arguments_: TArguments) => void,
    delay?: number,
    ...arguments_: TArguments
  ) {
    let handle: TimerHandle
    handle = originalSetTimeout(
      (...values: TArguments) => {
        timeouts.delete(handle)
        callback(...values)
      },
      delay,
      ...arguments_
    )
    timeouts.add(handle)
    return handle
  }
  function trackedSetInterval<TArguments extends unknown[]>(
    callback: (...arguments_: TArguments) => void,
    delay?: number,
    ...arguments_: TArguments
  ) {
    const handle = originalSetInterval(callback, delay, ...arguments_)
    intervals.add(handle)
    return handle
  }
  globalThis.setTimeout = trackedSetTimeout as typeof globalThis.setTimeout
  globalThis.setInterval = trackedSetInterval as typeof globalThis.setInterval
  globalThis.clearTimeout = ((handle?: TimerHandle) => {
    if (handle !== undefined) timeouts.delete(handle)
    originalClearTimeout(handle)
  }) as typeof globalThis.clearTimeout
  globalThis.clearInterval = ((handle?: TimerHandle) => {
    if (handle !== undefined) intervals.delete(handle)
    originalClearInterval(handle)
  }) as typeof globalThis.clearInterval

  return {
    pending: (): PendingTimers => ({
      intervals: intervals.size,
      timeouts: timeouts.size,
    }),
    restore: () => {
      for (const handle of timeouts) originalClearTimeout(handle)
      for (const handle of intervals) originalClearInterval(handle)
      globalThis.setTimeout = originalSetTimeout
      globalThis.clearTimeout = originalClearTimeout
      globalThis.setInterval = originalSetInterval
      globalThis.clearInterval = originalClearInterval
    },
  }
}
