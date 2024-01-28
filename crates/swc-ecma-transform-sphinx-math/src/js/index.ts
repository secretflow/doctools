import katex from "katex";

export function render({ code, inline }: { code: string; inline?: boolean }) {
  return katex.renderToString(code, {
    throwOnError: true,
    displayMode: !inline,
  });
}
