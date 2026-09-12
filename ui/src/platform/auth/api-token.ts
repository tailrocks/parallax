// Plan 109 — operator API token, browser side.
//
// The server accepts a bearer token on every developer-API route. Browsers
// cannot attach headers to `EventSource`, so the SSE live tail also accepts an
// `access_token` query parameter (scoped to the stream routes only). This
// module is the single owner of how the stored token reaches both transports.

const STORAGE_KEY = "parallax.api-token"

function browserStorage(): Storage | null {
  return typeof window === "undefined" ? null : window.localStorage
}

/** Stored token, or null when unset/empty (token auth is optional server-side). */
export function getApiToken(): string | null {
  const raw = browserStorage()?.getItem(STORAGE_KEY) ?? null
  const trimmed = raw?.trim()
  return trimmed ? trimmed : null
}

/** Save the token (empty/whitespace clears it). Callers decide when to reload. */
export function setApiToken(token: string): void {
  const storage = browserStorage()
  if (!storage) return
  const trimmed = token.trim()
  if (trimmed) {
    storage.setItem(STORAGE_KEY, trimmed)
  } else {
    storage.removeItem(STORAGE_KEY)
  }
}

/** Bearer headers for fetch-based transports; empty record when no token. */
export function apiAuthHeaders(): Record<string, string> {
  const token = getApiToken()
  return token ? { authorization: `Bearer ${token}` } : {}
}

/**
 * Append the `access_token` query parameter for EventSource URLs, which cannot
 * carry headers. Returns the URL untouched when no token is stored.
 */
export function withAccessTokenQuery(url: string): string {
  const token = getApiToken()
  if (!token) return url
  const separator = url.includes("?") ? "&" : "?"
  return `${url}${separator}access_token=${encodeURIComponent(token)}`
}
