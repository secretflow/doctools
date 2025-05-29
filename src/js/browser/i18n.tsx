import { match as matchLocale } from "@formatjs/intl-localematcher";
import type { I18n } from "@lingui/core";
import { setupI18n } from "@lingui/core";
import { I18nProvider } from "@lingui/react";
import { ConfigProvider } from "antd";
import type { Locale } from "antd/es/locale";
import antdEnUs from "antd/locale/en_US";
import antdZhCN from "antd/locale/zh_CN";
import type { PropsWithChildren } from "react";
import { useEffect } from "react";

import { messages as en } from "./locales/en/messages.po";
import { messages as zhCN } from "./locales/zh-CN/messages.po";

const messages = {
  ["en"]: en,
  ["zh-CN"]: zhCN,
};

const antdLocales: Record<string, Locale> = {
  ["en"]: antdEnUs,
  ["zh-CN"]: antdZhCN,
};

export function createI18n(initial: string | undefined) {
  const i18n = setupI18n();
  i18n.load(messages);
  if (initial) {
    i18n.activate(initial, Object.keys(messages));
  }
  return i18n;
}

export type LocaleInfo = {
  accepts: Intl.Locale[];
  translated: Intl.Locale[];
  requested: Intl.Locale | null;
  best: Intl.Locale;
};

export type CurrentLocaleHook = () => LocaleInfo;

export function createI18nProvider({
  i18n,
  useCurrentLocale,
}: {
  i18n: I18n;
  useCurrentLocale: CurrentLocaleHook;
}) {
  return function CurrentI18nProvider({ children }: PropsWithChildren) {
    const { best } = useCurrentLocale();
    const selected = best.toString();
    useEffect(() => {
      i18n.activate(selected, i18n.locales);
    }, [selected]);
    return (
      <I18nProvider i18n={i18n}>
        <ConfigProvider locale={antdLocales[selected]}>{children}</ConfigProvider>
      </I18nProvider>
    );
  };
}

export function requestLocale(requested: string | undefined): LocaleInfo {
  const supported = Object.keys(messages).map((locale) => new Intl.Locale(locale));

  supported.sort(sortBySpecificity);

  const fallback = supported.find((lang) => lang.language === "en")?.toString() ?? "en";

  const accepts = (() => {
    try {
      return globalThis.navigator.languages;
    } catch {
      return [];
    }
  })();

  const preferred = accepts.map((locale) => new Intl.Locale(locale));

  const parsed = (() => {
    if (requested) {
      const normalized = normalizeLocale(requested).split("-");
      for (let i = normalized.length; i > 0; i--) {
        try {
          return new Intl.Locale(normalized.slice(0, i).join("-"));
        } catch {
          continue;
        }
      }
      return null;
    } else {
      return null;
    }
  })();

  const selected = (() => {
    if (parsed) {
      return [parsed.toString()];
    } else if (preferred.length > 0) {
      return preferred.map((locale) => locale.toString());
    } else {
      return [fallback];
    }
  })();

  const best = new Intl.Locale(
    matchLocale(
      selected,
      supported.map((c) => c.toString()),
      fallback,
      { algorithm: "best fit" },
    ),
  );

  return {
    accepts: preferred,
    translated: supported,
    requested: parsed,
    best,
  };
}

function normalizeLocale(locale: string): string {
  return locale.toLowerCase().replaceAll(/[^A-Za-z0-9-]+/gu, "-");
}

function sortBySpecificity(a: Intl.Locale, b: Intl.Locale) {
  let x = 1;
  let y = 1;
  if (a.script) {
    x += 1;
  }
  if (a.region) {
    x += 1;
  }
  if (b.script) {
    y += 1;
  }
  if (b.region) {
    y += 1;
  }
  return y - x;
}

export function printLocaleNameIn(locale: string, toPrint?: string): string {
  let value = toPrint ?? locale;
  // oh well
  switch (value) {
    case "zh-CN":
      value = "zh-Hans";
      break;
    case "zh-HK":
    case "zh-MO":
    case "zh-TW":
      value = "zh-Hant";
      break;
    default:
      break;
  }
  try {
    // may not have been polyfilled
    const names = new Intl.DisplayNames(locale, { type: "language" });
    const name = names.of(value);
    if (!name) {
      throw new Error("Invalid locale");
    } else {
      return name;
    }
  } catch {
    if (locale === "en") {
      return value;
    } else {
      return printLocaleNameIn("en", value);
    }
  }
}

