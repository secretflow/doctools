module.exports = {
  printWidth: 88,
  singleQuote: false,
  trailingComma: "all",
  proseWrap: "always",
  overrides: [
    {
      files: ["**/*.md", "**/*.mdx", ".github/**/*.yml"],
      options: {
        proseWrap: "preserve",
      },
    },
  ],
};
