import type { LinguiConfig } from "@lingui/conf";

const config: LinguiConfig = {
  locales: ["en-US", "zh-Hans"],
  sourceLocale: "en-US",
  compileNamespace: "ts",
  catalogs: [
    {
      path: "<rootDir>/src/locales/{locale}",
      include: ["src"],
    },
  ],
};

export default config;
