import process from "node:process";

import { serve } from "@hono/node-server";
import { Hono } from "hono";
import type * as mdast from "mdast";
import rehypeParse from "rehype-parse";
import remarkDirective from "remark-directive";
import remarkFrontmatter from "remark-frontmatter";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import remarkMdx from "remark-mdx";
import remarkParse from "remark-parse";
import type { Options } from "remark-stringify";
import remarkStringify from "remark-stringify";
import { unified } from "unified";
import type * as unist from "unist";

import { toJSX } from "../mdx/hast-util-to-jsx.ts";
import { remarkEscapeMdx } from "../mdx/remark-escape-mdx.ts";
import { remarkFixBlockNodes } from "../mdx/remark-fix-block-nodes.ts";
import { remarkFixPhrasingNodes } from "../mdx/remark-fix-phrasing-nodes.ts";
import { remarkMergePhrasingNodes } from "../mdx/remark-merge-phrasing-nodes.ts";

export function ffi(port = 3000) {
  process.on("SIGINT", () => process.exit(0));
  process.on("SIGTERM", () => process.exit(0));

  const STRINGIFY_OPTIONS: Options = {
    emphasis: "_",
    resourceLink: true,
    rule: "-",
    bullet: "-",
    bulletOther: "+",
    bulletOrdered: ".",
    fences: true,
    listItemIndent: "one",
    join: [
      // remove extraneous new lines between JSX block elements
      (
        left: unist.Node,
        right: unist.Node,
        parent: unist.Parent,
      ): number | undefined => {
        if (parent.type === "root") {
          return 1;
        }
        if (
          left.type === "list" &&
          right.type === "list" &&
          !(left as mdast.List).spread &&
          !(right as mdast.List).spread
        ) {
          return 0;
        }
        const types = ["mdxJsxFlowElement", "mdxJsxTextElement"];
        if (types.includes(left.type) && types.includes(right.type)) {
          return 0;
        }
        return undefined;
      },
    ],
  };

  const mdProcessor = unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkFrontmatter)
    .use(remarkDirective)
    .use(remarkMath)
    .use(remarkMdx)
    .use(remarkFixBlockNodes)
    .use(remarkFixPhrasingNodes)
    .use(remarkMergePhrasingNodes)
    .use(remarkEscapeMdx)
    .use(remarkStringify, STRINGIFY_OPTIONS)
    .freeze();

  const htmlProcessor = unified()
    .use(rehypeParse, { fragment: true })
    .use(() => toJSX)
    .use(remarkStringify, STRINGIFY_OPTIONS)
    .freeze();

  const app = new Hono();

  app.post("/markdown/stringify", async (ctx) => {
    try {
      const body = await ctx.req.json();
      const transformed = await mdProcessor.run(body);
      const text = mdProcessor.stringify(transformed as mdast.Root);
      return ctx.text(text);
    } catch (e) {
      ctx.status(400);
      return ctx.text(String(e));
    }
  });

  app.post("/markdown/parse", async (ctx) => {
    try {
      const body = await ctx.req.text();
      const tree = mdProcessor.parse(body);
      const transformed = await mdProcessor.run(tree);
      return ctx.json(transformed);
    } catch (e) {
      ctx.status(400);
      return ctx.text(String(e));
    }
  });

  app.post("/html/parse", async (ctx) => {
    try {
      const body = await ctx.req.text();
      const tree = htmlProcessor.parse(body);
      const transformed = await htmlProcessor.run(tree);
      return ctx.json(transformed);
    } catch (e) {
      ctx.status(400);
      return ctx.text(String(e));
    }
  });

  if (typeof Deno === "object") {
    Deno.serve(
      {
        hostname: "127.0.0.1",
        port,
        onListen: ({ hostname, port }) =>
          process.stderr.write(`Listening on http://${hostname}:${port}\n`),
      },
      app.fetch,
    );
  } else {
    serve(
      {
        fetch: app.fetch,
        hostname: "127.0.0.1",
        port,
      },
      ({ address, port }) =>
        process.stderr.write(`Listening on http://${address}:${port}\n`),
    );
  }
}
