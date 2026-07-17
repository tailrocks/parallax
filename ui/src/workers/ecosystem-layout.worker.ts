import { runElkLayout } from "@/lib/ecosystem-layout"
import type {
  EcosystemLayoutRequest,
  EcosystemLayoutResponse,
} from "@/lib/ecosystem-layout"

const workerScope = globalThis as unknown as {
  onmessage: ((event: MessageEvent<EcosystemLayoutRequest>) => void) | null
  postMessage: (response: EcosystemLayoutResponse) => void
}

workerScope.onmessage = (event) => {
  void runElkLayout(event.data)
    .then((layout) => workerScope.postMessage({ ok: true, layout }))
    .catch((error: unknown) =>
      workerScope.postMessage({
        ok: false,
        error: error instanceof Error ? error.message : String(error),
      })
    )
}
