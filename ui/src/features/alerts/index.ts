// Public facade for alerts (plan 151 residual claim from lib/*).
// Named exports only — no wildcard barrel.

export {
  buildIncidentTimeline,
  severityMixSegments,
  type CheckSnapshot,
  type DeliveryEventSnapshot,
  type IncidentSnapshot,
  type IncidentTimelineEvent,
  type IncidentTimelineKind,
} from "@/features/alerts/model/alert-incident-timeline"
export {
  ALERT_RULE_TEMPLATES,
  draftFromTemplate,
  metricGraduationDraft,
  validateAlertRuleDraft,
  type AlertComparator,
  type AlertRuleDraft,
  type AlertRuleTemplate,
  type AlertRuleValidation,
  type AlertSeverity,
  type AlertSignalType,
} from "@/features/alerts/model/alert-rule-form"
export {
  ALERTS_INDEX_QUERY,
  ALERT_INCIDENT_FIELDS,
  ALERT_RULE_FIELDS,
  alertDestinationSaveMutation,
  alertRuleDetailQuery,
  alertRulePreviewQuery,
  alertRuleSaveMutation,
  parseStringArray,
  ruleConditionLabel,
  type AlertCheckRow,
  type AlertDestinationRow,
  type AlertIncidentRow,
  type AlertRuleRow,
  type AlertRuleStateRow,
} from "@/features/alerts/api/alerts-gql"
