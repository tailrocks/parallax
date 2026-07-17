// Plan 152 — local TypedDocumentNode type.
// Avoids the empty-main `@graphql-typed-document-node/core` package which
// Oxc resolution rejects (types-only package with main: "").

import type { DocumentNode } from "graphql"

/**
 * DocumentNode branded with result/variable types for static operations.
 * Compatible with the TypedDocumentNode community shape.
 */
export type TypedDocumentNode<
  TResult = { readonly [key: string]: unknown },
  TVariables = { readonly [key: string]: unknown },
> = DocumentNode & {
  /**
   * Phantom type carrier — never present at runtime.
   * @internal
   */
  readonly __apiType?: (variables: TVariables) => TResult
}
