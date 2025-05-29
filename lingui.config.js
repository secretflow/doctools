/** @type {import('@lingui/conf').LinguiConfig} */
const config = {
  locales: ["en", "zh-CN"],
  sourceLocale: "en",
  catalogs: [
    {
      include: ["<rootDir>/src/js/browser"],
      path: "<rootDir>/src/js/browser/locales/{locale}/messages",
    },
  ],
  format: "po",
  compileNamespace: "ts",
};

export default config;
