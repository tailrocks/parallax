import {
  classifyHealth,
  DEFAULT_ENDPOINT_LABEL,
  type AppStatus,
} from "@/features/app-status/model/app-status"

export async function loadAppStatus(signal?: AbortSignal): Promise<AppStatus> {
  try {
    const init: RequestInit = {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ query: "{ health }" }),
    }
    if (signal) init.signal = signal
    const response = await fetch("/graphql", init)
    const body = (await response.json()) as { data?: { health?: unknown } }
    return {
      healthy: classifyHealth(body.data?.health),
      endpointLabel: DEFAULT_ENDPOINT_LABEL,
    }
  } catch {
    return {
      healthy: false,
      endpointLabel: DEFAULT_ENDPOINT_LABEL,
    }
  }
}
