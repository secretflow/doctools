import type { IApi as DumiAPI } from 'dumi';

export const THEME_KEY = 'sphinx-theme';

export default async function preset(api: DumiAPI) {
  const { manifestPlugin } = await import('./manifest/index.mjs');

  api.registerPlugins([
    require.resolve('@secretflow/dumi-plugin-mdx'),
    require.resolve('@secretflow/dumi-plugin-search/plugin'),
  ]);

  api.describe({ key: THEME_KEY });

  manifestPlugin(api);
}
