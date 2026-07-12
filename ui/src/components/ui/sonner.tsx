"use client"

import {
  IconAlertCircleFilled,
  IconAlertTriangleFilled,
  IconCircleCheckFilled,
  IconInfoCircleFilled,
  IconLoader,
} from "@tabler/icons-react"
import { useTheme } from "next-themes"
import { Toaster as Sonner } from "sonner"
import type { ToasterProps } from "sonner"

const Toaster = ({ theme: themeProp, ...props }: ToasterProps) => {
  const { theme = "system" } = useTheme()

  return (
    <Sonner
      theme={themeProp ?? (theme as NonNullable<ToasterProps["theme"]>)}
      className="toaster group"
      icons={{
        success: <IconCircleCheckFilled className="size-4 text-green-500" />,
        info: <IconInfoCircleFilled className="size-4" />,
        warning: <IconAlertTriangleFilled className="size-4" />,
        error: <IconAlertCircleFilled className="size-4 text-red-500" />,
        loading: <IconLoader className="size-4 animate-spin" />,
      }}
      style={
        {
          "--normal-bg": "var(--popover)",
          "--normal-text": "var(--popover-foreground)",
          "--normal-border": "var(--border)",
          "--border-radius": "var(--radius)",
        } as React.CSSProperties
      }
      toastOptions={{
        classNames: {
          toast: "cn-toast",
        },
      }}
      {...props}
    />
  )
}

export { Toaster }
