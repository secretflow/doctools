import * as fs from "node:fs/promises";
import * as pathlib from "node:path";
import * as process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";

import { compile } from "@mdx-js/mdx";
import { insertMultiple, save } from "@orama/orama";
import { globby } from "globby";
import Handlebars from "handlebars";
import ora from "ora";
import rehypeKatex from "rehype-katex";
import remarkDirective from "remark-directive";
import remarkExtractFrontmatter from "remark-extract-frontmatter";
import remarkFrontmatter from "remark-frontmatter";
import remarkGFM from "remark-gfm";
import remarkMath from "remark-math";
import remarkMDXFrontmatter from "remark-mdx-frontmatter";
import tempDir from "temp-dir";
import { VFile } from "vfile";
import YAML from "yaml";

import type { Project, Sidebar, SidebarItem } from "../docs/types.ts";
import { ManifestV2 } from "../docs/types.ts";
import { rehypeArticleOutline } from "../mdx/rehype-article-outline.ts";
import { rehypeReactRouter } from "../mdx/rehype-react-router.ts";
import { rehypeRemoveEmptyElements } from "../mdx/rehype-remove-empty-elements.ts";
import { rehypeSourceCode } from "../mdx/rehype-source-code.ts";
import { rehypeTable } from "../mdx/rehype-table.ts";
import { remarkAdmonitions } from "../mdx/remark-admonitions.ts";
import {
  ESBUILD_STATIC,
  esbuildStatic,
  remarkEsbuildStatic,
} from "../mdx/remark-esbuild-static.ts";
import { remarkEscapeMath } from "../mdx/remark-escape-math.ts";
import { remarkSwagger } from "../mdx/remark-swagger.ts";
import { remarkTarget } from "../mdx/remark-target.ts";
import { remarkValidateDOMNesting } from "../mdx/remark-validate-dom-nesting.ts";
import { createDatabase } from "../search/index.ts";

import { esbuild, resolver } from "./esbuild.ts";
import PAGE_MODULE from "./templates/[instance]/render.js.hbs";
import PACKAGE_ENTRYPOINT from "./templates/index.js.hbs";

export const bundle = (options: Options) => new Bundler(options).build();

export type Options = {
  sourceDir: string;
  outputDir: string;
  targets: TargetProject[];
  mapping?: PathMap;
  options?: Pick<esbuild.BuildOptions, "minify">;
};

export type TargetProject = {
  kind: "github";
  repo: string;
  ref: string;
  lang: string;
};

export type PathMap = {
  [repo: string]: {
    [prefix: string]: {
      name: string;
      path: string;
    };
  };
};

type PageImport = {
  name: string;
  path: URL;
};

type SearchTarget = {
  url: string;
  title: string;
  content: string;
  type: string;
};

type SearchIndex = Record<string, SearchTarget[]>;

type Metadata = Required<Pick<Project, "kind" | "repo" | "ref" | "lang" | "triple">>;

type Entrypoint = {
  key: string;
  metadata: Metadata;
  manifest: ManifestV2;
  routes: { name: string; path: URL }[];
  search: string;
};

class Bundler {
  renderEntrypoint = Handlebars.compile<{ imports: Entrypoint[] }>(PACKAGE_ENTRYPOINT);
  renderPageModule = Handlebars.compile(PAGE_MODULE);

  spinner = ora({
    prefixText: "compiling to JavaScript",
    discardStdin: false,
  }).start();

  options: Options;
  workDir: string;
  distDir: string;

  constructor(options: Options) {
    this.options = options;
    this.distDir = pathlib.resolve(process.cwd(), this.options.outputDir);
    // this.workDir = this.distDir;
    this.workDir = tempDir;
    this.workDir = pathlib.join(this.workDir, `.tmp-secretflow-${Date.now()}`);
  }

  async build() {
    try {
      await fs.rm(this.workDir, { recursive: true, force: true });
    } catch {
      // ignore
    }

    const imports: Awaited<ReturnType<typeof this.buildOne>> = [];

    for (const target of this.options.targets) {
      imports.push(...(await this.buildOne(target)));
    }

    this.spinner.start();
    this.spinner.info("running esbuild");

    const entrypointPath = pathlib.join(this.workDir, "index.js");
    const entrypoint = this.renderEntrypoint({ imports });
    await fs.mkdir(pathlib.dirname(entrypointPath), { recursive: true });
    await fs.writeFile(entrypointPath, entrypoint, "utf-8");

    {
      await fs.mkdir(this.distDir, { recursive: true });
      await esbuild
        .build({
          entryPoints: [pathToFileURL(entrypointPath).href],
          assetNames: "assets/[hash]",
          chunkNames: "chunks/[hash]",
          format: "esm",
          plugins: [esbuildStatic(), resolver()],
          outdir: this.distDir,
          platform: "browser",
          target: "es2020",
          splitting: true,
          bundle: true,
          treeShaking: true,
          ...this.options.options,
        })
        .then(({ outputFiles = [] }) =>
          Promise.all(
            outputFiles.map(async ({ path, contents }) => {
              path = fileURLToPath(new URL(path, "file:"));
              await fs.mkdir(pathlib.dirname(path), { recursive: true });
              await fs.writeFile(path, contents);
            }),
          ),
        );
    }

    this.spinner.succeed("done").stop();
  }

