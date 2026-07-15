import { defineConfig } from "vitest/config"
import { devtools } from "@tanstack/devtools-vite"
import { tanstackStart } from "@tanstack/react-start/plugin/vite"
import viteReact from "@vitejs/plugin-react"
import tailwindcss from "@tailwindcss/vite"

const config = defineConfig({
  resolve: { tsconfigPaths: true },
  test: {
    include: ["src/**/*.test.{ts,tsx}", "tests/harness/**/*.test.{ts,tsx}"],
    exclude: ["tests/e2e/**"],
    setupFiles: ["src/test/setup.ts"],
    server: {
      // Bun executes Vitest in this repository. Inline Zod so Vite transforms
      // its conditional ESM exports instead of handing Bun an externalized
      // namespace with missing named exports.
      deps: { inline: ["zod"] },
    },
  },
  plugins: [
    devtools(),
    tailwindcss(),
    tanstackStart({
      prerender: {
        enabled: false,
        failOnError: false,
      },
      spa: {
        enabled: true,
        prerender: {
          enabled: false,
        },
      },
    }),
    viteReact(),
  ],
  server: {
    // Dev: forward API calls to the local parallax serve instance, so the UI
    // is same-origin in development exactly like the embedded prod build.
    proxy: {
      "/graphql": "http://127.0.0.1:4000",
    },
  },
})

export default config
