import type { Root } from 'mdast';
import remarkMdx from 'remark-mdx';
import remarkStringify from 'remark-stringify';
import { unified } from 'unified';
import { u } from 'unist-builder';
import { describe, test, expect } from 'vitest';

import { remarkFixPhrasingNodes } from './index.mjs';

describe('remark-strict-phrasing', () => {
  const processor = unified()
    .use(remarkMdx)
    .use(remarkFixPhrasingNodes)
    .use(remarkStringify);

  test('simple', async () => {
    const tree: Root = {
      type: 'root',
      children: [
        {
          type: 'paragraph',
          children: [
            { type: 'text', value: 'Hello, ' },
            // @ts-expect-error deliberately malformed
            { type: 'blockquote', children: [{ type: 'text', value: 'world!' }] },
          ],
        },
      ],
    };
    const result = await processor.run(tree);
    expect(result).toEqual({
      type: 'root',
      children: [
        {
          type: 'paragraph',
          children: [{ type: 'text', value: 'Hello, ' }],
        },
        { type: 'blockquote', children: [{ type: 'text', value: 'world!' }] },
      ],
    });
    expect(processor.stringify(result).trim()).toBe(
      `
Hello,&#x20;

> world!
    `.trim(),
    );
  });

  test('deeply nested', async () => {
    // @ts-expect-error deliberately malformed
    const tree: Root = u('root', [
      u('paragraph', [
        u('text', 'Hello, '),
        u('strong', [
          u('strong', [
            u('emphasis', [
              u('mdxJsxFlowElement', { name: 'p', attributes: [] }, [
                u('inlineCode', 'world!'),
              ]),
            ]),
          ]),
        ]),
      ]),
    ]);
    const result = await processor.run(tree);
    expect(result).toEqual({
      type: 'root',
      children: [
        {
          type: 'paragraph',
          children: [
            { type: 'text', value: 'Hello, ' },
            {
              type: 'strong',
              children: [
                {
                  type: 'strong',
                  children: [
                    {
                      type: 'emphasis',
                      children: [],
                    },
                  ],
                },
              ],
            },
          ],
        },
        {
          type: 'mdxJsxFlowElement',
          name: 'p',
          attributes: [],
          children: [
            {
              type: 'inlineCode',
              value: 'world!',
            },
          ],
        },
      ],
    });
    expect(processor.stringify(result).trim()).toBe(
      `
Hello, **********

<p>
  \`world!\`
</p>
    `.trim(),
    );
  });
});
