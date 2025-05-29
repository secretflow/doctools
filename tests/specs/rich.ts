import { expect, test } from "@playwright/test";

import { shouldExpectScreenshot, softScreenshot } from "./util/screenshot.ts";
import { waitForMain } from "./util/timeout.ts";

if (shouldExpectScreenshot) {
  test.describe.configure({ retries: 3 });
}

test("math", async ({ page }) => {
  await page.goto("/en/docs/demo/main/rich#math");
  await waitForMain(page);
  await softScreenshot({
    name: "math",
    elem: page.getByText("∇2f=r21∂r∂(r2∂r∂f)+r2sinθ1∂θ∂"),
    alternative: (elem) => expect(elem).toContainClass("katex-html"),
  });
});

test("graphviz", async ({ page }) => {
  await page.goto("/en/docs/demo/main/rich#graphviz");
  await waitForMain(page);
  await softScreenshot({
    name: "graphviz",
    elem: page.getByRole("img").getByText("mygraph //absl/random:random"),
    alternative: (elem) =>
      expect(elem).toContainText(
        "mygraph //absl/random:random //absl/random:random //absl/random:distributions //absl/random:distributions //absl/random:random->//absl/random:distributions //absl/random:seed_sequences //absl/random:seed_sequences //absl/random:random->//absl/random:seed_sequences //absl/random/internal:pool_urbg //absl/random/internal:pool_urbg //absl/random:random->//absl/random/internal:pool_urbg //absl/random/internal:nonsecure_base //absl/random/internal:nonsecure_base //absl/random:random->//absl/random/internal:nonsecure_base //absl/strings:strings //absl/strings:strings //absl/random:distributions->//absl/strings:strings //absl/random:seed_sequences->//absl/random/internal:pool_urbg //absl/random:seed_sequences->//absl/random/internal:nonsecure_base //absl/random/internal:seed_material //absl/random/internal:seed_material //absl/random:seed_sequences->//absl/random/internal:seed_material //absl/random/internal:salted_seed_seq //absl/random/internal:salted_seed_seq //absl/random:seed_sequences->//absl/random/internal:salted_seed_seq //absl/random/internal:pool_urbg->//absl/random/internal:seed_material //absl/random/internal:nonsecure_base->//absl/random/internal:pool_urbg //absl/random/internal:nonsecure_base->//absl/random/internal:seed_material //absl/random/internal:nonsecure_base->//absl/random/internal:salted_seed_seq //absl/random/internal:seed_material->//absl/strings:strings //absl/random/internal:salted_seed_seq->//absl/random/internal:seed_material",
      ),
  });
});

test("mermaid", async ({ page }) => {
  await page.goto("/en/docs/demo/main/rich#mermaid");
  await waitForMain(page);
  await softScreenshot({
    name: "mermaid",
    elem: page
      .getByRole("article")
      .locator("div")
      .filter({ hasText: "JohnBobAliceJohnBobAlice#" }),
    alternative: (elem) =>
      expect(elem.locator(".note")).toHaveCSS("fill", "rgb(255, 245, 173)"),
  });
});
