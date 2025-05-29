import { test, expect } from "@playwright/test";

import { waitForMain } from "./util/timeout.ts";

test("navigation", async ({ page, isMobile }) => {
  await page.goto("/en/docs/demo/main");

  await waitForMain(page);

  await expect(page.getByRole("navigation")).toMatchAriaSnapshot({
    name: "navigation.aria.yaml",
  });

  await expect(page.getByRole("article")).toMatchAriaSnapshot({
    name: "toctree.aria.yaml",
  });

  if (isMobile) {
    await page.getByRole("button", { name: "Chapters" }).click();
  }

  await page.getByRole("menuitem", { name: "Markdown" }).getByRole("link").click();

  await page.waitForURL("/en/docs/demo/main/markdown");

  if (isMobile) {
    await page.getByRole("button", { name: "Sections" }).click();
  }

  await expect(page.locator(".ant-anchor")).toMatchAriaSnapshot({
    name: "anchor.aria.yaml",
  });
});
