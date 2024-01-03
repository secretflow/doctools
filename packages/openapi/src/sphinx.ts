import { createElement } from 'react';
import { createRoot } from 'react-dom/client';
import styled from 'styled-components';

import * as enUS from '@/locales/en-US.mjs';
import * as zhHans from '@/locales/zh-Hans.mjs';

export const Container = styled.div`
  background-color: #fefefe;
  padding: 16px;
  border-radius: 8px;

  p {
    color: #1a1a1a;
  }

  code {
    color: #d63384;
  }

  pre {
    border: none;

    code {
      color: #1a1a1a;
    }
  }
`;

declare global {
  interface Window {
    define: unknown;
  }
}

document.addEventListener('DOMContentLoaded', async () => {
  const sites: [Element, string][] = [];

  document.querySelectorAll('div.highlight-swagger').forEach((elem) => {
    const raw = elem.querySelector('pre')?.textContent;
    if (!raw) {
      return;
    }
    sites.push([elem, raw]);
    createRoot(elem).render(createElement('div', null, 'Loading...'));
  });

  const i18n = await (async () => {
    // https://github.com/no-context/moo/blob/main/moo.js#L1-L9
    const define = window.define;
    window.define = undefined;
    const core = await import('@lingui/core');
    window.define = define;
    return core.i18n;
  })();

  const { I18nProvider } = await import('@lingui/react');
  const { OpenAPIViewer } = await import('./components/openapi/OpenAPIViewer');

  i18n.load({
    'en-US': enUS.messages,
    'zh-Hans': zhHans.messages,
  });
  i18n.activate('en-US');

  sites.forEach(([elem, raw]) => {
    createRoot(elem).render(
      createElement(
        I18nProvider,
        { i18n },
        createElement(Container, null, createElement(OpenAPIViewer, { schema: raw })),
      ),
    );
  });
});
