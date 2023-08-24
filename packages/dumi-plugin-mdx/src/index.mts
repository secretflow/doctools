import type { Pipeline } from '@secretflow/dumi-mdx-loader-core';
import { mdxLoader } from '@secretflow/dumi-mdx-loader-core';
import { rehypeArticleOutline } from '@secretflow/unified-toolkit/rehype-article-outline';
import { rehypeAssetModules } from '@secretflow/unified-toolkit/rehype-asset-modules';
import { rehypeBetterTable } from '@secretflow/unified-toolkit/rehype-better-table';
import { rehypeDumiMetadataModule } from '@secretflow/unified-toolkit/rehype-dumi-metadata-module';
import { rehypePrettyCode } from '@secretflow/unified-toolkit/rehype-pretty-code';
import { rehypeReactRouter } from '@secretflow/unified-toolkit/rehype-react-router';
import { rehypeRemoveEmptyElements } from '@secretflow/unified-toolkit/rehype-remove-empty-elements';
import { remarkAdmonitions } from '@secretflow/unified-toolkit/remark-admonitions';
import { remarkAttrs } from '@secretflow/unified-toolkit/remark-attrs';
import { remarkDumiContentModule } from '@secretflow/unified-toolkit/remark-dumi-content-module';
import { remarkValidDOMNesting } from '@secretflow/unified-toolkit/remark-valid-dom-nesting';
import { MDX_NODE_TYPES } from '@secretflow/unified-toolkit/utils';
import type ChainMap from '@umijs/bundler-webpack/compiled/webpack-5-chain/types';
import type { IApi as DumiAPI } from 'dumi';
import rehypeKatex from 'rehype-katex';
import rehypeRaw from 'rehype-raw';
import remarkDirective from 'remark-directive';
import remarkExtractFrontmatter from 'remark-extract-frontmatter';
import remarkFrontmatter from 'remark-frontmatter';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import YAML from 'yaml';

const DEFAULT_ASSETS = /\.(png|jpe?g|gif|tiff|svg|mp4|m4v|mov|webp|pdf)$/;

// Extract the zod builder interface from Umi
// Umi bundles zod (to a specific version) so we can't install zod ourselves otherwise
// there'd would be type mismatch (and we also don't want to pin the version)
type ZodBuilder = Parameters<
  NonNullable<NonNullable<Parameters<DumiAPI['describe']>[0]['config']>['schema']>
>[0]['zod'];

// Make this a standalone function instead of inlining it in api.describe so we can
// infer the type of the config
const configSchema = ({ zod }: { zod: ZodBuilder }) =>
  zod.object({
    swc: zod
      .boolean()
      .optional()
      .describe('Set to true to use SWC instead of Babel. Default is false.'),
    assets: zod
      .custom<RegExp>((x) => x instanceof RegExp || typeof x === 'string')
      .transform((x) => {
        if (typeof x === 'string') {
          return new RegExp(x);
        }
        return x;
      })
      .optional()
      .default(DEFAULT_ASSETS)
      .describe(
        'RegExp to test for assets files that will be bundled by webpack. Default: ' +
          DEFAULT_ASSETS,
      ),
    experimental: zod
      .object({
        replaceDefaultCompiler: zod
          .boolean()
          .optional()
          .default(false)
          .describe(
            'By default, this plugin only compiles .mdx files, and .md files will be' +
              " compiled using Dumi's own Markdown compiler. Set this to true to" +
              ' compile .md files using this plugin as well. Default: false.',
          ),
        searchIndex: zod
          .boolean()
          .optional()
          .default(false)
          .describe(
            '(Not ready) Enable the experimental search index bundler. Default: false.',
          ),
        reactContext: zod
          .boolean()
          .optional()
          .default(false)
          .describe(
            'Provide builtin components via [@mdx-js/react](https://mdxjs.com/packages/react/)' +
              ' instead of injecting import statements into each page.' +
              ' **You MUST provide the <MDXProvider /> component in your layout yourself.**' +
              ' Default: false.',
          ),
      })
      .optional(),
  });

type PluginConfig = Zod.infer<ReturnType<typeof configSchema>>;

const CONFIG_KEY = 'mdxLoader';

// Umi only uses the zod schema for validation, not actual parsing
// so things like `optional` and `default` are ignored
// so this function could've been more useful had we used configSchema to parse
// the config object, except we can't because we don't have access to the zod builder
// anymore by the time this function is called
function getConfig(api: DumiAPI): PluginConfig {
  return api.config[CONFIG_KEY] ?? {};
}

export function plugin(api: DumiAPI) {
  api.describe({ key: CONFIG_KEY, config: { schema: configSchema } });

  api.chainWebpack((memo: ChainMap) => {
    const { experimental } = getConfig(api);
    if (experimental?.replaceDefaultCompiler) {
      // take out Dumi's Markdown rule entirely
      memo.module.rules.delete('dumi-md');
    }
  });

  // configure Dumi to process .md and .mdx files using this loader
  mdxLoader(api)(() => {
    const {
      // Umi only uses the zod schema for validation, not actual parsing
      // so despite us providing defaults in the zod schema, we still need to
      // provide defaults here to avoid undefined errors
      assets = DEFAULT_ASSETS,
      swc = false,
      experimental,
    } = getConfig(api);

    const contentPipeline: Pipeline = {
      remarkPlugins: [
        // metadata
        [remarkFrontmatter],
        [remarkExtractFrontmatter, { yaml: YAML.parse, name: 'frontmatter' }],
        // markup parsing
        [remarkGfm],
        [remarkDirective],
        [remarkMath],
        [remarkAdmonitions],
        [remarkAttrs],
        // validation
        [remarkValidDOMNesting],
        // routing
        [rehypeReactRouter, { routes: api.appData['routes'] }],
        // bundling
        [rehypeAssetModules, { test: assets }],
      ],
      rehypePlugins: [
        // see https://github.com/rehypejs/rehype-raw#when-should-i-use-this
        // has no effect when file is .mdx, in which "HTML" elements are treated
        // as JSX instead of HTML
        // see https://mdxjs.com/packages/mdx/#optionsformat
        [rehypeRaw, { passThrough: MDX_NODE_TYPES }],
        // element transformation
        [rehypeKatex],
        [rehypeBetterTable],
        [rehypePrettyCode],
        // validation
        [rehypeRemoveEmptyElements],
      ],
    };

    if (!experimental?.reactContext) {
      // export
      contentPipeline.remarkPlugins?.push([
        remarkDumiContentModule,
        { builtins: api.service.themeData.builtins },
      ]);
    }

    const metadataPipeline: Pipeline = {
      remarkPlugins: [
        // metadata
        [remarkFrontmatter],
        [remarkExtractFrontmatter, { yaml: YAML.parse, name: 'frontmatter' }],
        // markup parsing
        [remarkGfm],
        [remarkDirective],
        [remarkMath],
        [remarkAdmonitions],
        [remarkAttrs],
      ],
      rehypePlugins: [
        // outline extraction
        [rehypeArticleOutline],
        // export
        [
          rehypeDumiMetadataModule,
          {
            routes: api.appData['routes'],
            legacySearch: !experimental?.searchIndex,
          },
        ],
      ],
    };

    const targetExtensions: string[] = ['mdx'];

    if (experimental?.replaceDefaultCompiler) {
      targetExtensions.push('md');
    }

    return {
      extensions: targetExtensions,
      swc,
      pipelines: {
        content: contentPipeline,
        metadata: metadataPipeline,
      },
    };
  });
}
