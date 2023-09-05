module.exports = {
  root: true,
  extends: [require.resolve('../../.eslintrc.js')],
  overrides: [
    {
      files: ['src/plugin/**/*'],
      extends: [require.resolve('../../.eslintrc.node.js')],
    },
  ],
};
