import type { BundledLanguage } from "shiki";
import { bundledLanguages, createHighlighter } from "shiki";

import type { FromWorker, IntoWorker } from "../types";
import { colorReplacements } from "../types";

const init = createHighlighter({
  langs: [],
  themes: ["github-light"],
});

async function language(unchecked: string) {
  const lang = checked(unchecked);
  if (lang === undefined) {
    return "plaintext";
  } else {
    const loader = (loaders[lang] ??= init //
      .then((highlighter) => highlighter.loadLanguage(lang)));
    await loader;
    return lang;
  }
}

const checked = (lang: string): BundledLanguage | undefined =>
  lang in bundledLanguages ? (lang as BundledLanguage) : undefined;

const loaders: Partial<Record<BundledLanguage, Promise<void>>> = {};

export async function highlight({ code, lang }: IntoWorker) {
  const highlighter = await init;
  lang = await language(lang);
  return {
    code,
    lang,
    root: highlighter.codeToHast(code, {
      lang,
      theme: "github-light",
      colorReplacements,
    }),
  } satisfies FromWorker;
}
