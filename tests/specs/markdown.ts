import { test, expect } from "@playwright/test";

import { softScreenshot } from "./util/screenshot.ts";
import { waitForMain } from "./util/timeout.ts";

test("markdown", async ({ page }) => {
  await page.goto("/en/docs/demo/main/markdown");

  await waitForMain(page);

  await expect(page.getByRole("article")).toMatchAriaSnapshot(`- strong: Lorem ipsum`);

  await expect(page.getByRole("article")).toMatchAriaSnapshot(
    `- emphasis: dummy or placeholder`,
  );

  await expect(page.getByRole("article")).toMatchAriaSnapshot(
    `- code: web development`,
  );

  await expect(page.locator("s")).toMatchAriaSnapshot(
    `- text: without meaningful text`,
  );

  await expect(page.getByRole("article")).toMatchAriaSnapshot(`
    - list:
      - listitem: List of lists of lists – this article itself is a list of lists, so it contains itself
      - listitem: Lists of academic journals
      - listitem: Lists of encyclopedias
      - listitem: Lists of important publications in science
      - listitem:
        - text: Lists of problems
        - list:
          - listitem: Lists of unsolved problems
    `);

  await expect(page.getByRole("article")).toMatchAriaSnapshot(`
    - list:
      - listitem: Lists of abbreviations
      - listitem: Lists of dictionaries
      - listitem:
        - text: Lists of English words
        - list:
          - listitem: Lists of collective nouns
          - listitem:
            - text: Lists of English words by country or language of origin
            - list:
              - listitem: Lists of English words of Celtic origin
              - listitem: Lists of English words of Scottish origin
          - listitem: Lists of Merriam-Webster’s Words of the Year
          - listitem: Lists of pejorative terms for people
          - listitem: Lists of words having different meanings in American and British English
          - listitem: Word lists by frequency
    `);

  await expect(page.getByRole("figure")).toMatchAriaSnapshot(`
    - 'figure "Source: https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/img"':
      - paragraph:
        - img
      - paragraph:
        - text: "Source:"
        - link "https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/img":
          - /url: https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/img
    `);

  await expect(page.getByRole("article")).toMatchAriaSnapshot(`
    - heading "Note" [level=4]
    - paragraph: Let’s give readers a helpful hint!
    `);

  await expect(page.getByRole("article")).toMatchAriaSnapshot(`
    - heading "Deprecated" [level=4]
    - paragraph: "Deprecated since version 1.2.3: Explanation of the deprecation."
    `);
});

test("attachments", async ({ page }) => {
  await page.goto("/en/docs/demo/main/markdown");

  await waitForMain(page);

  await softScreenshot({
    name: "grapefruit-slice",
    elem: page
      .getByRole("figure", { name: "Source: https://developer." })
      .getByRole("img"),
    alternative: () => Promise.resolve(),
  });
});
