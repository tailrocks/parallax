export const TEST_NOW_ISO = "2026-01-15T12:00:00.000Z"

export function testNow() {
  return new Date(TEST_NOW_ISO)
}

export function createTestIdFactory(prefix = "fixture") {
  let sequence = 0
  return () => `${prefix}-${String(++sequence).padStart(3, "0")}`
}
