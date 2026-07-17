// Plan 152 — unknown-first GraphQL operation client.
// Accepts generated TypedDocumentNode + generated Zod operation-result schema.
// Legacy raw-string transport remains in transport.ts until feature plans migrate.

import type { DocumentNode } from "graphql"
import { Kind, print } from "graphql"

import { graphqlError } from "@/platform/graphql/error"
import type { TypedDocumentNode } from "@/platform/graphql/typed-document"
import { encodeGraphqlVariables } from "@/platform/graphql/variables"

// Loaders are isomorphic (run on server AND client): relative URLs only work
// in the browser, so SSR/loader calls target the API directly.
const BASE = typeof window === "undefined" ? "http://127.0.0.1:4000" : ""

const CACHE_TTL_MS = 15_000
const CACHE_MAX = 100

/** In-flight dedup of identical operation keys (client-side only). */
const inflight = new Map<string, Promise<unknown>>()
/** Short-lived result cache (client-side only). */
const cache = new Map<string, { at: number; data: unknown }>()

/** Compatible with generated Zod operation-result schemas (safeParse). */
export interface OperationResultSchema<T> {
  safeParse(input: unknown):
    | { readonly success: true; readonly data: T }
    | {
        readonly success: false
        readonly error: {
          readonly issues?: readonly { readonly path?: readonly unknown[] }[]
        }
      }
}

export interface ExecuteGraphqlOptions {
  readonly signal?: AbortSignal
}

/** Test-only: clear the client GraphQL cache and inflight map. */
export function clearGraphqlOperationCache(): void {
  cache.clear()
  inflight.clear()
}

/**
 * Execute one static GraphQL operation: print document, send
 * `{ operationName, query, variables }`, decode envelope then result once.
 */
export async function executeGraphqlOperation<TResult, TVariables>(
  document: TypedDocumentNode<TResult, TVariables>,
  resultSchema: OperationResultSchema<TResult>,
  variables: TVariables,
  options?: ExecuteGraphqlOptions
): Promise<TResult> {
  const operationName = deriveOperationName(document)
  const query = print(document as DocumentNode)
  let encodedVariables: unknown
  try {
    encodedVariables = JSON.parse(encodeGraphqlVariables(variables))
  } catch (error) {
    if (error instanceof Error && error.name === "GraphqlBoundaryError") {
      throw error
    }
    throw graphqlError("invalid-variables", { operationName })
  }

  return fetchAndDecode(
    operationName,
    query,
    encodedVariables as Record<string, unknown> | null,
    resultSchema,
    options
  )
}

/**
 * Client-only TTL cache + in-flight dedup. Key = printed document + canonical
 * variables. SSR always bypasses so a shared module never leaks across requests.
 */
export async function executeCachedGraphqlOperation<TResult, TVariables>(
  document: TypedDocumentNode<TResult, TVariables>,
  resultSchema: OperationResultSchema<TResult>,
  variables: TVariables,
  options?: ExecuteGraphqlOptions
): Promise<TResult> {
  if (typeof window === "undefined") {
    return executeGraphqlOperation(document, resultSchema, variables, options)
  }

  const operationName = deriveOperationName(document)
  const query = print(document as DocumentNode)
  const variablesKey = encodeGraphqlVariables(variables)
  const cacheKey = `${operationName}\0${query}\0${variablesKey}`

  const hit = cache.get(cacheKey)
  if (hit && Date.now() - hit.at < CACHE_TTL_MS) {
    return hit.data as TResult
  }

  const pending = inflight.get(cacheKey)
  if (pending) return pending as Promise<TResult>

  const encodedVariables = JSON.parse(variablesKey) as Record<string, unknown>
  const p = fetchAndDecode(
    operationName,
    query,
    encodedVariables,
    resultSchema,
    options
  ).then(
    (data) => {
      if (cache.size >= CACHE_MAX && !cache.has(cacheKey)) {
        const oldest = cache.keys().next().value
        if (oldest !== undefined) cache.delete(oldest)
      }
      cache.set(cacheKey, { at: Date.now(), data })
      inflight.delete(cacheKey)
      return data
    },
    (error: unknown) => {
      inflight.delete(cacheKey)
      throw error
    }
  )
  inflight.set(cacheKey, p)
  return p
}

function deriveOperationName(document: DocumentNode): string {
  const operations = document.definitions.filter(
    (definition) => definition.kind === Kind.OPERATION_DEFINITION
  )
  if (operations.length !== 1) {
    throw graphqlError("invalid-document", {
      message: "document must contain exactly one operation",
    })
  }
  const name = operations[0]?.name?.value
  if (!name) {
    throw graphqlError("invalid-document", {
      message: "operation must be named",
    })
  }
  return name
}

async function fetchAndDecode<TResult>(
  operationName: string,
  query: string,
  variables: Record<string, unknown> | null,
  resultSchema: OperationResultSchema<TResult>,
  options?: ExecuteGraphqlOptions
): Promise<TResult> {
  if (options?.signal?.aborted) {
    throw graphqlError("abort", { operationName })
  }

  let response: Response
  try {
    const requestInit: RequestInit = {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ operationName, query, variables }),
    }
    if (options?.signal) requestInit.signal = options.signal
    response = await fetch(`${BASE}/graphql`, requestInit)
  } catch (error) {
    if (isAbortError(error) || options?.signal?.aborted) {
      throw graphqlError("abort", { operationName })
    }
    throw graphqlError("transport", { operationName })
  }

  if (!response.ok) {
    throw graphqlError("http", {
      operationName,
      status: response.status,
    })
  }

  let raw: unknown
  try {
    raw = await response.json()
  } catch {
    throw graphqlError("malformed-json", {
      operationName,
      status: response.status,
    })
  }

  return decodeGraphqlEnvelope(
    raw,
    resultSchema,
    operationName,
    response.status
  )
}

function decodeGraphqlEnvelope<TResult>(
  raw: unknown,
  resultSchema: OperationResultSchema<TResult>,
  operationName: string,
  status: number
): TResult {
  if (raw === null || typeof raw !== "object" || Array.isArray(raw)) {
    throw graphqlError("invalid-envelope", { operationName, status })
  }
  const body = raw as { data?: unknown; errors?: unknown }

  if (body.errors !== undefined) {
    if (!Array.isArray(body.errors)) {
      throw graphqlError("invalid-envelope", { operationName, status })
    }
    if (body.errors.length > 0) {
      // Preserve current behavior: reject any non-empty errors even with data.
      throw graphqlError("graphql-errors", {
        operationName,
        status,
        schemaIssueCount: body.errors.length,
      })
    }
  }

  if (!("data" in body) || body.data === null || body.data === undefined) {
    throw graphqlError("invalid-envelope", {
      operationName,
      status,
      message: "graphql response missing data",
    })
  }

  const parsed = resultSchema.safeParse(body.data)
  if (!parsed.success) {
    const issues = parsed.error.issues ?? []
    const paths = issues.slice(0, 8).map((issue) => {
      const path = issue.path ?? []
      return path.map(String).join(".") || "(root)"
    })
    throw graphqlError("invalid-operation-data", {
      operationName,
      status,
      schemaIssueCount: issues.length,
      schemaIssuePaths: paths,
    })
  }
  return parsed.data
}

function isAbortError(error: unknown): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    "name" in error &&
    (error as { name: string }).name === "AbortError"
  )
}
