import remarkFrontmatter from 'remark-frontmatter';
import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import remarkStringify from 'remark-stringify';
import type { Processor } from 'unified';
import { unified } from 'unified';
import { describe, beforeAll, test } from 'vitest';

import { expectToStringifyInto } from '../testing/index.js';

import { remarkDumiContentModule } from './index.js';

describe('remark-dumi-emit-page', () => {
  let processor: Processor;

  beforeAll(() => {
    processor = unified()
      .use(remarkParse)
      .use(remarkStringify)
      .use(remarkMdx)
      .use(remarkFrontmatter)
      .use(remarkDumiContentModule, {
        builtins: {
          SourceCode: {
            specifier: 'SourceCode',
            source: '@/components/SourceCode',
          },
        },
      });
  });

  test('pure markdown', async () => {
    await expectToStringifyInto({
      source: `
# Hello, world!

This is a paragraph.
      `,
      output: `
import SourceCode from "@/components/SourceCode";

import { DumiPage } from 'dumi';

<DumiPage>
  # Hello, world!

  This is a paragraph.
</DumiPage>
      `,
      processor,
    });
  });

  test('markdown with ESM import/export', async () => {
    await expectToStringifyInto({
      source: `
import { Foo } from 'bar';

export default function MyPage() {
  return <Foo />;
}

This is a paragraph.

<MyPage />
      `,
      output: `
import SourceCode from "@/components/SourceCode";

import { Foo } from 'bar';

export default function MyPage() {
  return <Foo />;
}

import { DumiPage } from 'dumi';

<DumiPage>
  This is a paragraph.

  <MyPage />
</DumiPage>
      `,
      processor,
    });
  });

  test('frontmatter hoisting', async () => {
    await expectToStringifyInto({
      source: `
---
title: Hello, world!
---

# Hello, universe!
      `,
      output: `
---
title: Hello, world!
---

import SourceCode from "@/components/SourceCode";

import { DumiPage } from 'dumi';

<DumiPage>
  # Hello, universe!
</DumiPage>
        `,
      processor,
    });
  });
});
