// @ts-check

import { tsup } from '@secretflow/repo-utils';
import { globbySync } from 'globby';
import { defineConfig } from 'tsup';

export default defineConfig((options) => ({
  entry: globbySync('src/*/index.ts'),
  outDir: 'dist',
  format: ['esm'],
  outExtension: tsup.outExtension,
  sourcemap: true,
  dts: false,
  onSuccess: tsup.emitDeclarations({
    src: 'src',
    out: 'dist/typing',
    tsconfig: 'tsconfig.build.json',
  }),
  clean: options.clean || !options.watch,
  ...options,
}));
