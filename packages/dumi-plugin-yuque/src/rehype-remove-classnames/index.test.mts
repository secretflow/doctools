import { expectToStringifyInto } from '@secretflow/unified-toolkit/testing';
import rehypeRaw from 'rehype-raw';
import rehypeStringify from 'rehype-stringify';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import type { Processor } from 'unified';
import { unified } from 'unified';
import { describe, beforeAll, test } from 'vitest';

import { rehypeRemoveClassNames } from './index.mjs';

describe('rehype-remove-classnames', () => {
  let processor: Processor;

  beforeAll(() => {
    processor = unified()
      .use(remarkParse)
      .use(remarkRehype, {
        allowDangerousHtml: true,
      })
      .use(rehypeRaw)
      .use(rehypeRemoveClassNames)
      .use(rehypeStringify);
  });

  test('remove classnames', async () => {
    await expectToStringifyInto({
      source: '<div data-answer="42" class="foo bar"></div>',
      output: '<div data-answer="42"></div>',
      processor,
    });
  });
});
