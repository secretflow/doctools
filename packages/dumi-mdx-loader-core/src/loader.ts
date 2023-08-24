import { createRequire } from 'node:module';

import type { Options as LoaderOptions } from '@mdx-js/loader';
import type ChainMap from '@umijs/bundler-webpack/compiled/webpack-5-chain/types';

import { extensionTest, uniqueKey } from './utils.js';

const require = createRequire(import.meta.url);

export type Pipeline = Pick<LoaderOptions, 'remarkPlugins' | 'rehypePlugins'>;

type Options = Pick<LoaderOptions, 'format'> & {
  /** Extensions of the doc files to be included, without the leading dot */
  extensions: string[];
  /**
   * Functions returning unified ecosystem plugins to be used for code transformation,
   * one for metadata modules and one for pages
   *
   * They are functions because access of some properties on Dumi's API must be deferred
   * until chainWebpack is called
   */
  pipelines: {
    content: Pipeline;
    metadata: Pipeline;
  };
  swc?: boolean;
  preprocessor?: (obj: ChainMap.Rule<ChainMap.Rule<ChainMap.Module>>) => void;
  resolve?: {
    '@mdx-js/loader'?: string;
    '@mdx-js/react'?: string;
    'swc-loader'?: string;
  };
};

const noop = () => {
  // noop
};

export function attachLoader(options: () => Options): (memo: ChainMap) => void {
  return (memo: ChainMap) => {
    const {
      format,
      extensions,
      pipelines,
      swc,
      resolve = {},
      preprocessor = noop,
    } = options();

    const {
      'swc-loader': swcLoader = 'swc-loader',
      '@mdx-js/loader': mdxLoader = '@mdx-js/loader',
      '@mdx-js/react': reactProvider = '@mdx-js/react',
    } = resolve;

    const test = extensionTest(extensions);

    const useBabel = (config: ChainMap.Rule<ChainMap.Rule<ChainMap.Module>>) => {
      const umiBabelLoader = memo.module.rule('src').use('babel-loader').entries();
      config
        .use('babel-loader')
        .loader(umiBabelLoader.loader)
        .options(umiBabelLoader.options)
        .end();
    };

    const useSWC = (config: ChainMap.Rule<ChainMap.Rule<ChainMap.Module>>) =>
      config.use('swc-loader').loader(require.resolve(swcLoader)).end();

    memo.module

      // compile MDX to components
      .rule(uniqueKey('mdx:loader', ...extensions))
      .type('javascript/auto')
      .test(test)

      // create metadata modules for .mdx files to be imported by Dumi
      // modules will be imported like this:
      // import meta from '/path/to/page.mdx?type=meta'
      .oneOf(uniqueKey('mdx:metadata', ...extensions))
      .resourceQuery(/meta$/)

      .when(swc === true, useSWC, useBabel)

      .use('@mdx-js/loader')
      .loader(require.resolve(mdxLoader))
      .options({ format, ...pipelines.metadata } satisfies LoaderOptions)
      .end()

      .when(Boolean(preprocessor), preprocessor)

      .end()

      // create page content modules for .mdx files to be imported by Dumi
      // modules will be imported like this:
      // React.lazy(() => import('/path/to/page.mdx'))
      .oneOf(uniqueKey('mdx:content', ...extensions))

      .when(swc === true, useSWC, useBabel)

      .use('@mdx-js/loader')
      .loader(require.resolve(mdxLoader))
      .options({
        format,
        providerImportSource: require.resolve(reactProvider),
        ...pipelines.content,
      } satisfies LoaderOptions)
      .end()

      .when(Boolean(preprocessor), preprocessor)

      .end()

      .end();
  };
}

export type { Options as WebpackOptions };
