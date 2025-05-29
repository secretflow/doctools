import { expect, test } from "@playwright/test";

import { shouldExpectScreenshot, softScreenshot } from "./util/screenshot.ts";
import { waitForMedia, waitForShiki, waitForMain } from "./util/timeout.ts";

if (shouldExpectScreenshot) {
  test.describe.configure({ retries: 3 });
  test.slow();
} else {
  test.skip(({ browserName }) => browserName !== "chromium");
}

for (const path of [
  "/en/docs/heu/v0.6.0b0/",
  "/en/docs/kuscia/v0.15.0b0/",
  "/en/docs/psi/v0.5.0b0/",
  "/en/docs/scql/0.9.3b1/",
  "/en/docs/secretflow/v1.12.0b0/",
  "/en/docs/serving/0.9.0b0/",
  "/en/docs/spec/v1.1.0b0/",
  "/en/docs/spu/0.9.3b0/",

  "/docs/heu/v0.6.0b0/",
  "/docs/kuscia/v0.15.0b0/",
  "/docs/scql/0.9.3b1/",
  "/docs/secretflow/v1.12.0b0/",
  "/docs/spu/0.9.3b0/",

  "/zh-CN/docs/interconnection/0.1.0b1/",
  "/zh-CN/docs/trustedflow/0.4.0b0/",

  "/en/docs/heu/v0.6.0b0/getting_started/algo_choice",
  "/en/docs/scql/0.9.3b1/reference/broker-api",
  "/en/docs/secretflow/v1.12.0b0/tutorial/Federate_Learning_for_Image_Classification",
  "/en/docs/secretflow/v1.12.0b0/tutorial/data_preprocessing_with_data_frame",
  "/en/docs/spu/0.9.3b0/development/type_system",
  "/en/docs/spu/0.9.3b0/reference/runtime_config",
  "/zh-CN/docs/kuscia/v0.14.0b0/reference/apis/kusciajob_cn",
  "/zh-CN/docs/kuscia/v0.14.0b0/troubleshoot/concept/kuscia_vs_ray",

  "/en/docs/secretflow/v1.12.0b0/developer/design/architecture#communication-and-scheduling",
  "/en/docs/secretflow/v1.12.0b0/source/secretflow.data.data.ndarray#secretflow.data.ndarray.ndarray.load",
  "/zh-CN/docs/secretpad/v0.12.0b0/deployment/guide#install-sh",
]) {
  test(path, async ({ page }) => {
    await page.goto(path);

    await waitForMain(page, 60_000);
    await waitForMedia(page, 60_000);
    await waitForShiki(page, 60_000);

    const { hash } = new URL(page.url());

    if (hash) {
      await page.locator(`[href="${hash}"]`).first().click();
    }

    await softScreenshot({
      name: path.replace(/\/+$/, ""),
      elem: page.locator("#root"),
      alternative: async () => {
        await expect(page.locator("article")) //
          .toMatchAriaSnapshot({ name: `${path}.article.aria.yaml` });
      },
      screenshot: { animations: "disabled" },
    });
  });
}
