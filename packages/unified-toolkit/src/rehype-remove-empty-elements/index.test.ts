import rehypeRaw from 'rehype-raw';
import rehypeStringify from 'rehype-stringify';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import type { Processor } from 'unified';
import { unified } from 'unified';
import { describe, beforeAll, test } from 'vitest';

import { expectToStringifyInto } from '../testing/index.js';

import { rehypeRemoveEmptyElements } from './index.js';

describe('rehype-remove-redundant-nodes', () => {
  let processor: Processor;

  beforeAll(() => {
    processor = unified()
      .use(remarkParse)
      .use(remarkRehype, {
        allowDangerousHtml: true,
      })
      .use(rehypeRaw)
      .use(rehypeRemoveEmptyElements)
      .use(rehypeStringify);
  });

  test('remove empty paragraph', async () => {
    await expectToStringifyInto({
      source: `
<p>
  <br />
</p>
      `,
      output: '',
      processor,
    });
  });

  test('remove empty paragraph 2', async () => {
    await expectToStringifyInto({
      source: '<p><br /><br /><br /><br /></p>',
      output: '',
      processor,
    });
  });

  test("don't remove non-empty paragraph", async () => {
    await expectToStringifyInto({
      source: `
<p>
  Lorem ipsum
  <br />
  dolor sit amet
</p>
      `,
      output: `
<p>
  Lorem ipsum
  <br>
  dolor sit amet
</p>
      `,
      processor,
    });
  });
});
