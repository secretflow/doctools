// @ts-check
import createOptions from 'tsup-utils';
import { globbySync } from 'globby';
import { defineConfig } from 'tsup';

export default defineConfig((overrides) => [
  {
    ...createOptions(overrides),
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
    ...createOptions(overrides),
    entry: ['./src/loader/index.cts', './src/plugin/index.cts'],
    external: ['./index.mjs'],
    outDir: 'dist',
    format: ['cjs'],
  },
]);
