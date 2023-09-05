import { createRequire } from 'node:module';

import type { IApi as DumiAPI } from 'dumi';

import { manifestPlugin } from './manifest/index.mjs';

const require = createRequire(import.meta.url);

export const THEME_KEY = 'sphinx-theme';

export async function plugin(api: DumiAPI) {
  api.describe({ key: THEME_KEY });
  api.registerPlugins([
    require.resolve('@secretflow/dumi-plugin-mdx'),
    require.resolve('@secretflow/dumi-plugin-search/plugin'),
  ]);
  manifestPlugin(api);
}
