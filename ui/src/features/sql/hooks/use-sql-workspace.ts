import { useEffect, useRef, useState, type RefObject } from "react"

import {
  deleteSqlSnippet,
  loadSqlSchema,
  loadSqlSnippets,
  runSql,
  saveSqlSnippet,
} from "@/features/sql/api/sql-api"
import { loadSqlHistory, recordSqlHistory } from "@/features/sql/api/sql-history-repository"
import { SQL_EXAMPLES } from "@/features/sql/model/sql-examples"
import { sqlErrorMessage } from "@/features/sql/model/sql-error"
import type { SqlResult } from "@/features/sql/model/sql-result"
import type { SchemaColumn } from "@/features/sql/model/sql-row"
import type { SqlSnippet } from "@/features/sql/model/sql-snippet"
import { monotonicNowMs } from "@/platform/browser/monotonic-now"

export type SqlWorkspace = {
  editorRef: RefObject<HTMLTextAreaElement | null>
  statement: string
  setStatement: (value: string) => void
  result: SqlResult | null
  error: string | null
  running: boolean
  elapsedMs: number | null
  history: string[]
  schema: Map<string, SchemaColumn[]>
  openTable: string | null
  setOpenTable: (table: string | null) => void
  snippets: SqlSnippet[]
  snippetError: string | null
  saveOpen: boolean
  setSaveOpen: (open: boolean) => void
  snippetName: string
  setSnippetName: (name: string) => void
  savingSnippet: boolean
  insertIdentifier: (identifier: string) => void
  run: (sql: string) => Promise<void>
  saveSnippet: () => Promise<void>
  deleteSnippet: (id: string) => Promise<void>
  toggleTable: (table: string) => void
}

export function useSqlWorkspace(searchQuery?: string): SqlWorkspace {
  const editorRef = useRef<HTMLTextAreaElement>(null)
  const [statement, setStatement] = useState(searchQuery ?? SQL_EXAMPLES[0]?.sql ?? "")
  const [result, setResult] = useState<SqlResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [running, setRunning] = useState(false)
  const [elapsedMs, setElapsedMs] = useState<number | null>(null)
  const [history, setHistory] = useState<string[]>(() => loadSqlHistory())
  const [schema, setSchema] = useState<Map<string, SchemaColumn[]>>(new Map())
  const [openTable, setOpenTable] = useState<string | null>(null)
  const [snippets, setSnippets] = useState<SqlSnippet[]>([])
  const [snippetError, setSnippetError] = useState<string | null>(null)
  const [saveOpen, setSaveOpen] = useState(false)
  const [snippetName, setSnippetName] = useState("")
  const [savingSnippet, setSavingSnippet] = useState(false)

  useEffect(() => {
    if (searchQuery) setStatement(searchQuery)
  }, [searchQuery])

  useEffect(() => {
    void loadSqlSchema()
      .then(setSchema)
      .catch((err: unknown) => setError(sqlErrorMessage(err)))
  }, [])

  useEffect(() => {
    void loadSqlSnippets()
      .then(setSnippets)
      .catch((err: unknown) => setSnippetError(sqlErrorMessage(err)))
  }, [])

  function insertIdentifier(identifier: string) {
    const textarea = editorRef.current
    if (!textarea) {
      setStatement((current) => `${current} ${identifier}`)
      return
    }
    const start = textarea.selectionStart
    const end = textarea.selectionEnd
    setStatement((current) => `${current.slice(0, start)}${identifier}${current.slice(end)}`)
    requestAnimationFrame(() => {
      textarea.focus()
      textarea.setSelectionRange(start + identifier.length, start + identifier.length)
    })
  }

  async function run(sql: string) {
    setRunning(true)
    setError(null)
    const startedAt = monotonicNowMs()
    try {
      const next = await runSql(sql)
      setResult(next)
      setElapsedMs(monotonicNowMs() - startedAt)
      setHistory((current) => recordSqlHistory(current, sql).entries)
    } catch (err) {
      setResult(null)
      setElapsedMs(null)
      setError(sqlErrorMessage(err))
    } finally {
      setRunning(false)
    }
  }

  async function saveSnippet() {
    const name = snippetName.trim()
    if (!name) return
    setSavingSnippet(true)
    setSnippetError(null)
    try {
      const saved = await saveSqlSnippet({ name, state: statement })
      setSnippets((current) => [saved, ...current.filter((snippet) => snippet.id !== saved.id)])
      setSaveOpen(false)
      setSnippetName("")
    } catch (err) {
      setSnippetError(sqlErrorMessage(err))
    } finally {
      setSavingSnippet(false)
    }
  }

  async function deleteSnippet(id: string) {
    setSnippetError(null)
    try {
      await deleteSqlSnippet(id)
      setSnippets((current) => current.filter((snippet) => snippet.id !== id))
    } catch (err) {
      setSnippetError(sqlErrorMessage(err))
    }
  }

  function toggleTable(table: string) {
    setOpenTable((current) => (current === table ? null : table))
  }

  return {
    editorRef,
    statement,
    setStatement,
    result,
    error,
    running,
    elapsedMs,
    history,
    schema,
    openTable,
    setOpenTable,
    snippets,
    snippetError,
    saveOpen,
    setSaveOpen,
    snippetName,
    setSnippetName,
    savingSnippet,
    insertIdentifier,
    run,
    saveSnippet,
    deleteSnippet,
    toggleTable,
  }
}
