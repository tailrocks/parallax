export type AppStatus = {
  healthy: boolean
  endpointLabel: string
}

export const DEFAULT_ENDPOINT_LABEL = "127.0.0.1:4000"

export function classifyHealth(value: unknown): boolean {
  return value === "ok" || value === true
}
