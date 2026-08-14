export function IncidentBundlePanel({
  markdown,
  canonicalHash,
}: {
  markdown: string | null | undefined
  canonicalHash: string | null | undefined
}) {
  if (!markdown) {
    return (
      <div
        data-testid="incident-bundle-panel"
        className="rounded-lg border border-dashed p-3 text-sm text-muted-foreground"
      >
        No evidence bundle on this incident.
      </div>
    )
  }
  return (
    <div data-testid="incident-bundle-panel" className="grid gap-2 rounded-lg border p-3">
      {canonicalHash ? (
        <p className="font-mono text-xs text-muted-foreground">bundle {canonicalHash}</p>
      ) : null}
      <pre className="max-h-80 overflow-auto font-mono text-xs leading-5 whitespace-pre-wrap">
        {markdown}
      </pre>
    </div>
  )
}
