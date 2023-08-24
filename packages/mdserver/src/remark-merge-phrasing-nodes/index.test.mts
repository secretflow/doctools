import type { Root } from 'mdast';
import remarkMdx from 'remark-mdx';
import remarkStringify from 'remark-stringify';
import { unified } from 'unified';
import { describe, test, expect } from 'vitest';

import { remarkMergePhrasingNodes } from './index.mjs';

describe('remark-merge-inline-elements', () => {
  const processor = unified()
    .use(remarkMdx)
    .use(remarkMergePhrasingNodes)
    .use(remarkStringify, { emphasis: '_' });

  test('simple', async () => {
    const tree: Root = {
      type: 'root',
      children: [
        {
          type: 'paragraph',
          children: [
            { type: 'emphasis', children: [{ type: 'text', value: 'Hello, ' }] },
            { type: 'emphasis', children: [{ type: 'text', value: 'world!' }] },
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
          children: [
            {
              type: 'emphasis',
              children: [
                { type: 'text', value: 'Hello, ' },
                { type: 'text', value: 'world!' },
              ],
            },
          ],
        },
      ],
    });
    expect(processor.stringify(result).trim()).toBe('_Hello, world!_');
  });

  test('nested', async () => {
    const tree: Root = {
      type: 'root',
      children: [
        {
          type: 'paragraph',
          children: [
            {
              type: 'emphasis',
              children: [
                { type: 'strong', children: [{ type: 'text', value: 'Lorem ' }] },
                { type: 'strong', children: [{ type: 'text', value: 'ipsum' }] },
              ],
            },
            { type: 'emphasis', children: [{ type: 'text', value: ' dolor ' }] },
            { type: 'emphasis', children: [{ type: 'text', value: 'sit amet' }] },
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
          children: [
            {
              type: 'emphasis',
              children: [
                {
                  type: 'strong',
                  children: [
                    { type: 'text', value: 'Lorem ' },
                    { type: 'text', value: 'ipsum' },
                  ],
                },
                { type: 'text', value: ' dolor ' },
                { type: 'text', value: 'sit amet' },
              ],
            },
          ],
        },
      ],
    });
    expect(processor.stringify(result).trim()).toBe('_**Lorem ipsum** dolor sit amet_');
  });

  test('python type signature', async () => {
    const tree: Root = {
      type: 'root',
      children: [
        {
          type: 'mdxJsxFlowElement',
          name: 'dd',
          attributes: [],
          children: [
            {
              type: 'paragraph',
              children: [
                { type: 'strong', children: [{ type: 'text', value: 'kind' }] },
                { type: 'text', value: ' (' },
                {
                  type: 'link',
                  url: 'https://docs.python.org/3/library/stdtypes.html#list',
                  children: [
                    { type: 'emphasis', children: [{ type: 'text', value: 'list' }] },
                  ],
                  title: '(in Python v3.11)',
                },
                { type: 'emphasis', children: [{ type: 'text', value: '[' }] },
                {
                  type: 'link',
                  url: 'https://docs.python.org/3/library/stdtypes.html#str',
                  children: [
                    { type: 'emphasis', children: [{ type: 'text', value: 'str' }] },
                  ],
                  title: '(in Python v3.11)',
                },
                { type: 'emphasis', children: [{ type: 'text', value: '] or ' }] },
                { type: 'emphasis', children: [{ type: 'text', value: 'None' }] },
                { type: 'text', value: ')' },
                { type: 'text', value: ' -- ' },
                { type: 'text', value: 'Optional "kind" of ingredients.' },
              ],
            },
          ],
        },
      ],
    };
    const result = await processor.run(tree);
    expect(processor.stringify(result).trim()).toBe(
      `
<dd>
  **kind** ([_list_](https://docs.python.org/3/library/stdtypes.html#list "(in Python v3.11)")_\\[_[_str_](https://docs.python.org/3/library/stdtypes.html#str "(in Python v3.11)")_] or None_) -- Optional "kind" of ingredients.
</dd>
      `.trim(),
    );
  });
});
