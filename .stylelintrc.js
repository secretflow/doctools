/** @type {import("stylelint").Config} */
const config = {
  root: true,
  ignoreFiles: ["src/js/browser/public/**/*"],
  extends: ["stylelint-config-standard", "stylelint-config-recess-order"],
  customSyntax: "postcss-styled-syntax",
  rules: {
    "property-no-vendor-prefix": [true, { ignoreProperties: "text-size-adjust" }],
  },
};

export default config;
