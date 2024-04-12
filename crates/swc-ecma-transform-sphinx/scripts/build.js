import { fileURLToPath } from "node:url";

import * as esbuild from "esbuild";
import { denoPlugins } from "jsr:@luca/esbuild-deno-loader@^0.10.3";

const relpath = (path) => fileURLToPath(new URL(path, import.meta.url));

await esbuild.build({
  plugins: [...denoPlugins({ configPath: relpath("../deno.json") })],
  entryPoints: [relpath("../src/js/server/index.ts")],
  outfile: relpath("../dist/server/index.js"),
  format: "esm",
  bundle: true,
  write: true,
  platform: "neutral",
  target: "deno1",
  minify: true,
});

await esbuild.stop();
