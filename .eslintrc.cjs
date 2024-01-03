module.exports = {
  root: true,
  env: { browser: true, es2022: true },
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:react/recommended',
    'plugin:react/jsx-runtime',
    'plugin:react-hooks/recommended',
    'plugin:promise/recommended',
    'prettier',
  ],
  parser: '@typescript-eslint/parser',
  plugins: ['@typescript-eslint', 'import', 'react-refresh'],
  ignorePatterns: ['dist', '.eslintrc.*'],
  rules: {
    eqeqeq: 'error',
    curly: 'error',
    '@typescript-eslint/no-explicit-any': ['warn', { ignoreRestArgs: true }],
    '@typescript-eslint/no-shadow': ['warn', { ignoreTypeValueShadow: true }],
    '@typescript-eslint/consistent-type-imports': [
      'warn',
      { disallowTypeAnnotations: false },
    ],
    'react-hooks/exhaustive-deps': 'error',
    'react-hooks/rules-of-hooks': 'error',
    'react/no-unknown-property': [
      'error',
      {
        ignore: [
          'jsx', // styled-jsx
        ],
      },
    ],
    'no-console': ['error', { allow: ['error', 'warn'] }],
    'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
    'import/newline-after-import': 'warn',
    'import/order': [
      'warn',
      {
        pathGroups: [
          {
            pattern: '@/**',
            group: 'internal',
            position: 'before',
          },
        ],
        distinctGroup: false,
        groups: [
          'builtin',
          'external',
          'internal',
          'parent',
          'sibling',
          'index',
          'object',
        ],
        'newlines-between': 'always',
        alphabetize: {
          order: 'asc',
          caseInsensitive: true,
        },
      },
    ],
  },
  overrides: [
    {
      files: ['*.mdx'],
      extends: ['plugin:mdx/recommended'],
      parserOptions: {
        extensions: ['.mdx'],
      },
      rules: {
        'react/jsx-no-undef': 'off',
        'react/no-unescaped-entities': 'off',
        'react/prop-types': 'off',
      },
    },
  ],
};
