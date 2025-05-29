import js from "@eslint/js";
import { defineConfig } from "eslint/config";
import importPlugin from "eslint-plugin-import";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import globals from "globals";
import tseslint from "typescript-eslint";

export default defineConfig([
  {
    ignores: [
      "src/py",
      "src/js/browser/locales",
      "src/js/browser/public",
      "**/build",
      "**/dist",
    ],
  },
  { files: ["**/*.{js,mjs,cjs,ts,jsx,tsx}"] },
  {
    files: ["**/*.{js,mjs,cjs,ts,jsx,tsx}"],
    languageOptions: { globals: { ...globals.browser, ...globals.node } },
  },
  {
    files: ["**/*.{js,mjs,cjs,ts,jsx,tsx}"],
    plugins: { js },
    extends: ["js/recommended"],
  },
  // eslint-disable-next-line import/no-named-as-default-member
  tseslint.configs.recommended,
  importPlugin.flatConfigs.recommended,
  importPlugin.flatConfigs.typescript,
  {
    settings: {
      "import/resolver": {
        typescript: {
          alwaysTryTypes: true,
        },
      },
    },
    rules: {
      eqeqeq: "error",
      curly: "error",
      "no-unused-vars": "off",
      "@typescript-eslint/consistent-type-imports": ["error"],
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          args: "after-used",
          argsIgnorePattern: "^_",
          ignoreRestSiblings: true,
        },
      ],
      "prefer-const": ["error", { destructuring: "all" }],
      "import/order": [
        "warn",
        {
          "newlines-between": "always",
          alphabetize: {
            order: "asc",
            caseInsensitive: false,
          },
          groups: ["builtin", "external", "parent", "sibling", "index"],
        },
      ],
      "import/consistent-type-specifier-style": ["error", "prefer-top-level"],
    },
  },
  react.configs.flat["recommended"],
  react.configs.flat["jsx-runtime"],
  {
    settings: { react: { version: "detect" } },
  },
  reactHooks.configs["recommended-latest"],
]);
