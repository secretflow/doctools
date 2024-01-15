import { setupI18n } from "@lingui/core";
import { useCallback, useEffect, useRef, useState } from "react";

import * as enUS from "./locales/en-US";
import * as zhHans from "./locales/zh-Hans";

export type SupportedLocale = "en-US" | "zh-Hans";

export const i18n = setupI18n({
  locale: "en-US",
  locales: ["en-US", "zh-Hans"],
  messages: { "en-US": enUS.messages, "zh-Hans": zhHans.messages },
});

export function useLocale(initial: SupportedLocale = "en-US") {
  const initialLocale = useRef(initial);

  const setLocale = useCallback((locale: SupportedLocale) => {
    i18n.activate(locale);
    setCurrent(locale);
  }, []);

  const [current, setCurrent] = useState(i18n.locale as SupportedLocale);

  useEffect(() => {
    setLocale(initialLocale.current);
  }, [setLocale]);

  return {
    locale: current,
    locales: i18n.locales as SupportedLocale[],
    setLocale,
  };
}

export { Trans } from "@lingui/react";

i18n.activate("en-US");
