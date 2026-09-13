import { useEffect, useState } from "react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  loadSourceMaps,
  uploadSourceMap,
  type SourceMapArtifactInfo,
} from "@/features/issues/api/issues-api"
import { RelativeTime } from "@/shared/console/relative-time"
import { SectionError } from "@/shared/console/error-state"

/** Basename of a frame file, mirroring the server's artifact matching. */
export function frameBasename(file: string): string {
  const noQuery = file.split(/[?#]/)[0] ?? file
  return noQuery.split("/").pop() ?? file
}

type ArtifactsState =
  | { readonly status: "loading" }
  | { readonly status: "error"; readonly message: string }
  | { readonly status: "ready"; readonly artifacts: readonly SourceMapArtifactInfo[] }

export function SourceMapCard({
  service,
  version,
  frameFile,
  onUploaded,
}: {
  service: string
  version: string | null
  frameFile: string | null
  onUploaded: () => void
}) {
  const [artifacts, setArtifacts] = useState<ArtifactsState>({ status: "loading" })
  const [file, setFile] = useState(frameFile ? frameBasename(frameFile) : "")
  const [debugId, setDebugId] = useState("")
  const [mapText, setMapText] = useState("")
  const [mapName, setMapName] = useState<string | null>(null)
  const [uploading, setUploading] = useState(false)
  const [uploadError, setUploadError] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)

  useEffect(() => {
    if (!version) {
      setArtifacts({ status: "ready", artifacts: [] })
      return
    }
    let active = true
    setArtifacts({ status: "loading" })
    loadSourceMaps(service, version)
      .then((loaded) => {
        if (active) setArtifacts({ status: "ready", artifacts: loaded })
      })
      .catch((error: unknown) => {
        if (active) {
          setArtifacts({
            status: "error",
            message: error instanceof Error ? error.message : String(error),
          })
        }
      })
    return () => {
      active = false
    }
  }, [service, version])

  useEffect(() => {
    setFile(frameFile ? frameBasename(frameFile) : "")
  }, [frameFile])

  if (!version) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-sm">Source maps</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">
            This event carries no service version, so minified frames cannot be mapped. Emit
            resource attribute <code className="font-mono text-xs">service.version</code> and upload
            the build&apos;s source map to resolve them.
          </p>
        </CardContent>
      </Card>
    )
  }

  async function submit() {
    const release = version
    if (!release || !mapText.trim() || !file.trim()) return
    setUploading(true)
    setUploadError(null)
    try {
      await uploadSourceMap({
        service,
        version: release,
        file: file.trim(),
        map: mapText,
        debugId: debugId.trim() ? debugId.trim() : null,
      })
      setMapText("")
      setMapName(null)
      setShowForm(false)
      const loaded = await loadSourceMaps(service, release)
      setArtifacts({ status: "ready", artifacts: loaded })
      onUploaded()
    } catch (error) {
      setUploadError(error instanceof Error ? error.message : String(error))
    } finally {
      setUploading(false)
    }
  }

  async function pickFile(input: HTMLInputElement | null) {
    const picked = input?.files?.[0]
    if (!picked) return
    try {
      const text = await picked.text()
      setMapText(text)
      setMapName(picked.name)
      setUploadError(null)
    } catch (error) {
      setUploadError(error instanceof Error ? error.message : String(error))
    }
  }

  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle className="text-sm">Source maps · {version}</CardTitle>
        <Button size="sm" variant="outline" onClick={() => setShowForm((value) => !value)}>
          {showForm ? "Cancel" : "Upload map"}
        </Button>
      </CardHeader>
      <CardContent className="space-y-3">
        {artifacts.status === "loading" ? (
          <p className="text-sm text-muted-foreground">Loading artifacts…</p>
        ) : artifacts.status === "error" ? (
          <SectionError message={artifacts.message} />
        ) : artifacts.artifacts.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            No source maps stored for {service} {version}. Upload the build&apos;s{" "}
            <code className="font-mono text-xs">.map</code> file to resolve minified frames.
          </p>
        ) : (
          <ul className="space-y-1">
            {artifacts.artifacts.map((artifact) => (
              <li
                key={artifact.file}
                className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground"
              >
                <span className="font-mono text-foreground">{artifact.file}</span>
                <Badge variant="secondary">{artifact.mapBytes.toLocaleString()} bytes</Badge>
                <span title={artifact.mapSha256}>sha {artifact.mapSha256.slice(0, 12)}</span>
                <RelativeTime nanos={artifact.uploadedAtNanos} />
              </li>
            ))}
          </ul>
        )}
        {showForm ? (
          <div className="grid gap-3 rounded-md border bg-muted/30 p-3">
            <div className="grid gap-1.5">
              <Label htmlFor="sourcemap-file">Minified file</Label>
              <Input
                id="sourcemap-file"
                value={file}
                placeholder="app.min.js"
                onChange={(event) => setFile(event.target.value)}
              />
            </div>
            <div className="grid gap-1.5">
              <Label htmlFor="sourcemap-debug">Debug ID (optional)</Label>
              <Input
                id="sourcemap-debug"
                value={debugId}
                placeholder="build debug id"
                onChange={(event) => setDebugId(event.target.value)}
              />
            </div>
            <div className="grid gap-1.5">
              <Label htmlFor="sourcemap-map">Source map (.map)</Label>
              <Input
                id="sourcemap-map"
                type="file"
                accept=".map,application/json"
                onChange={(event) => void pickFile(event.target)}
              />
              {mapName ? (
                <p className="text-xs text-muted-foreground">
                  {mapName} · {mapText.length.toLocaleString()} bytes
                </p>
              ) : null}
            </div>
            {uploadError ? <SectionError message={uploadError} /> : null}
            <div>
              <Button
                size="sm"
                disabled={uploading || !mapText.trim() || !file.trim()}
                onClick={() => void submit()}
              >
                {uploading ? "Uploading…" : "Upload"}
              </Button>
            </div>
          </div>
        ) : null}
      </CardContent>
    </Card>
  )
}
