import { shouldPolyfill as shouldPolyfillDisplayNames } from "@formatjs/intl-displaynames/should-polyfill";
import { shouldPolyfill as shouldPolyfillGetCanonicalLocales } from "@formatjs/intl-getcanonicallocales/should-polyfill";
import { shouldPolyfill as shouldPolyfillLocale } from "@formatjs/intl-locale/should-polyfill";

export async function intlPolyfill() {
  if (shouldPolyfillGetCanonicalLocales()) {
    await import("@formatjs/intl-getcanonicallocales/polyfill");
  }

  if (shouldPolyfillLocale()) {
    await import("@formatjs/intl-locale/polyfill");
  }

  if (shouldPolyfillDisplayNames("en")) {
    await import("@formatjs/intl-displaynames/polyfill");
    await import("@formatjs/intl-displaynames/locale-data/en");
  }

  if (shouldPolyfillDisplayNames("zh")) {
    await import("@formatjs/intl-displaynames/polyfill");
    await import("@formatjs/intl-displaynames/locale-data/zh");
    await import("@formatjs/intl-displaynames/locale-data/zh-Hans");
    await import("@formatjs/intl-displaynames/locale-data/zh-Hant");
  }
}
