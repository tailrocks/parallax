import { apiEndpointLabel } from "@/platform/graphql/transport"
import { classifyHealth, type AppStatus } from "@/features/app-status/model/app-status"

export async function loadAppStatus(signal?: AbortSignal): Promise<AppStatus> {
  const endpointLabel = apiEndpointLabel()
  try {
    const requestInit: RequestInit = {}
    if (signal !== undefined) requestInit.signal = signal
    const response = await fetch("/health", requestInit)
    if (response.ok) {
      const text = (await response.text()).trim().toLowerCase()
      return {
        healthy: text === "ok" || text.length === 0 || classifyHealth(text),
        endpointLabel,
      }
    }
    return { healthy: false, endpointLabel }
  } catch {
    return { healthy: false, endpointLabel }
  }
}
