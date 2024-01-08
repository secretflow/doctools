import { I18nProvider } from '@lingui/react';
import { MDXProvider } from '@mdx-js/react';
import { Alert, Skeleton, Divider } from 'antd';
import type OAS from 'oas';
import type { OASDocument } from 'oas/types';
import styled from 'styled-components';
import useSWR from 'swr';
import YAML from 'yaml';

import { i18n } from '@/i18n';
import { ThemeConfig } from '@/index';
import { lightTheme } from '@/theme';
import { intersperse } from '@/utils/itertools';

import type { OpenAPIComponents } from './injection';
import { OperationViewer } from './OperationViewer';

function resolveAPI(schema: unknown): () => Promise<OAS> {
  return async () => {
    const { default: OAS } = await import('oas');
    const { default: OASNormalize } = await import('oas-normalize');
    const raw = (() => {
      if (typeof schema === 'string') {
        return YAML.parse(schema);
      }
      return schema;
    })();
    const converted = await new OASNormalize(raw).validate({ convertToLatest: true });
    const api = new OAS(converted as OASDocument);
    await api.dereference();
    return api;
  };
}

const RootContainer = styled.div`
  box-sizing: border-box;
  font-family: ${lightTheme.vars.openapi.typography.sans};
  font-size: 14px;
`;

export function OpenAPIViewer({
  schema,
  components,
}: {
  schema: unknown;
  components?: OpenAPIComponents;
}) {
  const {
    data: api,
    isLoading,
    error,
  } = useSWR<OAS>(['openapi', schema], resolveAPI(schema));
  if (error) {
    return (
      <Alert
        showIcon
        type="error"
        message="Failed to parse OpenAPI schema"
        description={error.message}
      />
    );
  }
  if (!api || isLoading) {
    return <Skeleton active />;
  }
  const paths = api.getPaths();
  return (
    <I18nProvider i18n={i18n}>
      <lightTheme.ThemeVariables />
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
    </I18nProvider>
  );
}

export { type OpenAPIComponents };
