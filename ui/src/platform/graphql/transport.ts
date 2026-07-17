// Platform GraphQL transport (Plan 100 provisional).
// Plan 152 owns generated documents/schemas and decoded-hardening of this boundary.
// Behavior-preserving extraction: raw string queries, client TTL cache, and gqlString.

// The UI's only data path: GraphQL against the Parallax API (same-origin —
// the vite dev proxy and the embedded prod build both serve /graphql).

// Loaders are isomorphic (run on server AND client): relative URLs only work
// in the browser, so SSR/loader calls target the API directly.
const BASE = typeof window === "undefined" ? "http://127.0.0.1:4000" : ""

const CACHE_TTL_MS = 15_000
const CACHE_MAX = 100

/** In-flight dedup of identical query strings (client-side only). */
const inflight = new Map<string, Promise<unknown>>()
/** Short-lived result cache keyed by query string (client-side only). */
const cache = new Map<string, { at: number; data: unknown }>()

/** Test-only: clear the client GraphQL cache and inflight map. */
export function clearGraphqlCache(): void {
  cache.clear()
  inflight.clear()
}

export async function graphql<T>(query: string, init?: { signal?: AbortSignal }): Promise<T> {
  const requestInit: RequestInit = {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ query }),
  }
  if (init?.signal) requestInit.signal = init.signal
  const response = await fetch(`${BASE}/graphql`, {
    ...requestInit,
  })
  if (!response.ok) {
    throw new Error(`parallax api unreachable (${response.status})`)
  }
  const body = (await response.json()) as { data?: T; errors?: unknown[] }
  if (body.errors?.length) {
    throw new Error(`graphql error: ${JSON.stringify(body.errors)}`)
  }
  if (!body.data) {
    throw new Error("graphql response missing data")
  }
  return body.data
}

/**
 * Client-side query cache + in-flight dedup for route loaders and preload.
 *
 * Key = full query string (variables are embedded today). SSR always bypasses
 * the cache so a shared module never leaks data across requests. Pollers and
 * explicit Refresh paths should keep using raw `graphql`.
 */
export async function graphqlCached<T>(query: string, init?: { signal?: AbortSignal }): Promise<T> {
  // Cache is client-only — Bun/SSR may share the module across requests.
  if (typeof window === "undefined") {
    return graphql<T>(query, init)
  }

  const hit = cache.get(query)
  if (hit && Date.now() - hit.at < CACHE_TTL_MS) {
    return hit.data as T
  }

  const pending = inflight.get(query)
  if (pending) return pending as Promise<T>

  const p = graphql<T>(query, init).then(
    (data) => {
      // Insertion-order LRU-ish: drop oldest when over cap.
      if (cache.size >= CACHE_MAX && !cache.has(query)) {
        const oldest = cache.keys().next().value
        if (oldest !== undefined) cache.delete(oldest)
      }
      cache.set(query, { at: Date.now(), data })
      inflight.delete(query)
      return data
    },
    (error: unknown) => {
      inflight.delete(query)
      throw error
    }
  )
  inflight.set(query, p)
  return p
}

/** Escape a value for inclusion inside a GraphQL double-quoted literal. */
export function gqlString(value: string): string {
  // GraphQL string literals cannot contain raw newlines or other control chars.
  // oxlint-disable-next-line no-control-regex -- GraphQL forbids raw C0 controls
  const rawControlCharacters = /[\u0000-\u0008\u000b\u000c\u000e-\u001f]/g
  return value
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "")
    .replace(/\t/g, "\\t")
    .replace(rawControlCharacters, (c) => "\\u" + c.charCodeAt(0).toString(16).padStart(4, "0"))
}
