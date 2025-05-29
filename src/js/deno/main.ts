import "./globals.ts";

import * as pathlib from "node:path";
import * as process from "node:process";

import { Command } from "commander";
import { globby } from "globby";

import type { PathMap, TargetProject } from "./bundle.ts";
import { bundle } from "./bundle.ts";
import { ffi } from "./ffi.ts";

const program = new Command();

program.name("secretflow-doctools");

program
  .command("ffi")
  .option("-p, --port <port>", "listen on this port")
  .action(({ port = 3000 }) => ffi(Number(port)));

program
  .command("bundle")
  .requiredOption("-i, --srcdir <path>", "sphinx output directory")
  .requiredOption("-o, --outdir <path>", "module output directory")
  .option("--redirect [aliases...]", "path mappings")
  .action(async function ({ srcdir, outdir, redirect = [] }: Options) {
    const sourceDir = pathlib.resolve(process.cwd(), srcdir);
    const outputDir = pathlib.resolve(process.cwd(), outdir);

    const targets = await globby("*/*/*/manifest.yml", {
      cwd: sourceDir,
      onlyFiles: true,
      absolute: false,
    }).then((triples) =>
      triples.map((triple): TargetProject => {
        const [repo, ref, lang] = triple.split("/");
        return { kind: "github" as const, repo, ref, lang };
      }),
    );

    const mapping = parsePathMap(redirect);

    await bundle({ sourceDir, outputDir, targets, mapping });
  });

program.parse();

type Options = {
  srcdir: string;
  outdir: string;
  redirect?: string[];
};

function parsePathMap(mapping: string[]): PathMap {
  const map: PathMap = {};
  for (const directive of mapping) {
    const [repo, prefix, rename, redirect, ..._] = directive.split(":");
    if (
      _.length ||
      repo === undefined ||
      prefix === undefined ||
      rename === undefined ||
      redirect === undefined
    ) {
      throw new Error(`invalid mapping: ${directive}`);
    }
    (map[repo] ??= {})[prefix] = { name: rename, path: redirect };
  }
  return map;
}
