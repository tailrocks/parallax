export type Investigation = {
  readonly id: string
  readonly name: string
  readonly state: string
  readonly createdAtNanos: string
  readonly updatedAtNanos: string
}

export function mapInvestigation(raw: {
  readonly id: string
  readonly name: string
  readonly state: string
  readonly createdAtNanos: string
  readonly updatedAtNanos: string
}): Investigation {
  return { ...raw }
}
