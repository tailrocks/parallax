import { useState } from "react"
import { createFileRoute, redirect, useNavigate } from "@tanstack/react-router"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { loginWithPassword, loadAuthStatus } from "@/platform/auth/status"
import { setApiToken } from "@/platform/auth/api-token"

export const Route = createFileRoute("/login")({
  beforeLoad: async () => {
    if (typeof window === "undefined") return
    const status = await loadAuthStatus()
    if (!status.loginEnabled) {
      throw redirect({ to: "/" })
    }
  },
  component: LoginPage,
})

function LoginPage() {
  const navigate = useNavigate()
  const [error, setError] = useState<string | null>(null)
  const [pending, setPending] = useState(false)

  return (
    <div className="mx-auto flex min-h-[60vh] max-w-md flex-col justify-center gap-6">
      <div className="grid gap-2">
        <h1 className="font-heading text-2xl font-semibold tracking-tight">Sign in to Parallax</h1>
        <p className="text-sm text-muted-foreground">
          Use the operator account for this server. Guest access is off while login is enabled.
        </p>
      </div>
      <form
        className="grid gap-4"
        onSubmit={(event) => {
          event.preventDefault()
          const form = new FormData(event.currentTarget)
          const username = String(form.get("username") ?? "").trim()
          const password = String(form.get("password") ?? "")
          setPending(true)
          setError(null)
          void loginWithPassword(username, password)
            .then((session) => {
              setApiToken(session.token)
              void navigate({ to: "/" })
            })
            .catch((cause: unknown) => {
              setError(cause instanceof Error ? cause.message : "Sign in failed")
            })
            .finally(() => setPending(false))
        }}
      >
        <label className="grid gap-1.5 text-sm">
          <span className="font-medium">Username</span>
          <Input
            type="email"
            name="username"
            autoComplete="username"
            placeholder="operator@example.com"
            required
            autoFocus
          />
        </label>
        <label className="grid gap-1.5 text-sm">
          <span className="font-medium">Password</span>
          <Input type="password" name="password" autoComplete="current-password" required />
        </label>
        {error ? <p className="text-sm text-destructive">{error}</p> : null}
        <Button type="submit" disabled={pending}>
          {pending ? "Signing in…" : "Sign in"}
        </Button>
      </form>
    </div>
  )
}
