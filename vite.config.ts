import path from "path";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react()],

  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
  },

  resolve: {
    alias: {
      "~": path.resolve(__dirname, "./src"),
    },
  },
});