  async buildOne(project: TargetProject) {
    const sourceRoot = this.sourceDirFor(project);

    const matched = await globby(["**/*.mdx"], { cwd: sourceRoot });
    const imports: PageImport[] = [];
    const search: SearchIndex = {};

    const resolveDocName = this.srcPathToDocName(project);
    const resolveURLPath = this.srcPathToURLPath(project);

    for (const fileName of matched) {
      this.spinner.text = `${projectName(project)} ${fileName}`;

      const filePath = pathlib.resolve(pathlib.join(sourceRoot, fileName));
      const fileURL = new URL(pathToFileURL(filePath));

      const outPath = this.outputPathFor(project, filePath);
      const urlPath = resolveURLPath(fileURL);
      const docName = resolveDocName(fileURL);

      const file = new VFile({
        path: filePath,
        value: await fs.readFile(filePath, "utf-8"),
      });

      const result = await (() => {
        switch (pathlib.extname(fileName)) {
          case ".mdx":
            return this.compileMdx(project, file);
          default:
            throw new Error("unreachable");
        }
      })();

      result.value = this.renderPageModule({
        ESBUILD_STATIC,
        body: new Handlebars.SafeString(result.toString()),
        assets: result.data["assets"],
      });

      await fs.mkdir(pathlib.dirname(outPath), { recursive: true });
      await fs.writeFile(outPath, result.value, "utf-8");

      search[docName] =
        result.data.outline?.map(({ id, longTitle: title, content }) => ({
          url: id ? `${urlPath}#${id}` : urlPath,
          type: id ? "fragment" : "page",
          title,
          content,
        })) ?? [];

      imports.push({ name: docName, path: pathToFileURL(outPath) });
    }

    const manifest = await fs
      .readFile(pathlib.join(sourceRoot, "manifest.yml"), "utf-8")
      .then(YAML.parse);

    const instance = { imports, manifest, search };

    const { options } = this;

    this.spinner.text = `${projectName(project)} building search index`;

    function* broadcast(): Generator<{
      repo: string;
      mapped: [string, string] | undefined;
      mapper: (path: string) => string | undefined;
    }> {
      const mapped = options.mapping?.[project.repo];
      if (!mapped) {
        yield {
          repo: project.repo,
          mapper: (path) => path,
          mapped: undefined,
        };
      } else {
        for (const [prefix, { name, path }] of Object.entries(mapped)) {
          yield {
            repo: name,
            mapper: (path) => prefixedBy({ path, prefix }),
            mapped: [prefix, path],
          };
        }
      }
    }

    const results: {
      key: string;
      routes: PageImport[];
      metadata: Metadata;
      manifest: ManifestV2;
      search: string;
    }[] = [];

    for (const { repo, mapped, mapper } of broadcast()) {
      const triple: [string, string, string] = [repo, project.ref, project.lang];

      const key = triple.join("/");

      const routes = instance.imports.flatMap(({ name, ...item }) => {
        const suffix = mapper(name);
        if (suffix === undefined) {
          return [];
        } else {
          return [{ ...item, name: suffix }];
        }
      });

      const manifest: ManifestV2 = {
        ...ManifestV2.parse(instance.manifest),
        sidebar: runtimeSidebar(instance.manifest, mapped),
      };

      const database = await createDatabase();
      await insertMultiple(
        database,
        Object.entries(instance.search)
          .filter(([name]) => mapper(name) !== undefined)
          .flatMap(([, items]) => items),
      );

      const metadata: Metadata = { ...project, repo, triple };

      const search = await (async () => {
        const serialized = await save(database);
        const data = JSON.stringify(serialized);
        const out = pathlib.join(this.workDir, ...triple, "search.json");
        await fs.mkdir(pathlib.dirname(out), { recursive: true });
        await fs.writeFile(out, data, "utf-8");
        return out;
      })();

      results.push({ key, routes, metadata, manifest, search });
    }

    this.spinner.info(`${projectName(project)} done`).start();

    return results;
  }

  async compileMdx(project: TargetProject, file: VFile): Promise<VFile> {
    return await compile(file, {
      outputFormat: "function-body",
      format: "mdx",
      jsxImportSource: "react",
      jsxRuntime: "automatic",
      providerImportSource: "@mdx-js/react",
      remarkPlugins: [
        [remarkFrontmatter],
        [remarkExtractFrontmatter, { yaml: YAML.parse, name: "frontmatter" }],
        [remarkMDXFrontmatter],
        [remarkGFM],
        [remarkDirective],
        [remarkMath],
        [remarkEscapeMath],
        [remarkAdmonitions],
        [remarkTarget],
        [remarkValidateDOMNesting],
        [remarkSwagger],
        [remarkEsbuildStatic],
      ],
      rehypePlugins: [
        [rehypeKatex],
        [rehypeTable],
        [rehypeSourceCode],
        [
          rehypeReactRouter,
          {
            getLink: this.srcPathToURLPath(project),
            logger: { warn: (msg) => this.spinner.warn(msg) },
          } satisfies Parameters<typeof rehypeReactRouter>[0],
        ],
        [rehypeRemoveEmptyElements],
        [rehypeArticleOutline],
      ],
    });
  }

