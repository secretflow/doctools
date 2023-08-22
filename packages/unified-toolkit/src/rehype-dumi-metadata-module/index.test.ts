import remarkExtractFrontmatter from 'remark-extract-frontmatter';
import remarkFrontmatter from 'remark-frontmatter';
import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import type { Processor } from 'unified';
import { unified } from 'unified';
import { describe, beforeAll, test } from 'vitest';
import YAML from 'yaml';

import { rehypeArticleOutline } from '../rehype-article-outline/index.js';
import { expectToStringifyInto } from '../testing/index.js';
import {
  recmaStringify,
  rehypeRecma,
  MDX_NODE_TYPES,
  remarkMarkAndUnravel,
} from '../utils/index.js';

import { rehypeDumiMetadataModule } from './index.js';

describe('remark-dumi-emit-metadata', () => {
  let processor: Processor;

  beforeAll(() => {
    processor = unified()
      .use(remarkParse)
      .use(remarkFrontmatter)
      .use(remarkExtractFrontmatter, { yaml: YAML.parse, name: 'frontmatter' })
      .use(remarkMarkAndUnravel)
      .use(remarkMdx)
      .use(remarkRehype, {
        // simulate @mdx-js/mdx
        allowDangerousHtml: true,
        passThrough: MDX_NODE_TYPES,
      })
      .use(rehypeArticleOutline)
      .use(rehypeDumiMetadataModule)
      .use(rehypeRecma, {
        elementAttributeNameCase: 'react',
        stylePropertyNameCase: 'dom',
      })
      .use(recmaStringify);
  });

  test('extract title from first heading', async () => {
    await expectToStringifyInto({
      source: `
# In Congress, July 4, 1776
  `,
      output: `
export const frontmatter = {
  "title": "In Congress, July 4, 1776",
  "toc": "content",
  "filename": "docs/index.md"
};
export const toc = [{
  "id": "in-congress-july-4-1776",
  "title": "In Congress, July 4, 1776",
  "depth": 1,
  "order": 0
}];
export const demos = {};
export const texts = [];
<></>;
      `,
      processor,
      cwd: '/path/to/project',
      path: '/path/to/project/docs/index.md',
    });
  });

  test('extract title from first <h1> (inline)', async () => {
    await expectToStringifyInto({
      source: `
<h1>In Congress, July 4, 1776</h1>
## We hold these truths to be self-evident
  `,
      output: `
export const frontmatter = {
  "title": "In Congress, July 4, 1776",
  "toc": "content",
  "filename": "docs/index.md"
};
export const toc = [{
  "id": "in-congress-july-4-1776",
  "title": "In Congress, July 4, 1776",
  "depth": 1,
  "order": 0
}, {
  "id": "we-hold-these-truths-to-be-self-evident",
  "title": "We hold these truths to be self-evident",
  "depth": 2,
  "order": 1
}];
export const demos = {};
export const texts = [];
<></>;
        `,
      processor,
      cwd: '/path/to/project',
      path: '/path/to/project/docs/index.md',
    });
  });
  test('extract ToC from heading elements and preserve ids', async () => {
    await expectToStringifyInto({
      source: `
<h1 id="title">In Congress, July 4, 1776</h1>
## We hold these truths to be self-evident
  `,
      output: `
export const frontmatter = {
  "title": "In Congress, July 4, 1776",
  "toc": "content",
  "filename": "docs/index.md"
};
export const toc = [{
  "id": "title",
  "title": "In Congress, July 4, 1776",
  "depth": 1,
  "order": 0
}, {
  "id": "we-hold-these-truths-to-be-self-evident",
  "title": "We hold these truths to be self-evident",
  "depth": 2,
  "order": 1
}];
export const demos = {};
export const texts = [];
<></>;
        `,
      processor,
      cwd: '/path/to/project',
      path: '/path/to/project/docs/index.md',
    });
  });

  test('extract title from first <h1> (block)', async () => {
    await expectToStringifyInto({
      source: `
<h1>In Congress, July 4, 1776</h1>
## We hold these truths to be self-evident
  `,
      output: `
export const frontmatter = {
  "title": "In Congress, July 4, 1776",
  "toc": "content",
  "filename": "docs/index.md"
};
export const toc = [{
  "id": "in-congress-july-4-1776",
  "title": "In Congress, July 4, 1776",
  "depth": 1,
  "order": 0
}, {
  "id": "we-hold-these-truths-to-be-self-evident",
  "title": "We hold these truths to be self-evident",
  "depth": 2,
  "order": 1
}];
export const demos = {};
export const texts = [];
<></>;
        `,
      processor,
      cwd: '/path/to/project',
      path: '/path/to/project/docs/index.md',
    });
  });

  test('extract description from first paragraph', async () => {
    await expectToStringifyInto({
      source: `
This is a description.
        `,
      output: `
export const frontmatter = {
  "title": "Docs",
  "toc": "content",
  "filename": "docs/index.md",
  "description": "This is a description."
};
export const toc = [];
export const demos = {};
export const texts = [];
<></>;
        `,
      processor,
      cwd: '/path/to/project',
      path: '/path/to/project/docs/index.md',
    });
  });

  test('extract description from first <p> (inline)', async () => {
    await expectToStringifyInto({
      source: `
<p>This is a description.</p>

This is another description.
      `,
      output: `
export const frontmatter = {
  "title": "Docs",
  "toc": "content",
  "filename": "docs/index.md",
  "description": "This is a description."
};
export const toc = [];
export const demos = {};
export const texts = [];
<></>;
      `,
      processor,
      cwd: '/path/to/project',
      path: '/path/to/project/docs/index.md',
    });
  });

  test('extract description from first <p> (block)', async () => {
    await expectToStringifyInto({
      source: `
<p>
  This is a description.
</p>
This is another description.
      `,
      output: `
export const frontmatter = {
  "title": "Docs",
  "toc": "content",
  "filename": "docs/index.md",
  "description": "This is a description."
};
export const toc = [];
export const demos = {};
export const texts = [];
<></>;
      `,
      processor,
      cwd: '/path/to/project',
      path: '/path/to/project/docs/index.md',
    });
  });

  test('do not overwrite existing description', async () => {
    await expectToStringifyInto({
      source: `
---
description: The following text is a transcription of the Stone Engraving of the parchment Declaration of Independence.
---
# In Congress, July 4, 1776
The unanimous Declaration of the thirteen united States of America,When in the Course of
human events, it becomes necessary for one people to dissolve the political bands which
have connected them with another, and to assume among the powers of the earth, the
separate and equal station to which the Laws of Nature and of Nature's God entitle
them, a decent respect to the opinions of mankind requires that they should declare
the causes which impel them to the separation.
      `,
      output: `
export const frontmatter = {
  "title": "In Congress, July 4, 1776",
  "toc": "content",
  "filename": "docs/index.md",
  "description": "The following text is a transcription of the Stone Engraving of the parchment Declaration of Independence."
};
export const toc = [{
  "id": "in-congress-july-4-1776",
  "title": "In Congress, July 4, 1776",
  "depth": 1,
  "order": 0
}];
export const demos = {};
export const texts = [];
<></>;
      `,
      processor,
      cwd: '/path/to/project',
      path: '/path/to/project/docs/index.md',
    });
  });

  test('do not overwrite existing title', async () => {
    await expectToStringifyInto({
      source: `
---
title: Declaration of Independence
---
# In Congress, July 4, 1776
      `,
      output: `
export const frontmatter = {
  "title": "Declaration of Independence",
  "toc": "content",
  "filename": "docs/index.md"
};
export const toc = [{
  "id": "in-congress-july-4-1776",
  "title": "In Congress, July 4, 1776",
  "depth": 1,
  "order": 0
}];
export const demos = {};
export const texts = [];
<></>;
      `,
      processor,
      cwd: '/path/to/project',
      path: '/path/to/project/docs/index.md',
    });
  });

  test('generate titles from filename', async () => {
    await expectToStringifyInto({
      source: `
In Congress, July 4, 1776
          `,
      output: `
export const frontmatter = {
  "title": "Declaration of Independence",
  "toc": "content",
  "filename": "docs/declaration-of-independence.md",
  "description": "In Congress, July 4, 1776"
};
export const toc = [];
export const demos = {};
export const texts = [];
<></>;
      `,
      processor,
      cwd: '/path/to/project',
      path: '/path/to/project/docs/declaration-of-independence.md',
    });
  });
});
