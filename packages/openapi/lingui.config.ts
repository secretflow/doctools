import type { LinguiConfig } from '@lingui/conf';

const config: LinguiConfig = {
  locales: ['en-US', 'zh-Hans'],
  compileNamespace: 'es',
  catalogs: [
    {
      path: '<rootDir>/src/locales/{locale}',
      include: ['src'],
    },
  ],
};

export default config;
