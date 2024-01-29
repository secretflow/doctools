import type { IRoute } from 'dumi';
import type { Processor } from 'unified';

export type LoaderConfig = {
  backend: string;
  pipelines?: Record<
    string,
    {
      processor: (processor: Processor) => Processor;
      preprocessor?: (text: string) => string;
    }
  >;
  routes?: Record<string, IRoute>;
};

export type LoaderOptions = Pick<LoaderConfig, 'backend' | 'pipelines'>;
