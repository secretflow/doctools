import { expectToStringifyInto } from '@secretflow/unified-toolkit/testing';
import { rehypeRecma, recmaStringify } from '@secretflow/unified-toolkit/utils';
import rehypeRaw from 'rehype-raw';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import type { Processor } from 'unified';
import { unified } from 'unified';
import { describe, beforeEach, test } from 'vitest';

import type { RouteMappingOptions } from './index.mjs';
import { rehypeRewriteLinks } from './index.mjs';

describe('remark-mdx-auto-react-router', () => {
  let processor: Processor;

  beforeEach(() => {
    processor = unified()
      .use(remarkParse)
      .use(remarkRehype, {
        allowDangerousHtml: true,
      })
      .use(rehypeRaw);
  });

  function withOptions(options: RouteMappingOptions) {
    return processor
      .use(rehypeRewriteLinks, options)
      .use(rehypeRecma, {
        elementAttributeNameCase: 'react',
        stylePropertyNameCase: 'dom',
      })
      .use(recmaStringify);
  }

  test('do nothing', async () => {
    await expectToStringifyInto({
      source:
        '<a href="https://google.com/" target="_blank" rel="noreferrer noopener">Google</a>',
      output:
        '<><p><a href="https://google.com/" target="_blank" rel="noreferrer noopener">{"Google"}</a></p></>;',
      processor: withOptions({ match: /(.*)/, routes: {} }),
      path: '/path/to/project/docs/index.md',
    });
  });

  test('simple resolve', async () => {
    await expectToStringifyInto({
      source: '<a href="/docs/foo/bar">Google</a>',
      output: '<><p><Link to="/docs/foo/bar">{"Google"}</Link></p></>;',
      processor: withOptions({
        match: /(.*)/,
        routes: {
          '/docs/foo/bar': {
            id: '/docs/foo/bar',
            path: '/docs/foo/bar',
            absPath: '/docs/foo/bar',
            file: '/path/to/project/docs/foo/bar.md',
          },
        },
      }),
      path: '/path/to/project/docs/baz.md',
    });
  });

  test('resolve with function', async () => {
    await expectToStringifyInto({
      source: '<a href="https://google.com/bar">Google</a>',
      output: '<><p><Link to="/docs/foo/bar">{"Google"}</Link></p></>;',
      processor: withOptions({
        match: /https:\/\/google.com\/(.+)$/,
        resolve: (match, path) => path.endsWith(match),
        routes: {
          '/docs/foo/bar': {
            id: '/docs/foo/bar',
            path: '/docs/foo/bar',
            absPath: '/docs/foo/bar',
            file: '/path/to/project/docs/foo/bar.md',
          },
        },
      }),
      path: '/path/to/project/docs/baz.md',
    });
  });

  test('preserve query and hash', async () => {
    await expectToStringifyInto({
      source: '<a href="https://google.com/docs/foo/bar?foo=bar#baz">Google</a>',
      output: '<><p><Link to="/docs/foo/bar?foo=bar#baz">{"Google"}</Link></p></>;',
      processor: withOptions({
        match: /https:\/\/google.com\/(.+)$/,
        resolve: (match, path) => path.endsWith(match),
        routes: {
          '/docs/foo/bar': {
            id: '/docs/foo/bar',
            path: '/docs/foo/bar',
            absPath: '/docs/foo/bar',
            file: '/path/to/project/docs/foo/bar.md',
          },
        },
      }),
      path: '/path/to/project/docs/baz.md',
    });
  });

  test('transform intra-page hash redirect into <a />', async () => {
    await expectToStringifyInto({
      source: '<a href="https://google.com/docs/foo/bar?foo=bar#baz">Google</a>',
      output: '<><p><a href="#baz">{"Google"}</a></p></>;',
      processor: withOptions({
        match: /https:\/\/google.com\/(.+)$/,
        resolve: (match, path) => path.endsWith(match),
        routes: {
          '/docs/foo/bar': {
            id: '/docs/foo/bar',
            path: '/docs/foo/bar',
            absPath: '/docs/foo/bar',
            file: '/path/to/project/docs/foo/bar.md',
          },
        },
      }),
      path: '/path/to/project/docs/foo/bar.md',
    });
  });
});
