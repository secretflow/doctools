import type { ComponentProps } from "react";
import { Suspense, lazy } from "react";
import { styled } from "styled-components";

import { Loading } from "../../layout/Loading";
import { theme } from "../../theme";

import { h2 } from "./intrinsic";

const CSSOverrides = styled.div`
  div[class^="OpenAPIViewer"] {
    ul[class^="components__"] {
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
      &[class^="components__"] {
        margin: 6px 0;

        &:first-child {
          margin-block-start: 0;
        }

        &:last-child {
          margin-block-end: 0;
        }
      }
    }

    h2 {
      margin: 0;
    }

    code {
      font-family: ${theme.fonts.monospace};
      color: ${theme.colors.fg.default};

      &[class^="SchemaTree"] {
        color: inherit;
      }
    }

    .ant-collapse-header {
      padding: ${theme.spacing.xs} ${theme.spacing.s};
    }

    .ant-collapse-content-box {
      padding: ${theme.spacing.s};
    }

    section[class^="OperationViewer"] {
      h3 {
        margin: 0;
        font-family: ${theme.fonts.sansSerif};
        font-size: 0.9rem;
        font-weight: 600;
        line-height: 1rem;
      }

      h4[class^="OperationViewer"] {
        margin: 0;
        font-family: ${theme.fonts.sansSerif};
        font-size: 14px;
        font-weight: 500;
        line-height: 22px;
        color: ${theme.colors.fg.muted};
        user-select: none;
      }
    }

    section[class^="SchemaTree"] {
      & > div[class^="SchemaHeader"] > div[class^="SchemaHeader"] {
        line-height: 1.2rem;

        & > span[class^="SchemaHeader"] {
          line-height: 1.2rem;
        }
      }
    }

    .ant-collapse > .ant-collapse-item > .ant-collapse-header {
      align-items: flex-start;
    }

    ul[class^="SchemaTree"] {
      gap: 0;
      padding: 0;
    }
  }
`;

const OpenAPIViewer = lazy(() =>
  import("./OpenAPI/index").then((m) => ({ default: m.OpenAPIViewer })),
);

export function OpenAPI(props: ComponentProps<typeof OpenAPIViewer>) {
  return (
    <Suspense fallback={<Loading />}>
      <CSSOverrides>
        <OpenAPIViewer components={{ OperationTitle: h2 }} {...props} />
      </CSSOverrides>
    </Suspense>
  );
}
