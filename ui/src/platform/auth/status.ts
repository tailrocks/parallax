export type AuthStatus = {
  loginEnabled: boolean
  username: string | null
}

export async function loadAuthStatus(signal?: AbortSignal): Promise<AuthStatus> {
  try {
    const response = await fetch("/api/auth/status", { signal })
    if (!response.ok) {
      return { loginEnabled: false, username: null }
    }
    const body = (await response.json()) as {
      login_enabled?: boolean
      username?: string | null
    }
    return {
      loginEnabled: Boolean(body.login_enabled),
      username: body.username ?? null,
    }
  } catch {
    return { loginEnabled: false, username: null }
  }
}

export async function loginWithPassword(
  username: string,
  password: string,
): Promise<{ token: string; username: string }> {
  const response = await fetch("/api/auth/login", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ username, password }),
  })
  if (response.status === 404) {
    throw new Error("Login is disabled on this server")
  }
  if (!response.ok) {
    throw new Error("Invalid username or password")
  }
  return (await response.json()) as { token: string; username: string }
}
