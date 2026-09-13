import { SERVICE_MAP_NODE_PRESENTATION } from "@/features/ecosystem/model/service-map-presentation"
import { cn } from "@/lib/utils"

export function EcosystemLegend() {
  const nodes = Object.entries(SERVICE_MAP_NODE_PRESENTATION)
  return (
    <div
      aria-label="Investigation legend"
      className="rounded-lg border bg-card/50 p-3 text-xs text-muted-foreground"
    >
      <p className="mb-2 font-medium text-foreground">Investigation legend</p>
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        <div className="flex flex-wrap gap-x-3 gap-y-1">
          {nodes.map(([kind, presentation]) => (
            <span key={kind} className="inline-flex items-center gap-1">
              <presentation.Icon className={cn("size-3", presentation.color)} />
              {presentation.label}
            </span>
          ))}
        </div>
        <div className="flex flex-wrap gap-x-3 gap-y-1">
          <span className="inline-flex items-center gap-1">
            <svg aria-hidden="true" width="28" height="8">
              <line x1="0" y1="4" x2="28" y2="4" stroke="var(--border)" strokeWidth="1" />
            </svg>
            healthy
          </span>
          <span className="inline-flex items-center gap-1">
            <svg aria-hidden="true" width="28" height="8">
              <line x1="0" y1="4" x2="28" y2="4" stroke="var(--chart-error)" strokeWidth="2" />
            </svg>
            errors
          </span>
        </div>
        <div className="flex flex-wrap gap-x-3 gap-y-1">
          {(["low", "medium", "high"] as const).map((band) => (
            <span key={band} className="inline-flex items-center gap-1">
              <svg aria-hidden="true" width="28" height="8">
                <line
                  x1="0"
                  y1="4"
                  x2="28"
                  y2="4"
                  stroke="var(--muted-foreground)"
                  strokeWidth={band === "low" ? 1 : band === "medium" ? 4 : 8}
                  className={`service-map-edge--traffic-${band}`}
                />
              </svg>
              {band}
            </span>
          ))}
          <span>width + dashes; reduced-motion pauses flow</span>
        </div>
      </div>
    </div>
  )
}
