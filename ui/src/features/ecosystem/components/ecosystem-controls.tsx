import type { EcosystemSearch } from "@/features/ecosystem/model/ecosystem-search"
import type { TrafficPreset } from "@/features/ecosystem/model/ecosystem-topology"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

export function EcosystemControls({
  services,
  search,
  update,
}: {
  services: string[]
  search: EcosystemSearch
  update: (patch: Partial<EcosystemSearch>) => void
}) {
  return (
    <div className="flex flex-wrap items-center gap-2">
      <Select
        value={search.focus ?? "all"}
        onValueChange={(value) => update({ focus: !value || value === "all" ? undefined : value })}
      >
        <SelectTrigger size="sm" aria-label="Focus service">
          <SelectValue placeholder="All services" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">All services</SelectItem>
          {services.map((service) => (
            <SelectItem key={service} value={service}>
              {service}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Select
        value={String(search.hops ?? 1)}
        onValueChange={(value) => update({ hops: value === "2" ? 2 : undefined })}
      >
        <SelectTrigger size="sm" aria-label="Focus hops">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="1">1 hop</SelectItem>
          <SelectItem value="2">2 hops</SelectItem>
        </SelectContent>
      </Select>
      <Select
        value={search.focusMode ?? "dim"}
        onValueChange={(value) => update({ focusMode: value === "hide" ? "hide" : undefined })}
      >
        <SelectTrigger size="sm" aria-label="Outside focus behavior">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="dim">Dim outside</SelectItem>
          <SelectItem value="hide">Hide outside</SelectItem>
        </SelectContent>
      </Select>
      <Select
        value={search.minTraffic ?? "all"}
        onValueChange={(value) =>
          update({
            minTraffic: value && value !== "all" ? (value as TrafficPreset) : undefined,
          })
        }
      >
        <SelectTrigger size="sm" aria-label="Minimum traffic">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">All traffic</SelectItem>
          <SelectItem value="0.1%">&gt;0.1% traffic</SelectItem>
          <SelectItem value="1%">&gt;1% traffic</SelectItem>
          <SelectItem value="5%">&gt;5% traffic</SelectItem>
        </SelectContent>
      </Select>
    </div>
  )
}
