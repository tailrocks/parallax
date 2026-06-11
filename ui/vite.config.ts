import { defineConfig } from "vite"
import { devtools } from "@tanstack/devtools-vite"
import { tanstackStart } from "@tanstack/react-start/plugin/vite"
import viteReact from "@vitejs/plugin-react"
import tailwindcss from "@tailwindcss/vite"

const config = defineConfig({
  resolve: { tsconfigPaths: true },
  plugins: [devtools(), tailwindcss(), tanstackStart({ spa: { enabled: true } }), viteReact()],
  server: {
    // Dev: forward API calls to the local parallax serve instance, so the UI
    // is same-origin in development exactly like the embedded prod build.
    proxy: {
      "/graphql": "http://127.0.0.1:4000",
    },
  },
})

export default config
