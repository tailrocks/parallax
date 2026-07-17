import { IconAlertTriangle } from "@tabler/icons-react"

import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { EvidenceGap } from "@/lib/api"

export function EvidenceGapsCard({ gaps }: { gaps: EvidenceGap[] }) {
  if (gaps.length === 0) return null

  return (
    <Card data-testid="evidence-gaps-card">
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-sm">
          <IconAlertTriangle />
          Evidence gaps
        </CardTitle>
      </CardHeader>
      <CardContent>
        <ul className="flex flex-col gap-2">
          {gaps.map((gap) => (
            <li
              key={`${gap.kind}-${gap.subject}`}
              className="flex flex-col gap-1 rounded-lg border bg-background/70 px-3 py-2 text-sm"
            >
              <span className="flex flex-wrap items-center gap-2">
                <Badge variant="outline">{gap.kind}</Badge>
                <span className="font-mono text-xs text-muted-foreground">
                  {gap.subject}
                </span>
              </span>
              <span className="text-muted-foreground">{gap.detail}</span>
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  )
}
