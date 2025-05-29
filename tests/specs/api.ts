import { test, expect } from "@playwright/test";

import { waitForMain } from "./util/timeout.ts";

test("api", async ({ page }) => {
  await page.goto("/en/docs/demo/main/api");

  await waitForMain(page);

  await expect(page.getByRole("article")).toMatchAriaSnapshot({
    name: "api.aria.yaml",
  });
});
