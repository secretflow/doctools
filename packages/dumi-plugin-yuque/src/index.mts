import { createRequire } from 'node:module';

import { mdxLoader } from '@secretflow/dumi-mdx-loader-core';
import { rehypeArticleOutline } from '@secretflow/unified-toolkit/rehype-article-outline';
import { rehypeBetterTable } from '@secretflow/unified-toolkit/rehype-better-table';
import { rehypeDumiMetadataModule } from '@secretflow/unified-toolkit/rehype-dumi-metadata-module';
import { rehypePrettyCode } from '@secretflow/unified-toolkit/rehype-pretty-code';
import { rehypeRemoveEmptyElements } from '@secretflow/unified-toolkit/rehype-remove-empty-elements';
import { remarkDumiContentModule } from '@secretflow/unified-toolkit/remark-dumi-content-module';
import { MDX_NODE_TYPES } from '@secretflow/unified-toolkit/utils';
import type { IApi as DumiAPI } from 'dumi';
import rehypeRaw from 'rehype-raw';
import remarkExtractFrontmatter from 'remark-extract-frontmatter';
import remarkFrontmatter from 'remark-frontmatter';
import YAML from 'yaml';

import { rehypeAddReferrerpolicy } from './rehype-add-referrerpolicy/index.mjs';
import { rehypeRemoveClassNames } from './rehype-remove-classnames/index.mjs';
import { rehypeRemoveStyle } from './rehype-remove-styles/index.mjs';
import { rehypeRemoveDumiWrapper } from './rehype-remove-yuque-wrapper/index.mjs';
import type { RouteMappingOptions } from './rehype-rewrite-links/index.mjs';
import { rehypeRewriteLinks } from './rehype-rewrite-links/index.mjs';

const require = createRequire(import.meta.url);

export function plugin(api: DumiAPI) {
  mdxLoader(api)(() => ({
    format: 'md',
    extensions: ['html'],
    pipelines: {
      content: {
        remarkPlugins: [
          [remarkFrontmatter],
          [remarkDumiContentModule, { builtins: api.service.themeData.builtins }],
        ],
        rehypePlugins: [
          [rehypeRaw, { passThrough: MDX_NODE_TYPES }],
          [rehypePrettyCode],
          [rehypeRemoveClassNames],
          [rehypeBetterTable],
          [rehypeAddReferrerpolicy],
          [
            rehypeRemoveStyle,
            {
              properties: ['fontSize', 'lineHeight', { tr: 'height', td: 'width' }],
            },
          ],
          [
            rehypeRewriteLinks,
            {
              routes: api.appData['routes'],
              match: /admin(\/[0-9A-Za-z]+)$/,
              resolve: (id, path) => path.endsWith(id),
            } satisfies RouteMappingOptions,
          ],
          [rehypeRemoveEmptyElements],
          [rehypeRemoveDumiWrapper],
        ],
      },
      metadata: {
        remarkPlugins: [
          [remarkFrontmatter],
          [remarkExtractFrontmatter, { yaml: YAML.parse, name: 'frontmatter' }],
        ],
        rehypePlugins: [
          [rehypeRaw, { passThrough: MDX_NODE_TYPES }],
          [rehypeArticleOutline],
          [
            rehypeDumiMetadataModule,
            { legacySearch: true, routes: api.appData['routes'] },
          ],
        ],
      },
    },
    preprocessor: (config) =>
      config
        .use('html-preprocessor')
        .loader(require.resolve('./html-preprocessor/index.cjs'))
        .end(),
  }));
}
