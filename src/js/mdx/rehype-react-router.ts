import "mdast-util-to-hast";

import { createRequire } from "node:module";
import * as pathlib from "node:path";
import { fileURLToPath, pathToFileURL, URL } from "node:url";

import type { Element, ElementContent, Parent as HastParent } from "hast";
import type { Link, Parent as MdastParent } from "mdast";
import type { MdxJsxFlowElement, MdxJsxTextElement } from "mdast-util-mdx";
import type { Transformer } from "unified";
import type { Node } from "unist";
import { convert } from "unist-util-is";
import { SKIP, visit } from "unist-util-visit";

type GetLink = (url: URL) => string;

/**
 * Replace all relative links pointing to local Markdown files with react-router
 * <Link />s and all absolute URLs with <a target="_blank" />s.
 *
 * The following cases are supported:
 *
 * - Relative links that correctly resolves to a Markdown file on the filesystem
 *   (if you can click it in VSCode and it opens the correct file then it's supported)
 *
 * The following cases are NOT supported at the moment:
 *
 * - Links with extensions omitted
 * - Links with path variables
 * - Tabbed pages
 * - Implicit `index.md` pages
 *
 * Replacement for https://github.com/umijs/dumi/blob/7e250d8039ad59e4dea969313dfc8f012b2fb0df/src/loaders/markdown/transformer/rehypeLink.ts#L19
 */
export function rehypeReactRouter({
  getLink,
  logger,
}: {
  getLink: GetLink;
  logger?: { warn: (msg: string) => void };
}): Transformer {
  return (tree, file) => {
    const base = pathToFileURL(file.path);
    const require = createRequire(base);

    type AnyLink = Link | Element | MdxJsxFlowElement | MdxJsxTextElement;

    type AnyParent = HastParent | MdastParent;

    visit(
      tree,
      convert([
        "link",
        "mdxJsxTextElement",
        "mdxJsxFlowElement",
        (node: Node) =>
          node.type === "element" && "tagName" in node && node.tagName === "a",
      ]),
      (_node, idx = 0, parent: AnyParent) => {
        const node = _node as AnyLink;

        let href: string | undefined;

        if (node.type === "link") {
          href = node.url;
        } else if (
          node.type === "mdxJsxTextElement" ||
          node.type === "mdxJsxFlowElement"
        ) {
          const attr = node.attributes.find((x) => "name" in x && x.name === "href");
          if (!attr || typeof attr.value !== "string") {
            return;
          }
          href = attr.value;
        } else {
          href = node.properties["href"]?.toString();
        }

        if (!href || href.startsWith("#")) {
          return;
        }

        href = decodeURIComponent(href);

        let parsed: URL;

        try {
          parsed = new URL(href, "file://");
        } catch {
          // malformed or unprocessable URL
          return;
        }

        const props: Record<string, unknown> = (() => {
          if (node.type === "link") {
            const attrs = node.data?.hProperties || {};
            attrs["title"] = node.title;
            return attrs;
          } else if (
            node.type === "mdxJsxTextElement" ||
            node.type === "mdxJsxFlowElement"
          ) {
            return Object.fromEntries(
              node.attributes.flatMap((attr) => {
                if ("name" in attr) {
                  return [[attr.name, attr.value?.toString()]];
                } else {
                  return [];
                }
              }),
            );
          } else {
            return node.properties;
          }
        })();

        if (parsed.protocol !== "file:") {
          // external link
          if (
            node.type === "link" ||
            (node.type === "mdxJsxTextElement" && node.name === "a") ||
            (node.type === "element" && node.tagName === "a")
          ) {
            if (parsed.hostname.endsWith("www.secretflow.org.cn")) {
              // internal link written as external link
              let linkTo = parsed.pathname;
              if (parsed.searchParams.size > 0) {
                linkTo += parsed.search;
              }
              if (parsed.hash) {
                linkTo += parsed.hash;
              }
              props["to"] = linkTo;
              delete props["href"];
              parent.children[idx] = {
                type: "mdxJsxTextElement",
                name: "Link",
                attributes: Object.entries(props).map(([name, value]) => ({
                  type: "mdxJsxAttribute",
                  name,
                  value: value?.toString(),
                })),
                children: node.children as ElementContent[],
              };
            } else {
              // actual external link
              // replace with <a target="_blank" />
              props["href"] = href.toString();
              props["target"] = "_blank";
              props["rel"] = "noopener";
              parent.children[idx] = {
                type: "mdxJsxTextElement",
                name: "a",
                attributes: Object.entries(props).map(([name, value]) => ({
                  type: "mdxJsxAttribute",
                  name,
                  value: value?.toString(),
                })),
                children: node.children as ElementContent[],
              };
            }
            return SKIP;
          } else {
            // some other elements, do nothing
            return;
          }
        } else {
          // internal link, absolute or relative

          // make absolute
          const absolute = new URL(href, base);

          const resolved = (() => {
            for (const href of [
              absolute.href,
              absolute.href + ".mdx",
              absolute.href + "/index.mdx",
            ]) {
              try {
                return require.resolve(fileURLToPath(href));
              } catch {
                continue;
              }
            }
            return null;
          })();

          if (resolved) {
            absolute.pathname = pathToFileURL(resolved).pathname;
          } else {
            let rel: string;
            try {
              rel = pathlib.relative(file.path, fileURLToPath(absolute));
            } catch {
              rel = absolute.href;
            }
            const msg = `[react-router] cannot resolve ${rel} from ${file.path}`;
            logger?.warn(msg);
            parent.children[idx] = {
              type: "mdxJsxTextElement",
              name: "span",
              attributes: [],
              children: node.children as ElementContent[],
            };
            return SKIP;
          }

          const linkTo = getLink(absolute);

          if (
            node.type === "link" ||
            (node.type === "mdxJsxTextElement" && node.name === "a") ||
            (node.type === "element" && node.tagName === "a")
          ) {
            // replace with <Link />
            props["to"] = linkTo;
            delete props["href"];
            parent.children[idx] = {
              type: "mdxJsxTextElement",
              name: "Link",
              attributes: Object.entries(props).map(([name, value]) => ({
                type: "mdxJsxAttribute",
                name,
                value: value?.toString(),
              })),
              children: node.children as ElementContent[],
            };
            return SKIP;
          } else {
            // some other elements, replace the href attribute
            if (node.type === "element") {
              node.properties["href"] = linkTo;
            } else {
              const attr = node.attributes.find(
                (x) => "name" in x && x.name === "href",
              );
              if (!attr || typeof attr.value !== "string") {
                return;
              }
              attr.value = linkTo;
            }
            return;
          }
        }
      },
    );
  };
}
