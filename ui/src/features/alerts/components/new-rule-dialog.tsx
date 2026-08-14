import { useState } from "react"
import { IconPlus } from "@tabler/icons-react"

import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
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
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { graphql } from "@/platform/graphql/transport"
import {
  ALERT_RULE_TEMPLATES,
  alertRulePreviewQuery,
  alertRuleSaveMutation,
  draftFromTemplate,
  metricGraduationDraft,
  validateAlertRuleDraft,
  type AlertDestinationRow,
} from "@/features/alerts"

export function NewRuleDialog({
  destinations,
  graduation,
  onSaved,
}: {
  destinations: AlertDestinationRow[]
  graduation?: { metricName: string; metricAggregation: string } | null
  onSaved: () => void
}) {
  // A metric-explorer graduation handoff opens the dialog pre-filled.
  const [open, setOpen] = useState(Boolean(graduation))
  const [name, setName] = useState("")
  const [templateId, setTemplateId] = useState(ALERT_RULE_TEMPLATES[0]?.id ?? "high-error-rate")
  const [threshold, setThreshold] = useState("")
  const [selectedDestinations, setSelectedDestinations] = useState<string[]>([])
  const [error, setError] = useState<string | null>(null)
  const [preview, setPreview] = useState<string | null>(null)

  const template = ALERT_RULE_TEMPLATES.find((t) => t.id === templateId)

  function currentDraft() {
    const draft = graduation
      ? metricGraduationDraft(name, graduation.metricName, graduation.metricAggregation)
      : draftFromTemplate(templateId, name)
    if (!draft) return { draft: null, error: "unknown template" }
    if (threshold.trim()) {
      const parsed = Number(threshold)
      if (!Number.isFinite(parsed)) {
        return { draft: null, error: "threshold must be a number" }
      }
      draft.threshold = parsed
    }
    const validation = validateAlertRuleDraft(draft)
    if (!validation.ok) {
      return { draft: null, error: validation.errors.join("; ") }
    }
    return { draft, error: null }
  }

  async function runPreview() {
    setError(null)
    setPreview(null)
    const next = currentDraft()
    if (!next.draft) {
      setError(next.error ?? "invalid draft")
      return
    }
    try {
      const data = await graphql<{
        alertRulePreview: {
          windowMinutes: number
          groups: Array<{
            groupKey: string
            samplesSufficient: boolean
            points: Array<{ wouldFire: boolean }>
          }>
        }
      }>(alertRulePreviewQuery(next.draft))
      const fires = data.alertRulePreview.groups.reduce(
        (sum, group) => sum + group.points.filter((point) => point.wouldFire).length,
        0
      )
      setPreview(
        `${data.alertRulePreview.groups.length} groups, ${fires} would-fire points over ${data.alertRulePreview.windowMinutes}m`
      )
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  async function create() {
    setError(null)
    const next = currentDraft()
    if (!next.draft) {
      setError(next.error ?? "invalid draft")
      return
    }
    try {
      const draft = next.draft
      await graphql(
        alertRuleSaveMutation(draft, {
          destinationIds: selectedDestinations,
        })
      )
      setOpen(false)
      setName("")
      setThreshold("")
      setSelectedDestinations([])
      onSaved()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger render={<Button />}>
        <IconPlus data-icon="inline-start" />
        New rule
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New alert rule</DialogTitle>
          <DialogDescription>
            Start from a template; scope, thresholds, and hysteresis can be refined on the rule
            page.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-3">
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="alert-rule-name">Name</Label>
            <Input
              id="alert-rule-name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="Checkout error rate"
            />
          </div>
          {graduation ? (
            <div className="flex flex-col gap-1.5">
              <Label>Metric</Label>
              <p className="text-sm text-muted-foreground">
                <span className="font-mono">{graduation.metricName}</span> ·{" "}
                {graduation.metricAggregation}
              </p>
            </div>
          ) : (
            <div className="flex flex-col gap-1.5">
              <Label>Template</Label>
              <Select
                value={templateId}
                onValueChange={(value) => {
                  if (value) setTemplateId(value)
                }}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {ALERT_RULE_TEMPLATES.map((t) => (
                    <SelectItem key={t.id} value={t.id}>
                      {t.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="alert-rule-threshold">
              Threshold{" "}
              <span className="text-muted-foreground">
                (default {template?.draft.threshold ?? "—"})
              </span>
            </Label>
            <Input
              id="alert-rule-threshold"
              value={threshold}
              onChange={(event) => setThreshold(event.target.value)}
              placeholder={String(template?.draft.threshold ?? "")}
              inputMode="decimal"
            />
          </div>
          {destinations.length > 0 ? (
            <div className="flex flex-col gap-1.5">
              <Label>Destinations</Label>
              {destinations.map((destination) => (
                <label key={destination.id} className="flex items-center gap-2 text-sm">
                  <Checkbox
                    checked={selectedDestinations.includes(destination.id)}
                    onCheckedChange={(checked) =>
                      setSelectedDestinations((current) =>
                        checked
                          ? [...current, destination.id]
                          : current.filter((id) => id !== destination.id)
                      )
                    }
                  />
                  {destination.name}
                  <span className="text-muted-foreground">({destination.kind})</span>
                </label>
              ))}
            </div>
          ) : (
            <p className="text-xs text-muted-foreground">
              No destinations yet — the rule will open incidents in the UI only until a webhook
              destination is added.
            </p>
          )}
          {preview ? <p className="text-sm text-muted-foreground">{preview}</p> : null}
          {error ? <p className="text-sm text-destructive">{error}</p> : null}
        </div>
        <DialogFooter>
          <Button variant="outline" disabled={!name.trim()} onClick={() => void runPreview()}>
            Preview
          </Button>
          <Button disabled={!name.trim()} onClick={() => void create()}>
            Create rule
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
