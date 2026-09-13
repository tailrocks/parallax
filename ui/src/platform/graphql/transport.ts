// Platform GraphQL transport (Plan 100 provisional).
// Plan 152 owns generated documents/schemas and decoded-hardening of this boundary.
// Plan 133: raw-string path remains for legacy call sites; cache is TanStack Query.

// The UI's only data path: GraphQL against the Parallax API (same-origin —
// the vite dev proxy and the embedded prod build both serve /graphql).

import { apiAuthHeaders } from "@/platform/auth/api-token"
import { getBrowserQueryClient, graphqlRawQueryKey } from "@/platform/query/graphql-query"

// Loaders are isomorphic (run on server AND client): relative URLs only work
// in the browser, so SSR/loader calls target the API directly.
const SSR_API_BASE = "http://127.0.0.1:4000"
const BASE = typeof window === "undefined" ? SSR_API_BASE : ""

/**
 * Human label for the API this UI talks to: the same-origin host in the
 * browser (fetch uses a relative URL), the direct SSR base on the server.
 */
export function apiEndpointLabel(): string {
  if (typeof window === "undefined") return SSR_API_BASE
  return window.location.host || SSR_API_BASE
}

/** Test-only: clear the Query-backed raw GraphQL cache. */
export function clearGraphqlCache(): void {
  getBrowserQueryClient()?.removeQueries({ queryKey: ["graphql-raw"] })
}

export async function graphql<T>(query: string, init?: { signal?: AbortSignal }): Promise<T> {
  const requestInit: RequestInit = {
    method: "POST",
    headers: { "content-type": "application/json", ...apiAuthHeaders() },
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
 * Query-backed cache for raw string queries (plan 133).
 * SSR always bypasses. Prefer feature queryOptions + executeGraphqlOperation.
 */
export async function graphqlCached<T>(query: string, init?: { signal?: AbortSignal }): Promise<T> {
  if (typeof window === "undefined") {
    return graphql<T>(query, init)
  }

  const queryClient = getBrowserQueryClient()
  if (!queryClient) {
    return graphql<T>(query, init)
  }

  return queryClient.fetchQuery({
    queryKey: graphqlRawQueryKey(query),
    queryFn: ({ signal }) => graphql<T>(query, { signal: init?.signal ?? signal }),
  })
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
