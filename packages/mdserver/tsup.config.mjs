// @ts-check

import { defineConfig } from 'tsup';
import createOptions from 'tsup-utils';

export default defineConfig((options) => ({
  ...createOptions(options),
  entry: ['src/mdserver.mts'],
  format: ['cjs'],
  minify: true,
  sourcemap: false,
  dts: false,
}));
