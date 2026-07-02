import type { LucideIcon } from "lucide-react"
import type { CSSProperties } from "react"

type KpiStyle = CSSProperties & { "--kpi-color": string }

export function KpiCard({
  icon: Icon,
  label,
  value,
  delta,
  detail,
  tone = "blue",
  bars,
}: {
  icon: LucideIcon
  label: string
  value: string
  delta?: string
  detail?: string
  tone?: "blue" | "orange" | "rose" | "green" | "violet" | "fuchsia"
  bars?: number[]
}) {
  const colorVar = `var(--brand-${tone})`
  const max = Math.max(1, ...(bars ?? [1]))
  const toneStyle: KpiStyle = {
    "--kpi-color": colorVar,
    backgroundColor: "var(--kpi-color)",
  }

  return (
    <div className="parallax-panel p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="flex min-w-0 items-center gap-2">
          <span
            className="grid size-7 shrink-0 place-items-center rounded-xl text-white shadow-[0_0_18px_color-mix(in_srgb,var(--kpi-color),transparent_72%)]"
            style={toneStyle}
          >
            <Icon />
          </span>
          <span className="truncate text-sm text-muted-foreground">
            {label}
          </span>
        </div>
        {delta ? (
          <span
            className="rounded-full px-2 py-0.5 text-xs font-medium"
            style={{
              color: colorVar,
              backgroundColor: `color-mix(in srgb, ${colorVar}, transparent 86%)`,
            }}
          >
            {delta}
          </span>
        ) : null}
      </div>
      <div className="mt-4 flex items-end justify-between gap-3">
        <p className="text-2xl font-semibold tracking-tight tabular-nums">
          {value}
        </p>
        {detail ? (
          <p className="truncate text-xs text-muted-foreground">{detail}</p>
        ) : null}
      </div>
      {bars && bars.length > 0 ? (
        <div className="mt-5 flex h-7 items-end gap-1">
          {bars.slice(-24).map((bar, index) => (
            <span
              key={`${bar}-${index}`}
              className="w-1 flex-1 rounded-full bg-[color-mix(in_srgb,var(--kpi-color),transparent_12%)] opacity-90"
              style={
                {
                  "--kpi-color": colorVar,
                  height: `${Math.max(12, (bar / max) * 100)}%`,
                } as KpiStyle
              }
            />
          ))}
        </div>
      ) : null}
    </div>
  )
}
