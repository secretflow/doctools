import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import remarkStringify from 'remark-stringify';
import type { Processor } from 'unified';
import { unified } from 'unified';
import { describe, beforeAll, test } from 'vitest';

import { expectToStringifyInto } from '../testing/index.js';

import { rehypeAssetModules } from './index.js';

describe('remark-mdx-asset-loader', () => {
  let processor: Processor;

  beforeAll(async () => {
    processor = unified()
      .use(remarkParse)
      .use(remarkMdx)
      .use(rehypeAssetModules, { test: /\.(png|svg|webm|mp4)$/ })
      .use(remarkStringify);
  });

  test('simple image with relative path', async () => {
    await expectToStringifyInto({
      source: `
![image](./image.png)
![](./image.png)
    `,
      output: `
import asset0 from "./image.png";

import asset1 from "./image.png";

<img src={asset0} alt="image" />
<img src={asset1} alt="" />
    `,
      processor,
    });
  });

  test('ignore unknown asset types', async () => {
    await expectToStringifyInto({
      source: `
![font](./font.woff2)
<iframe src="./homepage.html" />
      `,
      output: `
![font](./font.woff2)

<iframe src="./homepage.html" />
      `,
      processor,
    });
  });

  test('ignore data URIs', async () => {
    const image =
      '![red](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAwAAAAMCAIAAADZF8uwAAAAAXNSR0IArs4c6QAAAMZlWElmTU0AKgAAAAgABgESAAMAAAABAAEAAAEaAAUAAAABAAAAVgEbAAUAAAABAAAAXgEoAAMAAAABAAIAAAExAAIAAAAVAAAAZodpAAQAAAABAAAAfAAAAAAAAABIAAAAAQAAAEgAAAABUGl4ZWxtYXRvciBQcm8gMy4zLjIAAAAEkAQAAgAAABQAAACyoAEAAwAAAAEAAQAAoAIABAAAAAEAAAAMoAMABAAAAAEAAAAMAAAAADIwMjM6MDU6MDkgMDk6MzI6MjkAO67pRwAAAAlwSFlzAAALEwAACxMBAJqcGAAAA7BpVFh0WE1MOmNvbS5hZG9iZS54bXAAAAAAADx4OnhtcG1ldGEgeG1sbnM6eD0iYWRvYmU6bnM6bWV0YS8iIHg6eG1wdGs9IlhNUCBDb3JlIDYuMC4wIj4KICAgPHJkZjpSREYgeG1sbnM6cmRmPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5LzAyLzIyLXJkZi1zeW50YXgtbnMjIj4KICAgICAgPHJkZjpEZXNjcmlwdGlvbiByZGY6YWJvdXQ9IiIKICAgICAgICAgICAgeG1sbnM6dGlmZj0iaHR0cDovL25zLmFkb2JlLmNvbS90aWZmLzEuMC8iCiAgICAgICAgICAgIHhtbG5zOmV4aWY9Imh0dHA6Ly9ucy5hZG9iZS5jb20vZXhpZi8xLjAvIgogICAgICAgICAgICB4bWxuczp4bXA9Imh0dHA6Ly9ucy5hZG9iZS5jb20veGFwLzEuMC8iPgogICAgICAgICA8dGlmZjpZUmVzb2x1dGlvbj43MjAwMDAvMTAwMDA8L3RpZmY6WVJlc29sdXRpb24+CiAgICAgICAgIDx0aWZmOlhSZXNvbHV0aW9uPjcyMDAwMC8xMDAwMDwvdGlmZjpYUmVzb2x1dGlvbj4KICAgICAgICAgPHRpZmY6UmVzb2x1dGlvblVuaXQ+MjwvdGlmZjpSZXNvbHV0aW9uVW5pdD4KICAgICAgICAgPHRpZmY6T3JpZW50YXRpb24+MTwvdGlmZjpPcmllbnRhdGlvbj4KICAgICAgICAgPGV4aWY6UGl4ZWxZRGltZW5zaW9uPjEyPC9leGlmOlBpeGVsWURpbWVuc2lvbj4KICAgICAgICAgPGV4aWY6UGl4ZWxYRGltZW5zaW9uPjEyPC9leGlmOlBpeGVsWERpbWVuc2lvbj4KICAgICAgICAgPHhtcDpNZXRhZGF0YURhdGU+MjAyMy0wNS0wOVQwOTozMjozNSswODowMDwveG1wOk1ldGFkYXRhRGF0ZT4KICAgICAgICAgPHhtcDpDcmVhdGVEYXRlPjIwMjMtMDUtMDlUMDk6MzI6MjkrMDg6MDA8L3htcDpDcmVhdGVEYXRlPgogICAgICAgICA8eG1wOkNyZWF0b3JUb29sPlBpeGVsbWF0b3IgUHJvIDMuMy4yPC94bXA6Q3JlYXRvclRvb2w+CiAgICAgIDwvcmRmOkRlc2NyaXB0aW9uPgogICA8L3JkZjpSREY+CjwveDp4bXBtZXRhPgrW0bvNAAAAGUlEQVQYGWNkWPiPgRBgIqQAJD+qiN5BAADiMgG3Iz4GSwAAAABJRU5ErkJggg==)';
    await expectToStringifyInto({
      source: image,
      output: image,
      processor,
    });
  });

  test('remote images', async () => {
    await expectToStringifyInto({
      source: `
![image](https://example.org/image.png)
      `,
      output: `
![image](https://example.org/image.png)
      `,
      processor,
    });
  });

  test('inline images', async () => {
    await expectToStringifyInto({
      source: `
# Emote
This is an emote: ![emote](./emote.svg). Isn't it cute?
      `,
      output: `
import asset0 from "./emote.svg";

# Emote

This is an emote: <img src={asset0} alt="emote" />. Isn't it cute?
      `,
      processor,
    });
  });

  test('custom components', async () => {
    await expectToStringifyInto({
      source: `
<object src="./image.svg" />
<video controls width="250">
  <source src="../../media/cc0-videos/flower.webm" type="video/webm" />
  <source src="../../media/cc0-videos/flower.mp4" type="video/mp4" />
  Download the <a href="/media/cc0-videos/flower.webm">WEBM</a> or
  <a href="/media/cc0-videos/flower.mp4">MP4</a> video.
</video>
      `,
      output: `
import asset0 from "./image.svg";

import asset1 from "../../media/cc0-videos/flower.webm";

import asset2 from "../../media/cc0-videos/flower.mp4";

<object src={asset0} />

<video controls width="250">
  <source src={asset1} type="video/webm" />

  <source src={asset2} type="video/mp4" />

  Download the <a href="/media/cc0-videos/flower.webm">WEBM</a> or
  <a href="/media/cc0-videos/flower.mp4">MP4</a> video.
</video>
      `,
      processor,
    });
  });

  test('downloadable links', async () => {
    await expectToStringifyInto({
      source: "<a href='./video.webm' download={true}>Download WebM</a>",
      output: `
import asset0 from "./video.webm";

<a href={asset0} download="video.webm">Download WebM</a>
      `,
      processor,
    });
  });

  test('downloadable links, preserve download attribute', async () => {
    await expectToStringifyInto({
      source: `
<a href='./video.webm' download='Example.webm'>Download WebM</a>
<a href='./video.webm' download={"Example.webm"}>Download WebM</a>
      `,
      output: `
import asset0 from "./video.webm";

import asset1 from "./video.webm";

<a href={asset0} download="Example.webm">Download WebM</a>
<a href={asset1} download={"Example.webm"}>Download WebM</a>
      `,
      processor,
    });
  });
});
