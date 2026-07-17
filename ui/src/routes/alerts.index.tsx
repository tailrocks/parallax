import { createFileRoute, useRouter } from "@tanstack/react-router"
import { useState } from "react"
import { IconBell, IconPlus } from "@tabler/icons-react"

import { EmptyState } from "@/components/console/empty-state"
import { PageHeader } from "@/components/page-header"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { gqlString, graphql } from "@/lib/api"
import {
  ALERT_RULE_TEMPLATES,
  draftFromTemplate,
  validateAlertRuleDraft,
} from "@/lib/alert-rule-form"
import type { AlertRuleDraft } from "@/lib/alert-rule-form"

/** Preliminary /alerts page (plan 167 step 4 skeleton).
 *
 * The GraphQL contract below (`alertRules` query, `alertRuleSave` mutation)
 * is the shape the backend resolvers are expected to expose over the landed
 * Turso CRUD; until they land the loader degrades to a "backend not wired"
 * empty state instead of crashing the route. Peer owns the full pages
 * (rule detail with threshold chart, incidents, destinations) and evidence.
 */

export interface AlertRuleRow {
  id: string
  name: string
  enabled: boolean
  signalType: string
  comparator: string
  threshold: number
  severity: string
  windowMinutes: number
}

interface LoaderData {
  rules: AlertRuleRow[] | null
}

export const Route = createFileRoute("/alerts/")({
  loader: async (): Promise<LoaderData> => {
    try {
      const { alertRules } = await graphql<{ alertRules: AlertRuleRow[] }>(`
        {
          alertRules {
            id
            name
            enabled
            signalType
            comparator
            threshold
            severity
            windowMinutes
          }
        }
      `)
      return { rules: alertRules }
    } catch {
      // Backend field not wired yet (plan 167 step 4) — render the
      // preliminary empty state rather than a route error.
      return { rules: null }
    }
  },
  component: AlertsPage,
})

function severityVariant(severity: string): "destructive" | "secondary" {
  return severity === "critical" ? "destructive" : "secondary"
}

function AlertsPage() {
  const { rules } = Route.useLoaderData()
  const router = useRouter()
  const [open, setOpen] = useState(false)
  const [name, setName] = useState("")
  const [templateId, setTemplateId] = useState(
    ALERT_RULE_TEMPLATES[0]?.id ?? "high-error-rate"
  )
  const [error, setError] = useState<string | null>(null)

  async function create() {
    setError(null)
    const draft = draftFromTemplate(templateId, name)
    if (!draft) {
      setError("unknown template")
      return
    }
    const validation = validateAlertRuleDraft(draft)
    if (!validation.ok) {
      setError(validation.errors.join("; "))
      return
    }
    try {
      await graphql<{
        alertRuleSave: { id: string }
      }>(`mutation { alertRuleSave(${draftToArgs(draft)}) { id } }`)
      setName("")
      setOpen(false)
      await router.invalidate()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        icon={IconBell}
        iconClassName="text-amber-500"
        title="Alerts"
        description="Threshold rules over error rate, latency, throughput, logs, and metrics."
        actions={
          <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger render={<Button />}>
              <IconPlus data-icon="inline-start" />
              New rule
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>New alert rule</DialogTitle>
                <DialogDescription>
                  Start from a template; thresholds and scope can be refined on
                  the rule page.
                </DialogDescription>
              </DialogHeader>
              <div className="flex flex-col gap-2">
                <Input
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  placeholder="Checkout error rate"
                />
                <div className="flex flex-wrap gap-2">
                  {ALERT_RULE_TEMPLATES.map((template) => (
                    <Button
                      key={template.id}
                      size="sm"
                      variant={
                        template.id === templateId ? "default" : "outline"
                      }
                      onClick={() => setTemplateId(template.id)}
                    >
                      {template.label}
                    </Button>
                  ))}
                </div>
              </div>
              {error ? (
                <p className="text-sm text-destructive">{error}</p>
              ) : null}
              <DialogFooter>
                <Button disabled={!name.trim()} onClick={() => void create()}>
                  Create
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        }
      />

      {rules === null ? (
        <EmptyState
          icon={IconBell}
          title="Alerting backend not wired yet"
          description="The alert rule store and evaluator are in place; the GraphQL surface lands with plan 167 step 4."
        />
      ) : rules.length === 0 ? (
        <EmptyState
          icon={IconBell}
          title="No alert rules"
          description="Create a rule from a template to get notified about breaches."
        />
      ) : (
        <ul className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {rules.map((rule) => (
            <li key={rule.id}>
              <Card>
                <CardHeader className="flex-row items-center justify-between">
                  <CardTitle className="truncate text-sm">
                    {rule.name}
                  </CardTitle>
                  <Badge variant={severityVariant(rule.severity)}>
                    {rule.severity}
                  </Badge>
                </CardHeader>
                <CardContent className="flex items-center justify-between gap-2 text-xs text-muted-foreground">
                  <span>
                    {rule.signalType} {rule.comparator} {rule.threshold} over{" "}
                    {rule.windowMinutes}m
                  </span>
                  <Badge variant={rule.enabled ? "secondary" : "outline"}>
                    {rule.enabled ? "enabled" : "disabled"}
                  </Badge>
                </CardContent>
              </Card>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}

/** Serialize a draft into GraphQL mutation arguments (string-typed API). */
export function draftToArgs(draft: AlertRuleDraft): string {
  const parts = [
    `name: "${gqlString(draft.name)}"`,
    `enabled: ${draft.enabled}`,
    `signalType: "${gqlString(draft.signalType)}"`,
    `comparator: "${gqlString(draft.comparator)}"`,
    `threshold: ${draft.threshold}`,
    `windowMinutes: ${draft.windowMinutes}`,
    `minimumSampleCount: ${draft.minimumSampleCount}`,
    `consecutiveBreachesRequired: ${draft.consecutiveBreachesRequired}`,
    `consecutiveHealthyRequired: ${draft.consecutiveHealthyRequired}`,
    `severity: "${gqlString(draft.severity)}"`,
    `renotifyIntervalMinutes: ${draft.renotifyIntervalMinutes}`,
  ]
  if (draft.thresholdUpper != null) {
    parts.push(`thresholdUpper: ${draft.thresholdUpper}`)
  }
  if (draft.metricName) {
    parts.push(`metricName: "${gqlString(draft.metricName)}"`)
  }
  if (draft.metricAggregation) {
    parts.push(`metricAggregation: "${gqlString(draft.metricAggregation)}"`)
  }
  if (draft.services && draft.services.length > 0) {
    const list = draft.services
      .map((service) => `"${gqlString(service)}"`)
      .join(", ")
    parts.push(`services: [${list}]`)
  }
  return parts.join(", ")
}
