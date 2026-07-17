import { IconHistory, IconPlayerPlay } from "@tabler/icons-react"
import type { RefObject } from "react"

import { SnippetsMenu } from "@/features/sql/components/snippets-menu"
import { SQL_EXAMPLES } from "@/features/sql/model/sql-examples"
import type { SqlSnippet } from "@/features/sql/model/sql-snippet"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Kbd, KbdGroup } from "@/components/ui/kbd"

export function SqlEditor({
  editorRef,
  statement,
  onStatementChange,
  onRun,
  running,
  elapsedMs,
  error,
  snippetError,
  snippets,
  history,
  onSelectSnippet,
  onDeleteSnippet,
  onOpenSave,
}: {
  editorRef: RefObject<HTMLTextAreaElement | null>
  statement: string
  onStatementChange: (value: string) => void
  onRun: (sql: string) => void
  running: boolean
  elapsedMs: number | null
  error: string | null
  snippetError: string | null
  snippets: readonly SqlSnippet[]
  history: readonly string[]
  onSelectSnippet: (snippet: SqlSnippet) => void
  onDeleteSnippet: (id: string) => void
  onOpenSave: () => void
}) {
  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle className="text-sm">Editor</CardTitle>
        <KbdGroup>
          <Kbd>⌘</Kbd>
          <Kbd>Enter</Kbd>
        </KbdGroup>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <textarea
          ref={editorRef}
          name="sql-statement"
          value={statement}
          onChange={(event) => onStatementChange(event.target.value)}
          onKeyDown={(event) => {
            if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
              event.preventDefault()
              onRun(statement)
            }
          }}
          rows={9}
          spellCheck={false}
          className="min-h-56 w-full rounded-md border bg-background p-3 font-mono text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring"
        />
        <div className="flex flex-wrap items-center gap-2">
          <Button onClick={() => onRun(statement)} disabled={running}>
            <IconPlayerPlay />
            Run query
          </Button>
          <SnippetsMenu
            snippets={snippets}
            onSelect={onSelectSnippet}
            onDelete={onDeleteSnippet}
            onSave={onOpenSave}
          />
          <ExamplesMenu onSelect={onStatementChange} />
          {history.length > 0 ? (
            <HistoryMenu history={history} onSelect={onStatementChange} />
          ) : null}
          {elapsedMs != null ? (
            <span className="text-xs text-muted-foreground">
              {elapsedMs.toFixed(0)} ms
            </span>
          ) : null}
        </div>
        {snippetError ? (
          <p className="text-sm text-destructive">{snippetError}</p>
        ) : null}
        {error ? <p className="text-sm text-destructive">{error}</p> : null}
      </CardContent>
    </Card>
  )
}

function ExamplesMenu({ onSelect }: { onSelect: (sql: string) => void }) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger render={<Button variant="outline" />}>
        Examples
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuGroup>
          {SQL_EXAMPLES.map((example) => (
            <DropdownMenuItem
              key={example.label}
              onClick={() => onSelect(example.sql)}
            >
              {example.label}
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

function HistoryMenu({
  history,
  onSelect,
}: {
  history: readonly string[]
  onSelect: (sql: string) => void
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger render={<Button variant="outline" />}>
        <IconHistory />
        History
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuGroup>
          {history.map((entry, index) => (
            <DropdownMenuItem
              key={`${index}-${entry.slice(0, 20)}`}
              onClick={() => onSelect(entry)}
            >
              {entry.replace(/\s+/g, " ").slice(0, 72)}
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
