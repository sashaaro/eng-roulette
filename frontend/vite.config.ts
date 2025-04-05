import { reactRouter } from "@react-router/dev/vite";
import tailwindcss from "tailwindcss";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

export default defineConfig({
  css: {
    postcss: {
      plugins: [tailwindcss],
    },
  },
  plugins: [reactRouter(), tsconfigPaths()],
  server: {
    port: 5157,
    host: "0.0.0.0",
    allowedHosts: true,
  },
});
