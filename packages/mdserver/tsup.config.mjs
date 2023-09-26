// @ts-check

import { tsup } from '@secretflow/repo-utils';
import { defineConfig } from 'tsup';

export default defineConfig((options) => ({
  ...tsup.defineOptions(options),
  entry: ['src/mdserver.mts'],
  format: ['cjs'],
  minify: true,
  sourcemap: false,
  dts: false,
}));
