import { RelativeTime } from "@/shared/console/relative-time"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { ServiceCatalogRow } from "@/features/services/model/service-summary"
import { formatCount } from "@/lib/format"

export function ServiceIdentityCard({
  identity,
  fallbackLastSeen,
}: {
  identity: ServiceCatalogRow | undefined
  fallbackLastSeen: string | undefined
}) {
  const sdk = [identity?.telemetrySdkName, identity?.telemetrySdkVersion]
    .filter(Boolean)
    .join(" ")
  const identityLastSeen = identity?.lastSeenNanos ?? fallbackLastSeen
  const values = [
    ["Version", identity?.serviceVersion],
    ["Namespace", identity?.serviceNamespace],
    ["Environment", identity?.deploymentEnvironment],
    ["Runtime", identity?.telemetrySdkLanguage],
    ["SDK", sdk || null],
    ["Instances", formatCount(Number(identity?.instanceCount ?? 0))],
    [
      "Last seen",
      identityLastSeen ? <RelativeTime nanos={identityLastSeen} /> : null,
    ],
  ] satisfies Array<[string, React.ReactNode]>

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Identity</CardTitle>
      </CardHeader>
      <CardContent className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        {values.map(([label, value]) => (
          <div key={label} className="space-y-1">
            <div className="text-xs text-muted-foreground">{label}</div>
            <div className="text-sm font-medium">
              {value || (
                <span className="text-muted-foreground">not emitted</span>
              )}
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}
