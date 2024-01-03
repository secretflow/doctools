import path from 'node:path';

import { lingui } from '@lingui/vite-plugin';
import react from '@vitejs/plugin-react-swc';
import { polyfillNode } from 'esbuild-plugin-polyfill-node';
import { defineConfig } from 'vite';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

import { dependencies, peerDependencies } from './package.json';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react({
      plugins: [
        ['@lingui/swc-plugin', {}],
        ['@swc/plugin-styled-components', {}],
      ],
    }),
    lingui(),
    nodePolyfills({
      overrides: {
        fs: 'memfs',
      },
    }),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      // ...Object.fromEntries(
      //   [...Object.entries(peerDependencies), ...Object.entries(dependencies)].map(
      //     ([k, v]) => [k, `https://esm.sh/${k}@${v}`],
      //   ),
      // ),
    },
  },
  define: {
    'process.env.NODE_ENV': JSON.stringify(process.env['NODE_ENV']),
  },
  build: {
    target: 'esnext',
    lib: {
      entry: ['./src/index.ts', './src/sphinx.ts'],
      formats: ['es'],
    },
    rollupOptions: {
      output: {
        entryFileNames: '[name].js',
      },
      external: [...Object.keys(dependencies), ...Object.keys(peerDependencies)].map(
        (k) => new RegExp(`^${k}(/|$)`),
      ),
    },
  },
  optimizeDeps: {
    esbuildOptions: {
      plugins: [
        polyfillNode({
          globals: { buffer: true },
          polyfills: {
            buffer: true,
            url: true,
            process: true,
            fs: true,
            'fs/promises': true,
          },
        }),
        {
          // silences "cannot be marked as external" errors
          // i don't know how this works
          // https://github.com/remorses/esbuild-plugins/issues/24#issuecomment-1369928859
          // https://github.com/evanw/esbuild/issues/2762
          name: 'fix-node-globals-polyfill',
          setup(build) {
            build.onResolve(
              { filter: /esbuild-plugin-polyfill-node\/polyfills/ },
              (r) => ({
                path: r.path,
                external: false,
              }),
            );
          },
        },
      ],
    },
  },
});
