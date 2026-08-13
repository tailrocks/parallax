import { createFileRoute, useRouter } from "@tanstack/react-router"
import { useState } from "react"
import { IconBell, IconBellFilled, IconPlus, IconTrash, IconWebhook } from "@tabler/icons-react"

import { EmptyState } from "@/shared/console/empty-state"
import { RelativeTime } from "@/shared/console/relative-time"
import { PageHeader } from "@/shared/components/page-header"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
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
import { Switch } from "@/components/ui/switch"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { gqlString, graphql } from "@/platform/graphql/transport"
import {
  ALERTS_INDEX_QUERY,
  alertDestinationSaveMutation,
  parseStringArray,
  ruleConditionLabel,
} from "@/features/alerts"
import { NewRuleDialog } from "@/features/alerts/components/new-rule-dialog"
import type { AlertDestinationRow, AlertIncidentRow, AlertRuleRow } from "@/features/alerts"

/** /alerts index (plan 167 step 4, preliminary): rules, incidents, and
 * destinations over the alert GraphQL surface. Peer owns the rule-detail
 * threshold chart, incident detail, and live breach evidence.
 */

interface LoaderData {
  alertRules: AlertRuleRow[]
  alertIncidents: AlertIncidentRow[]
  alertDestinations: AlertDestinationRow[]
}

interface AlertsSearch {
  signal_type?: string | undefined
  metric_name?: string | undefined
  metric_aggregation?: string | undefined
}

function searchString(value: unknown) {
  return typeof value === "string" && value ? value : undefined
}

export const Route = createFileRoute("/alerts/")({
  validateSearch: (search: Record<string, unknown>): AlertsSearch => ({
    signal_type: searchString(search["signal_type"]),
    metric_name: searchString(search["metric_name"]),
    metric_aggregation: searchString(search["metric_aggregation"]),
  }),
  loader: () => graphql<LoaderData>(ALERTS_INDEX_QUERY),
  component: AlertsPage,
})

function SeverityBadge({ severity }: { severity: string }) {
  return <Badge variant={severity === "critical" ? "destructive" : "secondary"}>{severity}</Badge>
}

