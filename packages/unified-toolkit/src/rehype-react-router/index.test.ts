import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import remarkStringify from 'remark-stringify';
import type { Processor } from 'unified';
import { unified } from 'unified';
import { describe, beforeEach, test } from 'vitest';

import { expectToStringifyInto } from '../testing/index.js';

import { rehypeReactRouter } from './index.js';

describe('remark-mdx-auto-react-router', () => {
  let processor: Processor;

  beforeEach(() => {
    processor = unified().use(remarkParse).use(remarkStringify).use(remarkMdx);
  });

  test('do nothing', async () => {
    await expectToStringifyInto({
      source: `
[link](./foo)
<a href="./foo">link</a>
    `,
      output: `
[link](./foo)
<a href="./foo">link</a>
    `,
      path: '/path/to/project/docs/index.md',
      processor: processor.use(rehypeReactRouter, { routes: {} }),
    });
  });

  test('simple resolve', async () => {
    await expectToStringifyInto({
      source: `
[link](foo/bar.md)
<a href="foo/bar.md">link</a>
    `,
      output: `
<Link to="/docs/foo/bar">link</Link>
<Link to="/docs/foo/bar">link</Link>
    `,
      path: '/path/to/project/docs/baz.md',
      processor: processor.use(rehypeReactRouter, {
        routes: {
          bar: {
            id: '1',
            path: '/docs/foo/bar',
            absPath: '/docs/foo/bar',
            file: '/path/to/project/docs/foo/bar.md',
          },
        },
      }),
    });
  });

  test('resolve when source is index.md', async () => {
    await expectToStringifyInto({
      source: `
[link](../foo/bar.md)
<a href="../foo/bar.md">link</a>
    `,
      output: `
<Link to="/docs/foo/bar">link</Link>
<Link to="/docs/foo/bar">link</Link>
    `,
      path: '/path/to/project/docs/baz/index.md',
      processor: processor.use(rehypeReactRouter, {
        routes: {
          bar: {
            id: '1',
            path: '/docs/foo/bar',
            absPath: '/docs/foo/bar',
            file: '/path/to/project/docs/foo/bar.md',
          },
        },
      }),
    });
  });

  test('resolve when target is index.md', async () => {
    await expectToStringifyInto({
      source: `
[link](foo/bar/index.md)
<a href="foo/bar/index.md">link</a>
    `,
      output: `
<Link to="/docs/foo/bar">link</Link>
<Link to="/docs/foo/bar">link</Link>
    `,
      path: '/path/to/project/docs/baz.md',
      processor: processor.use(rehypeReactRouter, {
        routes: {
          bar: {
            id: '1',
            path: '/docs/foo/bar',
            absPath: '/docs/foo/bar',
            file: '/path/to/project/docs/foo/bar/index.md',
          },
        },
      }),
    });
  });

  test('non-existent file', async () => {
    await expectToStringifyInto({
      source: `
[link](./foo/qux.md)
<a href="./foo/qux.md">link</a>
    `,
      output: `
[link](./foo/qux.md)
<a href="./foo/qux.md">link</a>
    `,
      path: '/path/to/project/docs/baz.md',
      processor: processor.use(rehypeReactRouter, {
        routes: {
          bar: {
            id: '1',
            path: '/docs/foo/bar',
            absPath: '/docs/foo/bar',
            file: '/path/to/project/docs/foo/bar/index.md',
          },
        },
      }),
    });
  });

  test('external link', async () => {
    await expectToStringifyInto({
      source: `
[Google](https://google.com)
<a href="https://google.com">Google</a>
    `,
      output: `
<a href="https://google.com" target="_blank" rel="noopener">Google</a>
<a href="https://google.com" target="_blank" rel="noopener">Google</a>
    `,
      path: '/path/to/project/docs/baz.md',
      processor: processor.use(rehypeReactRouter, { routes: {} }),
    });
  });

  test('link with Markdown', async () => {
    await expectToStringifyInto({
      source: `
[**Google**](https://google.com)
    `,
      output: `
<a href="https://google.com" target="_blank" rel="noopener">**Google**</a>
    `,
      path: '/path/to/project/docs/baz.md',
      processor: processor.use(rehypeReactRouter, { routes: {} }),
    });
  });

  test('same-page hashes are not transformed', async () => {
    await expectToStringifyInto({
      source: `
[Heading 1](#heading-1)
    `,
      output: `
[Heading 1](#heading-1)
    `,
      path: '/path/to/project/docs/baz/index.md',
      processor: processor.use(rehypeReactRouter, { routes: {} }),
    });
  });

  test('hashes on cross-page links are preserved', async () => {
    await expectToStringifyInto({
      source: `
[link](foo/bar.md#baz)
<a href="foo/bar.md#baz">link</a>
    `,
      output: `
<Link to="/docs/foo/bar#baz">link</Link>
<Link to="/docs/foo/bar#baz">link</Link>
    `,
      path: '/path/to/project/docs/baz.md',
      processor: processor.use(rehypeReactRouter, {
        routes: {
          bar: {
            id: '1',
            path: '/docs/foo/bar',
            absPath: '/docs/foo/bar',
            file: '/path/to/project/docs/foo/bar.md',
          },
        },
      }),
    });
  });

  test('additional extensions are preserved', async () => {
    await expectToStringifyInto({
      source: `
[link](foo.bar.baz.md#baz)
    `,
      output: `
<Link to="/docs/foo.bar.baz#baz">link</Link>
    `,
      path: '/path/to/project/docs/foo.bar.md',
      processor: processor.use(rehypeReactRouter, {
        routes: {
          bar: {
            id: '1',
            path: '/docs/foo.bar.baz',
            absPath: '/docs/foo.bar.baz',
            file: '/path/to/project/docs/foo.bar.baz.md',
          },
        },
      }),
    });
  });
});
