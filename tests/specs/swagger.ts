import { expect, test } from "@playwright/test";

import { waitForShiki, waitForMain } from "./util/timeout.ts";

test("swagger", async ({ page }) => {
  await page.goto("/en/docs/demo/main/swagger");

  await waitForMain(page);
  await waitForShiki(page);

  await expect(page.getByRole("article")).toMatchAriaSnapshot({
    name: "swagger.aria.yaml",
  });

  await page.getByRole("button", { name: "object Category" }).first().click();
  await page.getByRole("button", { name: "array" }).first().click();
  await page.getByRole("button", { name: "object Tag" }).click();
  await expect(page.getByRole("article")).toMatchAriaSnapshot(`
    - heading "Request body" [level=3]
    - list:
      - listitem:
        - code: id
        - text: int64
      - listitem:
        - code: category
        - text: object
        - button "object Category" [expanded]:
          - paragraph:
            - text: object
            - code: Category
        - list:
          - listitem:
            - code: id
            - text: int64
          - listitem:
            - code: name
            - text: string
      - listitem:
        - code: name
        - text: string
        - emphasis: required
        - paragraph:
          - strong: "example:"
          - code: doggie
      - listitem:
        - code: photoUrls
        - text: array of strings
        - emphasis: required
      - listitem:
        - code: tags
        - text: array of objects
        - button "array" [expanded]:
          - paragraph: array
        - button "object Tag" [expanded]:
          - paragraph:
            - text: object
            - code: Tag
        - list:
          - listitem:
            - code: id
            - text: int64
          - listitem:
            - code: name
            - text: string
      - listitem:
        - code: status
        - text: string
        - paragraph: pet status in the store
        - paragraph:
          - strong: one of
          - code: available
          - code: pending
          - code: sold
    `);
});
