import { reactRouter } from "@react-router/dev/vite";
import tailwindcss from "@tailwindcss/postcss";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";
import path from "path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  // prevent vite from obscuring rust errors
  clearScreen: false,
  server: {
    // Tauri expects a fixed port, fail if that port is not available
    strictPort: true,
    // if the host Tauri is expecting is set, use it
    host: host || false,
    port: 5173,
  },
  // Env variables starting with the item of `envPrefix` will be exposed in tauri's source code through `import.meta.env`.
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target:
      process.env.TAURI_ENV_PLATFORM == "windows" ? "chrome105" : "safari13",
    // don't minify for debug builds
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  css: {
    postcss: {
      plugins: [tailwindcss()],
    },
  },
  plugins: [reactRouter(), tsconfigPaths()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./app"),
    },
  },
});
