import { Skeleton } from 'antd';
import { Suspense, lazy } from 'react';
import styled from 'styled-components';

import { h2 } from '../../exports/intrinsic-elements.js';

const CSSOverrides = styled.div`
  div[class^='OpenAPIViewer'] {
    h2 {
      margin: 0;
    }

    code {
      color: var(--openapi-colors-default);
      font-family: var(--openapi-typography-monospace);

      &[class^='SchemaTree'] {
        color: inherit;
      }
    }

    .ant-collapse-header {
      padding: var(--openapi-spacing-xs);
    }

    .ant-collapse-content-box {
      padding: var(--openapi-spacing-s);
    }

    section[class^='OperationViewer'] {
      h3 {
        font-family: var(--openapi-typography-sans);
        font-weight: 600;
        line-height: 1rem;
        font-size: 0.9rem;
        margin: 0;
      }

      h4[class^='OperationViewer'] {
        margin: 0;
        font-family: var(--openapi-typography-sans);
        font-weight: 500;
        color: var(--openapi-colors-muted);
        user-select: none;
        font-size: 14px;
        line-height: 22px;
      }
    }

    section[class^='SchemaTree'] {
      & > div[class^='SchemaHeader'] > div[class^='SchemaHeader'] {
        line-height: 1.2rem;

        & > span[class^='SchemaHeader'] {
          line-height: 1.2rem;
        }
      }
    }

    .ant-collapse > .ant-collapse-item > .ant-collapse-header {
      align-items: flex-start;
    }

    ul[class^='SchemaTree'] {
      padding: 0;
      gap: 0;
    }

    ul[class^='components__'] {
      display: block;
    }

    p,
    ul,
    ol,
    h1,
    h2,
    h3,
    h4,
    h5,
    h6,
    blockquote,
    pre {
      &[class^='components__'] {
        margin: 6px 0;

        &:first-child {
          margin-block-start: 0;
        }

        &:last-child {
          margin-block-end: 0;
        }
      }
    }
  }
`;

const OpenAPIViewer = lazy(() =>
  import('@secretflow/openapi').then((m) => ({ default: m.OpenAPIViewer })),
);

export const OpenAPI = (props: React.ComponentProps<typeof OpenAPIViewer>) => (
  <Suspense fallback={<Skeleton active />}>
    <CSSOverrides>
      <OpenAPIViewer components={{ OperationTitle: h2 }} {...props} />
    </CSSOverrides>
  </Suspense>
);
