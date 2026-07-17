/**
 * Canonical visual golden manifest (plan 146).
 * Thresholds are exact match (maxDiffPixels: 0) unless an owner-bound exception.
 */
export interface VisualGolden {
  caseId: string
  project: "visual-chromium-linux"
  file: string
  width: number
  height: number
  theme: "dark" | "light"
  threshold: number
  sourceCommit?: string
}

export const visualManifest: VisualGolden[] = [
  {
    caseId: "pw-shell-visual",
    project: "visual-chromium-linux",
    file: "shell-root-dark.png",
    width: 1440,
    height: 900,
    theme: "dark",
    threshold: 0,
  },
  {
    caseId: "pw-investigations-visual",
    project: "visual-chromium-linux",
    file: "investigations-list-dark.png",
    width: 1440,
    height: 900,
    theme: "dark",
    threshold: 0,
  },
]
