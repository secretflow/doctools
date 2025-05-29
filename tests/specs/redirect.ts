import { expect, test } from "@playwright/test";

import { waitForMain } from "./util/timeout.ts";

for (const [canonical, aliases] of Object.entries({
  "/en/docs/demo/main/": [
    "/docs/demo",
    "/en/docs/demo",
    "/zh-CN/docs/demo",
    "/de-AT/docs/demo/latest",
  ],
  "/en/docs/demo/main/api#secretflow_doctools.builder.MdxBuilder": [
    "/docs/demo/api#secretflow_doctools.builder.MdxBuilder",
    "/docs/demo/main/api#secretflow_doctools.builder.MdxBuilder",
    "/en/docs/demo/api#secretflow_doctools.builder.MdxBuilder",
  ],
})) {
  for (const alias of aliases) {
    test(`redirect ${alias}`, async ({ page }) => {
      await page.goto(alias);
      await waitForMain(page);
      const expected = new URL(canonical, page.url());
      await expect(page).toHaveURL(expected.href);
    });
  }
}

test("page not found", async ({ page }) => {
  await page.goto("/en/docs/demo/main/api2");

  await waitForMain(page);

  await expect(page.getByText("Page not found")).toMatchAriaSnapshot({
    name: "not-found.page.aria.yaml",
  });

  await expect(page.getByRole("navigation")).toMatchAriaSnapshot({
    name: "not-found.navigation.aria.yaml",
  });
});

test("repo not found", async ({ page }) => {
  await page.goto("/en/docs/dem0/");

  await waitForMain(page);

  await expect(page.locator("#root")).toMatchAriaSnapshot({
    name: "not-found.repo.aria.yaml",
  });
});
