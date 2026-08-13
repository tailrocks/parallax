/** GraphQL shapes + query/mutation builders for the alerts pages (plan 167).
 *
 * Pure string builders over `gqlString` so the route components stay thin.
 * Preliminary (helper agent) — peer verifies field coverage against the API
 * and extends (threshold chart series, incident detail).
 */

import { gqlString } from "@/platform/graphql/transport"
import type { AlertRuleDraft } from "@/features/alerts/model/alert-rule-form"

export interface AlertRuleRow {
  id: string
  name: string
  enabled: boolean
  signalType: string
  services: string
  excludeServices: string
  groupBy: string | null
  comparator: string
  threshold: number
  thresholdUpper: number | null
  windowMinutes: number
  minimumSampleCount: number
  consecutiveBreachesRequired: number
  consecutiveHealthyRequired: number
  noDataBehavior: string
  severity: string
  renotifyIntervalMinutes: number
  destinationIds: string
  metricName: string | null
  metricAggregation: string | null
  updatedAtNanos: string
}

export interface AlertIncidentRow {
  id: string
  ruleId: string
  groupKey: string
  status: string
  severity: string
  firstTriggeredAtNanos: string
  lastTriggeredAtNanos: string
  resolvedAtNanos: string | null
  lastValue: number | null
  rule: { id: string; name: string } | null
}

export interface AlertDestinationRow {
  id: string
  name: string
  kind: string
  config: string
  updatedAtNanos: string
}

export interface AlertRuleStateRow {
  groupKey: string
  consecutiveBreaches: number
  consecutiveHealthy: number
  incidentOpen: boolean
  lastStatus: string | null
  lastValue: number | null
  lastSampleCount: number
  lastEvaluatedAtNanos: string | null
  lastError: string | null
}

export interface AlertCheckRow {
  groupKey: string
  checkedAtNanos: string
  value: number | null
  sampleCount: number
  status: string
  error: string | null
}

export const ALERT_RULE_FIELDS = `
  id name enabled signalType services excludeServices groupBy comparator
  threshold thresholdUpper windowMinutes minimumSampleCount
  consecutiveBreachesRequired consecutiveHealthyRequired noDataBehavior
  severity renotifyIntervalMinutes destinationIds metricName
  metricAggregation updatedAtNanos
`

export const ALERT_INCIDENT_FIELDS = `
  id ruleId groupKey status severity firstTriggeredAtNanos
  lastTriggeredAtNanos resolvedAtNanos lastValue rule { id name }
`

export const ALERTS_INDEX_QUERY = `
  {
    alertRules { ${ALERT_RULE_FIELDS} }
    alertIncidents { ${ALERT_INCIDENT_FIELDS} }
    alertDestinations { id name kind config updatedAtNanos }
  }
`

export function alertRuleDetailQuery(ruleId: string): string {
  const id = gqlString(ruleId)
  return `
    {
      alertRule(id: "${id}") { ${ALERT_RULE_FIELDS} }
      alertRuleStates(ruleId: "${id}") {
        groupKey consecutiveBreaches consecutiveHealthy incidentOpen
        lastStatus lastValue lastSampleCount lastEvaluatedAtNanos lastError
      }
      alertChecks(ruleId: "${id}") {
        groupKey checkedAtNanos value sampleCount status error
      }
      alertIncidents(ruleId: "${id}") { ${ALERT_INCIDENT_FIELDS} }
    }
  `
}

function numberField(name: string, value: number): string {
  return `${name}: ${Number.isFinite(value) ? String(value) : "0"}`
}

function draftInputFields(
  draft: AlertRuleDraft,
  options?: { id?: string; destinationIds?: string[] }
): string {
  return [
    options?.id ? `id: "${gqlString(options.id)}"` : null,
    `name: "${gqlString(draft.name)}"`,
    `enabled: ${draft.enabled ? "true" : "false"}`,
    `signalType: "${gqlString(draft.signalType)}"`,
    draft.services?.length
      ? `services: [${draft.services.map((service) => `"${gqlString(service)}"`).join(", ")}]`
      : null,
    `comparator: "${gqlString(draft.comparator)}"`,
    numberField("threshold", draft.threshold),
    draft.thresholdUpper != null ? numberField("thresholdUpper", draft.thresholdUpper) : null,
    numberField("windowMinutes", draft.windowMinutes),
    numberField("minimumSampleCount", Math.max(1, draft.minimumSampleCount)),
    numberField("consecutiveBreachesRequired", draft.consecutiveBreachesRequired),
    numberField("consecutiveHealthyRequired", draft.consecutiveHealthyRequired),
    `severity: "${gqlString(draft.severity)}"`,
    numberField("renotifyIntervalMinutes", draft.renotifyIntervalMinutes),
    options?.destinationIds?.length
      ? `destinationIds: [${options.destinationIds
          .map((destination) => `"${gqlString(destination)}"`)
          .join(", ")}]`
      : null,
    draft.metricName ? `metricName: "${gqlString(draft.metricName)}"` : null,
    draft.metricAggregation ? `metricAggregation: "${gqlString(draft.metricAggregation)}"` : null,
  ]
    .filter(Boolean)
    .join(", ")
}

/** Read-only draft evaluation (plan 171). Does not persist. */
export function alertRulePreviewQuery(draft: AlertRuleDraft): string {
  return `{ alertRulePreview(input: { ${draftInputFields(draft)} }) {
    windowMinutes
    groups { groupKey samplesSufficient points { tsNanos value sampleCount wouldFire } }
  } }`
}

/** Build the alertRuleSave mutation from a validated draft. */
export function alertRuleSaveMutation(
  draft: AlertRuleDraft,
  options?: { id?: string; destinationIds?: string[] }
): string {
  return `mutation { alertRuleSave(input: { ${draftInputFields(draft, options)} }) { id } }`
}

export function alertDestinationSaveMutation(
  name: string,
  kind: string,
  url: string,
  id?: string
): string {
  const config = JSON.stringify({ url })
  const idField = id ? `, id: "${gqlString(id)}"` : ""
  return `mutation { alertDestinationSave(name: "${gqlString(name)}", kind: "${gqlString(kind)}", config: "${gqlString(config)}"${idField}) { id } }`
}

/** Parse a JSON string array stored opaquely by the API; tolerant of junk. */
export function parseStringArray(json: string): string[] {
  try {
    const value: unknown = JSON.parse(json)
    return Array.isArray(value)
      ? value.filter((item): item is string => typeof item === "string")
      : []
  } catch {
    return []
  }
}

/** Human summary of a rule's condition, e.g. "error_rate > 0.2 over 5m". */
export function ruleConditionLabel(rule: {
  signalType: string
  comparator: string
  threshold: number
  thresholdUpper: number | null
  windowMinutes: number
}): string {
  const comparators: Record<string, string> = {
    gt: ">",
    gte: ">=",
    lt: "<",
    lte: "<=",
  }
  const op = comparators[rule.comparator]
  const range =
    rule.thresholdUpper != null
      ? `${rule.comparator === "not_between" ? "outside" : "within"} ${rule.threshold}–${rule.thresholdUpper}`
      : op
        ? `${op} ${rule.threshold}`
        : `${rule.comparator} ${rule.threshold}`
  return `${rule.signalType} ${range} over ${rule.windowMinutes}m`
}
