import { defineConfig } from 'dumi';

export default defineConfig({
  themeConfig: {
    name: 'Sphinx',
    nprogress: false,
  },
  mfsu: false,
  mdxLoader: {
    swc: true,
    experimental: {
      replaceDefaultCompiler: true,
      reactContext: true,
      searchIndex: true,
    },
  },
});
