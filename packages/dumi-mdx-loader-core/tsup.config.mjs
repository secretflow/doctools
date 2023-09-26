// @ts-check
import { tsup } from '@secretflow/repo-utils';
import { defineConfig } from 'tsup';

export default defineConfig((options) => ({
  ...tsup.defineOptions(options),
  entry: ['src/index.ts'],
  format: ['esm'],
  onSuccess: tsup.emitDeclarations({
    src: 'src',
    out: 'dist/typing',
    tsconfig: 'tsconfig.build.json',
  }),
}));
