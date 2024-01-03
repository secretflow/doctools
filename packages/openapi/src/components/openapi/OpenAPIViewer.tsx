import { Alert, Skeleton, Divider } from 'antd';
import OAS from 'oas';
import type { OASDocument } from 'oas/types';
import OASNormalize from 'oas-normalize';
import useSWR from 'swr';
import YAML from 'yaml';

import { intersperse } from '@/utils/itertools';

import { Operation } from './Operation';

function resolveAPI(schema: unknown): () => Promise<OAS> {
  return async () => {
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

export function OpenAPIViewer({ schema }: { schema: unknown }) {
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
    <div>
      {intersperse(
        Object.entries(paths).flatMap(([path, methods]) =>
          Object.entries(methods).map(([method, operation]) => {
            return (
              <Operation
                key={`${method} ${path}`}
                method={method}
                path={path}
                operation={operation}
              />
            );
          }),
        ),
        (i) => (
          <Divider key={`divider-${i}`} style={{ margin: '2rem 0' }} />
        ),
      )}
    </div>
  );
}
