import { compileSync, runSync } from "@mdx-js/mdx";
import type { RunOptions } from "@mdx-js/mdx";
import { useMDXComponents } from "@mdx-js/react";
import { memo } from "react";
import * as jsxRuntime from "react/jsx-runtime";
import remarkGfm from "remark-gfm";

export const MarkdownEval = memo(function MarkdownEval({
  content,
}: {
  content: string;
}) {
  const code = compileSync(content, {
    format: "md",
    outputFormat: "function-body",
    remarkPlugins: [remarkGfm],
  });
  const { default: Content } = runSync(code, {
    ...(jsxRuntime as RunOptions),
    baseUrl: import.meta.url,
  });
  return <Content components={useMDXComponents()} />;
});
