// @ts-check
import { tsup } from '@secretflow/repo-utils';
import { globbySync } from 'globby';
import { defineConfig } from 'tsup';

// TODO: Explain the whole ESM/CJS saga

export default defineConfig((options) => [
  {
    ...tsup.defineOptions(options),
    entry: globbySync([
      './src/exports/index.{ts,tsx,mts,mtsx}',
      './src/{builtins,layouts,slots}/*.{ts,tsx,mts,mtsx}',
      './src/{builtins,layouts,slots}/*/index.{ts,tsx,mts,mtsx}',
      './src/locales/*.json',
    ]),
    external: [/^@@\//],
    format: ['esm'],
    loader: {
      '.json': 'copy',
    },
    outExtension: () => ({ js: '.js' }),
    onSuccess: tsup.emitDeclarations({ src: 'src', out: 'dist/typing' }),
  },
  {
    ...tsup.defineOptions(options),
    entry: ['./src/plugin/index.mts'],
    outDir: './dist/plugin',
    format: ['esm'],
  },
  {
    ...tsup.defineOptions(options),
    entry: globbySync(['./src/plugin/index.cts', './src/plugin/package.json']),
    external: ['./index.mjs'],
    outDir: './dist/plugin',
    format: ['cjs'],
    loader: {
      '.json': 'copy',
    },
    outExtension: () => ({ js: '.js' }),
  },
]);
