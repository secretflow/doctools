import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov', 'json'],
      all: true,
      include: ['**/{.dumi,src}/**/*.{ts,mts,cts,tsx}'],
    },
  },
});
