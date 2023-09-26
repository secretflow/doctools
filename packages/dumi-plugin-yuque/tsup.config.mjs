// @ts-check
import { tsup } from '@secretflow/repo-utils';
import { defineConfig } from 'tsup';

export default defineConfig((options) => [
  {
    ...tsup.defineOptions(options),
    entry: ['src/index.mts'],
    external: ['./html-preprocessor/index.cjs'],
    outDir: 'dist',
    format: ['esm'],
    onSuccess: tsup.emitDeclarations({
      src: 'src',
      out: 'dist/typing',
      tsconfig: 'tsconfig.build.json',
    }),
  },
  {
    ...tsup.defineOptions(options),
    entry: ['src/index.cts', 'src/html-preprocessor/index.ts'],
    external: ['./index.mjs'],
    outDir: 'dist',
    format: ['cjs'],
  },
]);
