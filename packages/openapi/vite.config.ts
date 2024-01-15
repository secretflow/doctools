import { createRequire } from "node:module";
import path from "node:path";

import { lingui } from "@lingui/vite-plugin";
import react from "@vitejs/plugin-react-swc";
import { polyfillNode } from "esbuild-plugin-polyfill-node";
import { defineConfig } from "vite";
import { nodePolyfills } from "vite-plugin-node-polyfills";

import { dependencies, peerDependencies } from "./package.json";

const require = createRequire(import.meta.url);

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    nodePolyfills({
      overrides: {
        fs: "memfs",
      },
    }),
    react({
      plugins: [
        [
          "@lingui/swc-plugin",
          {
            runtimeModules: {
              i18n: [require.resolve("./src/i18n.ts"), "i18n"],
              trans: [require.resolve("./src/i18n.ts"), "Trans"],
            },
          },
        ],
        ["@swc/plugin-styled-components", { displayName: true }],
      ],
    }),
    lingui(),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  define: {
    "process.env.NODE_ENV": JSON.stringify(process.env["NODE_ENV"]),
  },
  build: {
    target: "esnext",
    outDir: "dist/esm",
    lib: {
      entry: {
        index: "./src/index.ts",
        sphinx: "./src/sphinx/index.ts",
      },
      formats: ["es"],
    },
    rollupOptions: {
      external: [...Object.keys(dependencies), ...Object.keys(peerDependencies)].map(
        (k) => new RegExp(`^${k}(/|$)`),
      ),
    },
    minify: process.env["NODE_ENV"] !== "development",
  },
  optimizeDeps: {
    esbuildOptions: {
      plugins: [
        polyfillNode({
          globals: { buffer: true },
          polyfills: {
            buffer: true,
            url: true,
            process: true,
            fs: true,
            "fs/promises": true,
          },
        }),
        {
          // silences "cannot be marked as external" errors
          // i don't know how this works
          // https://github.com/remorses/esbuild-plugins/issues/24#issuecomment-1369928859
          // https://github.com/evanw/esbuild/issues/2762
          name: "fix-node-globals-polyfill",
          setup(build) {
            build.onResolve(
              { filter: /esbuild-plugin-polyfill-node\/polyfills/ },
              (r) => ({
                path: r.path,
                external: false,
              }),
            );
          },
        },
      ],
    },
  },
});
