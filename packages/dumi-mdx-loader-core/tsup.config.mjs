// @ts-check
import createOptions, { emitDeclarations } from 'tsup-utils';
import { defineConfig } from 'tsup';

export default defineConfig((options) => ({
  ...createOptions(options),
  entry: ['src/index.ts'],
  format: ['esm'],
  onSuccess: emitDeclarations({
    src: 'src',
    out: 'dist/typing',
    tsconfig: 'tsconfig.build.json',
  }),
}));
