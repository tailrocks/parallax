import { IconBookmark, IconDeviceFloppy, IconTrash } from "@tabler/icons-react"

import type { SqlSnippet } from "@/features/sql/model/sql-snippet"
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

export function SnippetsMenu({
  snippets,
  onSelect,
  onDelete,
  onSave,
}: {
  snippets: readonly SqlSnippet[]
  onSelect: (snippet: SqlSnippet) => void
  onDelete: (id: string) => void
  onSave: () => void
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger render={<Button variant="outline" />}>
        <IconBookmark />
        Snippets
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-72">
        <DropdownMenuLabel>Named snippets</DropdownMenuLabel>
        <DropdownMenuGroup>
          {snippets.length === 0 ? (
            <DropdownMenuItem disabled>No snippets</DropdownMenuItem>
          ) : (
            snippets.map((snippet) => (
              <DropdownMenuItem key={snippet.id} onClick={() => onSelect(snippet)}>
                <IconBookmark />
                <span className="truncate">{snippet.name}</span>
              </DropdownMenuItem>
            ))
          )}
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={onSave}>
          <IconDeviceFloppy />
          Save current snippet
        </DropdownMenuItem>
        {snippets.length > 0 ? (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuLabel>Delete snippet</DropdownMenuLabel>
            {snippets.map((snippet) => (
              <DropdownMenuItem
                key={`delete-${snippet.id}`}
                variant="destructive"
                onClick={(event) => {
                  event.preventDefault()
                  onDelete(snippet.id)
                }}
              >
                <IconTrash />
                <span className="truncate">{snippet.name}</span>
              </DropdownMenuItem>
            ))}
          </>
        ) : null}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
