import { expect, test } from "@playwright/test";

import { softScreenshot } from "./util/screenshot.ts";
import { waitForMain } from "./util/timeout.ts";

test.beforeEach(async ({ page }) => {
  await page.goto("/en/docs/demo/main/notebook");
  await waitForMain(page);
});

test("loguru", async ({ page }) => {
  await softScreenshot({
    name: "loguru",
    elem: page.locator("pre").filter({ hasText: "__main__" }),
    alternative: (elem) =>
      expect(elem.innerHTML()).resolves
        .toBe(`<code><span class="ansi32">2025-05-19 17:34:52.769</span> | <span class="ansi34"></span><span class="ansi1 ansi34">DEBUG   </span> | <span class="ansi36">__main__</span>:<span class="ansi36">&lt;module&gt;</span>:<span class="ansi36">3</span> - <span class="ansi34"></span><span class="ansi1 ansi34">That's it, beautiful and simple logging!</span>
</code>`),
    screenshot: {
      mask: [page.getByText(/\d{4}-\d{2}-\d{2}/)],
    },
  });
});

test("ansi", async ({ page }) => {
  await softScreenshot({
    name: "ansi",
    elem: page.getByText("30 31 32 33 34 35 36 37 40 41"),
    alternative: (elem) =>
      expect(elem.innerHTML()).resolves.toBe(`<span class="ansi30">30</span>
<span class="ansi31">31</span>
<span class="ansi32">32</span>
<span class="ansi33">33</span>
<span class="ansi34">34</span>
<span class="ansi35">35</span>
<span class="ansi36">36</span>
<span class="ansi97">37</span>

<span class="ansi40"></span><span class="ansi30 ansi40">40</span>
<span class="ansi41"></span><span class="ansi30 ansi41">41</span>
<span class="ansi42"></span><span class="ansi30 ansi42">42</span>
<span class="ansi43"></span><span class="ansi30 ansi43">43</span>
<span class="ansi44"></span><span class="ansi30 ansi44">44</span>
<span class="ansi45"></span><span class="ansi30 ansi45">45</span>
<span class="ansi46"></span><span class="ansi30 ansi46">46</span>
<span class="ansi107"></span><span class="ansi30 ansi107">47</span>

<span class="ansi1">1</span>
<span class="ansi2">2</span>
<span class="ansi4">4</span>
<span class="ansi8">8</span>
<span class="ansi9">9</span>
`),
  });
});
