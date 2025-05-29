import { test, expect } from "@playwright/test";

test("search", async ({ page, isMobile }) => {
  await page.goto("/en/docs/demo/main/markdown");

  if (isMobile) {
    await page.getByRole("button", { name: "Chapters" }).click();
  }

  await page.getByRole("button", { name: "Search" }).click();

  await expect(page.getByRole("textbox", { name: "Search in demo main" })) //
    .toBeInViewport();

  await page.getByRole("textbox", { name: "Search in demo main" }).fill("mdxbuilder");

  await page.getByRole("link", { name: "class secretflow_doctools." }).click();

  await expect(page.locator('[id="secretflow_doctools\\.builder\\.MdxBuilder"]')) //
    .toBeInViewport();
});
