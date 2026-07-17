import { useEffect, useState } from "react"

import {
  browserPageVisibility,
  type PageVisibilitySource,
} from "@/platform/visibility/page-visibility"

/** True when the document is visible (or during SSR where document is absent). */
export function usePageVisible(
  source: PageVisibilitySource = browserPageVisibility
): boolean {
  const [visible, setVisible] = useState(() => source.isVisible())
  useEffect(() => {
    setVisible(source.isVisible())
    return source.subscribe(() => setVisible(source.isVisible()))
  }, [source])
  return visible
}
