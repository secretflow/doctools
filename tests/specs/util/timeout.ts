import type { Page } from "@playwright/test";
import { expect } from "@playwright/test";

export async function waitForMain(page: Page, timeout = 20_000) {
  const content = page
    .getByRole("article")
    .or(page.getByRole("navigation"))
    .or(page.getByRole("heading"))
    .first();
  await expect(content).toBeVisible({ timeout });
  await expect(page.locator(".ant-skeleton")).toHaveCount(0, { timeout });
}

export async function waitForMedia(page: Page, timeout = 20_000) {
  for (const img of await page.locator("img").all()) {
    await expect(img).toHaveJSProperty("complete", true, { timeout });
  }
}

export async function waitForShiki(page: Page, timeout = 20_000) {
  await expect(page.locator(".shiki-pending")).toHaveCount(0, { timeout });
}
