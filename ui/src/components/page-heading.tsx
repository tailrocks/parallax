import type { ReactNode } from "react"

import { PageHeader } from "@/components/page-header"

/** @deprecated Use PageHeader directly in redesigned route plans. */
export function PageHeading({
  title,
  description,
  action,
}: {
  eyebrow?: string
  title: string
  description?: string
  action?: ReactNode
}) {
  return (
    <PageHeader title={title} description={description} actions={action} />
  )
}
