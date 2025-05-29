import { expect, test } from "@playwright/test";

test.describe.configure({ retries: 3 });

test("anchors", async ({ page, isMobile }) => {
  await page.goto("/en/docs/demo/main/markdown");

  await page
    .getByRole("article")
    .getByRole("link", { name: "Society and social sciences", exact: true })
    .click();

  await expect(page.getByRole("heading", { name: "Society and social sciences" })) //
    .toBeInViewport();

  await page.reload();

  await expect(page.getByRole("heading", { name: "Society and social sciences" })) //
    .toBeInViewport();

  await page.getByRole("link", { name: "Poem" }).click();

  await expect(page.getByRole("blockquote")).toBeInViewport();

  await page.goBack();

  if (!isMobile) {
    await page
      .getByRole("emphasis")
      .filter({ hasText: "[1]" })
      .getByRole("link")
      .click();

    await page.waitForURL((u) => u.hash === "#id1");

    await expect(page.getByRole("superscript").getByRole("link", { name: "1" })) //
      .toBeInViewport();
  } else {
    // flaky
  }
});
