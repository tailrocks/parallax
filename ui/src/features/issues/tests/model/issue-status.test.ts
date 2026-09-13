import { describe, expect, it } from "vitest"

import {
  isIssueWorkflowStatus,
  issueNeedsAttention,
  issueStatusBadgeVariant,
} from "@/features/issues/model/issue-status"

describe("issue status", () => {
  it("accepts open, resolved, and regressed", () => {
    expect(isIssueWorkflowStatus("open")).toBe(true)
    expect(isIssueWorkflowStatus("resolved")).toBe(true)
    expect(isIssueWorkflowStatus("regressed")).toBe(true)
    expect(isIssueWorkflowStatus("ignored")).toBe(false)
  })

  it("maps badge variants and attention", () => {
    expect(issueStatusBadgeVariant("open")).toBe("rose")
    expect(issueStatusBadgeVariant("regressed")).toBe("amber")
    expect(issueStatusBadgeVariant("resolved")).toBe("emerald")
    expect(issueNeedsAttention("open")).toBe(true)
    expect(issueNeedsAttention("regressed")).toBe(true)
    expect(issueNeedsAttention("resolved")).toBe(false)
  })
})
