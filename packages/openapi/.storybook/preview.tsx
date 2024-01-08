import { I18nProvider } from '@lingui/react';
import type { Preview } from '@storybook/react';
import { useEffect } from 'react';

import { i18n } from '../src/i18n';
import { lightTheme } from '../src/theme';
import { ThemeConfig, ThemeResources } from '../src/theme/config';

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
      <ThemeConfig>
        <I18nContext locale={context.globals.locale}>
          <lightTheme.ThemeVariables />
          <ThemeResources />
          <Story />
        </I18nContext>
      </ThemeConfig>
    ),
  ],
};

export default preview;
