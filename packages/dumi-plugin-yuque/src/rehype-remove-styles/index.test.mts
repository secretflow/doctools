import { expectToStringifyInto } from '@secretflow/unified-toolkit/testing';
import rehypeRaw from 'rehype-raw';
import rehypeStringify from 'rehype-stringify';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import type { Processor } from 'unified';
import { unified } from 'unified';
import { describe, beforeAll, test } from 'vitest';

import { rehypeRemoveStyle } from './index.mjs';

describe('rehype-remove-style', () => {
  let processor: Processor;

  beforeAll(() => {
    processor = unified()
      .use(remarkParse)
      .use(remarkRehype, {
        allowDangerousHtml: true,
      })
      .use(rehypeRaw)
      .use(rehypeRemoveStyle, {
        properties: ['fontSize', { td: ['width', 'dataType'] }],
      })
      .use(rehypeStringify);
  });

  test('remove fontSize property', async () => {
    await expectToStringifyInto({
      source: `
<td style="font-size: 20px;"></td>
        `,
      output: `
<td></td>
        `,
      processor,
    });
  });

  test('remove width property', async () => {
    await expectToStringifyInto({
      source: `
<td width="100" height="50"></td>
      `,
      output: `
<td height="50"></td>
      `,
      processor,
    });
  });

  test('remove property', async () => {
    await expectToStringifyInto({
      source: `
<td data-type="example" height="50"></td>
      `,
      output: `
<td height="50"></td>
      `,
      processor,
    });
  });

  test('remove td width style', async () => {
    await expectToStringifyInto({
      source: `
<td style="width: 100px; height: 50px;"></td>
        `,
      output: `
<td style="height: 50px"></td>
        `,
      processor,
    });
  });
});
