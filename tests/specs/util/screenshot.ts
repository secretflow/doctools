import process from "node:process";

import type { Locator } from "@playwright/test";
import { test, expect } from "@playwright/test";

export const shouldExpectScreenshot = process.platform === "linux";

export async function softScreenshot({
  name,
  elem,
  alternative,
  screenshot,
}: {
  name: string;
  elem: Locator;
  alternative: (elem: Locator) => Promise<void>;
  screenshot?: ScreenshotOptions;
}) {
  const path = `${name}.png`;
  const body = await elem.screenshot({ ...screenshot, scale: "css" });
  test.info().attach(path, { body, contentType: "image/png" });
  if (shouldExpectScreenshot) {
    expect(body).toMatchSnapshot(path, screenshot);
  } else {
    await alternative(elem);
  }
}

type Expected<T> = ReturnType<typeof expect<T>>;

type ScreenshotOptions = NonNullable<
  Parameters<Expected<Locator>["toHaveScreenshot"]>[0]
>;
