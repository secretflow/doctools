// @ts-check
import createOptions, { emitDeclarations } from 'tsup-utils';
import { globbySync } from 'globby';
import { defineConfig } from 'tsup';

// TODO: Explain the whole ESM/CJS saga

export default defineConfig((options) => [
  {
    ...createOptions(options),
    entry: globbySync([
      './src/exports/index.{ts,tsx,mts,mtsx}',
      './src/{builtins,layouts,slots}/*.{ts,tsx,mts,mtsx}',
      './src/{builtins,layouts,slots}/*/index.{ts,tsx,mts,mtsx}',
      './src/locales/*.json',
    ]),
    format: ['esm'],
    loader: {
      '.json': 'copy',
    },
    outExtension: () => ({ js: '.js' }),
    onSuccess: emitDeclarations({ src: 'src', out: 'dist/typing' }),
  },
  {
    ...createOptions(options),
    entry: globbySync(['./src/plugin/index.cts', './src/plugin/package.json']),
    external: [/\.\/bundled/],
    outDir: './dist/plugin',
    format: ['cjs'],
    loader: {
      '.json': 'copy',
    },
    outExtension: () => ({ js: '.js' }),
  },
]);
