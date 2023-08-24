import { expectToStringifyInto } from '@secretflow/unified-toolkit/testing';
import rehypeRaw from 'rehype-raw';
import rehypeStringify from 'rehype-stringify';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import type { Processor } from 'unified';
import { unified } from 'unified';
import { describe, beforeAll, test } from 'vitest';

import { rehypeRemoveDumiWrapper } from './index.mjs';

describe('rehype-remove-yuque-wrapper', () => {
  let processor: Processor;

  beforeAll(() => {
    processor = unified()
      .use(remarkParse)
      .use(remarkRehype, {
        allowDangerousHtml: true,
      })
      .use(rehypeRaw)
      .use(rehypeRemoveDumiWrapper)
      .use(rehypeStringify);
  });

  test('remove yuque wrapper', async () => {
    await expectToStringifyInto({
      source: `
<div typography="classic">
  <h3>title</h3>
  <article>article</article>
</div>
      `,
      output: `
<h3>title</h3>
  <article>article</article>
      `,
      processor,
    });
  });
});
