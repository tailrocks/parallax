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
  // Plan 148 — production build contract (TanStack Start / Vite / Rolldown).
  // No direct Oxc minifier packages. Client embed must not ship source maps.
  build: {
    sourcemap: false,
    reportCompressedSize: true,
    chunkSizeWarningLimit: 150,
    rolldownOptions: {
      output: {
        // Keep heavy visualization stacks out of the shared vendor bucket when
        // the bundler can split them; route modules remain framework-owned.
        manualChunks(id: string) {
          if (!id.includes("node_modules")) return
          if (id.includes("@xyflow") || id.includes("elkjs")) return "graph-layout"
          if (id.includes("recharts")) return "charts"
          if (id.includes("@tanstack/react-virtual")) return "virtualizer"
          return
        },
      },
    },
  },
})

export default config
