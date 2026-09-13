export type IssueWorkflowStatus = "open" | "resolved" | "regressed"

export function isIssueWorkflowStatus(value: string): value is IssueWorkflowStatus {
  return value === "open" || value === "resolved" || value === "regressed"
}

export function issueStatusBadgeVariant(status: string): "rose" | "amber" | "emerald" {
  if (status === "resolved") return "emerald"
  if (status === "regressed") return "amber"
  return "rose"
}

export function issueNeedsAttention(status: string): boolean {
  return status === "open" || status === "regressed"
}
