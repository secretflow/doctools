import * as esbuild from "esbuild";

await esbuild.build({
  entryPoints: ["src/js/server/index.ts"],
  outfile: "dist/server/index.js",
  format: "esm",
  bundle: true,
  write: true,
  platform: "browser",
  target: "deno1",
  minify: true,
});
