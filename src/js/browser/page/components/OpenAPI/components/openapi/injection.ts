import { useMDXComponents } from "@mdx-js/react";
import type { ComponentType } from "react";

export type OpenAPIComponents = {
  OperationTitle?: ComponentType<{ id?: string }>;
};

export function useOpenAPIComponents(): OpenAPIComponents {
  return useMDXComponents() as OpenAPIComponents;
}