  sourceDirFor({ repo, ref, lang }: TargetProject) {
    return pathlib.join(this.options.sourceDir, repo, ref, lang);
  }

  outputDirFor({ repo, ref, lang }: TargetProject) {
    return pathlib.join(this.workDir, repo, ref, lang);
  }

  srcPathToDocName(project: TargetProject) {
    const root = this.sourceDirFor(project);
    return (url: URL) => {
      const relPath = pathlib.relative(root, fileURLToPath(url));
      const fileName = pathlib.basename(relPath, pathlib.extname(relPath));
      if (fileName === "index") {
        return pathlib.join(pathlib.dirname(relPath));
      } else {
        return pathlib.join(pathlib.dirname(relPath), fileName);
      }
    };
  }

  srcPathToURLPath(project: TargetProject, { repo, ref, lang } = project) {
    const language = normalizeLang(lang);
    return (url: URL) => {
      const href = new URL(url);
      let into = repo;
      let path = this.srcPathToDocName(project)(href);
      const mapping = this.options.mapping?.[repo];
      if (mapping) {
        for (const [prefix, { name, path: redirect }] of Object.entries(mapping)) {
          const suffix = prefixedBy({ path, prefix });
          if (suffix !== undefined) {
            into = name;
            path = withPrefix({ prefix: redirect, suffix });
          }
        }
      }
      href.pathname = ["", into, ref, language, path].join("/");
      return href.pathname + href.search + href.hash;
    };
  }

  outputPathFor(project: TargetProject, srcPath: string) {
    const srcRoot = this.sourceDirFor(project);
    const outRoot = this.outputDirFor(project);
    const relPath = pathlib.relative(srcRoot, srcPath);
    const filename = pathlib.basename(relPath, pathlib.extname(relPath)) + ".mjs";
    return pathlib.join(outRoot, pathlib.dirname(relPath), filename);
  }
}

function runtimeSidebar(raw: unknown, mapping: [string, string] | undefined): Sidebar {
  const manifest = ManifestV2.parse(raw);

  const fixPath = (path: string) => {
    const basename = pathlib.basename(path, pathlib.extname(path));
    return basename === "index"
      ? pathlib.dirname(path)
      : pathlib.join(pathlib.dirname(path), basename);
  };

  const mapPath = (path: string) => {
    if (!mapping) {
      return path;
    }
    const [prefix, redirect] = mapping;
    const suffix = prefixedBy({ path, prefix });
    if (!suffix) {
      return undefined;
    } else {
      return withPrefix({ prefix: redirect, suffix });
    }
  };

  const generateSidebar = (sidebar: Sidebar): Sidebar => {
    return sidebar.flatMap((item: SidebarItem): SidebarItem[] => {
      switch (item.kind) {
        case "link":
          // external link
          return [item];
        case "doc":
        case "category": {
          const children = generateSidebar(item.children ?? []);
          if (item.kind === "category" && !children.length) {
            return [];
          }
          const key = mapPath(fixPath(item.key));
          if (key === undefined) {
            return [];
          } else {
            return [{ ...item, key, children }];
          }
        }
      }
    });
  };

  return generateSidebar(manifest.sidebar);
}

const prefixedBy = ({
  path,
  prefix,
}: {
  path: string;
  prefix: string;
}): string | undefined => {
  if (prefix === "") {
    return undefined;
  }
  const actual = path.split("/");
  for (const expect of prefix.split("/")) {
    if (actual[0] === expect) {
      actual.shift();
    } else {
      return undefined;
    }
  }
  const result = actual.join("/") || ".";
  return result;
};

const withPrefix = ({ prefix, suffix }: { prefix: string; suffix: string }) => {
  if (prefix.endsWith("/")) {
    prefix = prefix.slice(0, -1);
  }
  if (!prefix) {
    return suffix;
  } else if (suffix === ".") {
    return prefix;
  } else {
    return `${prefix}/${suffix}`;
  }
};

/**
 * @param {string} lang
 * @returns {string}
 */
const normalizeLang = (lang: string): string => {
  const normalized = lang
    .replace(/^\/|\/$/g, "")
    .split("/")
    .shift()
    ?.replaceAll(/[^A-Za-z0-9-]+/gu, "-");
  if (!normalized) {
    throw new Error("invalid lang");
  }
  return new Intl.Locale(normalized).toString();
};

const projectName = (project: TargetProject) =>
  `${project.repo}/${project.ref}/${project.lang}`;

Handlebars.registerHelper(
  "json",
  (value) => new Handlebars.SafeString(JSON.stringify(value)),
);
