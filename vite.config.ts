import process from "node:process";

import { lingui } from "@lingui/vite-plugin";
import react from "@vitejs/plugin-react-swc";
import { polyfillNode } from "esbuild-plugin-polyfill-node";
import { defineConfig } from "vite";
import { nodePolyfills } from "vite-plugin-node-polyfills";

let {
  VITE_SERVER_HOST = "127.0.0.1",
  VITE_SERVER_PORT = 5173,
  FLASK_RUN_PORT = 5000,
} = process.env;

VITE_SERVER_PORT = Number(VITE_SERVER_PORT);
FLASK_RUN_PORT = Number(FLASK_RUN_PORT);

export default defineConfig((env) => ({
  root: relpath("src/js/browser"),
  plugins: [
    nodePolyfills({
      include: ["fs", "path", "stream"],
      overrides: {
        fs: "memfs",
      },
    }),
    lingui({
      configPath: relpath("./lingui.config.js"),
      failOnCompileError: true,
      failOnMissing: env.command !== "serve",
    }),
    react({
      plugins: [
        ["@lingui/swc-plugin", {}],
        ["@swc/plugin-styled-components", { displayName: true }],
      ],
    }),
  ],
  build: {
    outDir: relpath("dist/web"),
    copyPublicDir: false,
    emptyOutDir: true,
  },
  optimizeDeps: {
    esbuildOptions: {
      plugins: [
        polyfillNode({
          polyfills: {
            ["path"]: true,
            ["fs"]: true,
            ["fs/promises"]: true,
          },
        }),
        {
          name: "fix-node-globals-polyfill",
          setup(build) {
            build.onResolve(
              { filter: /esbuild-plugin-polyfill-node\/polyfills/ },
              (r) => ({ path: r.path, external: false }),
            );
          },
        },
      ],
    },
  },
  server: {
    host: VITE_SERVER_HOST,
    port: VITE_SERVER_PORT,
    strictPort: true,
    proxy: {
      "/static": `http://127.0.0.1:${FLASK_RUN_PORT}`,
    },
  },
  define: {
    ...Object.fromEntries(
      Object.entries({
        VITE_SERVER_HOST,
        VITE_SERVER_PORT,
        FLASK_RUN_PORT,
      }).map(([k, v]) => [`import.meta.env.${k}`, JSON.stringify(v)] as const),
    ),
  },
}));

function relpath(path: string) {
  return new URL(path, import.meta.url).pathname;
}
