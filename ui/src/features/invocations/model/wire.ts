/** Invocation / session / job wire types (plan 151 residual claim). */

export interface Invocation {
  invocationId: string
  registration: "cli" | "external"
  command: string | null
  appMode: string | null
  outcome: string | null
  status: string
  exitCode: number | null
  startedAtNanos: string
  endedAtNanos: string | null
  errorCount: number
  traceCount: number
  sessionCount: number
}

export interface ObservedInvocation {
  invocationId: string
  service: string
  lastCommand: string | null
  appMode: string | null
  firstNanos: string
  lastNanos: string
  spanCount: number
  logCount: number
}

export interface Session {
  sessionId: string
  previousSessionId: string | null
  startNanos: string
  endNanos: string | null
}

export interface ScreenVisit {
  screenId: string
  visitId: string
  sessionId: string | null
  navigationSequence: number | null
  transitionReason: string | null
  enteredNanos: string
  exitedNanos: string | null
}

export interface UiAction {
  name: string
  screenId: string | null
  widgetName: string | null
  sessionId: string | null
  traceId: string
  startNanos: string
  durationMs: number
  outcome: string | null
  hasError: boolean
}

export interface BackgroundCycle {
  name: string
  count: number
  errorCount: number
  p50Ms: number | null
  p95Ms: number | null
  lastNanos: string
  lastTraceId: string
}

export interface JobAttempt {
  startNanos: string
  durationMs: number
  outcome: string | null
  hasError: boolean
  traceId: string
}

export interface Job {
  jobId: string
  jobType: string | null
  producedNanos: string | null
  attempts: JobAttempt[]
  lastTraceId: string
}

export interface Conversation {
  conversationId: string
  agentName: string | null
  providerName: string | null
  firstNanos: string
  lastNanos: string
  spanCount: number
  inputTokens: number | null
  outputTokens: number | null
}
