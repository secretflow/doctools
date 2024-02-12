import * as esbuild from "esbuild";

await esbuild.build({
  entryPoints: ["src/js/server/index.ts"],
  outfile: "dist/server/index.js",
  format: "esm",
  bundle: true,
  write: true,
  platform: "browser",
  target: "deno1",
  define: {
    "process.env.NODE_ENV": JSON.stringify("production"),
  },
  jsx: "automatic",
  treeShaking: true,
  minify: true,
});