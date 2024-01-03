import type { Preview } from '@storybook/react';
import { I18nProvider } from '@lingui/react';
import { useEffect } from 'react';

import { i18n } from '../src/i18n';
import * as enUS from '../src/locales/en-US.mjs';
import * as zhHans from '../src/locales/zh-Hans.mjs';

i18n.load('en-US', enUS.messages);
i18n.load('zh-Hans', zhHans.messages);
i18n.activate('en-US');

function I18nContext({
  locale,
  children,
}: React.PropsWithChildren<{ locale: string }>) {
  useEffect(() => {
    i18n.activate(locale);
  }, [locale]);
  return <I18nProvider i18n={i18n}>{children}</I18nProvider>;
}

const preview: Preview = {
  parameters: {
    actions: { argTypesRegex: '^on[A-Z].*' },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
  globalTypes: {
    locale: {
      description: 'Language',
      defaultValue: 'en-US',
      toolbar: {
        icon: 'globe',
        items: [
          { value: 'en-US', title: 'English (US)' },
          { value: 'zh-Hans', title: '中文（简体）' },
        ],
      },
    },
  },
  decorators: [
    (Story, context) => (
      <I18nContext locale={context.globals.locale}>
        <Story />
      </I18nContext>
    ),
  ],
};

export default preview;
