/** Logs Patterns-mode URL codec (plan 165).
 *
 * Encodes the Patterns toggle + optional focused template for shareable
 * permalinks. Pure — peer wires into logs.tsx search schema.
 *
 * Preliminary — peer verify/extend + browser URL-reload evidence.
 */

export interface LogPatternsUrlState {
  /** When true, the page shows Drain cluster rows instead of raw logs. */
  patterns: boolean
  /** Optional template string focused for expand-to-samples. */
  patternTemplate: string | null
}

export const DEFAULT_LOG_PATTERNS_URL: LogPatternsUrlState = {
  patterns: false,
  patternTemplate: null,
}

export const LOG_PATTERNS_PARAM = "patterns"
export const LOG_PATTERN_TEMPLATE_PARAM = "pattern"

/** Parse `patterns=1|true|yes` (case-insensitive). */
export function parsePatternsFlag(raw: string | null | undefined): boolean {
  if (raw == null || raw === "") return false
  const v = raw.trim().toLowerCase()
  return v === "1" || v === "true" || v === "yes" || v === "on"
}

export function encodePatternsFlag(on: boolean): string {
  return on ? "1" : "0"
}

export function decodeLogPatternsUrl(
  params: URLSearchParams | Record<string, string | undefined>
): LogPatternsUrlState {
  const get = (key: string): string | undefined => {
    if (params instanceof URLSearchParams) {
      return params.get(key) ?? undefined
    }
    return params[key]
  }
  const patterns = parsePatternsFlag(get(LOG_PATTERNS_PARAM))
  const tmpl = get(LOG_PATTERN_TEMPLATE_PARAM)?.trim()
  return {
    patterns,
    // Template only meaningful when patterns mode is on; still decode if present.
    patternTemplate: tmpl ? tmpl : null,
  }
}

/**
 * Encode patterns state into search params.
 * Omits defaults (`patterns` off, no template) so clean logs URLs stay short.
 */
export function encodeLogPatternsUrl(state: LogPatternsUrlState): URLSearchParams {
  const params = new URLSearchParams()
  if (state.patterns) {
    params.set(LOG_PATTERNS_PARAM, "1")
  }
  if (state.patterns && state.patternTemplate) {
    params.set(LOG_PATTERN_TEMPLATE_PARAM, state.patternTemplate)
  }
  return params
}

/** Merge patterns keys onto an existing params object (mutates a copy). */
export function mergeLogPatternsParams(
  base: URLSearchParams,
  state: LogPatternsUrlState
): URLSearchParams {
  const next = new URLSearchParams(base)
  next.delete(LOG_PATTERNS_PARAM)
  next.delete(LOG_PATTERN_TEMPLATE_PARAM)
  const encoded = encodeLogPatternsUrl(state)
  for (const [k, v] of encoded.entries()) {
    next.set(k, v)
  }
  return next
}
