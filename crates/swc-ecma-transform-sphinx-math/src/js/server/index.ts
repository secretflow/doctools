import katex from "katex";

export function renderMath({ tex, inline }: { tex: string; inline?: boolean }) {
  return katex.renderToString(tex, {
    throwOnError: false,
    displayMode: !inline,
  });
}
