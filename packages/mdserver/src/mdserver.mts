import { toJSX } from '@secretflow/unified-toolkit/hast-util-to-jsx';
import express from 'express';
import type * as mdast from 'mdast';
import rehypeParse from 'rehype-parse';
import remarkDirective from 'remark-directive';
import remarkFrontmatter from 'remark-frontmatter';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import type { Options } from 'remark-stringify';
import remarkStringify from 'remark-stringify';
import { unified } from 'unified';
import type * as unist from 'unist';

import { remarkFixBlockNodes } from './remark-fix-block-nodes/index.mjs';
import { remarkFixPhrasingNodes } from './remark-fix-phrasing-nodes/index.mjs';
import { remarkMergePhrasingNodes } from './remark-merge-phrasing-nodes/index.mjs';

process.on('SIGINT', () => process.exit(0));
process.on('SIGTERM', () => process.exit(0));

const STRINGIFY_OPTIONS: Options = {
  emphasis: '_',
  resourceLink: true,
  rule: '-',
  bullet: '-',
  bulletOther: '+',
  bulletOrdered: '.',
  bulletOrderedOther: ')',
  fences: true,
  listItemIndent: 'one',
  join: [
    // remove extraneous new lines between JSX block elements
    (left: unist.Node, right: unist.Node, parent: unist.Parent): number | undefined => {
      if (parent.type === 'root') {
        return 1;
      }
      if (
        left.type === 'list' &&
        right.type === 'list' &&
        !(left as mdast.List).spread &&
        !(right as mdast.List).spread
      ) {
        return 0;
      }
      const types = ['mdxJsxFlowElement', 'mdxJsxTextElement'];
      if (types.includes(left.type) && types.includes(right.type)) {
        return 0;
      }
      return undefined;
    },
  ],
};

const markdownProcessor = unified()
  .use(remarkParse)
  .use(remarkGfm)
  .use(remarkFrontmatter)
  .use(remarkDirective)
  .use(remarkMath)
  .use(remarkMdx)
  .use(remarkFixBlockNodes)
  .use(remarkFixPhrasingNodes)
  .use(remarkMergePhrasingNodes)
  .use(remarkStringify, STRINGIFY_OPTIONS)
  .freeze();

const htmlProcessor = unified()
  .use(rehypeParse, { fragment: true })
  .use(() => toJSX)
  .use(remarkStringify, STRINGIFY_OPTIONS)
  .freeze();

const app = express();

app.post('/stringify/markdown', express.json({ limit: '50mb' }), async (req, res) => {
  try {
    const transformed = await markdownProcessor.run(req.body);
    const text = markdownProcessor.stringify(transformed);
    res.send(text);
  } catch (e) {
    res.status(400).send(String(e));
  }
});

app.post('/parse/markdown', express.text({ limit: '50mb' }), async (req, res) => {
  try {
    const tree = markdownProcessor.parse(req.body);
    const transformed = await markdownProcessor.run(tree);
    res.send(transformed);
  } catch (e) {
    res.status(400).send(String(e));
  }
});

app.post('/parse/html', express.text({ limit: '50mb' }), async (req, res) => {
  try {
    const tree = htmlProcessor.parse(req.body);
    const transformed = await htmlProcessor.run(tree);
    res.send(transformed);
  } catch (e) {
    res.status(400).send(String(e));
  }
});

const port = process.argv[2] || 3000;

app.listen(port, () => {
  console.log(`Listening on ${port}`);
});
