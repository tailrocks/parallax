import type { IssueDetail } from "@/features/issues/model/issue-detail"

export function issueGroupingCard(issue: IssueDetail) {
  const explanation = issue.groupingExplanation
  if (!explanation) {
    return null
  }
  return (
    <GroupingCard
      algorithmVersion={explanation.algorithmVersion}
      errorType={explanation.errorType}
      messageTemplate={explanation.messageTemplate}
      anchorFrame={explanation.anchorFrame}
      operation={explanation.operation}
    />
  )
}

export function GroupingCard({
  algorithmVersion,
  errorType,
  messageTemplate,
  anchorFrame,
  operation,
}: {
  algorithmVersion: string
  errorType: string
  messageTemplate: string
  anchorFrame: string
  operation: string | null
}) {
  const parts = [
    errorType || null,
    messageTemplate ? `template ${messageTemplate}` : null,
    anchorFrame ? `frame ${anchorFrame}` : null,
    operation ? `op ${operation}` : null,
  ].filter((part): part is string => Boolean(part))
  return (
    <div data-testid="grouping-card" className="rounded-lg border p-3 text-sm">
      <p className="text-xs text-muted-foreground">{algorithmVersion}</p>
      <p>Grouped by: {parts.join(" · ") || "fingerprint inputs unavailable"}</p>
    </div>
  )
}
