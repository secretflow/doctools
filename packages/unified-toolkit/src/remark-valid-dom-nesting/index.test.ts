import remarkGfm from 'remark-gfm';
import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import remarkStringify from 'remark-stringify';
import { unified } from 'unified';
import { describe, test } from 'vitest';

import { expectToStringifyInto } from '../testing/index.js';

import { remarkValidDOMNesting } from './index.js';

describe('remark-valid-dom-nesting', () => {
  const processor = unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkMdx)
    .use(remarkValidDOMNesting)
    .use(remarkStringify);

  test('a inside a', async () => {
    await expectToStringifyInto({
      source: '<a href="https://bing.com">https://google.com</a>',
      output: '<a href="https://bing.com">https\\://google.com</a>',
      processor,
    });
  });
});
