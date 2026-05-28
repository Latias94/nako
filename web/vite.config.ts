import react from "@vitejs/plugin-react"
import { fileURLToPath, URL } from "node:url"
import { defineConfig } from "vite"

const webRoot = fileURLToPath(new URL(".", import.meta.url))
const sdkRoot = fileURLToPath(new URL("../sdk/typescript", import.meta.url))

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": webRoot,
    },
  },
  server: {
    host: "127.0.0.1",
    port: 3000,
    strictPort: true,
    fs: {
      allow: [webRoot, sdkRoot],
    },
  },
  preview: {
    host: "127.0.0.1",
    port: 4173,
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "es2022",
  },
})
