// @ts-check

/** @type {import('@docusaurus/types').PluginModule} */
const plugin = async () => ({
  name: 'linaria',
  configureWebpack() {
    return {
      mergeStrategy: {
        'module.rules.test': 'match',
        'module.rules.use': 'append',
      },
      module: {
        rules: [
          {
            test: /\.(js|ts)x?$/,
            use: [
              {
                loader: '@linaria/webpack-loader',
                options: {
                  sourceMap: process.env.NODE_ENV === 'development',
                },
              },
            ],
          },
        ],
      },
    };
  },
});

module.exports = plugin;
