type TestReset = () => void | Promise<void>

const resets = new Set<TestReset>()

export function registerTestReset(reset: TestReset) {
  resets.add(reset)
  return () => resets.delete(reset)
}

export async function resetRegisteredTestState() {
  for (const reset of resets) {
    await reset()
  }
}
