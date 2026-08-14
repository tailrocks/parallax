import { IconMapQuestion } from "@tabler/icons-react"

import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty"

export function RouteNotFoundPanel() {
  return (
    <Empty className="max-w-2xl">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <IconMapQuestion />
        </EmptyMedia>
        <EmptyTitle>Nothing is mounted here</EmptyTitle>
        <EmptyDescription>Pick a Parallax surface from the navigation.</EmptyDescription>
      </EmptyHeader>
    </Empty>
  )
}
