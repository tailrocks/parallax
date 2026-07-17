// Plan 152 — bounded dynamic DocumentNode builder for dashboard widget series.
// Typed GraphQL AST only — never construct source text and parse it.

import {
  Kind,
  OperationTypeNode,
  type DocumentNode,
  type FieldNode,
  type OperationDefinitionNode,
  type VariableDefinitionNode,
} from "graphql"

/** Stay well under GraphQL complexity limit (~1000); 24 aliases per document. */
export const WIDGET_SERIES_CHUNK = 24

export interface WidgetSeriesInput {
  readonly metric: string
  readonly agg: string
  readonly groupBy?: string | null | undefined
}

export interface WidgetSeriesRange {
  readonly fromNanos: string
  readonly toNanos: string
}

export interface WidgetSeriesChunk {
  readonly document: DocumentNode
  readonly variables: Record<string, string | null>
  readonly aliases: readonly string[]
  /** Global zero-based ordinals covered by this chunk, in order. */
  readonly ordinals: readonly number[]
}

/**
 * Build chunked `DashboardWidgetSeries` documents.
 * Aliases are `series_<globalOrdinal>`; every argument is a Variable node.
 */
export function buildWidgetSeriesChunks(
  widgets: readonly WidgetSeriesInput[],
  range: WidgetSeriesRange
): WidgetSeriesChunk[] {
  if (widgets.length === 0) return []
  const chunks: WidgetSeriesChunk[] = []
  for (let offset = 0; offset < widgets.length; offset += WIDGET_SERIES_CHUNK) {
    const slice = widgets.slice(offset, offset + WIDGET_SERIES_CHUNK)
    chunks.push(buildOneChunk(slice, range, offset))
  }
  return chunks
}

function buildOneChunk(
  widgets: readonly WidgetSeriesInput[],
  range: WidgetSeriesRange,
  offset: number
): WidgetSeriesChunk {
  if (widgets.length > WIDGET_SERIES_CHUNK) {
    throw new Error(
      `widget series chunk exceeds ${WIDGET_SERIES_CHUNK} fields (got ${widgets.length})`
    )
  }

  const aliases: string[] = []
  const ordinals: number[] = []
  const selections: FieldNode[] = []
  const variableDefinitions: VariableDefinitionNode[] = []
  const variables: Record<string, string | null> = {}

  widgets.forEach((widget, index) => {
    const ordinal = offset + index
    const alias = `series_${ordinal}`
    aliases.push(alias)
    ordinals.push(ordinal)

    const nameVar = `name_${ordinal}`
    const fromVar = `from_${ordinal}`
    const toVar = `to_${ordinal}`
    const aggVar = `agg_${ordinal}`
    const groupByVar = `groupBy_${ordinal}`

    variables[nameVar] = widget.metric
    variables[fromVar] = range.fromNanos
    variables[toVar] = range.toNanos
    variables[aggVar] = widget.agg
    variables[groupByVar] = widget.groupBy ?? null

    for (const [varName, typeName, nonNull] of [
      [nameVar, "String", true],
      [fromVar, "String", true],
      [toVar, "String", true],
      [aggVar, "String", true],
      [groupByVar, "String", false],
    ] as const) {
      variableDefinitions.push(variableDefinition(varName, typeName, nonNull))
    }

    selections.push({
      kind: Kind.FIELD,
      alias: nameNode(alias),
      name: nameNode("metricSeries"),
      arguments: [
        argument("name", nameVar),
        argument("fromNanos", fromVar),
        argument("toNanos", toVar),
        argument("agg", aggVar),
        argument("groupBy", groupByVar),
      ],
      selectionSet: {
        kind: Kind.SELECTION_SET,
        selections: [
          {
            kind: Kind.FIELD,
            name: nameNode("groupValue"),
          },
          {
            kind: Kind.FIELD,
            name: nameNode("points"),
            selectionSet: {
              kind: Kind.SELECTION_SET,
              selections: [
                {
                  kind: Kind.FIELD,
                  name: nameNode("tsNanos"),
                },
                {
                  kind: Kind.FIELD,
                  name: nameNode("value"),
                },
              ],
            },
          },
        ],
      },
    })
  })

  const operation: OperationDefinitionNode = {
    kind: Kind.OPERATION_DEFINITION,
    operation: OperationTypeNode.QUERY,
    name: nameNode("DashboardWidgetSeries"),
    variableDefinitions,
    selectionSet: {
      kind: Kind.SELECTION_SET,
      selections,
    },
  }

  const document: DocumentNode = {
    kind: Kind.DOCUMENT,
    definitions: [operation],
  }

  return { document, variables, aliases, ordinals }
}

function nameNode(value: string) {
  return { kind: Kind.NAME as const, value }
}

function variableDefinition(
  name: string,
  typeName: string,
  nonNull: boolean
): VariableDefinitionNode {
  const named = {
    kind: Kind.NAMED_TYPE as const,
    name: nameNode(typeName),
  }
  return {
    kind: Kind.VARIABLE_DEFINITION,
    variable: {
      kind: Kind.VARIABLE,
      name: nameNode(name),
    },
    type: nonNull ? { kind: Kind.NON_NULL_TYPE as const, type: named } : named,
  }
}

function argument(name: string, variableName: string) {
  return {
    kind: Kind.ARGUMENT as const,
    name: nameNode(name),
    value: {
      kind: Kind.VARIABLE as const,
      name: nameNode(variableName),
    },
  }
}

/** Assert document has only Variable argument values and fixed selection. */
export function assertWidgetSeriesDocumentInvariants(
  document: DocumentNode
): void {
  const op = document.definitions[0]
  if (
    !op ||
    op.kind !== Kind.OPERATION_DEFINITION ||
    op.name?.value !== "DashboardWidgetSeries"
  ) {
    throw new Error("expected named DashboardWidgetSeries operation")
  }
  if (op.selectionSet.selections.length > WIDGET_SERIES_CHUNK) {
    throw new Error("too many top-level fields in widget series document")
  }
  for (const selection of op.selectionSet.selections) {
    if (selection.kind !== Kind.FIELD) {
      throw new Error("only field selections allowed")
    }
    if (selection.name.value !== "metricSeries") {
      throw new Error("only metricSeries fields allowed")
    }
    if (selection.directives && selection.directives.length > 0) {
      throw new Error("directives are not allowed")
    }
    for (const arg of selection.arguments ?? []) {
      if (arg.value.kind !== Kind.VARIABLE) {
        throw new Error("all arguments must be variables")
      }
    }
    const fieldNames = (selection.selectionSet?.selections ?? [])
      .filter((node) => node.kind === Kind.FIELD)
      .map((node) => (node.kind === Kind.FIELD ? node.name.value : ""))
    if (fieldNames[0] !== "groupValue" || fieldNames[1] !== "points") {
      throw new Error("fixed selection groupValue + points required")
    }
  }
}
