/** Alert rule form validation + template presets (plan 167 UI pure layer).
 *
 * Validates comparator/threshold_upper pairing and exposes the five template
 * presets from the plan. Pure — no GraphQL.
 *
 * Preliminary — peer must build /alerts routes, wire mutations, threshold
 * charts with plan-162 tokens, and live breach evidence.
 */

export type AlertSignalType =
  | "error_rate"
  | "p95_latency"
  | "p99_latency"
  | "throughput"
  | "log_count"
  | "metric"

export type AlertComparator = "gt" | "gte" | "lt" | "lte" | "between" | "not_between"

export type AlertSeverity = "warning" | "critical"

export interface AlertRuleDraft {
  name: string
  enabled: boolean
  signalType: AlertSignalType
  comparator: AlertComparator
  threshold: number
  thresholdUpper?: number
  windowMinutes: number
  minimumSampleCount: number
  consecutiveBreachesRequired: number
  consecutiveHealthyRequired: number
  severity: AlertSeverity
  renotifyIntervalMinutes: number
  services?: string[]
  metricName?: string
  metricAggregation?: string
}

export interface AlertRuleValidation {
  ok: boolean
  errors: string[]
}

/** Validate a draft before create/update mutation. */
export function validateAlertRuleDraft(draft: AlertRuleDraft): AlertRuleValidation {
  const errors: string[] = []
  if (!draft.name.trim()) {
    errors.push("name is required")
  }
  if (!(draft.windowMinutes > 0)) {
    errors.push("windowMinutes must be > 0")
  }
  if (!(draft.minimumSampleCount >= 0)) {
    errors.push("minimumSampleCount must be >= 0")
  }
  if (!(draft.consecutiveBreachesRequired >= 1)) {
    errors.push("consecutiveBreachesRequired must be >= 1")
  }
  if (!(draft.consecutiveHealthyRequired >= 1)) {
    errors.push("consecutiveHealthyRequired must be >= 1")
  }
  if (!Number.isFinite(draft.threshold)) {
    errors.push("threshold must be a finite number")
  }
  const needsUpper = draft.comparator === "between" || draft.comparator === "not_between"
  if (needsUpper) {
    if (draft.thresholdUpper == null || !Number.isFinite(draft.thresholdUpper)) {
      errors.push("thresholdUpper is required for between/not_between")
    } else if (draft.thresholdUpper < draft.threshold) {
      errors.push("thresholdUpper must be >= threshold")
    }
  }
  if (draft.signalType === "metric" && !draft.metricName?.trim()) {
    errors.push("metricName is required for signal_type=metric")
  }
  if (draft.signalType === "error_rate" && (draft.threshold < 0 || draft.threshold > 1)) {
    errors.push("error_rate threshold should be a fraction in [0, 1]")
  }
  return { ok: errors.length === 0, errors }
}

export interface AlertRuleTemplate {
  id: string
  label: string
  draft: Omit<AlertRuleDraft, "name" | "enabled">
}

/** Plan 167 template presets for the create form. */
export const ALERT_RULE_TEMPLATES: readonly AlertRuleTemplate[] = [
  {
    id: "high-error-rate",
    label: "High error rate",
    draft: {
      signalType: "error_rate",
      comparator: "gt",
      threshold: 0.2,
      windowMinutes: 5,
      minimumSampleCount: 20,
      consecutiveBreachesRequired: 2,
      consecutiveHealthyRequired: 2,
      severity: "critical",
      renotifyIntervalMinutes: 30,
    },
  },
  {
    id: "slow-p95",
    label: "Slow p95",
    draft: {
      signalType: "p95_latency",
      comparator: "gt",
      threshold: 500,
      windowMinutes: 5,
      minimumSampleCount: 20,
      consecutiveBreachesRequired: 2,
      consecutiveHealthyRequired: 2,
      severity: "warning",
      renotifyIntervalMinutes: 30,
    },
  },
  {
    id: "slow-p99",
    label: "Slow p99",
    draft: {
      signalType: "p99_latency",
      comparator: "gt",
      threshold: 1000,
      windowMinutes: 5,
      minimumSampleCount: 20,
      consecutiveBreachesRequired: 2,
      consecutiveHealthyRequired: 2,
      severity: "warning",
      renotifyIntervalMinutes: 30,
    },
  },
  {
    id: "throughput-drop",
    label: "Throughput drop",
    draft: {
      signalType: "throughput",
      comparator: "lt",
      threshold: 1,
      windowMinutes: 5,
      minimumSampleCount: 1,
      consecutiveBreachesRequired: 2,
      consecutiveHealthyRequired: 2,
      severity: "warning",
      renotifyIntervalMinutes: 30,
    },
  },
  {
    id: "log-error-burst",
    label: "Log error burst",
    draft: {
      signalType: "log_count",
      comparator: "gt",
      threshold: 50,
      windowMinutes: 5,
      minimumSampleCount: 1,
      consecutiveBreachesRequired: 2,
      consecutiveHealthyRequired: 2,
      severity: "critical",
      renotifyIntervalMinutes: 30,
    },
  },
] as const

/** Apply a template into a named draft. */
/** Draft for a metric-explorer graduation handoff (plan 168 → 167):
 * signal_type=metric with the explored metric/aggregation pre-filled. */
export function metricGraduationDraft(
  name: string,
  metricName: string,
  metricAggregation: string
): AlertRuleDraft {
  return {
    name,
    enabled: true,
    signalType: "metric",
    comparator: "gt",
    threshold: 0,
    windowMinutes: 5,
    minimumSampleCount: 1,
    consecutiveBreachesRequired: 2,
    consecutiveHealthyRequired: 2,
    severity: "warning",
    renotifyIntervalMinutes: 30,
    metricName,
    metricAggregation,
  }
}

export function draftFromTemplate(templateId: string, name: string): AlertRuleDraft | null {
  const t = ALERT_RULE_TEMPLATES.find((x) => x.id === templateId)
  if (!t) return null
  return {
    name,
    enabled: true,
    ...t.draft,
  }
}
