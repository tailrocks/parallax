// Query-backed GraphQL fetch helpers (plan 133). Feature-owned key factories
// are preferred; these keys are the temporary sole-cache path for adapters
// that have not yet declared feature query modules.

import type { QueryClient } from "@tanstack/react-query"

let browserQueryClient: QueryClient | null = null

/** Install the browser-owned client from app/router composition. */
export function installBrowserQueryClient(client: QueryClient): void {
  browserQueryClient = client
}

export function getBrowserQueryClient(): QueryClient | null {
  return browserQueryClient
}

export function requireBrowserQueryClient(): QueryClient {
  if (!browserQueryClient) {
    throw new Error("QueryClient not installed — wire AppQueryProvider / getRouter first")
  }
  return browserQueryClient
}

export function graphqlOperationQueryKey(
  operationName: string,
  variablesKey: string
): readonly ["graphql", string, string] {
  return ["graphql", operationName, variablesKey] as const
}

export function graphqlRawQueryKey(query: string): readonly ["graphql-raw", string] {
  return ["graphql-raw", query] as const
}
