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
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
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
            autoComplete="username"
            value={username}
            onChange={(event) => setUsername(event.target.value)}
            placeholder="operator@example.com"
            required
            autoFocus
          />
        </label>
        <label className="grid gap-1.5 text-sm">
          <span className="font-medium">Password</span>
          <Input
            type="password"
            autoComplete="current-password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            required
          />
        </label>
        {error ? <p className="text-sm text-destructive">{error}</p> : null}
        <Button type="submit" disabled={pending || !username.trim() || !password}>
          {pending ? "Signing in…" : "Sign in"}
        </Button>
      </form>
    </div>
  )
}