function AlertsPage() {
  const { alertRules, alertIncidents, alertDestinations } = Route.useLoaderData()
  const search = Route.useSearch()
  const graduation =
    search.signal_type === "metric" && search.metric_name
      ? {
          metricName: search.metric_name,
          metricAggregation: search.metric_aggregation ?? "avg",
        }
      : null
  const router = useRouter()
  const [actionError, setActionError] = useState<string | null>(null)

  async function mutate(mutation: string) {
    setActionError(null)
    try {
      await graphql(mutation)
      await router.invalidate()
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err))
    }
  }

  const openIncidents = alertIncidents.filter((incident) => incident.status === "open")

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        icon={IconBell}
        iconClassName="text-amber-500"
        title="Alerts"
        description="Threshold rules over live signals; incidents notify webhook destinations."
        actions={
          <NewRuleDialog
            destinations={alertDestinations}
            graduation={graduation}
            onSaved={() => void router.invalidate()}
          />
        }
      />

      {actionError ? <p className="text-sm text-destructive">{actionError}</p> : null}

      <Tabs defaultValue="rules">
        <TabsList>
          <TabsTrigger value="rules">Rules ({alertRules.length})</TabsTrigger>
          <TabsTrigger value="incidents">Incidents ({openIncidents.length} open)</TabsTrigger>
          <TabsTrigger value="destinations">Destinations ({alertDestinations.length})</TabsTrigger>
        </TabsList>

        <TabsContent value="rules">
          {alertRules.length === 0 ? (
            <EmptyState
              icon={IconBellFilled}
              title="No alert rules"
              description="Create a rule from a template — the evaluator checks it every minute once it is enabled."
            />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Rule</TableHead>
                  <TableHead>Condition</TableHead>
                  <TableHead>Scope</TableHead>
                  <TableHead>Severity</TableHead>
                  <TableHead>Enabled</TableHead>
                  <TableHead>Updated</TableHead>
                  <TableHead />
                </TableRow>
              </TableHeader>
              <TableBody>
                {alertRules.map((rule) => (
                  <TableRow key={rule.id}>
                    <TableCell>
                      <span className="font-medium">{rule.name}</span>
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {ruleConditionLabel(rule)}
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {parseStringArray(rule.services).join(", ") || "all services"}
                    </TableCell>
                    <TableCell>
                      <SeverityBadge severity={rule.severity} />
                    </TableCell>
                    <TableCell>
                      <Switch
                        checked={rule.enabled}
                        onCheckedChange={(checked) =>
                          void mutate(
                            `mutation { alertRuleSetEnabled(id: "${gqlString(rule.id)}", enabled: ${checked ? "true" : "false"}) { id enabled } }`
                          )
                        }
                        aria-label={`Enable ${rule.name}`}
                      />
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      <RelativeTime nanos={rule.updatedAtNanos} />
                    </TableCell>
                    <TableCell>
                      <DeleteButton
                        label={`Delete rule ${rule.name}?`}
                        onDelete={() =>
                          void mutate(`mutation { alertRuleDelete(id: "${gqlString(rule.id)}") }`)
                        }
                      />
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </TabsContent>

        <TabsContent value="incidents">
          {alertIncidents.length === 0 ? (
            <EmptyState
              icon={IconBell}
              title="No incidents"
              description="Incidents appear when an enabled rule breaches for its required consecutive windows."
            />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Rule</TableHead>
                  <TableHead>Group</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Severity</TableHead>
                  <TableHead className="text-right">Last value</TableHead>
                  <TableHead>First triggered</TableHead>
                  <TableHead>Last triggered</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {alertIncidents.map((incident) => (
                  <TableRow key={incident.id}>
                    <TableCell>
                      <span className="font-medium">{incident.rule?.name ?? incident.ruleId}</span>
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {incident.groupKey || "—"}
                    </TableCell>
                    <TableCell>
                      <Badge variant={incident.status === "open" ? "destructive" : "outline"}>
                        {incident.status}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <SeverityBadge severity={incident.severity} />
                    </TableCell>
                    <TableCell className="text-right tabular-nums">
                      {incident.lastValue ?? "—"}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      <RelativeTime nanos={incident.firstTriggeredAtNanos} />
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      <RelativeTime nanos={incident.lastTriggeredAtNanos} />
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </TabsContent>

        <TabsContent value="destinations">
          <div className="flex flex-col gap-3">
            <div>
              <NewDestinationDialog onSaved={() => void router.invalidate()} />
            </div>
            {alertDestinations.length === 0 ? (
              <EmptyState
                icon={IconWebhook}
                title="No destinations"
                description="Add a webhook or Slack webhook URL — rules deliver incident notifications to it."
              />
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>Kind</TableHead>
                    <TableHead>URL</TableHead>
                    <TableHead>Updated</TableHead>
                    <TableHead />
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {alertDestinations.map((destination) => (
                    <TableRow key={destination.id}>
                      <TableCell className="font-medium">{destination.name}</TableCell>
                      <TableCell className="text-muted-foreground">{destination.kind}</TableCell>
                      <TableCell className="max-w-64 truncate text-muted-foreground">
                        {destinationUrl(destination.config)}
                      </TableCell>
                      <TableCell className="text-xs text-muted-foreground">
                        <RelativeTime nanos={destination.updatedAtNanos} />
                      </TableCell>
                      <TableCell>
                        <DeleteButton
                          label={`Delete destination ${destination.name}?`}
                          onDelete={() =>
                            void mutate(
                              `mutation { alertDestinationDelete(id: "${gqlString(destination.id)}") }`
                            )
                          }
                        />
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}

function destinationUrl(config: string): string {
  try {
    const value: unknown = JSON.parse(config)
    if (value && typeof value === "object" && "url" in value) {
      const url = (value as { url?: unknown }).url
      if (typeof url === "string") return url
    }
  } catch {
    // opaque config — fall through to the raw string
  }
  return config
}

function DeleteButton({ label, onDelete }: { label: string; onDelete: () => void }) {
  return (
    <AlertDialog>
      <AlertDialogTrigger render={<Button variant="ghost-destructive" size="icon-xs" />}>
        <IconTrash />
        <span className="sr-only">Delete</span>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{label}</AlertDialogTitle>
          <AlertDialogDescription>This cannot be undone.</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction variant="destructive" onClick={onDelete}>
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

function NewDestinationDialog({ onSaved }: { onSaved: () => void }) {
  const [open, setOpen] = useState(false)
  const [name, setName] = useState("")
  const [kind, setKind] = useState("webhook")
  const [url, setUrl] = useState("")
  const [error, setError] = useState<string | null>(null)

  async function create() {
    setError(null)
    if (!/^https?:\/\//.test(url)) {
      setError("URL must start with http:// or https://")
      return
    }
    try {
      await graphql(alertDestinationSaveMutation(name, kind, url))
      setOpen(false)
      setName("")
      setUrl("")
      onSaved()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger render={<Button variant="outline" size="sm" />}>
        <IconPlus data-icon="inline-start" />
        Add destination
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New destination</DialogTitle>
          <DialogDescription>
            Incident notifications POST to this URL. Email is not available in V1.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-3">
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="alert-destination-name">Name</Label>
            <Input
              id="alert-destination-name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="Ops webhook"
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <Label>Kind</Label>
            <Select
              value={kind}
              onValueChange={(value) => {
                if (value) setKind(value)
              }}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="webhook">Webhook (JSON)</SelectItem>
                <SelectItem value="slack_webhook">Slack webhook</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="alert-destination-url">URL</Label>
            <Input
              id="alert-destination-url"
              value={url}
              onChange={(event) => setUrl(event.target.value)}
              placeholder="https://hooks.example.com/parallax"
            />
          </div>
          {error ? <p className="text-sm text-destructive">{error}</p> : null}
        </div>
        <DialogFooter>
          <Button disabled={!name.trim() || !url.trim()} onClick={() => void create()}>
            Add destination
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
