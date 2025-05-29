import * as fs from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";

import * as esbuild from "esbuild-wasm";
import esbuildWasm from "esbuild-wasm/esbuild.wasm";

await esbuild.initialize({
  wasmModule: await WebAssembly.compile(esbuildWasm),
  worker: false,
});

export { esbuild, resolver };

function resolver(): esbuild.Plugin {
  return {
    name: "deno-resolver",
    setup: (build) => {
      build.onResolve(
        {
          namespace: "",
          filter: /.*/,
        },
        ({ path, resolveDir }) => {
          const base = new URL(resolveDir, "file:");
          if (!base.pathname.endsWith("/")) {
            base.pathname += "/";
          }
          let resolved: URL | undefined;
          try {
            resolved = new URL(path, base);
          } catch {
            //
          }
          if (resolved?.protocol !== "file:") {
            resolved = new URL(import.meta.resolve(path));
          }
          if (resolved?.protocol === "file:") {
            return {
              path: resolved.pathname,
              namespace: "file",
            };
          } else {
            return {
              path: pathToFileURL(path).pathname,
              namespace: "file",
            };
          }
        },
      );
      build.onLoad(
        {
          namespace: "file",
          filter: /.*/,
        },
        async ({ path }) => {
          path = fileURLToPath(new URL(path, "file:"));
          const contents = await fs.readFile(path);
          return { contents, loader: "default" };
        },
      );
    },
  };
}
