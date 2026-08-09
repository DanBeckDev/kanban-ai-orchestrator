import path from "node:path";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "./src"),
    },
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    coverage: {
      all: true,
      provider: "v8",
      reporter: ["text", "json", "lcov"],
      include: ["src/**/*.{ts,tsx}", "scripts/**/*.mjs"],
      exclude: ["src/main.tsx", "src/test/**", "src/vite-env.d.ts"],
      thresholds: {
        branches: 80,
        functions: 80,
        lines: 80,
        statements: 80,
      },
    },
  },
});
