import { createFileRoute, useNavigate } from "@tanstack/react-router"
import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"

export const Route = createFileRoute("/traces/")({ component: TraceLookup })

/** Lifecycle-4 entry point: paste a trace id, land on its waterfall. */
function TraceLookup() {
  const [traceId, setTraceId] = useState("")
  const navigate = useNavigate()
  return (
    <div className="max-w-xl space-y-4">
      <h1 className="text-lg font-semibold">Trace lookup</h1>
      <p className="text-sm text-muted-foreground">
        Paste a trace ID — from an error page, a log line, or{" "}
        <code>parallax issue context</code> — to see the full cross-service
        picture.
      </p>
      <form
        className="flex gap-2"
        onSubmit={(event) => {
          event.preventDefault()
          const id = traceId.trim()
          if (id) {
            navigate({ to: "/traces/$traceId", params: { traceId: id } })
          }
        }}
      >
        <Input
          value={traceId}
          onChange={(event) => setTraceId(event.target.value)}
          placeholder="4bf92f3577b34da6a3ce929d0e0e4736"
          className="font-mono"
        />
        <Button type="submit">Open</Button>
      </form>
    </div>
  )
}
