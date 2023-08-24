// @ts-check
import createOptions, { emitDeclarations } from 'tsup-utils';
import { defineConfig } from 'tsup';

export default defineConfig((options) => [
  {
    ...createOptions(options),
    entry: ['src/index.mts'],
    external: ['./html-preprocessor/index.cjs'],
    outDir: 'dist',
    format: ['esm'],
    onSuccess: emitDeclarations({
      src: 'src',
      out: 'dist/typing',
      tsconfig: 'tsconfig.build.json',
    }),
  },
  {
    ...createOptions(options),
    entry: ['src/index.cts', 'src/html-preprocessor/index.ts'],
    external: ['./index.mjs'],
    outDir: 'dist',
    format: ['cjs'],
  },
]);
