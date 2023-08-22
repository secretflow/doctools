import { expectToStringifyInto } from '@secretflow/unified-toolkit/testing';
import rehypeRaw from 'rehype-raw';
import rehypeStringify from 'rehype-stringify';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import type { Processor } from 'unified';
import { unified } from 'unified';
import { describe, beforeAll, test } from 'vitest';

import { rehypeAddReferrerpolicy } from './index.mjs';

describe('rehype-add-referrerpolicy', () => {
  let processor: Processor;

  beforeAll(() => {
    processor = unified()
      .use(remarkParse)
      .use(remarkRehype, {
        allowDangerousHtml: true,
      })
      .use(rehypeRaw)
      .use(rehypeAddReferrerpolicy)
      .use(rehypeStringify);
  });

  test('add referrerpolicy', async () => {
    await expectToStringifyInto({
      source: '<img src="img.png" />',
      output: '<img src="img.png" referrerpolicy="no-referrer">',
      processor,
    });
  });

  test('add existed referrerpolicy', async () => {
    await expectToStringifyInto({
      source: '<img src="img.png" referrerpolicy="no-referrer">',
      output: '<img src="img.png" referrerpolicy="no-referrer">',
      processor,
    });
  });
});
