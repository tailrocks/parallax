import type {
  ConsoleMessage,
  Dialog,
  Download,
  Page,
  Request,
  TestInfo,
} from "@playwright/test"

export type DiagnosticKind =
  | "console-error"
  | "console-warning"
  | "pageerror"
  | "request-failed"
  | "external-network"
  | "dialog"
  | "download"

export interface DiagnosticEvent {
  kind: DiagnosticKind
  message: string
  url?: string
}

export interface DiagnosticSession {
  events: DiagnosticEvent[]
  dispose: () => void
  unexpected: () => DiagnosticEvent[]
  attach: (testInfo: TestInfo) => Promise<void>
}

const LOCAL_HOSTS = new Set(["127.0.0.1", "localhost", "[::1]"])

function isLocalUrl(raw: string): boolean {
  try {
    const url = new URL(raw)
    return LOCAL_HOSTS.has(url.hostname)
  } catch {
    return false
  }
}

/**
 * Capture unexpected browser diagnostics for a page.
 *
 * Same-origin request failures are recorded and attached but not treated as
 * unexpected during foundation stub teardown races. External network, console
 * errors/warnings, page errors, dialogs, and downloads fail the test.
 */
export function attachDiagnostics(page: Page): DiagnosticSession {
  const events: DiagnosticEvent[] = []

  const onConsole = (message: ConsoleMessage) => {
    const type = message.type()
    if (type === "error") {
      events.push({ kind: "console-error", message: message.text() })
    } else if (type === "warning") {
      events.push({ kind: "console-warning", message: message.text() })
    }
  }

  const onPageError = (error: Error) => {
    events.push({ kind: "pageerror", message: error.message })
  }

  const onRequest = (request: Request) => {
    if (!isLocalUrl(request.url())) {
      events.push({
        kind: "external-network",
        message: `external request ${request.method()} ${request.url()}`,
        url: request.url(),
      })
    }
  }

  const onRequestFailed = (request: Request) => {
    const failure = request.failure()
    events.push({
      kind: "request-failed",
      message: `${request.method()} ${request.url()} failed: ${failure?.errorText ?? "unknown"}`,
      url: request.url(),
    })
  }

  const onDialog = (dialog: Dialog) => {
    events.push({
      kind: "dialog",
      message: `${dialog.type()}: ${dialog.message()}`,
    })
    void dialog.dismiss().catch(() => undefined)
  }

  const onDownload = (download: Download) => {
    events.push({
      kind: "download",
      message: `unexpected download ${download.suggestedFilename()}`,
    })
  }

  page.on("console", onConsole)
  page.on("pageerror", onPageError)
  page.on("request", onRequest)
  page.on("requestfailed", onRequestFailed)
  page.on("dialog", onDialog)
  page.on("download", onDownload)

  return {
    events,
    dispose() {
      page.off("console", onConsole)
      page.off("pageerror", onPageError)
      page.off("request", onRequest)
      page.off("requestfailed", onRequestFailed)
      page.off("dialog", onDialog)
      page.off("download", onDownload)
    },
    unexpected() {
      return events.filter((event) => {
        if (
          event.kind === "request-failed" &&
          event.url !== undefined &&
          isLocalUrl(event.url)
        ) {
          return false
        }
        return true
      })
    },
    async attach(testInfo) {
      if (events.length === 0) return
      await testInfo.attach("diagnostics", {
        body: Buffer.from(JSON.stringify(events, null, 2)),
        contentType: "application/json",
      })
    },
  }
}
