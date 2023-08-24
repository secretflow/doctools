import type { IRoute } from 'dumi';
import type { Processor } from 'unified';

export type LoaderConfig = {
  backend: string;
  pipelines?: Record<string, (processor: Processor) => Processor>;
  routes?: Record<string, IRoute>;
};

export type LoaderOptions = Pick<LoaderConfig, 'backend' | 'pipelines'>;
