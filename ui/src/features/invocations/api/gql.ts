/** Temporary raw-string GraphQL helpers for invocations until plan 133/152 typed documents own them. */
export { clearGraphqlCache, gqlString, graphql, graphqlCached } from "@/platform/graphql/transport"
export type { Invocation, ObservedInvocation } from "@/features/invocations/model/wire"
