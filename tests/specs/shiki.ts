import { test, expect } from "@playwright/test";

import { softScreenshot } from "./util/screenshot.ts";
import { waitForShiki, waitForMain } from "./util/timeout.ts";

test("python", async ({ page }) => {
  await page.goto("/en/docs/demo/main/markdown");

  await waitForMain(page);
  await waitForShiki(page);

  await softScreenshot({
    name: "python",
    elem: page.locator("pre").filter({ hasText: "n = int(input('Type" }),
    alternative: (elem) =>
      expect(elem.innerHTML()).resolves
        .toBe(`<code><span class="line"><span style="color: rgb(36, 41, 46);">n </span><span style="color: rgb(215, 58, 73);">=</span><span style="color: rgb(0, 92, 197);"> int</span><span style="color: rgb(36, 41, 46);">(</span><span style="color: rgb(0, 92, 197);">input</span><span style="color: rgb(36, 41, 46);">(</span><span style="color: rgb(3, 47, 98);">'Type a number, and its factorial will be printed: '</span><span style="color: rgb(36, 41, 46);">))</span></span>
<span class="line"></span>
<span class="line"><span style="color: rgb(215, 58, 73);">if</span><span style="color: rgb(36, 41, 46);"> n </span><span style="color: rgb(215, 58, 73);">&lt;</span><span style="color: rgb(0, 92, 197);"> 0</span><span style="color: rgb(36, 41, 46);">:</span></span>
<span class="line"><span style="color: rgb(215, 58, 73);">    raise</span><span style="color: rgb(0, 92, 197);"> ValueError</span><span style="color: rgb(36, 41, 46);">(</span><span style="color: rgb(3, 47, 98);">'You must enter a non-negative integer'</span><span style="color: rgb(36, 41, 46);">)</span></span>
<span class="line"></span>
<span class="line"><span style="color: rgb(36, 41, 46);">factorial </span><span style="color: rgb(215, 58, 73);">=</span><span style="color: rgb(0, 92, 197);"> 1</span></span>
<span class="line"><span style="color: rgb(215, 58, 73);">for</span><span style="color: rgb(36, 41, 46);"> i </span><span style="color: rgb(215, 58, 73);">in</span><span style="color: rgb(0, 92, 197);"> range</span><span style="color: rgb(36, 41, 46);">(</span><span style="color: rgb(0, 92, 197);">2</span><span style="color: rgb(36, 41, 46);">, n </span><span style="color: rgb(215, 58, 73);">+</span><span style="color: rgb(0, 92, 197);"> 1</span><span style="color: rgb(36, 41, 46);">):</span></span>
<span class="line"><span style="color: rgb(36, 41, 46);">    factorial </span><span style="color: rgb(215, 58, 73);">*=</span><span style="color: rgb(36, 41, 46);"> i</span></span>
<span class="line"></span>
<span class="line"><span style="color: rgb(0, 92, 197);">print</span><span style="color: rgb(36, 41, 46);">(factorial)</span></span>
<span class="line"></span></code>`),
  });
});

test("json", async ({ page }) => {
  await page.goto("/en/docs/demo/main/swagger");

  await waitForMain(page);
  await waitForShiki(page);

  await softScreenshot({
    name: "json",
    elem: page.getByText('json{ "id": 0, "category').locator("pre").first(),
    alternative: (elem) =>
      expect(elem.innerHTML()).resolves
        .toBe(`<code><span class="line"><span style="color: rgb(36, 41, 46);">{</span></span>
<span class="line"><span style="color: rgb(0, 92, 197);">  "id"</span><span style="color: rgb(36, 41, 46);">: </span><span style="color: rgb(0, 92, 197);">0</span><span style="color: rgb(36, 41, 46);">,</span></span>
<span class="line"><span style="color: rgb(0, 92, 197);">  "category"</span><span style="color: rgb(36, 41, 46);">: {</span></span>
<span class="line"><span style="color: rgb(0, 92, 197);">    "id"</span><span style="color: rgb(36, 41, 46);">: </span><span style="color: rgb(0, 92, 197);">0</span><span style="color: rgb(36, 41, 46);">,</span></span>
<span class="line"><span style="color: rgb(0, 92, 197);">    "name"</span><span style="color: rgb(36, 41, 46);">: </span><span style="color: rgb(3, 47, 98);">"string"</span></span>
<span class="line"><span style="color: rgb(36, 41, 46);">  },</span></span>
<span class="line"><span style="color: rgb(0, 92, 197);">  "name"</span><span style="color: rgb(36, 41, 46);">: </span><span style="color: rgb(3, 47, 98);">"doggie"</span><span style="color: rgb(36, 41, 46);">,</span></span>
<span class="line"><span style="color: rgb(0, 92, 197);">  "photoUrls"</span><span style="color: rgb(36, 41, 46);">: [</span></span>
<span class="line"><span style="color: rgb(3, 47, 98);">    "string"</span></span>
<span class="line"><span style="color: rgb(36, 41, 46);">  ],</span></span>
<span class="line"><span style="color: rgb(0, 92, 197);">  "tags"</span><span style="color: rgb(36, 41, 46);">: [</span></span>
<span class="line"><span style="color: rgb(36, 41, 46);">    {</span></span>
<span class="line"><span style="color: rgb(0, 92, 197);">      "id"</span><span style="color: rgb(36, 41, 46);">: </span><span style="color: rgb(0, 92, 197);">0</span><span style="color: rgb(36, 41, 46);">,</span></span>
<span class="line"><span style="color: rgb(0, 92, 197);">      "name"</span><span style="color: rgb(36, 41, 46);">: </span><span style="color: rgb(3, 47, 98);">"string"</span></span>
<span class="line"><span style="color: rgb(36, 41, 46);">    }</span></span>
<span class="line"><span style="color: rgb(36, 41, 46);">  ],</span></span>
<span class="line"><span style="color: rgb(0, 92, 197);">  "status"</span><span style="color: rgb(36, 41, 46);">: </span><span style="color: rgb(3, 47, 98);">"available"</span></span>
<span class="line"><span style="color: rgb(36, 41, 46);">}</span></span></code>`),
  });
});
