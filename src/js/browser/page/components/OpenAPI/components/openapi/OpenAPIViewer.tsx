import { Trans } from "@lingui/react/macro";
import { MDXProvider } from "@mdx-js/react";
import { useQuery } from "@tanstack/react-query";
import { Alert, ConfigProvider, Divider } from "antd";
import OAS from "oas";
import type { OASDocument } from "oas/types";
import OASNormalize from "oas-normalize";
import type { PropsWithChildren } from "react";
import { Fragment } from "react";
import { styled } from "styled-components";
import YAML from "yaml";

import { Loading } from "../../../../../layout/Loading";
import { theme } from "../../../../../theme";
import { intersperse } from "../../utils/itertools";

import { OperationViewer } from "./OperationViewer";
import type { OpenAPIComponents } from "./injection";

const RootContainer = styled.div`
  box-sizing: border-box;
  font-family: ${theme.fonts.sansSerif};
  font-size: 14px;
`;

export function OpenAPIViewer({
  schema,
  components,
}: {
  schema: unknown;
  components?: OpenAPIComponents;
}) {
  const { data: api, error } = useQuery({
    queryFn: async () => {
      const raw = (() => {
        if (typeof schema === "string") {
          return YAML.parse(schema);
        }
        return schema;
      })();
      const converted = await new OASNormalize(raw).convert();
      const api = new OAS(converted as OASDocument);
      await api.dereference();
      return api;
    },
    queryKey: [String(OpenAPIViewer), schema],
  });
  if (error) {
    return (
      <Alert
        showIcon
        type="error"
        message={<Trans>Failed to parse OpenAPI schema</Trans>}
        description={error.message}
      />
    );
  }
  if (!api) {
    return <Loading />;
  }
  const paths = api.getPaths();
  return (
    <Fragment>
      <MDXProvider components={components}>
        <ThemeConfig>
          <RootContainer>
            {intersperse(
              Object.entries(paths).flatMap(([path, methods]) =>
                Object.entries(methods).map(([method, operation]) => (
                  <OperationViewer key={`${method} ${path}`} operation={operation} />
                )),
              ),
              (i) => (
                <Divider key={`divider-${i}`} />
              ),
            )}
          </RootContainer>
        </ThemeConfig>
      </MDXProvider>
    </Fragment>
  );
}

function ThemeConfig({ children }: PropsWithChildren) {
  return (
    <ConfigProvider
      theme={{
        components: {
          Collapse: {
            headerPadding: `${theme.spacing.xs} ${theme.spacing.sm}`,
            contentPadding: `${theme.spacing.sm} ${theme.spacing.s}`,
          },
        },
      }}
    >
      {children}
    </ConfigProvider>
  );
}

export type { OpenAPIComponents };