export const BrowserI18nProvider = (() => {
  const locale = requestLocale(undefined);
  const i18n = createI18n(locale.best.toString());
  const useCurrentLocale = () => locale;
  return createI18nProvider({ i18n, useCurrentLocale });
})();

const localeOverrides = new Map<string, ReturnType<typeof createI18nProvider>>();

export function OverrideLocale({
  locale,
  children,
}: PropsWithChildren<{ locale: string }>) {
  if (localeOverrides.has(locale)) {
    const Context = localeOverrides.get(locale)!;
    return <Context>{children}</Context>;
  }
  const core = createI18n(locale);
  function useFixedLocale() {
    const localeObject = new Intl.Locale(locale);
    return {
      best: localeObject,
      requested: localeObject,
      accepts: [localeObject],
      translated: [localeObject],
    };
  }
  const Context = createI18nProvider({
    i18n: core,
    useCurrentLocale: useFixedLocale,
  });
  localeOverrides.set(locale, Context);
  return <Context>{children}</Context>;
}

/**
 * Try (our best) to determine if a string is unlikely to be intended as a language tag.
 * Returns `true` if the string is **NOT** likely to be a language tag.
 *
 * This turns out to be very cursed.
 */
export const isSpuriousLocale = (() => {
  const memo: Map<string, boolean> = new Map();
  return (tag: string): boolean => {
    let isSpurious = memo.get(tag);
    if (isSpurious !== undefined) {
      return isSpurious;
    }
    /**
     * acceptable `tag` for Intl.Locale would be in the IANA Language Subtag Registry.
     *
     * the language subtag (first part) could be from ISO 639-1 (2 letters)
     * or ISO 639-3 (3 letters), among others.
     *
     * there are more than 8000 3-letter tags in the `language-subtag-registry` package.
     * for the sake of the program's sanity, we would like to consider most 3-letter
     * codes to be unsupported; therefore, a tag is considered supported only if
     * all of the following are true:
     *
     * 1. it is well-formed: `new Intl.Locale(tag)` does not throw
     * 2. the locale's `baseName` equals `tag`
     * 3. the browser knows the display name of the language
     *
     * step 2 filters away:
     *
     * - tags that contain additional subtags and/or BCP 47 extensions such as `de-DE-u-co-phonebk`
     * - tags that may be languages in ISO 639-3 that belong to ISO 639-1 macrolanguages, notable examples:
     *   - `src` Logudorese Sardinian, whose `baseName` is `sc` Sardinian
     *   - `cmn` Mandarin Chinese, whose `baseName` is `zh` Chinese
     *
     * step 3 filters away languages that are not commonly seen enough for the browser
     * to contain locale data for it.
     *
     * step 2 and 3 combined leaves the following 3-letter codes intact (on Chrome 136)
     *
     * - `ast` Asturian
     * - `bho` Bhojpuri
     * - `ceb` Cebuano
     * - `chr` Cherokee
     * - `ckb` Central Kurdish
     * - `doi` Dogri
     * - `fil` Filipino
     * - `haw` Hawaiian
     * - `hmn` Hmong
     * - `ilo` Iloko
     * - `kok` Konkani
     * - `kri` Krio
     * - `lus` Mizo
     * - `mai` Maithili
     * - `mni` Manipuri
     * - `nso` Northern Sotho
     * - `yue` Cantonese
     *
     * <https://en.wikipedia.org/wiki/IETF_language_tag>
     *
     * <https://www.npmjs.com/package/language-subtag-registry>
     */
    try {
      const { language, baseName } = new Intl.Locale(tag);
      if (baseName !== tag) {
        isSpurious = true;
      } else {
        const displayName = new Intl.DisplayNames("en", { type: "language" });
        const inEnglish = displayName.of(language);
        isSpurious = language === inEnglish;
      }
    } catch {
      // if it doesn't parse, consider it unlikely to be a language tag
      isSpurious = true;
    }
    memo.set(tag, isSpurious);
    return isSpurious;
  };
})();
