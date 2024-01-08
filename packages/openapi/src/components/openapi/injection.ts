import { useMDXComponents } from '@mdx-js/react';

export type OpenAPIComponents = {
  OperationTitle?: React.FC<{ id?: string }>;
};

export function useOpenAPIComponents(): OpenAPIComponents {
  return useMDXComponents() as OpenAPIComponents;
}
