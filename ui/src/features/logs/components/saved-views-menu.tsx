import { IconBookmark, IconDeviceFloppy, IconTrash } from "@tabler/icons-react"

import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
export interface SavedView {
  id: string
  name: string
  page: string
  state: string
  updatedAtNanos: string
}

export function SavedViewsMenu({
  views,
  onSelect,
  onDelete,
  onSave,
}: {
  views: SavedView[]
  onSelect: (view: SavedView) => void
  onDelete: (id: string) => void
  onSave: () => void
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger render={<Button type="button" variant="outline" size="sm" />}>
        <IconBookmark />
        Views
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-64">
        <DropdownMenuLabel>Saved views</DropdownMenuLabel>
        <DropdownMenuGroup>
          {views.length === 0 ? (
            <DropdownMenuItem disabled>No saved views</DropdownMenuItem>
          ) : (
            views.map((view) => (
              <DropdownMenuItem key={view.id} onClick={() => onSelect(view)}>
                <IconBookmark />
                <span className="truncate">{view.name}</span>
              </DropdownMenuItem>
            ))
          )}
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={onSave}>
          <IconDeviceFloppy />
          Save current view
        </DropdownMenuItem>
        {views.length > 0 ? (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuLabel>Delete view</DropdownMenuLabel>
            {views.map((view) => (
              <DropdownMenuItem
                key={`delete-${view.id}`}
                variant="destructive"
                onClick={(event) => {
                  event.preventDefault()
                  onDelete(view.id)
                }}
              >
                <IconTrash />
                <span className="truncate">{view.name}</span>
              </DropdownMenuItem>
            ))}
          </>
        ) : null}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
