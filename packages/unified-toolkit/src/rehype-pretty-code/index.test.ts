import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import { unified } from 'unified';
import { describe, test } from 'vitest';

import { expectToStringifyInto } from '../testing/index.js';
import { recmaStringify, rehypeRecma } from '../utils/index.js';

import { rehypePrettyCode } from './index.js';

describe('remark-dumi-source-code', () => {
  test('convert', async () => {
    const processor = unified()
      .use(remarkParse)
      .use(remarkMdx)
      .use(remarkRehype)
      .use(rehypePrettyCode)
      .use(rehypeRecma)
      .use(recmaStringify);

    await expectToStringifyInto({
      source: `
\`\`\`js
import type { Code, Parent } from 'mdast';

export const answer = 42;
\`\`\`
      `,
      output: `<><SourceCode lang="js">{"import type { Code, Parent } from 'mdast';\\n\\nexport const answer = 42;\\n"}</SourceCode></>;`,
      processor,
    });
  });
});
