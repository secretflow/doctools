import { createRequire } from 'node:module';

import { remarkAttrs } from '@secretflow/unified-toolkit/remark-attrs';
import { MDX_NODE_TYPES } from '@secretflow/unified-toolkit/utils';
import type ChainMap from '@umijs/bundler-webpack/compiled/webpack-5-chain/types';
import type { IApi as DumiAPI } from 'dumi';
import rehypeRaw from 'rehype-raw';
import remarkDirective from 'remark-directive';
import remarkExtractFrontmatter from 'remark-extract-frontmatter';
import remarkFrontmatter from 'remark-frontmatter';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import YAML from 'yaml';

import type { LoaderConfig } from '../loader/typing.d.js';

const require = createRequire(import.meta.url);

const CONFIG_KEY = 'search2';

// Extract the zod builder interface from Umi
// Umi bundles zod (to a specific version) so we can't install zod ourselves otherwise
// there'd would be type mismatch (and we also don't want to pin the version)
type ZodBuilder = Parameters<
  NonNullable<NonNullable<Parameters<DumiAPI['describe']>[0]['config']>['schema']>
>[0]['zod'];

const configSchema = ({ zod }: { zod: ZodBuilder }) =>
  zod.object({
    backend: zod
      .string()
      .optional()
      .describe(
        'The search backend to use.' +
          ' Must be a specifier resolving to an ES module.',
      ),
    resolve: zod
      .object({
        loader: zod
          .string()
          .optional()
          .describe(
            'The loader to use for generating the search index.' +
              ' Must be a specifier resolving to a CommonJS module.' +
              ' This could be useful if you want to customize the search index, or' +
              ' if you decide to bundle this plugin, in which case you would need to' +
              ' re-export the loader from your bundle.',
          ),
        worker: zod
          .string()
          .optional()
          .describe(
            'The Web Worker to use.' +
              ' Must be a specifier resolving to a CommonJS or ES module,' +
              ' which will be bundled by webpack.' +
              ' This could be useful if you decide to bundle this plugin, in which case' +
              ' you would need to re-export it from your bundle.',
          ),
      })
      .optional(),
  });

type PluginConfig = Zod.infer<ReturnType<typeof configSchema>>;

function getConfig(api: DumiAPI): PluginConfig {
  return api.config[CONFIG_KEY] ?? {};
}

export async function plugin(api: DumiAPI) {
  api.describe({ key: CONFIG_KEY, config: { schema: configSchema } });

  api.chainWebpack((memo: ChainMap) => {
    const {
      backend = require.resolve('../backends/orama/index.mjs'),
      resolve: {
        loader = require.resolve('../loader/index.cjs'),
        worker = require.resolve('../worker/index.mjs'),
      } = {},
    } = getConfig(api);

    memo.resolve.alias
      .set('dumi-plugin-search/runtime/worker', worker)
      .set('dumi-plugin-search/runtime/backend', backend)
      .end();

    memo.module
      .rule('dumi-plugin-search/runtime/index')
      .test(/^$/)
      .resourceQuery(/dumi-plugin-search\/runtime\/index/)
      .type('json')
      .use('dumi-plugin-search/runtime/index')
      .loader(loader)
      .options({
        backend,
        routes: api.appData['routes'],
        pipelines: {
          mdx: (processor) =>
            processor
              .use(remarkParse)
              .use(remarkMdx)
              .use(remarkFrontmatter)
              .use(remarkExtractFrontmatter, { yaml: YAML.parse, name: 'frontmatter' })
              .use(remarkGfm)
              .use(remarkDirective)
              .use(remarkMath)
              .use(remarkAttrs)
              .use(remarkRehype),
          md: (processor) =>
            processor
              .use(remarkParse)
              .use(remarkFrontmatter)
              .use(remarkExtractFrontmatter, { yaml: YAML.parse, name: 'frontmatter' })
              .use(remarkGfm)
              .use(remarkDirective)
              .use(remarkMath)
              .use(remarkAttrs)
              .use(remarkRehype)
              .use(rehypeRaw, { passThrough: MDX_NODE_TYPES }),
        },
      } satisfies LoaderConfig)
      .end();
  });
}

export { CONFIG_KEY };
