// Framework-neutral story beat value (Plan 149).
// Presentation and links live in features/story.

export type StoryBeat = {
  readonly tsNanos: string
  readonly lane: string
  readonly kind: string
  readonly title: string
  readonly traceId: string
  readonly spanId: string | null
  readonly severity: string | null
  readonly durationNs: string | null
}
