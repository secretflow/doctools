module.exports = {
  extends: [
    "stylelint-config-standard",
    "stylelint-config-styled-components",
    "stylelint-config-recess-order",
  ],
  rules: {
    "custom-property-empty-line-before": null,
  },
  customSyntax: "postcss-styled-syntax",
};
