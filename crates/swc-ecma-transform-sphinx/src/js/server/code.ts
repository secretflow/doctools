import { codeToHtml } from "shiki";

type Options = {
  code: string;
  lang: string | null;
  /** 1-indexed */
  lineHighlight: number[] | null;
};

export async function renderCode({ code, lang, lineHighlight }: Options) {
  const language = lang || "text";
  const highlighted = lineHighlight || [];
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
