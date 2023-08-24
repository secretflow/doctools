import type { IApi as DumiAPI } from 'dumi';

import { findDocs } from './finder.js';
import type { WebpackOptions } from './loader.js';
import { attachLoader } from './loader.js';

export function mdxLoader(api: DumiAPI): (options: () => WebpackOptions) => void {
  return (options) => {
    api.chainWebpack(attachLoader(options));
    api.modifyRoutes({
      fn: findDocs(() => ({
        extensions: options().extensions,
        cwd: api.cwd,
        dirs: api.config.resolve.docDirs,
      })),
    });
  };
}

export type { Pipeline } from './loader.js';
