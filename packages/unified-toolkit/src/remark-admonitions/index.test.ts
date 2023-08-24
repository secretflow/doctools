import remarkDirective from 'remark-directive';
import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import remarkStringify from 'remark-stringify';
import { unified } from 'unified';
import { describe, test } from 'vitest';

import { expectToStringifyInto } from '../testing/index.js';

import { remarkAdmonitions } from './index.js';

describe('remark-dumi-admonitions', () => {
  test('convert', async () => {
    const processor = unified()
      .use(remarkParse)
      .use(remarkStringify)
      .use(remarkMdx)
      .use(remarkDirective)
      .use(remarkAdmonitions);

    await expectToStringifyInto({
      source: `
:::info{title="Info"}
This is an info message.
:::
      `,
      output: `
<Container type="info" title="Info">
  This is an info message.
</Container>
      `,
      processor,
    });
  });
});
