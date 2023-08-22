import rehypeStringify from 'rehype-stringify';
import remarkDirective from 'remark-directive';
import remarkGfm from 'remark-gfm';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import { unified } from 'unified';
import { describe, test } from 'vitest';

import { expectToStringifyInto } from '../testing/index.js';

import { remarkAttrs } from './index.js';

describe('rehype-inline-id', () => {
  const processor = unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkDirective)
    .use(remarkAttrs)
    .use(remarkRehype)
    .use(rehypeStringify);

  test('title', async () => {
    await expectToStringifyInto({
      source: `
:target{#title}

# Title
      `,
      output: `
<p></p>
<h1 id="title">Title</h1>
      `,
      processor,
    });
  });

  test('footnote', async () => {
    await expectToStringifyInto({
      source: `
Lorem ipsum :target{#footnote}[^1]

[^1]: Dolor sit amet.
      `,
      output: `
<p>Lorem ipsum <sup id="footnote"><a href="#user-content-fn-1" id="user-content-fnref-1" data-footnote-ref aria-describedby="footnote-label">1</a></sup></p>
<section data-footnotes class="footnotes"><h2 class="sr-only" id="footnote-label">Footnotes</h2>
<ol>
<li id="user-content-fn-1">
<p>Dolor sit amet. <a href="#user-content-fnref-1" data-footnote-backref class="data-footnote-backref" aria-label="Back to content">↩</a></p>
</li>
</ol>
</section>
      `,
      processor,
    });
  });
});
