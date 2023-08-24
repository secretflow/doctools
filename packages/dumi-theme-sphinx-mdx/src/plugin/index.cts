import type { IApi as DumiAPI } from 'dumi';

export default function preset(api: DumiAPI) {
  api.registerPlugins([
    require.resolve('@secretflow/dumi-plugin-mdx'),
    require.resolve('@secretflow/dumi-plugin-search/plugin'),
  ]);
}
