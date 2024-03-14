import { codeToHtml } from "shiki";

export async function renderCode({ code, lang }: { code: string; lang: string }) {
  return await codeToHtml(code, {
    lang,
    theme: "ayu-dark",
  });
}
