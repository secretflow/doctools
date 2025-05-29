import * as fs from "node:fs/promises";
import { createRequire } from "node:module";
import * as pathlib from "node:path";
import { fileURLToPath, pathToFileURL, URL } from "node:url";

import { fromJs } from "esast-util-from-js";
import type * as esbuild from "esbuild";
import type * as mdast from "mdast";
import type { MdxJsxFlowElement, MdxJsxTextElement } from "mdast-util-mdx";
import type { Transformer } from "unified";
import type * as unist from "unist";
import { convert } from "unist-util-is";
import { SKIP, visit } from "unist-util-visit";

export const ESBUILD_STATIC = "__esbuild_static__";

export function remarkEsbuildStatic(): Transformer {
  return (tree, file) => {
    type ImportSpecifier = {
      path: string;
      variable: string;
    };

    const refs: ImportSpecifier[] = [];

    const base = pathToFileURL(file.path);
    const require = createRequire(base);

    function resolveFile(href: string) {
      const parsed = (() => {
        try {
          const maybeURL = new URL(href, base);
          if (maybeURL.protocol !== "file:") {
            // Not a local asset
            return null;
          } else {
            return maybeURL;
          }
        } catch {
          // Not a valid WHATWG URL
          return null;
        }
      })();
      if (parsed === null) {
        return null;
      }
      return require.resolve(fileURLToPath(parsed));
    }

    function modifyJSXElement(node: MdxJsxFlowElement | MdxJsxTextElement) {
      let attr;

      if (node.name === "a") {
        // process download link
        attr = node.attributes.find((a) => "name" in a && a.name === "href");
      } else {
        attr = node.attributes.find((a) => "name" in a && a.name === "src");
      }

      if (
        // No available attribute
        attr === undefined ||
        // is a spread expression like {...attrs}
        attr.type === "mdxJsxExpressionAttribute" ||
        // is a JSX expression like <MyComponent src={...} />
        // This could be something like <MyComponent src={"./image.png"} />
        // but if it is written like that then the author should know what
        // they are doing.
        typeof attr.value === "object" ||
        attr.value === undefined
      ) {
        return;
      }

      const ref = resolveFile(attr.value);

      if (ref === null) {
        return;
      }

      const variable = `asset${refs.length}`;
      const info = { path: ref, variable };

      refs.push(info);

      const idx = node.attributes.findIndex((a) => a === attr);

      node.attributes[idx] = {
        type: "mdxJsxAttribute",
        name: attr.name,
        value: {
          type: "mdxJsxAttributeValueExpression",
          value: info.variable,
          data: { estree: fromJs(info.variable, { module: true }) },
        },
      };

      // modify the `download` attribute if necessary

      const download = node.attributes.find(
        (a) => a.type === "mdxJsxAttribute" && a.name === "download",
      );
      if (
        download &&
        download.value &&
        typeof download.value !== "string" &&
        download.value.value === JSON.stringify(true)
      ) {
        download.value = pathlib.basename(ref);
      }
    }

    visit(tree, "image", (node: mdast.Image, idx = 0, parent: mdast.Parent) => {
      if (resolveFile(node.url) === null) {
        return;
      }
      const imgElem = {
        type: "mdxJsxTextElement" as const,
        name: "img",
        attributes: [
          {
            type: "mdxJsxAttribute" as const,
            name: "src",
            value: node.url,
          },
        ],
        children: [],
      };
      if (typeof node.alt === "string") {
        imgElem.attributes.push({
          type: "mdxJsxAttribute" as const,
          name: "alt",
          value: node.alt,
        });
      }
      // This is then processed by the next traversal
      parent.children[idx] = imgElem;
      return SKIP;
    });

    visit(
      tree,
      convert((n): n is MdxJsxFlowElement | MdxJsxTextElement => {
        if (n.type !== "mdxJsxFlowElement" && n.type !== "mdxJsxTextElement") {
          return false;
        }
        const node = n as MdxJsxFlowElement | MdxJsxTextElement;
        if (
          node.name === "a" &&
          node.attributes.some((a) => "name" in a && a.name === "download")
        ) {
          return true;
        }
        const idx = node.attributes.findIndex(
          (attr) => "name" in attr && attr.name === "src",
        );
        if (idx === -1) {
          return false;
        }
        return true;
      }),
      modifyJSXElement,
    );

    const root: unist.Parent = { children: [], ...tree };

    file.data["assets"] = refs;

    return {
      ...root,
      children: [
        ...refs.map((ref) => {
          const stmt = `export const ${ref.variable} = ${ESBUILD_STATIC}[
            ${JSON.stringify(ref.variable)}
          ];`;
          return {
            type: "mdxjsEsm",
            value: stmt,
            data: { estree: fromJs(stmt, { module: true }) },
          };
        }),
        ...root.children,
      ],
    };
  };
}

export function esbuildStatic(): esbuild.Plugin {
  return {
    name: "asset-url-module",
    setup(builder) {
      builder.onResolve(
        { filter: /^.*$/, namespace: "file" },
        async ({ path: importPath, resolveDir, kind }) => {
          if (kind !== "import-statement" && kind !== "dynamic-import") {
            return undefined;
          }
          if (/^.*\.(js|cjs|mjs|ts|tsx|mts|cts)$/.test(importPath)) {
            return undefined;
          }
          resolveDir = fileURLToPath(new URL(resolveDir, "file:"));
          try {
            const test = new URL(importPath);
            if (test.protocol === "file:") {
              importPath = fileURLToPath(test);
            }
          } catch {
            //
          }
          let assetPath = pathlib.resolve(resolveDir, importPath);
          assetPath = await fs.realpath(assetPath);
          return {
            namespace: "asset-url-module",
            path: assetPath + ".js",
            watchFiles: [assetPath],
          };
        },
      );

      builder.onLoad(
        { filter: /^.*\.js$/, namespace: "asset-url-module" },
        ({ path: modulePath }) => {
          const assetPath = modulePath.replace(/\.js$/, "");
          const assetFile = "./" + pathlib.basename(assetPath);
          const resolveDir = pathlib.dirname(assetPath);
          const contents = `
          import url from ${JSON.stringify(assetFile)};
          export default new URL(url, import.meta.url).toString();
          `.trim();
          return { contents, resolveDir, loader: "js" };
        },
      );

      builder.onLoad(
        { filter: /^.*$/, namespace: "file" },
        async ({ path: filePath }) => {
          if (/^.*\.(js|cjs|mjs|ts|tsx|mts|cts)$/.test(filePath)) {
            return undefined;
          }
          filePath = fileURLToPath(new URL(filePath, "file:"));
          const contents = await fs.readFile(filePath);
          return { contents, loader: "file" };
        },
      );
    },
  };
}
