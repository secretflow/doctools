// @ts-check

import { fileURLToPath } from "node:url";

import * as esbuild from "esbuild";

await esbuild.build({
  entryPoints: [relpath("../src/js/deno/main.ts")],
  outfile: relpath("../src/py/secretflow_doctools/js/cli.js"),
  format: "esm",
  bundle: true,
  minify: false,
  sourcemap: false,
  platform: "neutral",
  target: ["es2022"],
  external: [
    "node:buffer",
    "node:child_process",
    "node:crypto",
    "node:events",
    "node:fs",
    "node:http",
    "node:http2",
    "node:module",
    "node:os",
    "node:path",
    "node:process",
    "node:stream",
    "node:timers",
    "node:tty",
    "node:url",
    "node:util",
  ],
  mainFields: ["browser", "module", "main"],
  conditions: ["node"],
  loader: {
    ".hbs": "text",
    ".wasm": "binary",
  },
  plugins: [importNode()],
  logOverride: {
    "ignored-bare-import": "info",
  },
});

/**
 * @param {string} path
 * @returns {string}
 */
function relpath(path) {
  return fileURLToPath(new URL(path, import.meta.url));
}

/**
 * @returns {esbuild.Plugin}
 */
function importNode() {
  return {
    name: "import-node",
    setup: (build) => {
      const expected =
        build.initialOptions.external
          ?.filter((m) => m.startsWith("node:"))
          .map((m) => m.slice(5)) ?? [];
      build.onResolve(
        {
          filter: new RegExp(`^(${expected.join("|")})$`),
          namespace: "file",
        },
        ({ path }) => ({ path: `node:${path}`, external: true }),
      );
    },
  };
}
