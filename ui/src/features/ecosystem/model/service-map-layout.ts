import {
  ecosystemTopologyKey,
  fallbackEcosystemLayout,
} from "@/features/ecosystem/model/service-map-layout-engine"
import type {
  EcosystemLayout,
  EcosystemLayoutRequest,
  EcosystemLayoutResponse,
} from "@/features/ecosystem/model/service-map-layout-engine"

export type {
  EcosystemLayout,
  EcosystemLayoutRequest,
  EcosystemLayoutResponse,
  EcosystemPosition,
} from "@/features/ecosystem/model/service-map-layout-engine"
export {
  ECOSYSTEM_NODE_WIDTH,
  ECOSYSTEM_NODE_HEIGHT,
  ecosystemTopologyKey,
  runElkLayout,
  fallbackEcosystemLayout,
} from "@/features/ecosystem/model/service-map-layout-engine"

const LAYOUT_CACHE_CAP = 32
const cache = new Map<string, Promise<EcosystemLayout>>()

function workerLayout(request: EcosystemLayoutRequest): Promise<EcosystemLayout> {
  return new Promise((resolve, reject) => {
    const worker = new Worker(new URL("../workers/ecosystem-layout.worker.ts", import.meta.url), {
      type: "module",
    })
    worker.onmessage = (event: MessageEvent<EcosystemLayoutResponse>) => {
      worker.terminate()
      if (event.data.ok) resolve(event.data.layout)
      else reject(new Error(event.data.error))
    }
    worker.onerror = (event) => {
      worker.terminate()
      reject(new Error(event.message || "ELK layout worker failed"))
    }
    worker.postMessage(request)
  })
}

function remember(key: string, value: Promise<EcosystemLayout>): Promise<EcosystemLayout> {
  if (cache.size >= LAYOUT_CACHE_CAP) {
    const oldest = cache.keys().next().value
    if (oldest !== undefined) cache.delete(oldest)
  }
  cache.set(key, value)
  return value
}

/** Worker in browsers; direct bundled engine in SSR/Vitest. A worker startup
 * failure retries through the direct engine so graph rendering still works. */
export function layoutEcosystem(request: EcosystemLayoutRequest): Promise<EcosystemLayout> {
  const key = ecosystemTopologyKey(request)
  const cached = cache.get(key)
  if (cached) return cached
  const canUseWorker = typeof window !== "undefined" && typeof window.Worker !== "undefined"
  const pending = canUseWorker
    ? workerLayout(request).catch(() => fallbackEcosystemLayout(request))
    : Promise.resolve(fallbackEcosystemLayout(request))
  return remember(key, pending)
}

export function clearEcosystemLayoutCache(): void {
  cache.clear()
}
