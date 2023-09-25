import { MDXProvider } from '@mdx-js/react';
import { Alert } from 'antd';
import { Link } from 'dumi';
import Badge from 'dumi/theme-default/builtins/Badge';
import Container from 'dumi/theme-default/builtins/Container';
import SourceCode from 'dumi/theme-default/builtins/SourceCode';
import Table from 'dumi/theme-default/builtins/Table';
import React, { forwardRef } from 'react';
import styled, { ThemeProvider } from 'styled-components';

import * as intrinsic from './intrinsic-elements.js';

import { TrackPagePosition } from '~/internals/common/positioning.js';
import * as components from '~/internals/components/index.js';
import * as theming from '~/internals/theming/index.js';

const Article = styled.article`
  height: 100%;

  overflow-x: hidden;
  overflow-y: auto;

  flex: 1 1 auto;
  min-width: 0;

  display: flex;
  flex-flow: column nowrap;
  gap: 1rem;

  font-family: ${(props) => props.theme.typography.text.fontFamily};
  color: ${(props) => props.theme.colors.text};

  line-height: 1.65rem;

  * {
    box-sizing: border-box;
  }

  ${theming.typesetting}
`;

type ForeignComponent = (props: React.PropsWithChildren) => React.ReactElement;

export const DocumentRenderer = forwardRef<HTMLElement, React.PropsWithChildren>(
  function DocumentRenderer({ children }, ref) {
    return (
      <TrackPagePosition>
        <ThemeProvider theme={theming.defaultTokens}>
          <MDXProvider
            components={{
              Link,
              Badge: Badge as unknown as ForeignComponent,
              Container: Container as unknown as ForeignComponent,
              SourceCode: SourceCode as unknown as ForeignComponent,
              Table: Table as unknown as ForeignComponent,
              ...intrinsic,
              ...components,
            }}
          >
            <Alert.ErrorBoundary>
              <Article ref={ref}>{children}</Article>
            </Alert.ErrorBoundary>
          </MDXProvider>
        </ThemeProvider>
      </TrackPagePosition>
    );
  },
);
