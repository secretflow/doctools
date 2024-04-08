import { fileURLToPath } from "node:url";

import * as esbuild from "esbuild";

const relpath = (path) => fileURLToPath(new URL(path, import.meta.url));

await esbuild.build({
  entryPoints: [relpath("../src/js/server/index.ts")],
  outfile: relpath("../dist/server/index.js"),
  format: "esm",
  bundle: true,
  write: true,
  platform: "browser",
  target: "deno1",
  minify: true,
});

await esbuild.stop();
