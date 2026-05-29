import react from "@vitejs/plugin-react"
import { fileURLToPath, URL } from "node:url"
import { defineConfig } from "vitest/config"

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
    fs: {
      allow: [webRoot, sdkRoot],
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
    css: true,
    clearMocks: true,
    restoreMocks: true,
    testTimeout: 15000,
  },
})
