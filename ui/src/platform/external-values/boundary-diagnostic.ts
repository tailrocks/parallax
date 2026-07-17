// Plan 153 — secret-safe, injectable diagnostic sink (silent by default).

import type { BoundaryError } from "./boundary-error"

const DEFAULT_CAP = 240

export interface BoundaryDiagnosticSink {
  report(error: BoundaryError): void
}

const silentSink: BoundaryDiagnosticSink = {
  report() {
    // Silent by default — no console or telemetry noise from this foundation.
  },
}

let activeSink: BoundaryDiagnosticSink = silentSink

/** Inject a diagnostic sink (tests/operators). Returns restore function. */
export function setBoundaryDiagnosticSink(sink: BoundaryDiagnosticSink | null): () => void {
  const previous = activeSink
  activeSink = sink ?? silentSink
  return () => {
    activeSink = previous
  }
}

/**
 * Render a bounded operator-facing diagnostic string.
 * Never includes raw JSON, storage content/key, URL query, origins, stacks, or
 * caught error text — only stable IDs/codes/kinds/numeric meta.
 */
export function formatBoundaryDiagnostic(error: BoundaryError, cap: number = DEFAULT_CAP): string {
  const parts = [
    `boundary=${truncate(error.boundaryId, 64)}`,
    `code=${error.code}`,
    `kind=${error.observedKind}`,
  ]
  if (error.meta) {
    for (const [key, value] of Object.entries(error.meta)) {
      if (!Number.isFinite(value)) continue
      parts.push(`${truncate(key, 32)}=${value}`)
    }
  }
  return truncate(parts.join(" "), Math.max(16, cap))
}

export function reportBoundaryError(error: BoundaryError): void {
  try {
    activeSink.report(error)
  } catch {
    // Sink failures must not break product paths.
  }
}

function truncate(value: string, max: number): string {
  if (value.length <= max) return value
  return `${value.slice(0, Math.max(0, max - 1))}…`
}
