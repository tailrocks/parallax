import { IconDatabase } from "@tabler/icons-react"

import { SaveSnippetDialog } from "@/features/sql/components/save-snippet-dialog"
import { SqlEditor } from "@/features/sql/components/sql-editor"
import { SqlResultBody } from "@/features/sql/components/sql-result-body"
import { SqlSchemaBrowser } from "@/features/sql/components/sql-schema-browser"
import { useSqlWorkspace } from "@/features/sql/hooks/use-sql-workspace"
import { PageHeader } from "@/shared/components/page-header"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { formatCount } from "@/shared/format"

export function SqlPage({ searchQuery }: { searchQuery?: string | undefined }) {
  const workspace = useSqlWorkspace(searchQuery)

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        icon={IconDatabase}
        iconClassName="text-yellow-500"
        title="SQL"
        description="Read-only queries over telemetry tables."
      />

      <div className="grid gap-4 lg:grid-cols-[16rem_1fr]">
        <SqlSchemaBrowser
          schema={workspace.schema}
          openTable={workspace.openTable}
          onToggleTable={workspace.toggleTable}
          onInsertIdentifier={workspace.insertIdentifier}
        />

        <div className="flex flex-col gap-3">
          <SqlEditor
            editorRef={workspace.editorRef}
            statement={workspace.statement}
            onStatementChange={workspace.setStatement}
            onRun={(sql) => void workspace.run(sql)}
            running={workspace.running}
            elapsedMs={workspace.elapsedMs}
            error={workspace.error}
            snippetError={workspace.snippetError}
            snippets={workspace.snippets}
            history={workspace.history}
            onSelectSnippet={(snippet) => workspace.setStatement(snippet.state)}
            onDeleteSnippet={(id) => void workspace.deleteSnippet(id)}
            onOpenSave={() => {
              workspace.setSnippetName("")
              workspace.setSaveOpen(true)
            }}
          />

          <SaveSnippetDialog
            open={workspace.saveOpen}
            name={workspace.snippetName}
            saving={workspace.savingSnippet}
            onOpenChange={workspace.setSaveOpen}
            onNameChange={workspace.setSnippetName}
            onSave={() => void workspace.saveSnippet()}
          />

          {workspace.result ? (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-sm">
                  Query result
                  <Badge variant="outline">{formatCount(workspace.result.rowCount)} rows</Badge>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <SqlResultBody result={workspace.result} />
              </CardContent>
            </Card>
          ) : null}
        </div>
      </div>
    </div>
  )
}
