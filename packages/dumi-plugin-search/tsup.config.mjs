// @ts-check
import { tsup } from '@secretflow/repo-utils';
import { globbySync } from 'globby';
import { defineConfig } from 'tsup';

export default defineConfig((overrides) => [
  {
    ...tsup.defineOptions(overrides),
    entry: [
      './src/client/index.mts',
      './src/worker/index.mts',
      './src/loader/index.mts',
      './src/plugin/index.mts',
      ...globbySync('./src/backends/*/index.{ts,mts}'),
    ],
    external: [/dumi-plugin-search\/runtime/],
    outDir: 'dist',
    format: ['esm'],
  },
  {
    ...tsup.defineOptions(overrides),
    entry: ['./src/loader/index.cts', './src/plugin/index.cts'],
    external: ['./index.mjs'],
    outDir: 'dist',
    format: ['cjs'],
  },
]);
