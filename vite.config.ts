import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Vite options tailored for Tauri development and only applied in `tauri dev`
  // or `tauri build`.
  //
  // 1. Prevent vite from obscuring Rust errors.
  clearScreen: false,
  // 2. Tauri expects a fixed port. Fail if that port is not available.
  server: {
    port: 1420,
    strictPort: true,
  }
}));
