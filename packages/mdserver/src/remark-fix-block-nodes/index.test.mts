import type { Root } from 'mdast';
import remarkMdx from 'remark-mdx';
import remarkStringify from 'remark-stringify';
import { unified } from 'unified';
import { u } from 'unist-builder';
import { describe, test, expect } from 'vitest';

import { remarkFixBlockNodes } from './index.mjs';

describe('remark-implicit-flow', () => {
  const processor = unified()
    .use(remarkMdx)
    .use(remarkFixBlockNodes)
    .use(remarkStringify);

  test('simple', async () => {
    const tree = u('root', [u('text', 'Hello, '), u('text', 'world!')]);
    const result = await processor.run(tree);
    expect(result).toEqual(
      u('root', [u('paragraph', [u('text', 'Hello, '), u('text', 'world!')])]),
    );
    expect(processor.stringify(result).trim()).toBe('Hello, world!');
  });

  test('JSX', async () => {
    // @ts-expect-error deliberately malformed
    const tree: Root = u('root', [
      u(
        'mdxJsxFlowElement',
        {
          name: 'p',
          attributes: [],
        },
        [
          u(
            'mdxJsxTextElement',
            {
              name: 'strong',
              attributes: [],
            },
            [u('text', 'Lorem')],
          ),
          u(
            'mdxJsxTextElement',
            {
              name: 'span',
              attributes: [],
            },
            [u('mdxTextExpression', '" "')],
          ),
          u(
            'mdxJsxTextElement',
            {
              name: 'em',
              attributes: [],
            },
            [u('text', 'ipsum')],
          ),
        ],
      ),
    ]);
    const result = await processor.run(tree);
    const expectedTree: Root = u('root', [
      u(
        'mdxJsxFlowElement',
        {
          name: 'p',
          attributes: [],
        },
        [
          u('paragraph', [
            u(
              'mdxJsxTextElement',
              {
                name: 'strong',
                attributes: [],
              },
              [u('text', 'Lorem')],
            ),
            u(
              'mdxJsxTextElement',
              {
                name: 'span',
                attributes: [],
              },
              [u('mdxTextExpression', '" "')],
            ),
            u(
              'mdxJsxTextElement',
              {
                name: 'em',
                attributes: [],
              },
              [u('text', 'ipsum')],
            ),
          ]),
        ],
      ),
    ]);
    const expectedText = `
<p>
  <strong>Lorem</strong><span>{" "}</span><em>ipsum</em>
</p>
    `.trim();
    expect(result).toEqual(expectedTree);
    expect(processor.stringify(result).trim()).toBe(expectedText);
  });

  test('JSX 2', async () => {
    // @ts-expect-error deliberately malformed
    const tree: Root = u('root', [
      u(
        'mdxJsxFlowElement',
        {
          name: 'LineBlock',
          attributes: [],
        },
        [
          u('paragraph', [u('text', 'This is a second line block.')]),
          u(
            'mdxJsxTextElement',
            {
              name: 'br',
              attributes: [],
            },
            [],
          ),
          u('paragraph', [
            u(
              'text',
              'Blank lines are permitted internally, but they must begin with a "|".',
            ),
          ]),
        ],
      ),
    ]);
    const result = await processor.run(tree);
    const expectedTree: Root = u('root', [
      u(
        'mdxJsxFlowElement',
        {
          name: 'LineBlock',
          attributes: [],
        },
        [
          u('paragraph', [u('text', 'This is a second line block.')]),
          u('paragraph', [
            u(
              'mdxJsxTextElement',
              {
                name: 'br',
                attributes: [],
              },
              [],
            ),
          ]),
          u('paragraph', [
            u(
              'text',
              'Blank lines are permitted internally, but they must begin with a "|".',
            ),
          ]),
        ],
      ),
    ]);
    const expectedText = `
<LineBlock>
  This is a second line block.

  <br />

  Blank lines are permitted internally, but they must begin with a "|".
</LineBlock>
    `.trim();
    expect(result).toEqual(expectedTree);
    expect(processor.stringify(result).trim()).toBe(expectedText);
  });
});
