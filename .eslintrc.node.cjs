module.exports = {
  root: true,
  env: { node: true },
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:promise/recommended',
    'prettier',
  ],
  plugins: ['import'],
  rules: {
    'no-console': 'off',
    'import/order': require('./.eslintrc.cjs').rules['import/order'],
  },
};
