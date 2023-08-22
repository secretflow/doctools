import rehypeParse from 'rehype-parse';
import remarkMdx from 'remark-mdx';
import remarkStringify from 'remark-stringify';
import { unified } from 'unified';
import { describe, test } from 'vitest';

import { expectToStringifyInto } from '../testing/index.js';

import { toJSX } from './index.js';

import 'mdast-util-mdx';

describe('hast-util-to-jsx', () => {
  const processor = unified()
    .use(rehypeParse, { fragment: true })
    .use(() => toJSX)
    .use(remarkMdx)
    .use(remarkStringify);

  test('simple element', async () => {
    await expectToStringifyInto({
      source: '<div />',
      output: '<div />',
      processor,
    });
  });

  test('element with attributes', async () => {
    await expectToStringifyInto({
      source: '<div id="foo" class="bar baz">',
      output: '<div id="foo" className="bar baz" />',
      processor,
    });
  });

  test('element with text content', async () => {
    await expectToStringifyInto({
      source: '<div>foo</div>',
      output: '<div>{"foo"}</div>',
      processor,
    });
  });

  test('multiline text content', async () => {
    await expectToStringifyInto({
      source: `
<pre>
  import React from 'react';
</pre>
      `,
      output: `
<pre>
  {"  import React from 'react';\\n"}
</pre>
      `,
      processor,
    });
  });

  test('preserve data attributes', async () => {
    await expectToStringifyInto({
      source: '<div data-foo="bar">',
      output: '<div data-foo="bar" />',
      processor,
    });
  });

  test('preserve aria attributes', async () => {
    await expectToStringifyInto({
      source: '<div aria-label="foo">',
      output: '<div aria-label="foo" />',
      processor,
    });
  });

  test('transform styles using CSS-in-JS', async () => {
    await expectToStringifyInto({
      source: '<div style="color: red; display: flex;">',
      output: '<div style={{"color":"red","display":"flex"}} />',
      processor,
    });
  });

  test('ignoring doctypes', async () => {
    await expectToStringifyInto({
      source: '<!DOCTYPE html>',
      output: '',
      processor,
    });
  });

  test('example.org', async () => {
    await expectToStringifyInto({
      source: `
<div>
  <h1>Example Domain</h1>
  <p>This domain is for use in illustrative examples in documents. You may use this
  domain in literature without prior coordination or asking for permission.</p>
  <p><a href="https://www.iana.org/domains/example">More information...</a></p>
</div>
      `,
      output: `
<div>
  <h1>{"Example Domain"}</h1>

  <p>
    {"This domain is for use in illustrative examples in documents. You may use this\\n  domain in literature without prior coordination or asking for permission."}
  </p>

  <p><a href="https://www.iana.org/domains/example">{"More information..."}</a></p>
</div>
      `,
      processor,
    });
  });

  test('remove script tags', async () => {
    await expectToStringifyInto({
      source: '<script>alert("foo")</script>',
      output: '',
      processor,
    });
  });
});
