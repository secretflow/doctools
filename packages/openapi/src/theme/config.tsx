import { ConfigProvider } from 'antd';
import { Helmet } from 'react-helmet';

import { lightTheme } from './index';

export const ThemeConfig = ({ children }: React.PropsWithChildren) => (
  <ConfigProvider
    theme={{
      components: {
        Collapse: {
          headerPadding: `${lightTheme.vars.openapi.spacing.xs} ${lightTheme.vars.openapi.spacing.s}`,
          contentPadding: lightTheme.vars.openapi.spacing.s,
        },
      },
    }}
  >
    {children}
  </ConfigProvider>
);

export const ThemeResources = () => (
  <Helmet>
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
    <link
      href="https://fonts.googleapis.com/css2?family=Noto+Sans+SC:wght@400;500;600;700&family=Roboto:ital,wght@0,300;0,400;0,500;0,700;1,300;1,400;1,500;1,700&display=swap"
      rel="stylesheet"
    />
    <link rel="preconnect" href="https://rsms.me/" crossOrigin="anonymous" />
    <link rel="stylesheet" href="https://rsms.me/inter/inter.css" />
  </Helmet>
);
