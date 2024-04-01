import { codeToHtml } from "shiki";

type Options = {
  code: string;
  lang: string | null;
  /** 1-indexed */
  highlightedLines: number[] | null;
};

export async function renderCode({ code, lang, highlightedLines }: Options) {
  const language = lang || "plaintext";
  const highlighted = highlightedLines || [];
  return await codeToHtml(code, {
    lang: language,
    themes: {
      light: "github-light",
      dark: "ayu-dark",
    },
    defaultColor: false,
    decorations: highlighted.map((lineNo) => ({
      start: { line: lineNo - 1, character: 0 },
      end: { line: lineNo, character: 0 },
      properties: { class: "highlighted" },
    })),
  });
}
