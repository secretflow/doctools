import { fileURLToPath } from "node:url";

import { defineConfig, mergeConfig } from "vite";

import packageJson from "./package.json" with { type: "json" };
import config from "./vite.config.ts";

export default defineConfig((env) =>
  mergeConfig(
    config(env),
    defineConfig({
      build: {
        outDir: relpath("dist/esm/browser"),
        target: "esnext",
        lib: {
          formats: ["es"],
          entry: {
            ["lib.js"]: relpath("src/js/browser/lib.ts"),
            ["i18n/polyfill.js"]: relpath("src/js/browser/i18n/polyfill.ts"),
          },
          fileName: (_, entry) => entry,
          cssFileName: "lib",
        },
        rollupOptions: {
          external: Object.keys(packageJson.dependencies)
            .map((name) => `^${name}(|/.*)$`)
            .map((pat) => new RegExp(pat)),
        },
        minify: false,
        sourcemap: true,
        emptyOutDir: true,
        copyPublicDir: false,
      },
      worker: {
        format: "iife",
      },
      esbuild: {
        target: "esnext",
      },
    }),
  ),
);

function relpath(path: string) {
  return fileURLToPath(new URL(path, import.meta.url));
}
